use crate::common::common_error_logic::send_error_to_requester_channel;
use crate::communication::communication_types::{
    BasicResponse, GenericRoomIdAndPeerId, RoomPermissions,
    VoiceServerClosePeer, VoiceServerCreateRoom, VoiceServerDestroyRoom,
    VoiceServerRequest,
};
use crate::communication::data_capturer::CaptureResult;
use crate::communication::{data_capturer, data_fetcher};
use crate::data_store::db_models::{DBRoom, DBRoomBlock, DBRoomPermissions};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rabbitmq::rabbit;
use crate::state::state::ServerState;
use crate::state::state_types::Room;
use futures::lock::Mutex;
use lapin::Channel;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::mem::drop;
use std::sync::Arc;

use super::permission_configs;
pub type EncounteredError = bool;
pub type AllPermissionsResult = (EncounteredError, HashMap<i32, RoomPermissions>);
pub type ListenerOrSpeaker = String;
pub type RoomOwnerAndSettings = (bool, i32, String);

// Managing rooms happens in a pub-sub fashion:
//  - The client waits on the response from this server.
//  - This server waits on the response from the voice server.
//  - We gather voice server messages in a separate task.
//  - Once this server gathers a response, it fans it to all targets.
//
// For Example, kicking someone:
//  - Admin/Mod requests user x to be kicked
//  - This server sends the request to the voice server
//  - Once the voice server responds, if it is successful
//       the user is removed from the state of the server
//       and this update is fanned/brodcasted across all users in the room.

pub async fn block_user_from_room(
    user_id: i32,
    room_id: i32,
    requester_id: i32,
    server_state: &mut ServerState,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) {
    let mut handler = execution_handler.lock().await;
    let owner_gather: (bool, i32, String) =
        data_fetcher::get_room_owner_and_settings(&mut handler, &room_id).await;

    // ensure the requester is the owner.
    // no errors were encountered gathering the owner
    if owner_gather.0 == false && owner_gather.1 == requester_id {
        //capture new block and send request to voice server
        let new_block = DBRoomBlock {
            id: -1,
            owner_room_id: room_id.clone(),
            blocked_user_id: user_id.clone(),
        };
        let capture_result = data_capturer::capture_new_room_block(&mut handler, &new_block).await;
        drop(handler);
        handle_user_block_capture_result(
            capture_result,
            requester_id,
            user_id.clone(),
            server_state,
            room_id,
            publish_channel,
        )
        .await;
        return;
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_blocking_user".to_string(),
    );
}

pub async fn create_room(
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    public: bool,
) {
    let mut handler = execution_handler.lock().await;
    let db_room = DBRoom {
        id: -1,
        owner_id: requester_id,
        chat_mode: "fast".to_owned(),
    };
    let room_id = data_capturer::capture_new_room(&mut handler, &db_room).await;
    if room_id == -1 {
        send_error_to_requester_channel(
            "internal error".to_string(),
            requester_id,
            server_state,
            "issue_creating_room".to_string(),
        );
    } else {
        let channel = publish_channel.lock().await;
        continue_with_successful_room_creation(room_id, &channel, public, server_state).await;
    }
}

pub async fn destroy_room(
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    room_id: &i32,
) {
    // remove from db
    let mut handler = execution_handler.lock().await;
    data_capturer::capture_room_removal(&mut handler, room_id).await;
    drop(handler);

    // remove from state
    server_state.rooms.remove(room_id);

    // remove from voice server
    let request_to_voice_server = VoiceServerDestroyRoom {
        roomId: room_id.to_string(),
    };
    let request_str =
        create_voice_server_request("destroy-room", &"-1".to_owned(), request_to_voice_server);
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str).await;
}

pub async fn remove_user_from_room_basic(
    request_to_voice_server: VoiceServerClosePeer,
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) {
    server_state
        .rooms
        .get_mut(&request_to_voice_server.roomId.parse().unwrap())
        .unwrap()
        .user_ids
        .remove(&request_to_voice_server.peerId.parse().unwrap());
    let request_str: String = create_voice_server_request(
        "close-peer",
        &request_to_voice_server.peerId.clone(),
        request_to_voice_server,
    );
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str).await;
}

