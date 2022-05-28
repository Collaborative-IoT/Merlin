use crate::common::response_logic::send_to_requester_channel;
use crate::communication::data_fetcher;
use crate::communication::helpers;
use crate::communication::types::{
    AllUsersInRoomResponse, BasicRequest, BasicRoomCreation, BlockUserFromRoom, CommunicationRoom,
    GenericRoomId, GenericRoomIdAndPeerId, GetFollowList, User, UserPreview,
};
use crate::data_store::db_models::{DBFollower, DBUserBlock};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::integration::types::DisconnectMsg;
use crate::integration::types::GeneralMessage;
use crate::integration::types::HOIActionDataIncoming;
use crate::integration::types::HOIActionDataOutgoing;
use crate::integration::types::HouseOfIoTCredentials;
use crate::logging;
use crate::rabbitmq::rabbit;
use crate::rooms::handler::EncounteredError;
use crate::state::state::ServerState;
use crate::state::types::Room;
use crate::{rooms, ws_fan};
use futures::lock::Mutex;
use serde_json::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::mem::drop;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::data_capturer::{self, CaptureResult};
use super::types::ExistingIotServer;
use super::types::GiveOrRevokeIot;
use super::types::InitRoomData;
use super::types::JoinTypeInfo;
use super::types::LooseUserPreviewRequest;
use super::types::NewModStatus;
use super::types::RoomDetails;
use super::types::SingleUserDataResults;
use super::types::SingleUserPermissionResults;
use super::types::UserProfileEdit;
use super::types::{
    BasicResponse, DeafAndMuteStatus, DeafAndMuteStatusUpdate, GenericUserId, RoomUpdate,
};

/*
Handles all functionality that has to be carried out by communication and
handles repetitive pre-checks.

For example:
    before a user makes a request to join a room, are they banned?
    before a user makes a request to add a speaker, is the speaker in the room?

Small checks like this are pre-checks that usually are no brainers and
aren't included in the core logic of different modules.
*/

