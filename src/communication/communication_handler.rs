use crate::common::common_response_logic::send_to_requester_channel;
use crate::communication::communication_handler_helpers;
use crate::communication::communication_types::{
    AllUsersInRoomResponse, BasicRequest, BasicRoomCreation, BlockUserFromRoom, CommunicationRoom,
    GenericRoomId, GenericRoomIdAndPeerId, GetFollowList, User, UserPreview,
};
use crate::communication::data_fetcher;
use crate::data_store::db_models::DBFollower;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rooms;
use crate::state::state::ServerState;
use crate::state::state_types::Room;
use futures::lock::Mutex;
use serde_json::Result;
use std::collections::{HashMap, HashSet};
use std::mem::drop;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::communication_types::GenericUserId;
use super::data_capturer::{self, CaptureResult};

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
    let read_state = server_state.read().await;
    //Make sure the user exist and they aren't in a room
    if read_state.active_users.contains_key(&requester_id)
        && read_state
            .active_users
            .get(&requester_id)
            .unwrap()
            .current_room_id
            == -1
    {
        drop(read_state);
        let mut write_state = server_state.write().await;
        rooms::room_handler::create_room(
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
    // If the request is invalid
    send_error_response_to_requester(read_state, requester_id, server_state).await;
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
    let read_state = server_state.read().await;

    // Make sure this room actually exists
    if read_state.rooms.contains_key(&request_data.room_id) {
        let room = read_state.rooms.get(&request_data.room_id).unwrap();

        // Make sure both users are in the room
        // The owner checking happens in the room handler
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&request_data.user_id) {
            drop(read_state);
            let mut write_state = server_state.write().await;
            rooms::room_handler::block_user_from_room(
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
    send_error_response_to_requester(read_state, requester_id, server_state).await;
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
    let read_state = server_state.read().await;
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;

    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;
    //Ensure the room exist,the user isn't already in a room and this room is public
    if read_state.rooms.contains_key(&room_id)
        && read_state
            .active_users
            .get(&peer_id)
            .unwrap()
            .current_room_id
            == -1
        && read_state.rooms.get(&room_id).unwrap().public
        && peer_id == requester_id
    {
        //make sure the user isn't blocked from the room
        let mut handler = execution_handler.lock().await;
        let blocked_result: (bool, HashSet<i32>) =
            data_fetcher::get_blocked_user_ids_for_room(&mut handler, &room_id).await;
        // Nothing went wrong gathering blocked user ids
        // and user isn't blocked
        if blocked_result.0 == false && !blocked_result.1.contains(&peer_id) {
            drop(read_state);
            drop(handler);
            let mut write_state = server_state.write().await;
            rooms::room_handler::join_room(
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
    send_error_response_to_requester(read_state, requester_id, server_state).await;
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
    let read_state = server_state.read().await;
    //ensure request parsing is successful
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;
    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;

    // Make sure the room being requested exists
    if read_state.rooms.contains_key(&room_id) {
        let room = read_state.rooms.get(&room_id).unwrap();

        // Make sure the requester and requestee is in the
        // room that is being requested
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&peer_id) {
            drop(read_state);
            let mut write_state = server_state.write().await;
            if add_or_remove == "add" {
                rooms::room_handler::add_speaker(
                    request_data,
                    publish_channel,
                    &requester_id,
                    &mut write_state,
                    execution_handler,
                )
                .await;
            } else {
                rooms::room_handler::remove_speaker(
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
    send_error_response_to_requester(read_state, requester_id, server_state).await;
    return Ok(());
}

pub async fn handle_web_rtc_request(
    request: BasicRequest,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
) -> Result<()> {
    let request_data: serde_json::Value = serde_json::from_str(&request.request_containing_data)?;
    let read_state = server_state.read().await;

    if communication_handler_helpers::web_rtc_request_is_valid(
        &read_state,
        &request_data,
        &requester_id,
    ) {
        rooms::room_handler::handle_web_rtc_specific_requests(
            request_data,
            publish_channel,
            &request.request_op_code,
        )
        .await;
        return Ok(());
    }
    send_error_response_to_requester(read_state, requester_id, server_state).await;
    return Ok(());
}

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
        return Ok(());
    }
    drop(write_state);
    drop(handler);
    send_error_response_to_requester(server_state.read().await, requester_id, server_state).await;
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
    communication_handler_helpers::send_follow_list(
        target,
        server_state,
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
    let read_state = server_state.read().await;
    let mut all_rooms: Vec<&Room> = read_state.rooms.values().into_iter().collect();
    all_rooms.sort_by_key(|room| room.amount_of_users);
    let mut handler = execution_handler.lock().await;
    let mut communication_rooms: Vec<CommunicationRoom> = Vec::new();
    for room in all_rooms {
        let all_room_user_ids: Vec<i32> = room.user_ids.iter().cloned().collect();
        //(encountered_error) is the first of the tuple values
        let previews: (bool, HashMap<i32, UserPreview>) =
            data_fetcher::get_user_previews_for_users(all_room_user_ids, &mut handler).await;

        let owner_data_and_chat_mode: (bool, i32, String) =
            data_fetcher::get_room_owner_and_settings(&mut handler, &room.room_id).await;

        //if encountered errors getting data needed
        if previews.0 || owner_data_and_chat_mode.0 {
            continue;
        }

        communication_handler_helpers::construct_communication_room(
            previews.1,
            room,
            &mut communication_rooms,
            owner_data_and_chat_mode.1,
            owner_data_and_chat_mode.2,
        );
    }
    //clean up old mutexes and send the response
    drop(handler);
    drop(read_state);
    let response_containing_data = serde_json::to_string(&communication_rooms).unwrap();
    let mut write_state = server_state.write().await;
    send_to_requester_channel(
        response_containing_data,
        requester_id,
        &mut write_state,
        "top_rooms".to_owned(),
    );
}

pub async fn raise_hand_or_lower_hand(
    request: BasicRequest,
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    type_of_hand_action: &str,
) -> Result<()> {
    let read_state = server_state.read().await;
    let request_data: GenericRoomIdAndPeerId =
        serde_json::from_str(&request.request_containing_data)?;
    let room_id: i32 = request_data.roomId;
    let peer_id: i32 = request_data.peerId;

    //you can only raise your own hand
    if type_of_hand_action == "raise" && requester_id != peer_id {
        return Ok(());
    }

    //room exist
    if read_state.rooms.contains_key(&room_id) {
        let room = read_state.rooms.get(&room_id).unwrap();
        //both users are in this room
        if room.user_ids.contains(&requester_id) && room.user_ids.contains(&peer_id) {
            drop(read_state);
            let mut write_state = server_state.write().await;
            if type_of_hand_action == "lower" {
                rooms::room_handler::lower_hand(
                    &mut write_state,
                    &room_id,
                    &peer_id,
                    &requester_id,
                    execution_handler,
                )
                .await;
            } else {
                rooms::room_handler::raise_hand(
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
    send_error_response_to_requester(read_state, requester_id, server_state).await;
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
    let read_state = server_state.read().await;
    //if the room exist
    if read_state.rooms.contains_key(&room_id) {
        let room = read_state.rooms.get(&room_id).unwrap();
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
            //remove old lock for state and get users
            drop(read_state);
            let mut write_state = server_state.write().await;
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
    send_error_response_to_requester(read_state, requester_id, server_state).await;
    return Ok(());
}

async fn send_error_response_to_requester(
    read_state: tokio::sync::RwLockReadGuard<'_, ServerState>,
    requester_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
) {
    drop(read_state);
    let mut write_state = server_state.write().await;
    send_to_requester_channel(
        "issue with request".to_owned(),
        requester_id,
        &mut write_state,
        "invalid_request".to_owned(),
    );
}
