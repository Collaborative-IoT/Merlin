use chrono::{DateTime, Utc};
use crate::DataStore::{sql_execution_handler::ExecutionHandler,tests};
use tokio_postgres::{Client,row::Row,NoTls,Error};

#[tokio::test]
pub async fn test(){
    let mut execution_handler = setup_execution_handler().await.unwrap();
    setup_tables(&mut execution_handler).await;
    let user_id = tests::user::test_insert_and_gather_user(&mut execution_handler).await;   
    tests::user::test_updating_user_avatar(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_ban_status_of_user(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_user_display_name(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_user_bio(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_last_online(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_github_access_token(&mut execution_handler, user_id.clone()).await;
    tests::user::test_updating_discord_access_token(&mut execution_handler, user_id.clone()).await;
    tests::user::test_update_contributions(&mut execution_handler, user_id.clone()).await;
    tests::user::test_update_banner_url(&mut execution_handler, user_id.clone()).await;
    tests::user::test_update_user_name(&mut execution_handler, user_id.clone()).await;
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