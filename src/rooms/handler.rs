use super::permission_configs;
use crate::common::response_logic::send_to_requester_channel;
use crate::communication::data_capturer::CaptureResult;
use crate::communication::types::{
    BasicResponse, GenericRoomIdAndPeerId, RoomPermissions, RoomUpdate, VoiceServerClosePeer,
    VoiceServerCreateRoom, VoiceServerDestroyRoom, VoiceServerRequest,
};
use crate::communication::{data_capturer, data_fetcher};
use crate::data_store::db_models::{DBRoom, DBRoomBlock, DBRoomPermissions};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::logging;
use crate::rabbitmq::rabbit;
use crate::state::owner_queue::OwnerQueue;
use crate::state::state::ServerState;
use crate::state::types::Room;
use crate::ws_fan::{self, fan};
use chrono::Utc;
use futures::lock::Mutex;
use lapin::Channel;
use serde::Serialize;
use std::collections::{HashMap, HashSet, LinkedList};
use std::mem::drop;
use std::sync::Arc;
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
    let all_room_permissions =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;

    if can_block_this_user_from_room(
        all_room_permissions.1,
        owner_gather.1,
        requester_id,
        user_id,
    ) {
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
    send_to_requester_channel(
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
    name: String,
    desc: String,
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
        send_to_requester_channel(
            "internal error".to_string(),
            requester_id,
            server_state,
            "issue_creating_room".to_string(),
        );
        logging::console::log_failure(&format!("user({}) create room failure", requester_id));
    } else {
        let channel = publish_channel.lock().await;
        continue_with_successful_room_creation(
            room_id,
            &channel,
            public,
            server_state,
            name,
            desc,
            requester_id,
        )
        .await;
    }
}

/// Handles room deletion in the following areas
/// 1.Server state
/// 2.Database
/// 3.VoiceServer(via rabbitmq published message)
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
    server_state.owner_queues.remove(room_id);
    // remove from voice server
    let request_to_voice_server = VoiceServerDestroyRoom {
        roomId: room_id.to_string(),
    };
    let request_str =
        create_voice_server_request("destroy-room", &"-1".to_owned(), request_to_voice_server);
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str)
        .await
        .unwrap_or_default();
    logging::console::log_event(&format!("Destroyed room:{}", room_id));
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
    rabbit::publish_message(&channel, request_str)
        .await
        .unwrap_or_default();
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
) {
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId;
    let user_id: i32 = request_to_voice_server.peerId;
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    let state_room_option = server_state.rooms.get(&room_id);

    if !state_room_option.is_some() {
        return;
    };
    let state_room = state_room_option.unwrap();

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
            &request_to_voice_server.peerId.to_string(),
            request_to_voice_server,
        );
        add_user_to_room_state(&room_id, user_id, server_state);
        rabbit::publish_message(&channel, request_str)
            .await
            .unwrap_or_default();

        //make sure this user is now reflected in our queue
        //for next-in-line ownership
        insert_user_into_owner_queue(user_id, &room_id, server_state);
        logging::console::log_success(&format!("user({}) joined room({})", requester_id, room_id));

        return;
    };
    logging::console::log_failure(&format!(
        "user({}) issue joining room({})",
        requester_id, room_id
    ));
    send_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_joining_room".to_string(),
    );
}

