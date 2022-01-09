use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use crate::communication::data_capturer;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::communication::communication_types::{UserRemovedFromRoom};

pub async fn remove_user_from_user(
    user_id:i32, 
    requester_id:i32, 
    server_state:&Arc<Mutex<ServerState>>,
    execution_handler:&Arc<Mutex<ServerState>>
    ){
        
}