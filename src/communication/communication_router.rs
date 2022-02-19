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

use super::data_fetcher;

pub async fn route_msg(
    msg: String,
    user_id: i32,
    server_state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) -> Result<()> {
    let basic_request: BasicRequest = serde_json::from_str(&msg)?;

    //Route the request
    //We could use the basic_request op code for checking
    //different requests like add/remove user inside of the method
    //instead of using a different parameter, but this way it is
    //cleaner and opcodes are abstracted away from function implementation.
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

        "@connect-transport" | "@send-track" | "@get-recv-tracks" => {
            communication_handler::handle_web_rtc_request(
                basic_request,
                publish_channel,
                server_state,
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
        "join-as-speaker" => {
            communication_handler::join_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
                "join-as-speaker",
            )
            .await
        }
        "join-as-new-peer" => {
            communication_handler::join_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
                "join-as-new-peer",
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
        "gather_all_users_in_room" => {
            communication_handler::gather_all_users_in_room(
                basic_request,
                server_state,
                user_id,
                execution_handler,
            )
            .await
        }
        "follow_user" | "unfollow_user" => {
            communication_handler::follow_or_unfollow_user(
                basic_request,
                execution_handler,
                user_id,
                server_state,
            )
            .await
        }
        "block_user" | "unblock_user" => {
            communication_handler::block_or_unblock_user_from_user(
                basic_request,
                server_state,
                user_id,
                execution_handler,
            )
            .await
        }
        "leave_room" => {
            communication_handler::leave_room(
                basic_request,
                publish_channel,
                server_state,
                execution_handler,
                user_id,
            )
            .await
        }

        "update_room_meta" => {
            communication_handler::change_room_metadata(
                basic_request,
                server_state,
                user_id,
                execution_handler,
            )
            .await
        }
        "update_deaf_and_mute" => {
            communication_handler::update_mute_and_deaf_status(basic_request, server_state, user_id)
                .await
        }
        "all_room_permissions" => Ok(communication_handler::get_room_permissions_for_users(
            server_state,
            user_id,
            execution_handler,
        )
        .await),
        _ => Ok(()),
    }
}