/// Removes users from a room, this method
/// is used for both direct user triggered
/// requests and unexpected disconnections
pub async fn leave_room(
    server_state: &mut ServerState,
    requester_id: &i32,
    room_id: &i32,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    if let Some(user) = server_state.active_users.get_mut(&requester_id) {
        user.current_room_id = -1;
        let mut room = server_state.rooms.get_mut(room_id).unwrap();
        room.amount_of_users -= 1;
        room.user_ids.remove(&requester_id);
        //Always cleanup empty rooms
        //this is handled during unexpected
        //disconnects too.
        if room.amount_of_users == 0 {
            destroy_room(server_state, publish_channel, execution_handler, room_id).await;
        } else {
            select_new_owner_if_current_user_is_owner(
                requester_id,
                server_state,
                execution_handler,
                room_id.clone(),
            )
            .await;
        }
        let request_to_voice_server = VoiceServerClosePeer {
            roomId: room_id.to_string(),
            peerId: requester_id.to_string(),
            kicked: false,
        };
        let request_str: String = create_voice_server_request(
            "close-peer",
            &requester_id.to_string(),
            request_to_voice_server,
        );
        let channel = publish_channel.lock().await;
        rabbit::publish_message(&channel, request_str)
            .await
            .unwrap_or_default();
        logging::console::log_success(&format!("user({}) left room({})", requester_id, room_id));
    }
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
    let room_id: i32 = request_to_voice_server.roomId;
    let user_id: i32 = request_to_voice_server.peerId;
    let all_room_permissions: AllPermissionsResult =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;

    if all_room_permissions.0 == false {
        let requester_permissions: &RoomPermissions =
            all_room_permissions.1.get(requester_id).unwrap();
        let requestee_permissions: &RoomPermissions = all_room_permissions.1.get(&user_id).unwrap();
        let owner_and_settings =
            data_fetcher::get_room_owner_and_settings(&mut handler, &room_id).await;
        // Can the requester even add you as a speaker?
        // Did you even ask to speak?
        if requester_can_add_speaker(
            requester_permissions,
            requestee_permissions,
            &owner_and_settings.1,
            requester_id,
        ) {
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
            rabbit::publish_message(&channel, request_str)
                .await
                .unwrap_or_default();
            logging::console::log_success(&format!(
                "user({}) added user({}) as to speakers",
                requester_id, user_id
            ));
            return;
        }
    }
    logging::console::log_failure(&format!(
        "user({}) failure to add speaker({})",
        requester_id, user_id
    ));
    send_to_requester_channel(
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
    println!("{:?}", request_to_voice_server);
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId;
    let user_id: i32 = request_to_voice_server.peerId;
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
            rabbit::publish_message(&channel, request_str)
                .await
                .unwrap_or_default();

            //notify the users in the room.
            let basic_response = BasicResponse {
                response_op_code: "speaker_removed".to_owned(),
                response_containing_data: user_id.to_string(),
            };
            ws_fan::fan::broadcast_message_to_room(
                serde_json::to_string(&basic_response).unwrap(),
                server_state,
                room_id,
            )
            .await;
            logging::console::log_success(&format!(
                "user({}) removed user({}) from speaker",
                requester_id, user_id
            ));
            return;
        }
    }
    logging::console::log_failure(&format!(
        "user({}) invalid remove speaker request",
        requester_id
    ));
    send_to_requester_channel(
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
    rabbit::publish_message(&channel, request_str)
        .await
        .unwrap_or_default();
}

// A user can only raise a hand if:
// They aren't a speaker. If they do, it is a
// clear illegal request, no need to
// respond.
pub async fn raise_hand(
    server_state: &mut ServerState,
    room_id: &i32,
    requester_id: &i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut handler = execution_handler.lock().await;
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    if all_room_permissions.0 {
        return;
    }

    let current_user_permissions: &RoomPermissions =
        all_room_permissions.1.get(requester_id).unwrap();
    if current_user_permissions.is_speaker {
        return;
    }
    //no one should ever ask to speak if they are mods, because
    //the frontend will make the add speaker request on
    //the mod's behalf automatically which will be accepted
    //by the server. Mods can add themselves as a speaker.
    let new_db_permissions = permission_configs::create_non_preset(
        room_id.clone(),
        requester_id.clone(),
        true,
        false,
        current_user_permissions.is_mod,
    );
    data_capturer::capture_new_room_permissions_update(&new_db_permissions, &mut handler).await;
    let basic_response = BasicResponse {
        response_op_code: "user_asking_to_speak".to_owned(),
        response_containing_data: requester_id.to_string(),
    };
    let basic_response_str = serde_json::to_string(&basic_response).unwrap();
    fan::broadcast_message_to_room(basic_response_str, server_state, room_id.clone()).await;
    logging::console::log_success(&format!(
        "user({}) successfully raised their hand",
        requester_id
    ));
}

