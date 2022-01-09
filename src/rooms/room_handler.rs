use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use crate::communication::{data_capturer, data_fetcher};
use crate::data_store::db_models::DBRoomBlock;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::communication::communication_types::{UserRemovedFromRoom};
use crate::communication::data_capturer::CaptureResult;
use crate::rabbitmq::rabbit;
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
    user_id:i32, 
    room_id:i32,
    requester_id:i32, 
    server_state:&Arc<Mutex<ServerState>>,
    execution_handler:&Arc<Mutex<ExecutionHandler>>
    ){
        let mut state = server_state.lock().await;
        let mut handler = execution_handler.lock().await;

        //this room exist and the user is inside of the room
        if state.rooms.contains_key(&room_id) && state.rooms.user_ids.contains(&user_id){
            let owner_gather:(bool, i32, String) = data_fetcher::get_room_owner_and_settings(&mut handler, &room_id).await;

            //ensure the requester is the owner.
            //no errors were encountered gathering the owner
            if owner_gather.0 == false && owner_gather.1 == requester_id{
                //capture new block and send request to voice server
                let new_block = DBRoomBlock{
                    id:-1,
                    owner_room_id: room_id.clone(),
                    blocked_user_id: user_id.clone()
                };
                let capture_result = data_capturer::capture_new_room_block(&mut handler, &new_block).await;
                handle_user_block_capture_result(capture_result, requester_id, user_id).await;
            }
            else{
                //send error
            }
        }
}

pub async fn handle_user_block_capture_result(capture_result:CaptureResult, requester_id:i32, user_id:i32){
    if capture_result.encountered_error == false{
        let remove_request = UserRemovedFromRoom{
            user_id:user_id,
            type_of_ban:"user",
            requester:requester_id
        };
        let remove_request_str:String = remove_request.to_string();
        rabbit::publish_message_to_voice_server(remove_request_str).await;
    }
    else{
        //let requester know there was an issue
    }
}