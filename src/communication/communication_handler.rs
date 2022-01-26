use crate::common::common_error_logic::send_error_to_requester_channel;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
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
    let state = server_state.read().await;
    //make sure the user exist and they aren't in a room
    if state.active_users.contains_key(&requester_id)
        && state
            .active_users
            .get(&requester_id)
            .unwrap()
            .current_room_id
            == -1
    {
        return;
    }
    drop(state);
    let mut new_state = server_state.write().await;
    send_error_to_requester_channel(
        "issue".to_owned(),
        requester_id,
        &mut new_state,
        "issue_with_request".to_owned(),
    );
}
