use futures::lock::Mutex;
use futures_util::stream::StreamExt;
use lapin::{
    message::Delivery, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Result,
};
use std::sync::Arc;
use tokio_amqp::*;

use crate::state::state::ServerState;

pub async fn setup_rabbit_connection() -> Result<Connection> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn: Connection =
        Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    return Ok(conn);
}

pub async fn setup_consume_task(
    conn: &Connection,
    server_state: Arc<Mutex<ServerState>>,
) -> Result<()> {
    let channel = conn.create_channel().await?;
    //declare/create new main queue
    channel
        .queue_declare(
            "main",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "main",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    //listen for messages forever and handle messages
    tokio::task::spawn(async move {
        let mut state = server_state.lock().await;
        while let Some(delivery) = consumer.next().await {
            let (_, delivery) = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
            let message = parse_message(delivery);
            handle_message(message, &mut state).await;
        }
    });
    return Ok(());
}

pub async fn publish_message(publish_channel: &Channel, data: String) -> Result<bool> {
    let confirm = publish_channel
        .basic_publish(
            "main",
            "",
            BasicPublishOptions::default(),
            convert_string_to_vec_u8(data),
            BasicProperties::default(),
        )
        .await?
        .await?;
    return Ok(confirm == Confirmation::NotRequested);
}

fn convert_string_to_vec_u8(data: String) -> Vec<u8> {
    let str_data: &str = &data;
    let bytes: Vec<u8> = str_data.as_bytes().to_vec();
    return bytes;
}

async fn handle_message(message: String, server_state: &mut ServerState) {}

//this gives us the type of request that is
//sent by the voice server which is actually
//either an update for all users of a room
//or one user a room
//"room" or "user"
pub fn type_of_request(json_string: String) -> String {
    let data: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    if data["uid"] == serde_json::Value::Null {
        return "room".to_string();
    } else {
        return "user".to_string();
    }
}

pub fn parse_message(delivery: Delivery) -> String {
    //fill out the del parsing
    return "".to_string();
}

pub async fn publish_message_to_voice_server(message: String) {}
