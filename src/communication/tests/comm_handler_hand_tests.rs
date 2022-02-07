use crate::communication::communication_router;
use crate::communication::tests::comm_handler_test_helpers::helpers;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn users_in_room_as_listener_can_raise(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - USERS IN THE ROOM AS A LISTENER CAN REQUEST TO SPEAK
    // Make sure hand raising works for users in the room as listeners.
    // From previous tests, we know user number 34 is a listener.
    // The listener rx is user num 34.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;
    helpers::clear_message_that_was_fanned(vec![speaker_rx]).await;
}

pub async fn users_can_lower_their_own_hand(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - USERS CAN LOWER THEIR OWN HAND
    // Make sure user who was declined to speak can request again
    // and lower their own hand.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;

    let lower_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        lower_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_hand_lowered", "34").await;
    helpers::clear_message_that_was_fanned(vec![speaker_rx]).await;
}

pub async fn users_not_in_room_cannot_make_requests(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    // TESTCASE - USERS NOT IN THE ROOM CAN'T MAKE LOWER/RAISE REQUESTS
    // Make sure no user not in the room
    // can make a raise hand request.
    // We create a new user that has no current room
    // and make the request.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 35),
    );
    let mut mock_temp_user_rx =
        helpers::create_and_add_new_user_channel_to_peer_map(35, state).await;
    communication_router::route_msg(
        raise_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;

    //make sure no one not in the room can
    //make a lower hand request
    let raise_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 35),
    );
    communication_router::route_msg(
        raise_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;
}
