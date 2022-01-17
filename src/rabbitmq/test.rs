/* The following tests are for testing the ability to publish and consume messages
    from the rabbit mq instance spawned. This test doesn't test every thing
    in "rabbit.rs" but rather things that can be tested without integration testing.
*/
use crate::rabbitmq::rabbit;
use futures_util::stream::StreamExt;
use tokio_amqp::*;
use lapin::{
    message::Delivery, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Result, Error
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct MockVoiceServerResponseForUser{
    pub op:String,
    pub d:String,
    pub uid:String
}

#[derive(Deserialize, Serialize)]
struct MockVoiceServerResponseForRoom{
    pub op:String,
    pub d:String,
    pub rid:String
}

pub async fn test(){
    test_publish_and_consume().await;
    test_type_of_request().await;
}

pub async fn test_publish_and_consume(){
    let conn:Connection = rabbit::setup_rabbit_connection().await.unwrap();
    let channel_one = conn.create_channel().await.unwrap();
    let channel_two = conn.create_channel().await.unwrap();
    let data_to_publish:String = "test".to_owned();
    let publish_result = rabbit::publish_message(&channel_one,data_to_publish.clone()).await;
    assert_eq!(publish_result.is_ok(),true);
    let mut consumer = channel_two
    .basic_consume(
        "main",
        "my_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    )
    .await.unwrap();

    //should not hang or be None due to previous publish
    let delivery = consumer.next().await.unwrap().unwrap().1;
    let parsed_msg = rabbit::parse_message(delivery);
    assert_eq!(parsed_msg,data_to_publish);
}

pub async fn test_type_of_request(){
    let room_struct_data = MockVoiceServerResponseForRoom{
        op:"test".to_string(),
        d:"test".to_string(),
        rid:"test".to_string()
    };

    let user_struct_data = MockVoiceServerResponseForUser{
        op:"test".to_string(),
        d:"test".to_string(),
        uid:"test".to_string()
    };

    let room_str_data:String = serde_json::to_string(&room_struct_data).unwrap();
    let user_str_data:String = serde_json::to_string(&user_struct_data).unwrap();

    let room_json_value = serde_json::from_str(&room_str_data).unwrap();
    let user_json_value = serde_json::from_str(&user_str_data).unwrap();

    let type_of_request_room = rabbit::type_of_request(&room_json_value);
    let type_of_request_user = rabbit::type_of_request(&user_json_value);
    
    assert_eq!(type_of_request_room, "room");
    assert_eq!(type_of_request_user, "user");
}