pub async fn lower_hand(
    server_state: &mut ServerState,
    room_id: &i32,
    requestee_id: &i32,
    requester_id: &i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut handler = execution_handler.lock().await;
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    if all_room_permissions.0 {
        return;
    }
    let requester_user_permissions: &RoomPermissions =
        all_room_permissions.1.get(requester_id).unwrap();
    let requestee_user_permissions: &RoomPermissions =
        all_room_permissions.1.get(requestee_id).unwrap();
    //A hand cannot be lowered if you are a speaker,
    //or you aren't requesting to speak.
    if requestee_user_permissions.is_speaker == true
        || requestee_user_permissions.asked_to_speak == false
    {
        return;
    }
    //- Mods can accept/lower any hand requests
    //the assumption being, there should never
    //be a situation where a person with mod permissions
    //is requesting to speak, because they will
    //automatically request to add themselves
    //and skip the ask stage.
    //
    //- Anyone can lower their own hand.
    if requester_user_permissions.is_mod || requester_id == requestee_id {
        let new_db_permissions = permission_configs::create_non_preset(
            room_id.clone(),
            requestee_id.clone(),
            false,
            false,
            requestee_user_permissions.is_mod,
        );
        data_capturer::capture_new_room_permissions_update(&new_db_permissions, &mut handler).await;
        let basic_response = BasicResponse {
            response_op_code: "user_hand_lowered".to_owned(),
            response_containing_data: requestee_id.to_string(),
        };
        let basic_response_str = serde_json::to_string(&basic_response).unwrap();
        fan::broadcast_message_to_room(basic_response_str, server_state, room_id.clone()).await;
        logging::console::log_success(&format!(
            "User({}) hand successfully lowered by user({})",
            requestee_id, requester_id
        ));
        return;
    }
    logging::console::log_failure(&format!(
        "Failed lower hand request from user({})",
        requester_id
    ));
    send_to_requester_channel(
        "issue with request".to_owned(),
        requester_id.clone(),
        server_state,
        "invalid_request".to_owned(),
    );
}

//changes the room name,desc and other data.
pub async fn update_room_meta_data(
    server_state: &mut ServerState,
    room_id: &i32,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    request_data: RoomUpdate,
) {
    let mut handler = execution_handler.lock().await;
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(room_id, &mut handler).await;
    if all_room_permissions.0 {
        return;
    }

    let requester_permissions: &RoomPermissions =
        all_room_permissions.1.get(&requester_id).unwrap();
    if requester_permissions.is_mod {
        let mut room = server_state.rooms.get_mut(&room_id).unwrap();
        room.auto_speaker = request_data.auto_speaker.clone();
        room.chat_throttle = request_data.chat_throttle.clone();
        room.public = request_data.public.clone();
        room.desc = request_data.description.clone();
        room.name = request_data.name.clone();
        //let the users know about the update
        let basic_response = BasicResponse {
            response_op_code: "room_meta_update".to_owned(),
            response_containing_data: serde_json::to_string(&request_data).unwrap(),
        };
        let basic_response_str = serde_json::to_string(&basic_response).unwrap();
        ws_fan::fan::broadcast_message_to_room(basic_response_str, server_state, room_id.clone())
            .await;
        logging::console::log_success(&format!(
            "user({}) updated room({}) metadata",
            requester_id, room_id
        ));
        return;
    }
    logging::console::log_failure(&format!(
        "Issue updating room metadata by user({})",
        requester_id
    ));
    send_to_requester_channel(
        "issue with request".to_owned(),
        requester_id.clone(),
        server_state,
        "invalid_request".to_owned(),
    );
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
    send_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_blocking_user".to_string(),
    );
}

fn add_user_to_room_state(room_id: &i32, user_id: i32, state: &mut ServerState) {
    let room = state.rooms.get_mut(&room_id).unwrap();
    room.user_ids.insert(user_id.clone());
    room.amount_of_users += 1;
    state
        .active_users
        .get_mut(&user_id)
        .unwrap()
        .current_room_id = room_id.clone();
}

