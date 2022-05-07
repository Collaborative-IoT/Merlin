use crate::state::state::ServerState;
use crate::vs_response::router;
use futures_util::stream::StreamExt;
use lapin::{
    message::Delivery, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Result,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_amqp::*;

pub async fn setup_rabbit_connection() -> Result<Connection> {
    let addr =
        std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".into());
    let conn: Connection =
        Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; //  Note the `with_tokio()` here
    return Ok(conn);
}

pub async fn setup_voice_publish_channel(conn: &Connection) -> Result<Channel> {
    let channel = conn.create_channel().await?;
    // declare/create new main queue
    channel
        .queue_declare(
            "voice_server_consume",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    return Ok(channel);
}

pub async fn setup_voice_consume_task(
    conn: &Connection,
    server_state: Arc<RwLock<ServerState>>,
) -> Result<()> {
    let channel = conn.create_channel().await?;
    // declare/create new main queue
    channel
        .queue_declare(
            "voice_server_publish",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "voice_server_publish",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // listen for messages forever and handle messages
    tokio::task::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            let (_, delivery) = delivery.expect("error in consumer");
            delivery.ack(BasicAckOptions::default()).await.expect("ack");
            let message = parse_message(delivery);
            let mut state = server_state.write().await;
            router::route_msg(message, &mut state).await;
        }
    });
    return Ok(());
}

pub async fn publish_voice_message(publish_channel: &Channel, data: String) -> Result<bool> {
    //voice server consume must be created prior aka queue declare.
    let confirm = publish_channel
        .basic_publish(
            "",
            "voice_server_consume",
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

pub fn parse_message(delivery: Delivery) -> String {
    let result = String::from_utf8(delivery.data);
    if result.is_ok() {
        return result.unwrap();
    } else {
        return "".to_owned();
    }
}
