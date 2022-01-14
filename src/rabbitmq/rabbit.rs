
use futures_util::stream::StreamExt;
use tokio_amqp::*;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};

pub async fn setup_rabbit_connection()->Result<Connection>{
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn:Connection = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    return Ok(conn);
}

pub async fn publish_message_to_voice_server(message: String) {}