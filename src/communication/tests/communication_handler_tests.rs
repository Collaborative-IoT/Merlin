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
use crate::communication::communication_types::{VoiceServerRequest,BasicRequest, BasicResponse, BasicRoomCreation};
use crate::communication::{data_capturer, communication_router};
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
use serde::{Serialize, Deserialize};
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
    let consumer = consume_channel
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
    let rx_user_one = create_and_add_new_user_channel_to_peer_map(33, &mock_state);
    let rx_user_two = create_and_add_new_user_channel_to_peer_map(34, &mock_state);
    insert_starting_user_state(&mock_state).await;
}

async fn test_creating_room(
    consume_channel: &Consumer,
    publish_channel :&Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>
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
    let mut server_state = state.write().await;
    server_state
        .active_users
        .get_mut(&33)
        .unwrap()
        .current_room_id = 2;
    drop(server_state);
    let create_room_msg = basic_request_for_room_creation().await;
    communication_router::route_msg(create_room_msg.clone(), 33, state, publish_channel, execution_handler).await;

    // Check that user is getting an error response
    // to their task channel.
    grab_and_assert_request_response(user_one_rx,"invalid_request","issue with request").await;
    
    // Set the user's room state back to -1, signifying
    // that they aren't in a room, which means they 
    // can successfully create a room
    let mut server_state = state.write().await;
    server_state
    .active_users
    .get_mut(&33)
    .unwrap()
    .current_room_id = -1;
    communication_router::route_msg(create_room_msg, 33, state, publish_channel, execution_handler).await;
    // The second attempt for room creation should be successful,
    // resulting in a new room in state and a message to the voice
    // server via RabbitMQ. So, we can check these side effects.
    

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

async fn grab_and_assert_message_to_voice_server<T:serde::de::DeserializeOwned+Serialize>(consume_channel: &mut Consumer ,d:String, uid:String,op:String){
    let message = consume_message(consume_channel).await;
    let vs_message:VoiceServerRequest<T> = serde_json::from_str(&message).unwrap();
    assert_eq!(serde_json::to_string(&vs_message.d).unwrap(), d);
    assert_eq!(op, vs_message.op);
    assert_eq!(uid, vs_message.uid);
}

async fn basic_request_for_room_creation() -> String {
    let room_creation = BasicRoomCreation {
        name: "test".to_owned(),
        desc: "test".to_owned(),
        public: true,
    };

    let request = BasicRequest {
        request_op_code: "create_room".to_owned(),
        request_containing_data: serde_json::to_string(&room_creation).unwrap(),
    };
    return serde_json::to_string(&request).unwrap();
}
