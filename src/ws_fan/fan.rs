use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use warp::ws::Message;

pub async fn broadcast_message_to_all_active_users(
    new_msg: String,
    server_state: &Arc<Mutex<ServerState>>,
) {
    for (&uid, tx) in server_state.lock().await.peer_map.iter() {
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            //user disconnection is handled in another task
        }
    }
}

pub async fn broadcast_message_to_room(
    new_msg: String,
    server_state: &Arc<Mutex<ServerState>>,
    room_id: i32,
) {
}
