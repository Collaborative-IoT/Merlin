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

This test isn't integration based, so we manually
grab the messages intended for the voice server after
publish and assert them.

*/

use crate::common::common_response_logic::send_to_requester_channel;
use crate::communication::communication_handler_helpers;
use crate::communication::communication_types::{
    AllUsersInRoomResponse, BasicRequest, BasicRoomCreation, BlockUserFromRoom, CommunicationRoom,
    GenericRoomId, GenericRoomIdAndPeerId, GetFollowList, User, UserPreview,
};
use crate::communication::data_fetcher;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rooms;
use crate::state::state::ServerState;
use crate::state::state_types::Room;
use futures::lock::Mutex;
use futures_util::stream::StreamExt;
use lapin::{
    message::Delivery, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer, Error, Result,
};
use std::collections::{HashMap, HashSet};
use std::mem::drop;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::rabbitmq::rabbit;
use crate::server::setup_execution_handler;

pub async fn tests() {
    let connection = rabbit::setup_rabbit_connection().await.unwrap();
    let publish_channel: Arc<Mutex<lapin::Channel>> =
        Arc::new(Mutex::new(setup_channel(&connection).await));
    let consume_channel: lapin::Channel = setup_channel(&connection).await;
    let consumer = consume_channel
        .basic_consume(
            "main",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    let mock_state: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState::new()));
    let execution_handler: Arc<Mutex<ExecutionHandler>> =
        Arc::new(Mutex::new(setup_execution_handler().await.unwrap()));
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
