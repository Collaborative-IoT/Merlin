use crate::DataStore::db_models::{
    DBRoom,
    DBRoomPermissions,
    DBFollower,
    DBUser,
    DBUserBlock,
    DBRoomBlock,
    DBScheduledRoom,
    DBScheduledRoomAttendance
};

use chrono::{DateTime, Utc};
use crate::DataStore::sql_execution_handler::ExecutionHandler;
use tokio_postgres::{Client,row::Row,NoTls,Error};

#[tokio::test]
pub async fn test(){
    let mut execution_handler = setup_execution_handler().await.unwrap();
    setup_tables(&mut execution_handler).await;

}

async fn setup_tables(execution_handler:&mut ExecutionHandler){
    let result = execution_handler.create_all_tables_if_needed().await;
    assert_eq!(result.is_ok(),true);
}

async fn test_insert_and_gather_user(execution_handler:&mut ExecutionHandler){
    let mock_user:DBUser = gather_user_struct();
    let insert_row_result = execution_handler.insert_user(mock_user).await;
    assert_eq!(insert_row_result.is_ok(),true);
    let inserted_rows = insert_row_result.unwrap();
    assert_eq!(inserted_rows.len(),1)

    //gather the row id
    let row_num:i32 = inserted_rows[0].get(0);

    //search for the row
    let select_row_result = execution_handler.select_user_by_id(row_num).await?;
    assert_eq!(select_row_result.is_ok(),true);

    //make sure the data is fine
    let selected_rows = select_row_result.unwrap();
    assert_eq(selected_rows.len(),1)
}

async fn setup_execution_handler()->Result<ExecutionHandler,Error>{ 
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let mut handler = ExecutionHandler::new(client);
    return Ok(handler);
}

fn gather_user_struct()->DBUser{
    let user:DBUser = DBUser{
        id:0,//doesn't matter in insertion
        display_name:"test1".to_string(),
        avatar_url:"test.com/avatar".to_string(),
        user_name:"ultimate_tester".to_string(),
        last_online:Utc::now().to_string(),
        github_id:"1".to_string(),
        discord_id:"2".to_string(),
        github_access_token:"23232".to_string(),
        discord_access_token:"29320".to_string(),
        banned:true,
        banned_reason:"ban evading".to_string(),
        bio:"test".to_string(),
        contributions:40,
        banner_url:"test.com/test_banner".to_string(),
    };
    return user;
}