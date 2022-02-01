/*
Handles all of the communication op_code_routing
to the intended functionality
*/
use crate::communication::communication_handler;
use crate::communication::communication_types::BasicRequest;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use serde_json::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn route_msg(
    msg: String,
    user_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let basic_request: BasicRequest = serde_json::from_str(&msg)?;

    //route the request
    //We could use the basic_request op code for checking
    //different requests like add/remove user inside of the method
    //instead of using a different parameter, but this way it is
    //cleaner and opcodes are abstracted away from function implementation
    match basic_request.request_op_code.as_str() {
        "create_room" => {
            communication_handler::create_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
            )
            .await
        }
        "add_speaker" => {
            communication_handler::add_or_remove_speaker(
                basic_request,
                publish_channel,
                user_id,
                server_state,
                execution_handler,
                "add",
            )
            .await
        }
        "remove_speaker" => {
            communication_handler::add_or_remove_speaker(
                basic_request,
                publish_channel,
                user_id,
                server_state,
                execution_handler,
                "remove",
            )
            .await
        }
        "block_user_from_room" => {
            communication_handler::block_user_from_room(
                basic_request,
                user_id,
                server_state,
                execution_handler,
                publish_channel,
            )
            .await
        }
        "get_followers" => {
            communication_handler::get_followers_or_following_list(
                basic_request,
                execution_handler,
                server_state,
                user_id,
                "followers",
            )
            .await
        }
        "get_following" => {
            communication_handler::get_followers_or_following_list(
                basic_request,
                execution_handler,
                server_state,
                user_id,
                "following",
            )
            .await
        }
        "join_room_as_speaker" => {
            communication_handler::join_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
                "speaker",
            )
            .await
        }
        "join_room_as_listener" => {
            communication_handler::join_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
                "listener",
            )
            .await
        }
        "get_top_rooms" => {
            Ok(
                communication_handler::get_top_rooms(server_state, user_id, execution_handler)
                    .await,
            )
        }
        "raise_hand" => {
            communication_handler::raise_hand_or_lower_hand(
                basic_request,
                server_state,
                user_id,
                execution_handler,
                "raise",
            )
            .await
        }
        "lower_hand" => {
            communication_handler::raise_hand_or_lower_hand(
                basic_request,
                server_state,
                user_id,
                execution_handler,
                "lower",
            )
            .await
        }
    }
}
