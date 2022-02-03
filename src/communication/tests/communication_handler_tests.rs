/*
The communication handler tests aims at
testing the logic that the handler enforces.

The testing is done through usage of the communication
router, which is responsible for routing requests to
the handler.

It doesn't test its consuming modules like the
execution handler, it makes sure
requests fail under certain circumstances,
state is being modified and messages are being
persisted to the voice server via RabbitMq.

This test isn't fully integration based, so we manually
grab the messages intended for the voice server after
publish and assert them.

*/
use crate::communication::communication_types::{
    BasicRequest, BasicResponse, BasicRoomCreation, GenericRoomIdAndPeerId, VoiceServerCreateRoom,
    VoiceServerRequest,
};
use crate::communication::{communication_router, data_capturer};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rabbitmq::rabbit;
use crate::rooms::permission_configs;
use crate::server::setup_execution_handler;
use crate::state::state::ServerState;
use crate::state::state_types::{Room, User};
use chrono::Utc;
use futures::lock::Mutex;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryFutureExt};
use lapin::{options::*, types::FieldTable, Channel, Connection, Consumer};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn tests() {
    //setup rabbit channels
    let connection = rabbit::setup_rabbit_connection().await.unwrap();
    let publish_channel: Arc<Mutex<lapin::Channel>> =
        Arc::new(Mutex::new(setup_channel(&connection).await));
    let consume_channel: lapin::Channel = setup_channel(&connection).await;
    let mut consumer = consume_channel
        .basic_consume(
            "voice_server_consume",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    //setup mock state/execution handler
    let mock_state: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState::new()));
    let execution_handler: Arc<Mutex<ExecutionHandler>> =
        Arc::new(Mutex::new(setup_execution_handler().await.unwrap()));
    //Setup mock inner user channels
    //
    //We use channels to direct messages
    //to tasks, after the message is
    //gathered by the task it is
    //forwarded to the user via
    //websocket connection.
    //
    //So we will exclude the forwarding portion
    //and act as though this channel belongs
    //to a real user.
    let mut rx_user_one = create_and_add_new_user_channel_to_peer_map(33, &mock_state).await;
    let mut rx_user_two = create_and_add_new_user_channel_to_peer_map(34, &mock_state).await;
    insert_starting_user_state(&mock_state).await;
    test_creating_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
    )
    .await;
    //speaker
    test_joining_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
        "join-as-speaker",
        33,
    )
    .await;
    //listener
    test_joining_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut &mut rx_user_two,
        "join-as-new-peer",
        34,
    )
    .await;
    test_raising_and_lowering_hand(
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
        &mut &mut rx_user_two,
    )
    .await;
}

async fn test_creating_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
) {
    // Make sure users cannot create a room if they
    // are currently in a room.
    //
    // Each user has a current room id, which helps us
    // not have to search all rooms to find a user which
    // is inefficent. If a user has a room value of -1,
    // they are not in a room, but if they have a non-negative
    // room number they are in a room. This is handled by the
    // communication handler internally.

    // Set the mock user's room as 2(even though room 2
    // doesn't exist).
    //
    // This should make the request to
    // create a room fail.
    let create_room_msg = basic_request("create_room".to_owned(), basic_room_creation());
    send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        2,
    )
    .await;
    // Check that user is getting an error response
    // to their task channel.
    grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request").await;

    // Set the user's room state back to -1, signifying
    // that they aren't in a room, which means they
    // can successfully create a room
    send_create_or_join_room_request(
        state,
        create_room_msg,
        publish_channel,
        execution_handler,
        -1,
    )
    .await;
    // The second attempt for room creation should be successful,
    // resulting in a new room in state and a message to the voice
    // server via RabbitMQ. So, we can check these side effects.
    grab_and_assert_message_to_voice_server::<VoiceServerCreateRoom>(
        consume_channel,
        basic_room_creation(),
        "33".to_owned(),
        "create_room".to_owned(),
    )
    .await;

    //Check the server state after the successful creations etc.
    let server_state = state.read().await;
    assert_eq!(server_state.rooms.len(), 1);
    assert_eq!(server_state.rooms.contains_key(&1), true);
}

