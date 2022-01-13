use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use warp::ws::Message;

//this notifies all of periodic server updates
//like someone leaving/joining a room
pub async fn broadcast_update(new_msg: String, server_state: &Arc<Mutex<ServerState>>) {
    for (&uid, tx) in server_state.lock().await.peer_map.iter() {
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            //user disconnection is handled in another task
        }
    }
}
