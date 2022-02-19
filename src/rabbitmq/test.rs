/* The following tests are for testing the ability to publish and consume messages
    from the rabbit mq instance spawned. This test doesn't test every thing
    in "rabbit.rs" but rather things that can be tested without integration testing.
*/
use crate::rabbitmq::rabbit;
use futures_util::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection};
use serde::{Deserialize, Serialize};
use tokio_amqp::*;

#[derive(Deserialize, Serialize)]
struct MockVoiceServerResponseForUser {
    pub op: String,
    pub d: String,
    pub uid: String,
}

#[derive(Deserialize, Serialize)]
struct MockVoiceServerResponseForRoom {
    pub op: String,
    pub d: String,
    pub rid: String,
}

pub async fn test() {
    test_publish_and_consume().await;
}

pub async fn test_publish_and_consume() {
    let conn: Connection = rabbit::setup_rabbit_connection().await.unwrap();
    let channel_one = conn.create_channel().await.unwrap();
    channel_one
        .queue_declare(
            "voice_server_consume",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    let channel_two = conn.create_channel().await.unwrap();
    let data_to_publish: String = "test".to_owned();
    let publish_result = rabbit::publish_message(&channel_one, data_to_publish.clone()).await;
    assert_eq!(publish_result.unwrap(), true);
    let mut consumer = channel_two
        .basic_consume(
            "voice_server_consume",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    //should not hang or be None due to previous publish
    let delivery = consumer.next().await.unwrap().unwrap().1;
    //let queue know we got the message acknowledge aka ack
    delivery.ack(BasicAckOptions::default()).await.expect("ack");
    let parsed_msg = rabbit::parse_message(delivery);
    assert_eq!(parsed_msg, data_to_publish);
}
