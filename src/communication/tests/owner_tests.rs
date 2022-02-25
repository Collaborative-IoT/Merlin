use crate::communication::router;
use crate::communication::tests::helpers::helpers;
use crate::communication::types::{BlockUserFromRoom, VoiceServerClosePeer};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use lapin::Consumer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn owner_can_block_from_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    //clear room and add new real user
    //(see begining of file for real vs mock users)
    helpers::clear_all_users_except_owner(state).await;
    let new_real_user_id = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "wipo3102ondwidsnm9o2w".to_string(),
        "3ork23-9kjwefm29".to_string(),
    )
    .await
    .0;

    let data = serde_json::to_string(&BlockUserFromRoom {
        user_id: new_real_user_id.clone(),
        room_id: 3,
    })
    .unwrap();
    let request = helpers::basic_request("block_user_from_room".to_string(), data.clone());
    router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    //check result
    helpers::grab_and_assert_message_to_voice_server::<VoiceServerClosePeer>(
        consume_channel,
        helpers::generic_close_peer(new_real_user_id, 3),
        new_real_user_id.to_string(),
        "close-peer".to_owned(),
    )
    .await;

    //make sure the user we just blocked is no longer in the room state
    assert!(
        state
            .write()
            .await
            .rooms
            .get_mut(&3)
            .unwrap()
            .user_ids
            .contains(&new_real_user_id)
            == false
    );
}

pub async fn non_owner_can_not_block_from_room(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
) {
    //User 34 is not the owner, so this should fail
    let data = serde_json::to_string(&BlockUserFromRoom {
        user_id: 38,
        room_id: 3,
    })
    .unwrap();
    let request = helpers::basic_request("block_user_from_room".to_string(), data);
    router::route_msg(request, 34, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "issue_blocking_user", "38").await;
}