fn construct_basic_room_for_state(room_id: i32, public: bool, name: String, desc: String) -> Room {
    return Room {
        room_id: room_id,
        muted: HashSet::new(),
        voice_server_id: 0.to_string(),
        /// not yet implemented(feature)
        deaf: HashSet::new(),
        user_ids: HashSet::new(),
        public: public,
        auto_speaker: true,
        amount_of_users: 0,
        name: name,
        desc: desc,
        chat_throttle: 1000,
        created_at: Utc::now().to_string(),
    };
}

/// executed after database insertion is proven to be successful.
async fn continue_with_successful_room_creation(
    room_id: i32,
    channel: &Channel,
    public: bool,
    server_state: &mut ServerState,
    name: String,
    desc: String,
    user_id: i32,
) {
    let request_to_voice_server = VoiceServerCreateRoom {
        roomId: room_id.clone().to_string(),
    };
    let new_room_state: Room = construct_basic_room_for_state(room_id.clone(), public, name, desc);
    server_state.rooms.insert(room_id, new_room_state);
    server_state.owner_queues.insert(
        room_id,
        OwnerQueue {
            user_queue: LinkedList::new(),
            room_id: room_id,
        },
    );
    let request_str =
        create_voice_server_request("create-room", &user_id.to_string(), request_to_voice_server);
    rabbit::publish_message(channel, request_str)
        .await
        .unwrap_or_default();
    logging::console::log_success(&format!(
        "user({}) successfully created room({})",
        user_id, room_id
    ));
}

