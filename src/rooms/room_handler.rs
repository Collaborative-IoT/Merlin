use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use crate::communication::data_capturer;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::communication::communication_types::{UserRemovedFromRoom};


//managing rooms happens in a pub-sub fashion
//the client waits on the response from this server
//and this server waits on the response from the
//voice server via rabbitMQ(spawned in another task)
//once this server gathers a response, it fans it
//to all involved parties(usually everyone in he room)
//For Example, kicking someone:
//1. admin requests user x to be kicked
//2. this server sends the request to the voice server
//3. once the voice server responds, if it is success
//the user is removes from the state of the server
//and this update is fanned/brodcasted across all users in the room.

pub async fn remove_user_from_room(
    user_id:i32, 
    requester_id:i32, 
    server_state:&Arc<Mutex<ServerState>>,
    execution_handler:&Arc<Mutex<ServerState>>
    ){
        
}