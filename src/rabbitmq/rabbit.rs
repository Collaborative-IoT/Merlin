
use futures_util::stream::StreamExt;
use tokio_amqp::*;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,message::Delivery
};

pub async fn setup_rabbit_connection()->Result<Connection>{
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn:Connection = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    return Ok(conn);
}

pub async fn setup_consume_task(conn:&Connection, )->Result<()>{
    let channel = conn.create_channel().await?;
    //declare/create new main queue
    channel.queue_declare(
        "main",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await?;

    let mut consumer = channel
        .basic_consume(
            "main",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    //listen for messages forever and forward messages to
    //the correct user channel
    //Background: messages from the voice server are responses that
    //are directly send and handled by the end user on the frontend/client.
    tokio::task::spawn(async move{
        while let Some(delivery) = consumer.next().await {
            let (_, delivery) = delivery.expect("error in consumer");
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("ack");
            
        }
    });
    return Ok(());
}

//this gives us the type of request that is 
//sent by the voice server which is actually
//either an update for all users of a room 
//or one user a room
//"room" or "user"
pub fn type_of_request(json_string:String)->String{
    let data:serde_json::Value = serde_json::from_str(&json_string).unwrap();
    if data["uid"] == serde_json::Value::Null{
        return "room".to_string();
    }
    else{
        return "user".to_string();
    }
}

pub fn parse_message(delivery:Delivery)->String{
    //fill out the del parsing
    return "".to_string();
}


pub async fn publish_message_to_voice_server(message: String) {}