/// - Handles both speaker join and peer join
/// - peer join-> Only consumes audio from other speakers
/// - speaker join-> Consumes audio and produces
/// - Block checks are handled outside of this function's scope
pub async fn join_room(
    request_to_voice_server: GenericRoomIdAndPeerId,
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    type_of_join: &str,
) -> Option<()> {
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId.parse().unwrap();
    let user_id: i32 = request_to_voice_server.peerId.parse().unwrap();
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    let state_room: &Room = server_state.rooms.get(&room_id)?;

    // ensure the user has the permissions to join
    let result: EncounteredError = check_or_insert_initial_permissions(
        state_room,
        type_of_join,
        &user_id,
        all_room_permissions,
        &mut handler,
    )
    .await;
    drop(handler);

    // if the user has this permission
    if result == false {
        let channel = publish_channel.lock().await;
        let request_str = create_voice_server_request(
            type_of_join,
            &request_to_voice_server.peerId.clone(),
            request_to_voice_server,
        );
        rabbit::publish_message(&channel, request_str).await;
        add_user_to_room_state(&room_id, user_id, server_state);
        return None;
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_joining_room".to_string(),
    );
    return None;
}

///  Adds a speaker that is already an existing peer in a room
///  this happens and is only allowed for users who have already
///  requested to speak.
///
///  - Mods who are listeners want to come to the stage.
///  - Mods accept other user's request to come to the stage.
pub async fn add_speaker(
    request_to_voice_server: GenericRoomIdAndPeerId,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    requester_id: &i32,
    server_state: &mut ServerState,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId.parse().unwrap();
    let user_id: i32 = request_to_voice_server.peerId.parse().unwrap();
    let all_room_permissions: AllPermissionsResult =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;

    if all_room_permissions.0 == false {
        let requester_permissions: &RoomPermissions =
            all_room_permissions.1.get(requester_id).unwrap();
        let requestee_permissions: &RoomPermissions = all_room_permissions.1.get(&user_id).unwrap();

        // you can only be added as a speaker if you requested.
        if requester_permissions.is_mod && requestee_permissions.asked_to_speak {
            let new_permission_config = permission_configs::regular_speaker(room_id, user_id);
            data_capturer::capture_new_room_permissions_update(
                &new_permission_config,
                &mut handler,
            )
            .await;
            drop(handler);
            let request_str = create_voice_server_request(
                "add-speaker",
                &user_id.to_string(),
                request_to_voice_server,
            );
            let channel = publish_channel.lock().await;
            rabbit::publish_message(&channel, request_str).await;
            return;
        }
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id.clone(),
        server_state,
        "issue_adding_speaker".to_string(),
    );
}

pub async fn remove_speaker(
    request_to_voice_server: GenericRoomIdAndPeerId,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    requester_id: &i32,
    server_state: &mut ServerState,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId.parse().unwrap();
    let user_id: i32 = request_to_voice_server.peerId.parse().unwrap();
    let all_room_permissions: AllPermissionsResult =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    let room_owner_data: RoomOwnerAndSettings =
        data_fetcher::get_room_owner_and_settings(&mut handler, &room_id).await;
    // make sure we didn't encounter errors getting essential information
    if room_owner_data.0 == false && all_room_permissions.0 == false {
        let requester_permissions: &RoomPermissions =
            all_room_permissions.1.get(requester_id).unwrap();
        let requestee_permissions: &RoomPermissions = all_room_permissions.1.get(&user_id).unwrap();
        if requester_can_remove_speaker(
            (requester_id, &requester_permissions),
            (&user_id, &requestee_permissions),
            &room_owner_data.1,
        ) {
            // modify the database with the new permissions(no longer a speaker)
            let new_permissions = get_new_removed_speaker_permission_config(
                requestee_permissions,
                &user_id,
                &room_id,
            );
            data_capturer::capture_new_room_permissions_update(&new_permissions, &mut handler)
                .await;
            drop(handler);
            let request_str = create_voice_server_request(
                "remove-speaker",
                &user_id.to_string(),
                request_to_voice_server,
            );
            let channel = publish_channel.lock().await;
            rabbit::publish_message(&channel, request_str).await;
            return;
        }
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id.clone(),
        server_state,
        "issue_removing_speaker".to_string(),
    );
}

///  This function handles the following requests
///  - Sending tracks
///  - Connection transports
///  - Getting recv tracks
///
///  We make usage of the dynamic json value
///  to remove the need to create
///  types for all of the media soup objects
///  being transfered to the voice server.
pub async fn handle_web_rtc_specific_requests(
    request_to_voice_server: serde_json::Value,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    op_code: &str,
) {
    let user_id = request_to_voice_server["peerId"].to_string();
    let request_str = create_voice_server_request(op_code, &user_id, request_to_voice_server);
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str).await;
}

async fn handle_user_block_capture_result(
    capture_result: CaptureResult,
    requester_id: i32,
    user_id: i32,
    server_state: &mut ServerState,
    room_id: i32,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) {
    if capture_result.encountered_error == false {
        let request = VoiceServerClosePeer {
            roomId: room_id.to_string(),
            peerId: user_id.to_string(),
            kicked: true,
        };
        remove_user_from_room_basic(request, server_state, publish_channel).await;
        return;
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_blocking_user".to_string(),
    );
}

