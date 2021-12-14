use chrono::{DateTime, Utc};
use crate::DataStore::{sql_execution_handler::ExecutionHandler,tests};
use tokio_postgres::{Client,row::Row,NoTls,Error};

#[tokio::test]
pub async fn test(){
    let mut execution_handler = setup_execution_handler().await.unwrap();
    setup_tables(&mut execution_handler).await;
    tests::user::test_insert_and_gather_user(&mut execution_handler);
}

async fn setup_tables(execution_handler:&mut ExecutionHandler){
    let result = execution_handler.create_all_tables_if_needed().await;
    result.unwrap();
}

async fn setup_execution_handler()->Result<ExecutionHandler,Error>{ 
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres port=5432 password=password", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let mut handler = ExecutionHandler::new(client);
    return Ok(handler);
}