async fn test_joining_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
    type_of_join: &str,
    user_id: i32,
) {
    // Based on the previous test we know
    // the room id->1 exists.

    // Set user to a fake room to test illegal requests,
    // no user can join a room if they are already in a room.
    let create_room_msg = basic_request(type_of_join.to_owned(), basic_join_as(user_id.clone()));
    send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        2,
    )
    .await;
    // Check the channel and make sure there is an error.
    grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request").await;
    // The second attempt should pass due to the room being changed to -1
    // which means the user isn't in a room.
    send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        -1,
    )
    .await;
    // Ensure that we published the correct message
    grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        basic_join_as(user_id),
        user_id.to_string(),
        type_of_join.to_owned(),
    )
    .await;

    //Check:The user in the room?
    //Check:There only one user in the room?
    //Check:The user's current room state is updated?
    let server_state = state.read().await;
    assert_eq!(server_state.rooms.get(&1).unwrap().user_ids.len(), 1);
    assert_eq!(
        server_state
            .rooms
            .get(&user_id)
            .unwrap()
            .user_ids
            .contains(&user_id),
        true
    );
    assert_eq!(
        server_state
            .active_users
            .get(&user_id)
            .unwrap()
            .current_room_id,
        1
    );
}

// Raising/lowering your
// hand causes no interaction
// with the voice server so we
// just check the broadcasted
// messages.
async fn test_raising_and_lowering_hand(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
) {
    // Make sure no user not in the room
    // can make a raise hand request.
    // We create a new user that has no current room
    // and make the request.
    let raise_hand_message =
        basic_request("raise_hand".to_owned(), basic_hand_raise_or_lower(1, 35));
    let mut mock_temp_user_rx = create_and_add_new_user_channel_to_peer_map(35, state).await;
    communication_router::route_msg(
        raise_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;

    // Make sure hand raising works for users in the room as listeners.
    // From previous tests, we know user number 34 is a listener.
    // The listener rx is user num 34.
    let raise_hand_message =
        basic_request("raise_hand".to_owned(), basic_hand_raise_or_lower(1, 34));
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;
    clear_message_that_was_fanned(vec![speaker_rx]).await;

    // Make sure no user not in the room
    // can make a lower hand request.
    let lower_hand_message =
        basic_request("lower_hand".to_owned(), basic_hand_raise_or_lower(1, 34));
    let mut mock_temp_user_rx = create_and_add_new_user_channel_to_peer_map(35, state).await;
    communication_router::route_msg(
        lower_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;

    // Make sure the room owner can lower the hand, the speaker rx is user num 33
    let lower_hand_message =
        basic_request("lower_hand".to_owned(), basic_hand_raise_or_lower(1, 33));
    communication_router::route_msg(
        lower_hand_message,
        33,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(speaker_rx, "user_hand_lowered", "34").await;
    clear_message_that_was_fanned(vec![listener_rx]).await;

    // Make sure user who was declined to speak can request again
    // and lower their own hand.
    let raise_hand_message =
        basic_request("raise_hand".to_owned(), basic_hand_raise_or_lower(1, 34));
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;

    let lower_hand_message =
        basic_request("lower_hand".to_owned(), basic_hand_raise_or_lower(1, 33));
    communication_router::route_msg(
        lower_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    grab_and_assert_request_response(listener_rx, "user_hand_lowered", "34").await;
    clear_message_that_was_fanned(vec![speaker_rx]).await;
}

//All users must be present in memory before operation
async fn insert_starting_user_state(server_state: &Arc<RwLock<ServerState>>) {
    let mut state = server_state.write().await;
    let user_one = User {
        last_online: Utc::now(),
        muted: true,
        deaf: true,
        ip: "test".to_string(),
        current_room_id: -1,
    };

    let user_two = User {
        last_online: Utc::now(),
        muted: true,
        deaf: false,
        ip: "test".to_string(),
        current_room_id: -1,
    };
    state.active_users.insert(33, user_one);
    state.active_users.insert(34, user_two);
}

//starts rabbitmq connection channel
async fn setup_channel(conn: &Connection) -> Channel {
    let publish_channel = conn.create_channel().await.unwrap();
    publish_channel
        .queue_declare(
            "voice_server_consume",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    return publish_channel;
}

async fn consume_message(consumer: &mut Consumer) -> String {
    let delivery = consumer.next().await.unwrap().unwrap().1;
    delivery.ack(BasicAckOptions::default()).await.expect("ack");
    let parsed_msg = rabbit::parse_message(delivery);
    return parsed_msg;
}

async fn create_and_add_new_user_channel_to_peer_map(
    mock_id: i32,
    mock_state: &Arc<RwLock<ServerState>>,
) -> UnboundedReceiverStream<Message> {
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    //add initial peer state to state
    //we will use th
    mock_state.write().await.peer_map.insert(mock_id, tx);
    return rx;
}

async fn grab_and_assert_request_response(
    rx: &mut UnboundedReceiverStream<Message>,
    op_code: &str,
    containing_data: &str,
) {
    let message = rx.next().await.unwrap().to_str().unwrap().to_owned();
    let parsed_json: BasicResponse = serde_json::from_str(&message).unwrap();
    assert_eq!(parsed_json.response_op_code, op_code);
    assert_eq!(parsed_json.response_containing_data, containing_data);
}

async fn grab_and_assert_message_to_voice_server<T: serde::de::DeserializeOwned + Serialize>(
    consume_channel: &mut Consumer,
    d: String,
    uid: String,
    op: String,
) {
    let message = consume_message(consume_channel).await;
    let vs_message: VoiceServerRequest<T> = serde_json::from_str(&message).unwrap();
    assert_eq!(serde_json::to_string(&vs_message.d).unwrap(), d);
    assert_eq!(op, vs_message.op);
    assert_eq!(uid, vs_message.uid);
}

fn basic_request(op: String, data: String) -> String {
    let request = BasicRequest {
        request_op_code: op,
        request_containing_data: data,
    };
    return serde_json::to_string(&request).unwrap();
}

fn basic_hand_raise_or_lower(room_id: i32, peer_id: i32) -> String {
    let raise_or_lower = GenericRoomIdAndPeerId {
        roomId: room_id,
        peerId: peer_id,
    };
    return serde_json::to_string(&raise_or_lower).unwrap();
}

fn basic_room_creation() -> String {
    let room_creation = BasicRoomCreation {
        name: "test".to_owned(),
        desc: "test".to_owned(),
        public: true,
    };
    return serde_json::to_string(&room_creation).unwrap();
}
fn basic_join_as(user_id: i32) -> String {
    let join = GenericRoomIdAndPeerId {
        roomId: 1,
        peerId: user_id,
    };
    return serde_json::to_string(&join).unwrap();
}

async fn send_create_or_join_room_request(
    state: &Arc<RwLock<ServerState>>,
    msg: String,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    curr_room: i32,
) {
    let mut server_state = state.write().await;
    server_state
        .active_users
        .get_mut(&33)
        .unwrap()
        .current_room_id = curr_room;
    drop(server_state);
    communication_router::route_msg(msg, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
}

//This is used to clear the messages that get fanned
// to other mock users in a room for our tests.
//
//The way we do our tests, requires the user's
//channel to be completely clear.
async fn clear_message_that_was_fanned(rxs: Vec<&mut UnboundedReceiverStream<Message>>) {
    for rx in rxs {
        rx.next().await.unwrap();
    }
}
