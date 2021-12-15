use chrono::{DateTime, Utc};
use crate::data_store::{sql_execution_handler::ExecutionHandler,tests};
use tokio_postgres::{Client,row::Row,NoTls,Error};

#[tokio::test]
pub async fn test(){
    let mut execution_handler = setup_execution_handler().await.unwrap();
    setup_tables(&mut execution_handler).await;
    test_users(&mut execution_handler).await;
    test_room(&mut execution_handler).await;
    test_follower(&mut execution_handler).await;
}

async fn test_follower(execution_handler:&mut ExecutionHandler){
    tests::follower::test_follower_insertion_and_gather(execution_handler).await;
    tests::follower::test_gather_following(execution_handler).await;
}

async fn test_room(execution_handler:&mut ExecutionHandler){
    let room_id = tests::room::test_room_insert_and_gather(execution_handler).await;
    let sch_room_id = tests::room::test_scheduled_room_insert_and_gather(execution_handler).await;
    //live-non scheduled
    tests::room::test_update_room_owner(execution_handler,room_id.clone()).await;
    tests::room::test_delete_room(execution_handler,room_id.clone()).await;
    //scheduled
    tests::room::test_update_scheduled_room(execution_handler,sch_room_id.clone()).await;
    tests::room::test_inserting_scheduled_room_attendance(execution_handler).await;
    tests::room::test_deleting_scheduled_room(execution_handler,sch_room_id.clone()).await;
    tests::room::test_deleting_scheduled_room_attendance(execution_handler).await;
    tests::room::test_deleting_all_scheduled_room_attendance(execution_handler).await;
    //permissions
    tests::room::test_room_permission_insert_and_gather(execution_handler).await;
    tests::room::test_update_room_permission_for_user(execution_handler).await;
    tests::room::test_delete_room_permissions(execution_handler).await;
}

async fn test_users(execution_handler: &mut ExecutionHandler){
    let user_id = tests::user::test_insert_and_gather_user(execution_handler).await;   
    tests::user::test_updating_user_avatar(execution_handler, user_id.clone()).await;
    tests::user::test_updating_ban_status_of_user(execution_handler, user_id.clone()).await;
    tests::user::test_updating_user_display_name(execution_handler, user_id.clone()).await;
    tests::user::test_updating_user_bio(execution_handler, user_id.clone()).await;
    tests::user::test_updating_last_online(execution_handler, user_id.clone()).await;
    tests::user::test_updating_github_access_token(execution_handler, user_id.clone()).await;
    tests::user::test_updating_discord_access_token(execution_handler, user_id.clone()).await;
    tests::user::test_update_contributions(execution_handler, user_id.clone()).await;
    tests::user::test_update_banner_url(execution_handler, user_id.clone()).await;
    tests::user::test_update_user_name(execution_handler, user_id.clone()).await;
    tests::user::test_updating_entire_user(execution_handler).await;
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