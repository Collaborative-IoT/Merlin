use crate::communication::communication_router;
use crate::communication::communication_types::GenericRoomIdAndPeerId;
use crate::communication::tests::comm_handler_test_helpers::helpers;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use lapin::Consumer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn mods_can_remove_speaker(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("remove_speaker".to_string(), data.clone());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        data,
        "34".to_owned(),
        "remove-speaker".to_owned(),
    )
    .await;
}

pub async fn non_mods_can_not_bring_up_speakers(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    //joins room 3 as a listener
    let mut mock_user_rx = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        37,
        consume_channel,
    )
    .await;
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("add_speaker".to_owned(), data.clone());
    communication_router::route_msg(request, 37, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(&mut mock_user_rx, "issue_adding_speaker", "34")
        .await;
}

pub async fn mods_can_bring_up_speakers(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    //TESTCASE - Mods can bring up speakers

    //our actual test
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("add_speaker".to_owned(), data.clone());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        data,
        "34".to_string(),
        "add-speaker".to_string(),
    )
    .await;
}

pub async fn non_mods_can_not_remove_speaker(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    let mut mock_user_rx = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        38,
        consume_channel,
    )
    .await;
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("remove_speaker".to_string(), data);
    communication_router::route_msg(request, 38, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(&mut mock_user_rx, "issue_removing_speaker", "34")
        .await;
}

pub async fn non_mods_can_not_lower_hands(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    // TESTCASE - NON MODS CAN'T LOWER HANDS
    // Make sure a non-mod user can't lower another user's hand
    // Make another user join as a listenr
    // and try to lower user 34's hand who is requesting.

    //joins room 3 as a listener
    let mut mock_temp_user_rx_two = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        36,
        consume_channel,
    )
    .await;
    //try to lower 34's hand(non mod) as 36(non mod)
    let raise_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        36,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx_two,
        "invalid_request",
        "issue with request",
    )
    .await;
}

pub async fn mods_can_lower_hands(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - MODS CAN LOWER HANDS
    // Make sure the room owner can lower the hand of 34 ,
    // the speaker rx is user num 33 aka the owner
    let lower_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        lower_hand_message,
        33,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(speaker_rx, "user_hand_lowered", "34").await;
    helpers::clear_message_that_was_fanned(vec![listener_rx]).await;
}