fn add_user_to_room_state(room_id: &i32, user_id: i32, state: &mut ServerState) {
    state
        .rooms
        .get_mut(&room_id)
        .unwrap()
        .user_ids
        .insert(user_id);
}

fn construct_basic_room_for_state(room_id: i32, public: bool) -> Room {
    return Room {
        room_id: room_id,
        muted: HashSet::new(),
        voice_server_id: 0.to_string(),
        /// not yet implemented(feature)
        deaf: HashSet::new(),
        user_ids: HashSet::new(),
        public: public,
        auto_speaker: false,
    };
}

/// executed after database insertion is proven to be successful.
async fn continue_with_successful_room_creation(
    room_id: i32,
    channel: &Channel,
    public: bool,
    server_state: &mut ServerState,
) {
    let request_to_voice_server = VoiceServerCreateRoom {
        roomId: room_id.clone().to_string(),
    };
    let request_str: String = serde_json::to_string(&request_to_voice_server).unwrap();
    rabbit::publish_message(channel, request_str).await;
    let new_room_state: Room = construct_basic_room_for_state(room_id.clone(), public);
    server_state.rooms.insert(room_id, new_room_state);
}

async fn check_or_insert_initial_permissions(
    room: &Room,
    join_as: &str,
    requester_id: &i32,
    permissions: AllPermissionsResult,
    handler: &mut ExecutionHandler,
) -> EncounteredError {
    if permissions.0 == false {
        // if the user already has permissions just return them
        if permissions.1.contains_key(requester_id) {
            let current_user_permissions = permissions.1.get(&requester_id).unwrap();
            // If the user is requesting to join as speaker:
            // - But isn't a speaker in the database and the room is auto speaker
            //    we accept this request because the user could have been a peer
            //    previously and now they are joining as a speaker.
            if join_as == "speaker"
                && current_user_permissions.is_speaker == false
                && room.auto_speaker == false
            {
                return false;
            }
            return true;
        }
        // if the user doesn't have permissions insert them
        else {
            let result: EncounteredError =
                create_initial_user_permissions(handler, join_as, room, requester_id).await;
            return result;
        }
    }
    return true;
}

async fn create_initial_user_permissions(
    handler: &mut ExecutionHandler,
    type_of_join: &str,
    room: &Room,
    requester_id: &i32,
) -> EncounteredError {
    if type_of_join == "join-as-speaker" {
        // rooms that are auto speaker
        // doesn't require hand raising
        if room.auto_speaker {
            let init_permissions =
                permission_configs::regular_speaker(requester_id.clone(), room.room_id.clone());
            let result =
                data_capturer::capture_new_room_permissions(&init_permissions, handler).await;
            return result;
        } else {
            return true; // can't join as speaker initially if auto speaker is off
        }
    } else {
        let init_permissions =
            permission_configs::regular_listener(requester_id.clone(), room.room_id.clone());
        let result = data_capturer::capture_new_room_permissions(&init_permissions, handler).await;
        return result;
    }
}

fn create_voice_server_request<T: Serialize>(op_code: &str, uid: &String, data: T) -> String {
    let voice_server_req = VoiceServerRequest {
        op: op_code.to_owned(),
        uid: uid.to_owned(),
        d: data,
    };
    return serde_json::to_string(&voice_server_req).unwrap();
}

/// When Users can only be removed from speaker:
/// - If the owner requests(doesn't matter if the user is a mod)
/// - If the person being removed is not a mod and the requester is a mod
///     and the person being removed is a speaker.
///
/// If none of these conditions are met, it is an invalid request.
fn requester_can_remove_speaker(
    remover_permissions: (&i32, &RoomPermissions),
    removee_permissions: (&i32, &RoomPermissions),
    owner_id: &i32,
) -> bool {
    if remover_permissions.0 == owner_id && removee_permissions.0 != remover_permissions.0 {
        return true;
    }
    if remover_permissions.1.is_mod
        && removee_permissions.1.is_mod == false
        && removee_permissions.1.is_speaker
    {
        return true;
    }
    return false;
}

/// We know that the speaker was removed
///     so we just need to gather the config
///     that represents the old version of the config
///     for this user, with the "is_speaker" field set
///     to false.
fn get_new_removed_speaker_permission_config(
    current_permissions: &RoomPermissions,
    user_id: &i32,
    room_id: &i32,
) -> DBRoomPermissions {
    if current_permissions.is_mod {
        return permission_configs::modded_non_speaker(room_id.clone(), user_id.clone());
    } else {
        return permission_configs::regular_listener(room_id.clone(), user_id.clone());
    }
}
