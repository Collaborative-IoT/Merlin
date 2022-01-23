use crate::communication::communication_types::{
    BasicResponse, RoomPermissions, VoiceServerClosePeer, VoiceServerCreateRoom,
    VoiceServerDestroyRoom, VoiceServerJoinAsSpeaker,
};
use crate::communication::data_capturer::CaptureResult;
use crate::communication::{data_capturer, data_fetcher};
use crate::data_store::db_models::{DBRoom, DBRoomBlock};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rabbitmq::rabbit;
use crate::state::state::ServerState;
use crate::state::state_types::Room;
use futures::lock::Mutex;
use lapin::Channel;
use std::collections::{HashMap, HashSet};
use std::mem::drop;
use std::sync::Arc;
use warp::ws::Message;

//managing rooms happens in a pub-sub fashion
//the client waits on the response from this server
//and this server waits on the response from the
//voice server via rabbitMQ(spawned in another task)
//once this server gathers a response, it fans it
//to all involved parties(usually everyone in the room)
//For Example, kicking someone:
//1. admin requests user x to be kicked
//2. this server sends the request to the voice server
//3. once the voice server responds, if it is success
//the user is removes from the state of the server
//and this update is fanned/brodcasted across all users in the room.

pub async fn block_user_from_room(
    user_id: i32,
    room_id: i32,
    requester_id: i32,
    server_state: &Arc<Mutex<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) {
    let mut state = server_state.lock().await;

    //this room exist and the user is inside of the room
    if state.rooms.contains_key(&room_id)
        && state
            .rooms
            .get(&room_id)
            .unwrap()
            .user_ids
            .contains(&user_id)
    {
        let mut handler = execution_handler.lock().await;
        let owner_gather: (bool, i32, String) =
            data_fetcher::get_room_owner_and_settings(&mut handler, &room_id).await;

        //ensure the requester is the owner.
        //no errors were encountered gathering the owner
        if owner_gather.0 == false && owner_gather.1 == requester_id {
            //capture new block and send request to voice server
            let new_block = DBRoomBlock {
                id: -1,
                owner_room_id: room_id.clone(),
                blocked_user_id: user_id.clone(),
            };
            let capture_result =
                data_capturer::capture_new_room_block(&mut handler, &new_block).await;
            drop(handler); //don't hold guard longer than needed
            handle_user_block_capture_result(
                capture_result,
                requester_id,
                user_id.clone(),
                &mut state,
                room_id,
                publish_channel,
            )
            .await;
            return;
        }
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        &mut state,
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
    //remove from db
    let mut handler = execution_handler.lock().await;
    data_capturer::capture_room_removal(&mut handler, room_id).await;
    drop(handler);

    //remove from state
    server_state.rooms.remove(room_id);

    //remove from voice server
    let request_to_voice_server = VoiceServerDestroyRoom {
        roomId: room_id.to_string(),
    };
    let request_str: String = serde_json::to_string(&request_to_voice_server).unwrap();
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str).await;
}

pub async fn remove_user_from_room_basic(
    request_to_voice_server: VoiceServerClosePeer,
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
) {
    let request_str: String = serde_json::to_string(&request_to_voice_server).unwrap();
    let channel = publish_channel.lock().await;
    rabbit::publish_message(&channel, request_str).await;
    server_state
        .rooms
        .get_mut(&request_to_voice_server.roomId.parse().unwrap())
        .unwrap()
        .user_ids
        .remove(&request_to_voice_server.peerId.parse().unwrap());
}

pub async fn join_room_as_speaker(
    request_to_voice_server: VoiceServerJoinAsSpeaker,
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    requester_id: i32,
) {
    let mut handler = execution_handler.lock().await;
    let room_id: i32 = request_to_voice_server.roomId.parse().unwrap();
    let user_id: i32 = request_to_voice_server.peerId.parse().unwrap();
    //could be more efficient by only selecting single row
    let all_room_permissions: (bool, HashMap<i32, RoomPermissions>) =
        data_fetcher::get_room_permissions_for_users(&room_id, &mut handler).await;
    //if no errors gathering data
    if all_room_permissions.0 == false && all_room_permissions.contains_keys(&user_id){
        let current_user_permissions: &RoomPermissions =
            all_room_permissions.1.get(&user_id).unwrap();
        if current_user_permissions.is_speaker {
            let channel = publish_channel.lock().await;
            let request_str = serde_json::to_string(&request_to_voice_server).unwrap();
            rabbit::publish_message(&channel, request_str).await;
            add_user_to_room_state(&room_id, user_id, server_state);
            return;
        }
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_joining_room_as_speaker".to_string(),
    );
}

pub async fn join_room_as_peer(    
    request_to_voice_server: VoiceServerJoinAsNewPeer,
    server_state: &mut ServerState,
    publish_channel: &Arc<Mutex<lapin::Channel>>){
    
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
    }
    send_error_to_requester_channel(
        user_id.to_string(),
        requester_id,
        server_state,
        "issue_blocking_user".to_string(),
    );
}

fn send_error_to_requester_channel(
    response_data: String,
    requester_id: i32,
    server_state: &mut ServerState,
    op_code: String,
) {
    let response = BasicResponse {
        response_op_code: op_code,
        response_containing_data: response_data,
    };
    //TODO:handle error
    server_state
        .peer_map
        .get(&requester_id)
        .unwrap()
        .send(Message::text(serde_json::to_string(&response).unwrap()));
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
        voice_server_id: 0.to_string(), //not yet implemented(feature)
        deaf: HashSet::new(),
        user_ids: HashSet::new(),
        public: public,
        auto_speaker: false,
    };
}

//executed after database insertion is proven to be successful
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
