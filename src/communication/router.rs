/*
Handles all of the communication op_code_routing
to the intended functionality
*/
use crate::communication::handler;
use crate::communication::types::BasicRequest;
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
    println!("{}", basic_request.request_op_code);
    println!("{}", basic_request.request_containing_data);
    //Route the request
    //We could use the basic_request op code for checking
    //different requests like add/remove user inside of the method
    //instead of using a different parameter, but this way it is
    //cleaner and opcodes are abstracted away from function implementation.
    match basic_request.request_op_code.as_str() {
        "create_room" => {
            handler::create_room(
                basic_request,
                server_state,
                publish_channel,
                execution_handler,
                user_id,
            )
            .await
        }
        "@connect-transport" | "@send-track" | "@get-recv-tracks" => {
            handler::handle_web_rtc_request(basic_request, publish_channel, server_state, user_id)
                .await
        }
        "add_speaker" => {
            handler::add_or_remove_speaker(
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
            handler::add_or_remove_speaker(
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
            handler::block_user_from_room(
                basic_request,
                user_id,
                server_state,
                execution_handler,
                publish_channel,
            )
            .await
        }
        "get_followers" => {
            handler::get_followers_or_following_list(
                basic_request,
                execution_handler,
                server_state,
                user_id,
                "followers",
            )
            .await
        }
        "get_following" => {
            handler::get_followers_or_following_list(
                basic_request,
                execution_handler,
                server_state,
                user_id,
                "following",
            )
            .await
        }
        "join-as-speaker" => {
            handler::join_room(
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
            handler::join_room(
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
            Ok(handler::get_top_rooms(server_state, user_id, execution_handler).await)
        }
        "raise_hand" => {
            handler::raise_hand_or_lower_hand(
                basic_request,
                server_state,
                user_id,
                execution_handler,
                "raise",
            )
            .await
        }
        "lower_hand" => {
            handler::raise_hand_or_lower_hand(
                basic_request,
                server_state,
                user_id,
                execution_handler,
                "lower",
            )
            .await
        }
        "gather_all_users_in_room" => {
            handler::gather_all_users_in_room(
                basic_request,
                server_state,
                user_id,
                execution_handler,
            )
            .await
        }
        "follow_user" | "unfollow_user" => {
            handler::follow_or_unfollow_user(
                basic_request,
                execution_handler,
                user_id,
                server_state,
            )
            .await
        }
        "block_user" | "unblock_user" => {
            handler::block_or_unblock_user_from_user(
                basic_request,
                server_state,
                user_id,
                execution_handler,
            )
            .await
        }
        "leave_room" => {
            handler::leave_room(
                basic_request,
                publish_channel,
                server_state,
                execution_handler,
                user_id,
            )
            .await
        }
        "initial_room_data" => {
            handler::get_initial_room_data(server_state, user_id, basic_request, execution_handler)
                .await
        }

        "update_room_meta" => {
            handler::change_room_metadata(basic_request, server_state, user_id, execution_handler)
                .await
        }
        "update_deaf_and_mute" => {
            handler::update_mute_and_deaf_status(basic_request, server_state, user_id).await
        }
        "all_room_permissions" => {
            Ok(
                handler::get_room_permissions_for_users(server_state, user_id, execution_handler)
                    .await,
            )
        }
        "user_previews" => {
            handler::gather_previews(basic_request, server_state, user_id, execution_handler).await
        }
        "send_chat_msg" => {
            handler::send_chat_message(server_state, user_id, basic_request.request_containing_data)
                .await
        }
        "join_type" => {
            handler::gather_type_of_room_join(
                basic_request,
                user_id,
                execution_handler,
                server_state,
            )
            .await
        }
        "my_data" => Ok(handler::gather_base_user(user_id, execution_handler, server_state).await),
        "single_user_data" => {
            handler::gather_single_user(basic_request, execution_handler, user_id, server_state)
                .await
        }
        "single_user_permissions" => {
            handler::gather_single_user_permission(
                basic_request,
                execution_handler,
                user_id,
                server_state,
            )
            .await
        }

        _ => Ok(handler::normal_invalid_request(server_state, user_id).await),
    }
}
