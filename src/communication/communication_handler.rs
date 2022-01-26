use crate::common::common_error_logic::send_error_to_requester_channel;
use crate::communication::communication_types::{
    BasicRequest, BlockUserFromRoom, GenericRoomIdAndPeerId,
};
use crate::communication::data_fetcher;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rooms;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use serde_json::Result;
use std::collections::HashSet;
use std::mem::drop;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    server_state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
    public: bool,
) {
    let read_state = server_state.read().await;
    //make sure the user exist and they aren't in a room
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
            public,
        )
        .await;
        return;
    }
    //if the request is invalid
    drop(read_state);
    let mut write_state = server_state.write().await;
    send_error_to_requester_channel(
        "issue with request".to_owned(),
        requester_id,
        &mut write_state,
        "invalid_request".to_owned(),
    );
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

    //Make sure this room actually exists
    if read_state.rooms.contains_key(&request_data.room_id) {
        let room = read_state.rooms.get(&request_data.room_id).unwrap();

        //Make sure both users are in the room
        //The owner checking happens in the room handler
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
    drop(read_state);
    let mut write_state = server_state.write().await;
    send_error_to_requester_channel(
        "issue with request".to_owned(),
        requester_id,
        &mut write_state,
        "invalid_request".to_owned(),
    );
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

    //Return if the request data isn't parse-able
    let room_id_parse_res = request_data.roomId.parse();
    let user_id_parse_res = request_data.peerId.parse();
    if !room_id_parse_res.is_ok() || !user_id_parse_res.is_ok() {
        return Ok(());
    }
    let room_id: i32 = room_id_parse_res.unwrap();
    let user_id: i32 = user_id_parse_res.unwrap();
    //Ensure the room exist,the user isn't already in a room and this room is public
    if read_state.rooms.contains_key(&room_id)
        && read_state
            .active_users
            .get(&user_id)
            .unwrap()
            .current_room_id
            == -1
        && read_state.rooms.get(&room_id).unwrap().public
    {
        //make sure the user isn't blocked from the room
        let mut handler = execution_handler.lock().await;
        let blocked_result: (bool, HashSet<i32>) =
            data_fetcher::get_blocked_user_ids_for_room(&mut handler, &room_id).await;
        // Nothing went wrong gathering blocked user ids
        // and user isn't blocked
        if blocked_result.0 == false && !blocked_result.1.contains(&user_id) {
            drop(read_state);
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
    drop(read_state);
    let mut write_state = server_state.write().await;
    send_error_to_requester_channel(
        "issue with request".to_owned(),
        requester_id,
        &mut write_state,
        "invalid_request".to_owned(),
    );
    return Ok(());
}
