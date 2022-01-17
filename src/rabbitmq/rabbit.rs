use crate::communication::communication_types::BasicResponse;
use crate::ws_fan;
use futures::lock::Mutex;
use futures_util::stream::StreamExt;
use lapin::{
    message::Delivery, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Error, Result,
};
use std::sync::Arc;
use tokio_amqp::*;
use warp::ws::Message;

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

async fn handle_message(message: String, server_state: &mut ServerState) {
    let data: serde_json::Value = serde_json::from_str(&message).unwrap();
    let request_type = type_of_request(&data);
    //all room messages(events from voice server) need to be brodcasted across the room
    //all user messages(events from voice server) need to be brodcasted to the user alone
    //*NOTE*-> error for sending to websocket channels are handled in a different task
    let message_for_user = BasicResponse {
        response_op_code: "voice_server_msg".to_owned(),
        response_containing_data: message,
    };
    let string_basic_response = serde_json::to_string(&message_for_user).unwrap();
    if request_type == "room" {
        let room_id: i32 = data["rid"].to_string().parse().unwrap();
        ws_fan::fan::broadcast_message_to_room(string_basic_response, server_state, room_id).await;
    } else {
        let user_id: i32 = data["uid"].to_string().parse().unwrap();
        let user_websocket_channel = server_state.peer_map.get(&user_id).unwrap();
        user_websocket_channel.send(Message::text(string_basic_response.clone()));
    }
}

//this gives us the type of request that is
//sent by the voice server which is actually
//either an update for all users of a room
//or one user a room
//"room" or "user"
pub fn type_of_request(data: &serde_json::Value) -> String {
    if data["uid"] == serde_json::Value::Null {
        return "room".to_string();
    } else {
        return "user".to_string();
    }
}

pub fn parse_message(delivery: Delivery) -> String {
    let result = String::from_utf8(delivery.data);
    if result.is_ok() {
        return result.unwrap();
    } else {
        return "".to_owned();
    }
}
