use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use warp::ws::Message;

pub async fn broadcast_message_to_all_active_users(
    new_msg: String,
    server_state: &Arc<Mutex<ServerState>>,
) {
    for (&_uid, tx) in server_state.lock().await.peer_map.iter() {
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            //user disconnection is handled in another task
        }
    }
}

pub async fn broadcast_message_to_room(
    new_msg: String,
    server_state: &mut ServerState,
    room_id: i32,
) {
    if let Some(room) = server_state.rooms.get(&room_id) {
        let room_users: Vec<&i32> = room.user_ids.iter().collect();
        for id in room_users {
            let user_websocket_channel = server_state.peer_map.get(id).unwrap();
            user_websocket_channel
                .send(Message::text(new_msg.clone()))
                .unwrap_or_else(|e| eprintln!("issue sending msg:{}", e));
        }
    } else {
        println!("Room removed before message was send: {}", new_msg);
    }
}

pub async fn broadcast_message_to_room_excluding_user(
    new_msg: String,
    server_state: &mut ServerState,
    room_id: i32,
    user_id: i32,
) {
    if let Some(room) = server_state.rooms.get(&room_id) {
        let room_users: Vec<&i32> = room.user_ids.iter().filter(|x| x != &&user_id).collect();

        for id in room_users {
            let user_websocket_channel = server_state.peer_map.get(id).unwrap();
            user_websocket_channel
                .send(Message::text(new_msg.clone()))
                .unwrap_or_else(|e| eprintln!("issue sending msg:{}", e));
        }
    } else {
        println!("Room removed before message was send: {}", new_msg);
    }
}

pub async fn broadcast_message_to_single_user(
    new_msg: String,
    server_state: &mut ServerState,
    user_id: &i32,
) {
    if let Some(user_websocket_channel) = server_state.peer_map.get(user_id) {
        user_websocket_channel
            .send(Message::text(new_msg.clone()))
            .unwrap_or_else(|e| eprintln!("issue sending msg:{}", e));
    }
}