pub async fn create_room(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: BasicRoomCreation = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    //Make sure the user exist and they aren't in a room
    if let Some(user) = write_state.active_users.get(&requester_id) {
        if user.current_room_id == -1 {
            rooms::handler::create_room(
                &mut write_state,
                publish_channel,
                execution_handler,
                requester_id,
                request_data.name,
                request_data.desc,
                request_data.public,
            )
            .await;
            return Ok(());
        }
    }
    // If the request is invalid
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn block_user_from_room(
    request: BasicRequest,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) -> Result<()> {
    let request_data: BlockUserFromRoom = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;

    // Make sure this room actually exists
    if let Some(room) = write_state.rooms.get(&request_data.room_id) {
        // Make sure both users are in the room
        // The owner checking happens in the room handler
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&request_data.user_id) {
            rooms::handler::block_user_from_room(
                request_data.user_id,
                request_data.room_id,
                requester_id,
                &mut write_state,
                execution_handler,
                publish_channel,
            )
            .await;
            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn join_room(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    type_of_join: &str,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;

    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;
    //Ensure the room exist,the user isn't already in a room and this room is public
    if room_is_joinable(&write_state, &peer_id, &requester_id, &room_id) {
        //make sure the user isn't blocked from the room
        let mut handler = execution_handler.lock().await;
        let blocked_result: (bool, HashSet<i32>) =
            data_fetcher::get_blocked_user_ids_for_room(&mut handler, &room_id).await;
        // Nothing went wrong gathering blocked user ids
        // and user isn't blocked
        if blocked_result.0 == false && !blocked_result.1.contains(&peer_id) {
            drop(handler);
            rooms::handler::join_room(
                request_data,
                &mut write_state,
                publish_channel,
                execution_handler,
                requester_id,
                type_of_join,
            )
            .await;
            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn add_or_remove_speaker(
    request: BasicRequest,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    add_or_remove: &str,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    //ensure request parsing is successful
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;
    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;

    // Make sure the room being requested exists
    if let Some(room) = write_state.rooms.get(&room_id) {
        // Make sure the requester and requestee is in the
        // room that is being requested
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&peer_id) {
            if add_or_remove == "add" {
                rooms::handler::add_speaker(
                    request_data,
                    publish_channel,
                    &requester_id,
                    &mut write_state,
                    execution_handler,
                )
                .await;
            } else {
                rooms::handler::remove_speaker(
                    request_data,
                    publish_channel,
                    &requester_id,
                    &mut write_state,
                    execution_handler,
                )
                .await;
            }
            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn handle_web_rtc_request(
    request: BasicRequest,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: serde_json::Value = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;

    if helpers::web_rtc_request_is_valid(&write_state, &request_data, &requester_id) {
        rooms::handler::handle_web_rtc_specific_requests(
            request_data,
            publish_channel,
            &request.request_op_code,
        )
        .await;
        return Ok(());
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

#[allow(unused_assignments)]
pub async fn follow_or_unfollow_user(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let user_id_data: GenericUserId = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let mut result: Option<CaptureResult> = None;
    let mut response_op: Option<String> = None;
    if request.request_op_code == "follow_user" {
        let db_follower = DBFollower {
            id: -1,
            follower_id: requester_id,
            user_id: user_id_data.user_id,
        };
        result = Some(data_capturer::capture_new_follower(&mut handler, &db_follower).await);
        response_op = Some("user_follow_successful".to_owned());
        //no errors
    } else {
        //unfollow
        result = Some(
            data_capturer::capture_follower_removal(
                &mut handler,
                &requester_id,
                &user_id_data.user_id,
            )
            .await,
        );
        response_op = Some("user_unfollow_successful".to_owned());
    }
    //if we didn't get any error from capture execution(saving to db)
    if result.is_some() && !result.unwrap().encountered_error {
        send_to_requester_channel(
            user_id_data.user_id.to_string(),
            requester_id,
            &mut write_state,
            response_op.unwrap().to_owned(),
        );
        logging::console::log_success(&format!(
            "User({}) successfully followed/unfollowed user({})",
            requester_id, user_id_data.user_id
        ));
        return Ok(());
    }
    logging::console::log_failure(&format!("User({}) follow/unfollow failure", requester_id));
    drop(handler);
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn get_followers_or_following_list(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    type_of_request: &str,
) -> Result<()> {
    //gather all
    let mut handler = execution_handler.lock().await;
    let target;
    let request_data: GetFollowList = serde_json::from_str(&request.request_containing_data)?;

    if type_of_request == "followers" {
        //(encountered_error, user_ids)
        target =
            data_fetcher::get_follower_user_ids_for_user(&mut handler, &request_data.user_id).await;
    } else {
        target = data_fetcher::get_following_user_ids_for_user(&mut handler, &request_data.user_id)
            .await;
    }
    helpers::send_follow_list(
        target,
        server_state,
        &mut handler,
        requester_id,
        request_data.user_id,
    )
    .await;
    return Ok(());
}

// Currently top rooms are rooms with the most people.
// In the future, top rooms will be user driven and
// will need to be limited with pagination techniques.
pub async fn get_top_rooms(
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut write_state = server_state.write().await;
    let mut all_rooms: Vec<&Room> = write_state.rooms.values().into_iter().collect();
    all_rooms.sort_by_key(|room| room.amount_of_users);
    let mut handler = execution_handler.lock().await;
    let mut communication_rooms: Vec<CommunicationRoom> = Vec::new();
    for room in all_rooms {
        let all_room_user_ids: Vec<i32> = room.user_ids.iter().cloned().collect();
        let previews: (EncounteredError, HashMap<i32, UserPreview>) =
            data_fetcher::get_user_previews_for_users(all_room_user_ids, &mut handler).await;

        let owner_data_and_chat_mode: (bool, i32, String) =
            data_fetcher::get_room_owner_and_settings(&mut handler, &room.room_id).await;

        //if encountered errors getting data needed
        if previews.0 || owner_data_and_chat_mode.0 {
            continue;
        }

        helpers::construct_communication_room(
            previews.1,
            room,
            &mut communication_rooms,
            owner_data_and_chat_mode.1,
            owner_data_and_chat_mode.2,
        );
    }
    //clean up old mutexes and send the response
    drop(handler);
    let response_containing_data = serde_json::to_string(&communication_rooms).unwrap();

    send_to_requester_channel(
        response_containing_data,
        requester_id,
        &mut write_state,
        "top_rooms".to_owned(),
    );
}

pub async fn get_initial_room_data(
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    request_data: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let room_id: GenericRoomId = serde_json::from_str(&request_data.request_containing_data)?;

    if let Some(room) = write_state.rooms.get(&room_id.room_id) {
        let owner_data_and_chat_mode: (EncounteredError, i32, String) =
            data_fetcher::get_room_owner_and_settings(&mut handler, &room.room_id).await;

        //if encountered errors getting data needed
        if owner_data_and_chat_mode.0 {
            send_error_response_to_requester(requester_id.clone(), &mut write_state);
            return Ok(());
        }
        let init_data = InitRoomData {
            details: RoomDetails {
                name: room.name.clone(),
                chat_throttle: room.chat_throttle.clone(),
                is_private: !room.public,
                description: room.desc.clone(),
            },
            creator_id: owner_data_and_chat_mode.1,
            auto_speaker_setting: room.auto_speaker.clone(),
            created_at: room.created_at.clone(),
            chat_mode: owner_data_and_chat_mode.2,
        };
        //clean up old mutexes and send the response
        drop(handler);
        let response_containing_data = serde_json::to_string(&init_data).unwrap();

        send_to_requester_channel(
            response_containing_data,
            requester_id,
            &mut write_state,
            "initial_room_data".to_owned(),
        );
    }
    send_error_response_to_requester(requester_id.clone(), &mut write_state);
    Ok(())
}

pub async fn leave_room(
    request: BasicRequest,
    voice_server_publish_channel: &Arc<Mutex<lapin::Channel>>,
    integration_publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: GenericRoomId = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    //the user is in this room
    if let Some(user) = write_state.active_users.get(&requester_id) {
        if user.current_room_id == request_data.room_id {
            rooms::handler::leave_room(
                &mut write_state,
                &requester_id,
                &request_data.room_id,
                voice_server_publish_channel,
                integration_publish_channel,
                execution_handler,
            )
            .await;

            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn raise_hand_or_lower_hand(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    type_of_hand_action: &str,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;
    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;

    //you can only raise your own hand
    if type_of_hand_action == "raise" && requester_id != peer_id {
        return Ok(());
    }

    //room exist
    if let Some(room) = write_state.rooms.get(&room_id) {
        //both users are in this room
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&peer_id) {
            if type_of_hand_action == "lower" {
                rooms::handler::lower_hand(
                    &mut write_state,
                    &room_id,
                    &peer_id,
                    &requester_id,
                    execution_handler,
                )
                .await;
            } else {
                rooms::handler::raise_hand(
                    &mut write_state,
                    &room_id,
                    &requester_id,
                    execution_handler,
                )
                .await;
            }
            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

#[allow(unused_assignments)]
pub async fn block_or_unblock_user_from_user(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let request_data: GenericUserId = serde_json::from_str(&request.request_containing_data)?;
    //no user can block or unblock themselves
    if request_data.user_id == requester_id {
        return Ok(());
    }
    let mut capture_result: Option<CaptureResult> = None;
    let mut response_op: Option<String> = None;
    let mut handler = execution_handler.lock().await;
    if request.request_op_code == "block_user" {
        let user_block = DBUserBlock {
            id: -1,
            owner_user_id: requester_id.clone(),
            blocked_user_id: request_data.user_id.clone(),
        };
        capture_result =
            Some(data_capturer::capture_new_user_block(&mut handler, &user_block).await);
        response_op = Some("user_personally_blocked".to_owned());
    } else {
        capture_result = Some(
            data_capturer::capture_user_block_removal(
                &mut handler,
                &requester_id,
                &request_data.user_id,
            )
            .await,
        );
        response_op = Some("user_personally_unblocked".to_owned());
    }

    if capture_result.is_some() && !capture_result.unwrap().encountered_error {
        send_to_requester_channel(
            request_data.user_id.to_string(),
            requester_id,
            &mut write_state,
            response_op.unwrap().to_owned(),
        );
        return Ok(());
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

// Gathering all users in a specific room
// doesn't require you to be in that room,
// any public room can be queried by outside
// users even banned ones. The banned ones
// can't join, but they can query it.
pub async fn gather_all_users_in_room(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let room_id_obj: GenericRoomId = serde_json::from_str(&request.request_containing_data)?;
    let room_id = room_id_obj.room_id;
    let mut write_state = server_state.write().await;
    //if the room exist
    if let Some(room) = write_state.rooms.get(&room_id) {
        let all_room_user_ids: Vec<i32> = room
            .user_ids
            .iter()
            .filter(|x| x != &&requester_id)
            .cloned()
            .collect();
        let mut handler = execution_handler.lock().await;
        let users: (bool, Vec<User>) =
            data_fetcher::get_users_for_user(requester_id.clone(), all_room_user_ids, &mut handler)
                .await;
        //no error was encountered
        if users.0 == false {
            //generate response with all users and send
            let response = AllUsersInRoomResponse {
                room_id: room_id,
                users: users.1,
            };
            let response_str = serde_json::to_string(&response).unwrap();
            send_to_requester_channel(
                response_str,
                requester_id.clone(),
                &mut write_state,
                "all_users_for_room".to_owned(),
            );
            return Ok(());
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn gather_single_user(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let user_id_obj: GenericUserId = serde_json::from_str(&request.request_containing_data)?;
    let user_id = user_id_obj.user_id;
    let mut users: (bool, Vec<User>) =
        data_fetcher::get_users_for_user(requester_id.clone(), vec![user_id.clone()], &mut handler)
            .await;
    //no error was encountered and this user exists
    if users.0 == false && users.1.len() == 1 {
        let response = SingleUserDataResults {
            user_id: user_id,
            data: users.1.remove(0),
        };
        let response_str = serde_json::to_string(&response).unwrap();
        send_to_requester_channel(
            response_str,
            requester_id.clone(),
            &mut write_state,
            "single_user_data".to_owned(),
        );
        return Ok(());
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn gather_single_user_permission(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let data_obj: GenericRoomIdAndPeerId = serde_json::from_str(&request.request_containing_data)?;
    let permissions_res =
        data_fetcher::get_single_user_permissions(&data_obj.roomId, &data_obj.peerId, &mut handler)
            .await;
    if let Some(permissions) = permissions_res {
        let response = SingleUserPermissionResults {
            user_id: data_obj.peerId,
            data: permissions,
        };
        let response_str = serde_json::to_string(&response).unwrap();
        send_to_requester_channel(
            response_str,
            requester_id.clone(),
            &mut write_state,
            "single_user_permissions".to_owned(),
        );
        return Ok(());
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn update_entire_user(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let data_obj: UserProfileEdit = serde_json::from_str(&request.request_containing_data)?;
    let result = data_capturer::capture_user_update(&mut handler, &requester_id, data_obj).await;
    if !result.encountered_error {
        // If this user exists and is in a room
        // we should notify the room that he is in
        // that this user has updated their own data.
        if let Some(user) = write_state.active_users.get_mut(&requester_id) {
            if user.current_room_id != -1 {
                let response = BasicResponse {
                    response_op_code: "user_info_updated".to_owned(),
                    response_containing_data: requester_id.to_string(),
                };
                let room_id = user.current_room_id.clone();
                drop(user);
                ws_fan::fan::broadcast_message_to_room_excluding_user(
                    serde_json::to_string(&response).unwrap(),
                    &mut write_state,
                    room_id,
                    requester_id.clone(),
                )
                .await;
            }
        }
        send_to_requester_channel(
            "".to_owned(),
            requester_id,
            &mut write_state,
            "profile_updated".to_owned(),
        );

        return Ok(());
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

/// Gives mod to someone in the room
pub async fn change_user_mod_status(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let data_obj: NewModStatus = serde_json::from_str(&request.request_containing_data)?;
    if let Some(user) = write_state.active_users.get(&requester_id) {
        let user_current_room = user.current_room_id.clone();
        // We will need the server state again
        // have to drop to comply with borrow rules
        drop(user);
        // Is this user even in a room?
        if user_current_room != -1 {
            let owner_and_settings =
                data_fetcher::get_room_owner_and_settings(&mut handler, &user_current_room).await;
            // 1. We haven't encountered an error gathering from db
            // 2. Are we the owner of the room?
            // Only the owner of the room can
            // appoint mods
            if owner_and_settings.1 == requester_id && !owner_and_settings.0 {
                let result = data_capturer::mod_or_unmod_user_capture(
                    data_obj.new_status,
                    &user_current_room,
                    &data_obj.user_id,
                    &mut handler,
                )
                .await;
                if !result.encountered_error {
                    let basic_response = BasicResponse {
                        response_op_code: type_of_mod_op(data_obj.new_status.clone()),
                        response_containing_data: data_obj.user_id.to_string(),
                    };
                    println!("{:?}", basic_response);
                    ws_fan::fan::broadcast_message_to_room(
                        serde_json::to_string(&basic_response).unwrap(),
                        &mut write_state,
                        user_current_room,
                    )
                    .await;
                    logging::console::log_success("Mod status changed");
                } else {
                    send_error_response_to_requester(requester_id, &mut write_state);
                    logging::console::log_failure("Issue with changing mod status");
                }
            }
        }
    }
    Ok(())
}

pub async fn change_room_metadata(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let room_update: RoomUpdate = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    let user = write_state.active_users.get(&requester_id).unwrap();
    let user_room_id = user.current_room_id.clone();
    //never go through with requests when the user isn't
    //in a room
    if user_room_id != -1 {
        rooms::handler::update_room_meta_data(
            &mut write_state,
            &user_room_id,
            requester_id,
            execution_handler,
            room_update,
        )
        .await;
        return Ok(());
    }

    send_error_response_to_requester(requester_id, &mut write_state);
    return Ok(());
}

pub async fn update_mute_and_deaf_status(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let mute_and_deaf: DeafAndMuteStatus = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    if let Some(user) = write_state.active_users.get_mut(&requester_id) {
        //you can only update your muted/deaf status if you aren't in a room
        if user.current_room_id != -1 {
            user.deaf = mute_and_deaf.deaf.clone();
            user.muted = mute_and_deaf.muted.clone();
            let user_room_id = user.current_room_id.clone();
            //send everyone the deaf/mute update
            let deaf_mute_response = DeafAndMuteStatusUpdate {
                deaf: mute_and_deaf.deaf,
                muted: mute_and_deaf.muted,
                user_id: requester_id,
            };
            let basic_response = BasicResponse {
                response_op_code: "user_mute_and_deaf_update".to_owned(),
                response_containing_data: serde_json::to_string(&deaf_mute_response).unwrap(),
            };
            let basic_response_str = serde_json::to_string(&basic_response).unwrap();
            ws_fan::fan::broadcast_message_to_room(
                basic_response_str,
                &mut write_state,
                user_room_id,
            )
            .await;
            return Ok(());
        }
        send_error_response_to_requester(requester_id, &mut write_state);
    }
    return Ok(());
}

// Used to just get user previews loosely,
// normally user previews are included with
// a "CommunicationRoom". Some functionality
// on the clients require previews without
// rooms, like the "people" list on the
// dash board.
pub async fn gather_previews(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let request_data: LooseUserPreviewRequest =
        serde_json::from_str(&request.request_containing_data)?;
    let result =
        data_fetcher::get_user_previews_for_users(request_data.user_ids, &mut handler).await;
    send_to_requester_channel(
        serde_json::to_string(&result.1).unwrap(),
        requester_id,
        &mut write_state,
        "user_previews".to_owned(),
    );
    Ok(())
}

// Used for when a user first authenticates
// when you first authenticate you need your
// own information.
pub async fn gather_base_user(
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    server_state: &Arc<RwLock<ServerState>>,
) {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let user_information = data_fetcher::gather_base_user(&mut handler, &requester_id).await;
    send_to_requester_channel(
        serde_json::to_string(&user_information).unwrap(),
        requester_id,
        &mut write_state,
        "your_data".to_owned(),
    );
}

/// Needed to let the frontend know
/// what type of successful join
/// request options there are.
pub async fn gather_type_of_room_join(
    request: BasicRequest,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    let request_data: GenericRoomId = serde_json::from_str(&request.request_containing_data)?;

    if write_state.active_users.contains_key(&requester_id) {
        let room_permissions =
            data_fetcher::get_room_permissions_for_users(&request_data.room_id, &mut handler).await;
        if !room_permissions.0 {
            //IN FUTURE: We may restrict listeners for rooms
            //that is why is_listner is added here
            let mut response_data = JoinTypeInfo {
                as_speaker: false,
                as_listener: true,
                room_id: request_data.room_id,
            };
            if let Some(permissions) = room_permissions.1.get(&requester_id) {
                response_data.as_speaker = permissions.is_speaker;
            }
            send_to_requester_channel(
                serde_json::to_string(&response_data).unwrap(),
                requester_id,
                &mut write_state,
                "join_type_info".to_owned(),
            );
        }
    }
    Ok(())
}

pub async fn send_chat_message(
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    message: String,
) -> Result<()> {
    //we need to inject the message with the
    //requester id so the frontend can know
    //who sent the message
    let mut message: Value = serde_json::from_str(&message)?;
    message["userId"] = Value::String(requester_id.to_string());
    let new_message = serde_json::to_string(&message)?;

    //broadcast to the room
    let mut write_state = server_state.write().await;
    if let Some(user) = write_state.active_users.get_mut(&requester_id) {
        let user_room_id = user.current_room_id.clone();
        if user_room_id != -1 {
            let basic_response = BasicResponse {
                response_op_code: "new_chat_message".to_owned(),
                response_containing_data: new_message,
            };
            let basic_response_str = serde_json::to_string(&basic_response).unwrap();
            ws_fan::fan::broadcast_message_to_room(
                basic_response_str,
                &mut write_state,
                user_room_id,
            )
            .await;
            return Ok(());
        }
        send_error_response_to_requester(requester_id, &mut write_state);
    }
    Ok(())
}

pub async fn normal_invalid_request(server_state: &Arc<RwLock<ServerState>>, requester_id: i32) {
    let mut state = server_state.write().await;
    send_error_response_to_requester(requester_id, &mut state);
}

pub async fn get_room_permissions_for_users(
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    if let Some(user) = write_state.active_users.get(&requester_id) {
        let result =
            data_fetcher::get_room_permissions_for_users(&user.current_room_id, &mut handler).await;
        drop(handler);
        send_to_requester_channel(
            serde_json::to_string(&result.1).unwrap(),
            requester_id,
            &mut write_state,
            "room_permissions".to_owned(),
        );
    }
}

pub async fn give_owner(
    request: BasicRequest,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) -> Result<()> {
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    // Is this user in this room to even give them ownership?
    if let Some(room) = write_state.rooms.get(&request_data.roomId) {
        if room.user_ids.contains(&request_data.peerId) {
            let is_owner = rooms::handler::user_is_owner_of_room(
                requester_id,
                &mut handler,
                &request_data.roomId,
            )
            .await;
            // Is the requester the owner? only the owner can reassign ownership
            if is_owner {
                rooms::handler::update_room_owner(
                    &mut write_state,
                    &mut handler,
                    &request_data.roomId,
                    &request_data.peerId,
                )
                .await;
                return Ok(());
            }
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn create_hoi_connection(
    request: BasicRequest,
    integration_publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: HouseOfIoTCredentials =
        serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    let mut handler = execution_handler.lock().await;
    if let Some(user) = write_state.active_users.get(&requester_id) {
        //ensure our user is actually in a room
        if user.current_room_id != -1 {
            let room_permissions =
                data_fetcher::get_room_permissions_for_users(&user.current_room_id, &mut handler)
                    .await;
            if let Some(permissions) = room_permissions.1.get(&requester_id) {
                // Only mods can actually make the request to connect to a server
                // and users can only make this request on behalf of themselves.
                if permissions.is_mod && request_data.user_id == requester_id {
                    let channel = integration_publish_channel.lock().await;
                    let new_general_msg = GeneralMessage {
                        category: "connect_hoi".to_owned(),
                        data: serde_json::to_string(&request_data).unwrap(),
                        server_id: "-1".to_owned(),
                    };
                    rabbit::publish_integration_message(
                        &channel,
                        serde_json::to_string(&new_general_msg).unwrap(),
                    )
                    .await
                    .unwrap_or_default();
                    return Ok(());
                }
            }
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn remove_hoi_connection(
    request: BasicRequest,
    integration_publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: DisconnectMsg = serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    let remove_result = remove_hoi_connection_directly(
        request_data.server_id,
        integration_publish_channel,
        &mut write_state,
        requester_id,
    )
    .await;

    if let Ok(res) = remove_result {
        if res == false {
            send_error_response_to_requester(requester_id, &mut write_state);
        }
    } else {
        send_error_response_to_requester(requester_id, &mut write_state);
    }

    Ok(())
}

pub async fn remove_hoi_connection_directly(
    server_id: String,
    integration_publish_channel: &Arc<Mutex<lapin::Channel>>,
    write_state: &mut ServerState,
    requester_id: i32,
) -> Result<bool> {
    if let Some(user) = write_state.active_users.get(&requester_id) {
        let current_room_id = user.current_room_id.clone();
        drop(user);
        if let Some(room) = write_state.rooms.get_mut(&current_room_id) {
            if let Some(board) = room.iot_server_connections.get(&server_id) {
                // only the owner of each board can remove the connection
                // a board is essentially an IoT server connection and
                // those who hold permissions for it etc.
                if requester_id == board.owner_user_id {
                    let channel = integration_publish_channel.lock().await;
                    let new_general_msg = GeneralMessage {
                        category: "disconnect_hoi".to_owned(),
                        data: String::new(),
                        server_id: server_id.clone(),
                    };
                    rabbit::publish_integration_message(
                        &channel,
                        serde_json::to_string(&new_general_msg).unwrap(),
                    )
                    .await
                    .unwrap_or_default();
                    // let the room know that this server has been
                    // removed.
                    ws_fan::fan::broadcast_message_to_room(
                        serde_json::to_string(&BasicResponse {
                            response_op_code: "hoi_server_disconnected".to_owned(),
                            response_containing_data: server_id,
                        })
                        .unwrap(),
                        write_state,
                        current_room_id,
                    )
                    .await;
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub async fn request_hoi_action(
    request: BasicRequest,
    integration_publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: HOIActionDataIncoming =
        serde_json::from_str(&request.request_containing_data)?;
    let mut write_state = server_state.write().await;
    if let Some(user) = write_state.active_users.get(&requester_id) {
        //ensure our user is actually in a room
        if let Some(room) = write_state.rooms.get(&user.current_room_id) {
            // Only people with permission can make requests
            if let Some(board) = room.iot_server_connections.get(&request_data.server_id) {
                if board.users_with_permission.contains(&requester_id)
                    || board.owner_user_id == requester_id
                {
                    logging::console::log_event(&format!(
                        "Executing HOI Action:{:?}",
                        request_data
                    ));
                    let channel = integration_publish_channel.lock().await;
                    let new_general_msg = GeneralMessage {
                        category: "action_hoi".to_owned(),
                        data: serde_json::to_string(&HOIActionDataOutgoing {
                            bot_name: request_data.bot_name,
                            action: request_data.action,
                        })
                        .unwrap(),
                        server_id: request_data.server_id,
                    };
                    rabbit::publish_integration_message(
                        &channel,
                        serde_json::to_string(&new_general_msg).unwrap(),
                    )
                    .await
                    .unwrap_or_default();

                    return Ok(());
                }
            }
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

/// Give a users permission to control an iot server
/// if that user is in your current room.
pub async fn give_or_revoke_iot_permission(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let mut write_state = server_state.write().await;
    let request_data: GiveOrRevokeIot = serde_json::from_str(&request.request_containing_data)?;
    // Can't give yourself permission since you must be the owner
    // in the beginning. You already have permission.
    if request_data.user_id != requester_id {
        if let Some(user) = write_state.active_users.get(&requester_id) {
            let current_room_id = user.current_room_id.clone();
            drop(user);
            if let Some(room) = write_state.rooms.get_mut(&current_room_id) {
                if let Some(board) = room
                    .iot_server_connections
                    .get_mut(&request_data.external_id)
                {
                    if board.owner_user_id == requester_id {
                        let mut outgoing_op_code = "removed_hoi_controller";
                        if request_data.now_has_permission {
                            board.users_with_permission.insert(request_data.user_id);
                            outgoing_op_code = "new_hoi_controller";
                        } else {
                            board.users_with_permission.remove(&request_data.user_id);
                        }
                        //Let the room know this user has been removed
                        //from controlling this board
                        ws_fan::fan::broadcast_message_to_room(
                            serde_json::to_string(&BasicResponse {
                                response_op_code: outgoing_op_code.to_owned(),
                                response_containing_data: request_data.user_id.to_string(),
                            })
                            .unwrap(),
                            &mut write_state,
                            current_room_id,
                        )
                        .await;
                        return Ok(());
                    }
                }
            }
        }
    }
    send_error_response_to_requester(requester_id, &mut write_state);
    Ok(())
}

pub async fn get_passive_data_snapshot(server_state: &Arc<RwLock<ServerState>>, requester_id: i32) {
    let mut write_state = server_state.write().await;
    if let Some(user) = write_state.active_users.get(&requester_id) {
        let current_room_id = user.current_room_id;
        if let Some(room) = write_state.rooms.get(&current_room_id) {
            let mut all_existing: Vec<ExistingIotServer> = Vec::new();
            for server in room.iot_server_connections.values() {
                let mut existing = ExistingIotServer {
                    owner_id: server.owner_user_id.clone(),
                    external_id: server.external_server_id.clone(),
                    controllers_of_room: Vec::new(),
                    passive_data_snap_shot: server.passive_data_snapshot.clone(),
                    outside_name: server.outside_name.clone(),
                };
                for controller in server.users_with_permission.iter() {
                    existing.controllers_of_room.push(controller.clone());
                }
                all_existing.push(existing);
            }
            send_to_requester_channel(
                serde_json::to_string(&all_existing).unwrap(),
                requester_id,
                &mut write_state,
                "existing_iot_data".to_owned(),
            );
        }
    }
}

pub fn room_is_joinable(
    read_state: &ServerState,
    peer_id: &i32,
    requester_id: &i32,
    room_id: &i32,
) -> bool {
    if let Some(user) = read_state.active_users.get(peer_id) {
        if let Some(room) = read_state.rooms.get(room_id) {
            if user.current_room_id == -1 && room.public && peer_id == requester_id {
                return true;
            }
        }
    }
    return false;
}

fn type_of_mod_op(mod_status: bool) -> String {
    if mod_status == true {
        return "new_mod".to_owned();
    } else {
        return "removed_mod".to_owned();
    }
}

fn send_error_response_to_requester(requester_id: i32, write_state: &mut ServerState) {
    send_to_requester_channel(
        "issue with request".to_owned(),
        requester_id,
        write_state,
        "invalid_request".to_owned(),
    );
}
