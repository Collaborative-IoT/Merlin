/*
abstracts usage of the sql execution handler 
by fetching and converts rows to correct response types.
*/

use tokio_postgres::{row::Row};
use crate::communication::communication_types{
    User
};
use crate::data_store::{sql_execution_handler::ExecutionHandler};
use crate::state::state::ServerState;

pub async fn get_users(
    requester_user_id:i32,
    server_state:state::state::ServerState
    )->(bool,Vec<User>){
    let mut encountered_error = false;
    let mut users:Vec<User> = Vec::new();
    let user_ids:Vec<String> = server_state.users.values().cloned().collect();
    for user in user_ids{
        let user_results = handler.select_user_by_id(&user_id).await;
        let user_blocks = handler.select_all_blocked_for_user(&user_id).await;
    }
    return (encountered_error,users);
}

pub async fn get_all_blocked_user_ids_for_user(){
    
}