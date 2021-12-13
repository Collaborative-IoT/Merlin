use crate::DataStore::db_models::{DBUser};
use chrono::{DateTime, Utc};
use crate::DataStore::sql_execution_handler::ExecutionHandler;
use tokio_postgres::{row::Row,Error};

async fn test_insert_and_gather_user(execution_handler:&mut ExecutionHandler)->i32{
    println!("Testing inserting/updating user");
    let mock_user = gather_user_struct();
    let insert_row_result = execution_handler.insert_user(&mock_user).await;
    assert_eq!(insert_row_result.is_ok(),true);

    //user id num
    let user_id:i32 = insert_row_result.unwrap();

    //search for the row
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    assert_eq!(select_row_result.is_ok(),true);

    //make sure the data is correct
    let selected_rows = select_row_result.unwrap();
    assert_eq!(selected_rows.len(),1);
    let target_row:&Row = &selected_rows[0];
    compare_user_to_db_user(&mock_user,target_row);
    return user_id;
}

async fn test_updating_user_avatar(execution_handler:&mut ExecutionHandler, user_id:i32){
    println!("Testing updating user avatar");
    //get the user avatar url
    //we know this user exist because of the test that run prior to this one
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    //double check there is only one result
    assert_eq!(selected_rows.len(),1);
    let new_avatar_url = "test.com/new_test_url".to_string();
    let avatar_url:&str = selected_rows[0].get(2);
    assert_eq!(avatar_url,"test.com/avatar");

    let result = execution_handler.update_user_avatar(new_avatar_url,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);

    //check after update
    let select_row_result_second = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows_second = select_row_result_second.unwrap();

    let after_update_avatar_url:&str = selected_rows_second[0].get(2);
    assert_eq!(after_update_avatar_url,"test.com/new_test_url");
}

//asserts db results against the original user inserted
fn compare_user_to_db_user(user_one:&DBUser,row:&Row){
    let display_name:&str = row.get(1);
    let avatar_url:&str = row.get(2);
    let user_name:&str = row.get(3);
    let last_online:&str = row.get(4);
    let github_id:&str = row.get(5);
    let discord_id:&str = row.get(6);
    let github_access_token:&str = row.get(7);
    let discord_access_token:&str = row.get(8);
    let banned:bool = row.get(9);
    let banned_reason:&str = row.get(10);
    let bio:&str = row.get(11);
    let contributions:i32 = row.get(12);
    let banner_url:&str = row.get(13);

    assert_eq!(user_one.display_name,display_name);
    assert_eq!(user_one.avatar_url,avatar_url);
    assert_eq!(user_one.user_name,user_name);
    assert_eq!(user_one.last_online,last_online);
    assert_eq!(user_one.github_id,github_id);
    assert_eq!(user_one.discord_id,discord_id);
    assert_eq!(user_one.github_access_token,github_access_token);
    assert_eq!(user_one.discord_access_token,discord_access_token);
    assert_eq!(user_one.banned,banned);
    assert_eq!(user_one.banned_reason,banned_reason);
    assert_eq!(user_one.bio,bio);
    assert_eq!(user_one.contributions,contributions);
    assert_eq!(user_one.banner_url,banner_url);
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