async fn check_or_insert_initial_permissions(
    room: &Room,
    join_as: &str,
    requester_id: &i32,
    permissions: AllPermissionsResult,
    handler: &mut ExecutionHandler,
) -> EncounteredError {
    if permissions.0 == false {
        // if the user already has permissions
        if permissions.1.contains_key(requester_id) {
            let current_user_permissions = permissions.1.get(&requester_id).unwrap();
            // If the user is requesting to join as speaker:
            // - But isn't a speaker in the database and the room is auto speaker
            //    we accept this request because the user could have been a peer
            //    previously and now they are joining as a speaker.
            if join_as == "join-as-speaker"
                && current_user_permissions.is_speaker == false
                && room.auto_speaker == true
            {
                return false;
            }
            //if this person was in the room before and is a speaker
            if join_as == "join-as-speaker" && current_user_permissions.is_speaker == true {
                return false;
            }
            //Allow any join as peer requests
            if join_as == "join-as-new-peer" {
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
    //this is the first person in the room
    //aka the owner so they must have mod
    //permissions
    if room.user_ids.len() == 0 {
        let init_permissions =
            permission_configs::modded_speaker(room.room_id.clone(), requester_id.clone());
        let result = data_capturer::capture_new_room_permissions(&init_permissions, handler).await;
        return result;
    }

    if type_of_join == "join-as-speaker" {
        // rooms that are auto speaker
        // doesn't require hand raising
        if room.auto_speaker {
            let init_permissions =
                permission_configs::regular_speaker(room.room_id.clone(), requester_id.clone());
            let result =
                data_capturer::capture_new_room_permissions(&init_permissions, handler).await;
            return result;
        } else {
            return true; // can't join as speaker initially if auto speaker is off
        }
    } else {
        let init_permissions =
            permission_configs::regular_listener(room.room_id.clone(), requester_id.clone());
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
/// - If the person requesting is removing themselves and they aren't
///   the owner
/// If none of these conditions are met, it is an invalid request.
fn requester_can_remove_speaker(
    remover_permissions: (&i32, &RoomPermissions),
    removee_permissions: (&i32, &RoomPermissions),
    owner_id: &i32,
) -> bool {
    if remover_permissions.0 == owner_id && removee_permissions.0 != remover_permissions.0 {
        return true;
    }
    // Anyone can make themselves a listener
    if remover_permissions.0 == removee_permissions.0 && removee_permissions.1.is_speaker {
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

fn requester_can_add_speaker(
    requester_permissions: &RoomPermissions,
    requestee_permissions: &RoomPermissions,
    owner_id: &i32,
    requester_id: &i32,
) -> bool {
    if requestee_permissions.asked_to_speak && requester_permissions.is_mod
        || requester_id == owner_id
    {
        return true;
    }
    false
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

fn insert_user_into_owner_queue(user_id: i32, room_id: &i32, server_state: &mut ServerState) {
    if let Some(queue) = server_state.owner_queues.get_mut(room_id) {
        queue.insert_new_user(user_id);
    } else {
        logging::console::log_failure("Issue adding user into owner queue");
    }
}

async fn select_new_owner_if_current_user_is_owner(
    requester_id: &i32,
    server_state: &mut ServerState,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    room_id: i32,
) {
    // Update the room owner if this user that is leaving
    // is the current owner and broadcast this update
    // to the other users in the room
    let mut handler = execution_handler.lock().await;
    let need_to_update_room_owner =
        user_is_owner_of_room(requester_id.clone(), &mut handler, &room_id).await;
    if need_to_update_room_owner {
        update_room_owner_in_line(server_state, &mut handler, &room_id).await;
    }
}

/// Only used to update the owner if the original
/// owner leaves and didn't select new owner
pub async fn update_room_owner_in_line(
    server_state: &mut ServerState,
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> Option<i32> {
    if let Some(owner_queue) = server_state.owner_queues.get_mut(room_id) {
        let new_owner = owner_queue.find_new_owner(&server_state.active_users);
        if let Some(new_owner_id) = new_owner {
            data_capturer::capture_new_room_owner_update(room_id, &new_owner_id, execution_handler)
                .await;
            let response = BasicResponse {
                response_op_code: "new_owner".to_owned(),
                response_containing_data: new_owner_id.to_string(),
            };
            ws_fan::fan::broadcast_message_to_room(
                serde_json::to_string(&response).unwrap(),
                server_state,
                room_id.clone(),
            )
            .await;
            logging::console::log_success(&format!("New Owner for room:{}", room_id));
            return Some(new_owner_id);
        }
    }
    None
}

pub async fn update_room_owner(
    server_state: &mut ServerState,
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
    user_id: &i32,
) {
    if let Some(owner_queue) = server_state.owner_queues.get_mut(room_id) {
        owner_queue.insert_new_user(user_id.clone());
        data_capturer::capture_new_room_owner_update(room_id, &user_id, execution_handler).await;
        let response = BasicResponse {
            response_op_code: "new_owner".to_owned(),
            response_containing_data: user_id.to_string(),
        };
        ws_fan::fan::broadcast_message_to_room(
            serde_json::to_string(&response).unwrap(),
            server_state,
            room_id.clone(),
        )
        .await;
        logging::console::log_success(&format!("New Owner for room:{}", room_id));
    } else {
        logging::console::log_failure(&format!("New Owner for room:{}", room_id));
    }
}

pub async fn user_is_owner_of_room(
    user_id: i32,
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> bool {
    let res = data_fetcher::get_room_owner_and_settings(execution_handler, room_id).await;
    if res.0 {
        return false;
    }
    return res.1 == user_id;
}

pub fn can_block_this_user_from_room(
    permissions: HashMap<i32, RoomPermissions>,
    owner_id: i32,
    requester_id: i32,
    target_to_block: i32,
) -> bool {
    //no one can block themselves
    if requester_id != target_to_block {
        if let Some(requester_permissions) = permissions.get(&requester_id) {
            if let Some(target_permissions) = permissions.get(&target_to_block) {
                // The owner can block anyone
                if requester_id == owner_id {
                    return true;
                }
                // The requester can only block the
                // target if they(requester) is a mod
                // and the target is not a mod/the owner
                if requester_permissions.is_mod
                    && !target_permissions.is_mod
                    && target_to_block != owner_id
                {
                    return true;
                }
            }
        }
    }
    false
}
