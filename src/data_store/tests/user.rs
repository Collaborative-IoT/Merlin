use crate::data_store::db_models::{DBUser};
use chrono::{DateTime, Utc};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use tokio_postgres::{row::Row,Error};

//dry violations help reduce the confusion and makes sure tests are clear.
//massive generic functions would make things harder to follow in this 
//scenario.

pub async fn test_insert_and_gather_user(execution_handler:&mut ExecutionHandler)->i32{
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

pub async fn test_updating_user_avatar(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating user avatar");
    //get the user avatar url
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    //check the current avatar
    let new_avatar_url = "test.com/new_test_url".to_string();
    let avatar_url:&str = selected_rows[0].get(2);
    assert_eq!(avatar_url,"test.com/avatar");
    //perform update
    let result = execution_handler.update_user_avatar(new_avatar_url.to_owned(),&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    test_user_column_after_update(2 as usize, new_avatar_url,&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_ban_status_of_user(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating ban status of user");
    //get the current ban_status
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    //check
    let new_banned = false;
    let new_banned_reason = "no_reason".to_string();
    let current_banned:bool = selected_rows[0].get(9);
    let current_banned_reason:&str = selected_rows[0].get(10);
    assert_eq!(current_banned, true);
    assert_eq!(current_banned_reason,"ban evading");
    //update
    let result = execution_handler.update_ban_status_of_user(new_banned,new_banned_reason,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    let select_row_result_second = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows_second = select_row_result_second.unwrap();
    let after_update_current_banned:bool = selected_rows_second[0].get(9);
    let after_update_banned_reason:&str = selected_rows_second[0].get(10);
    assert_eq!(after_update_current_banned,false);
    assert_eq!(after_update_banned_reason,"no_reason");
    return user_id;
}

pub async fn test_updating_user_display_name(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating user display name");
    //get the user display name 
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_display_name = "test_update_name".to_string();
    let display_name:&str = selected_rows[0].get(1);
    assert_eq!(display_name,"test1");
    //update 
    let result = execution_handler.update_display_name(new_display_name.to_owned(),&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    test_user_column_after_update(1 as usize, new_display_name,&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_user_bio(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating user bio");
    //get the user bio
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_bio = "test bio 123".to_string();
    let bio:&str = selected_rows[0].get(11);
    assert_eq!(bio,"test");
    //update
    let result = execution_handler.update_user_bio(new_bio.to_owned(),&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check_after_update
    test_user_column_after_update(11 as usize, new_bio.to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_last_online(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating last online");
    //get the last online
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_last_online = Utc::now().to_string();
    let last_online:&str = selected_rows[0].get(4);
    //update
    let result = execution_handler.update_last_online(new_last_online.clone(),&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    test_user_column_after_update(4 as usize, new_last_online.to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_github_access_token(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating github access token");
    //get the last access token
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_access_token = "-40kp2rm3ro".to_string();
    let access_token:&str = selected_rows[0].get(7);
    assert_eq!(access_token,"23232");
    //update
    let result = execution_handler.update_github_access_token(new_access_token,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check_after_update
    test_user_column_after_update(7 as usize,"-40kp2rm3ro".to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_discord_access_token(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating discord access token");
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_discord_access_token = "lkwefif".to_string();
    let access_token:&str = selected_rows[0].get(8);
    assert_eq!(access_token,"29320");
    //update
    let result = execution_handler.update_discord_access_token(new_discord_access_token,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    test_user_column_after_update(8 as usize, "lkwefif".to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_update_contributions(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing updating contributions");
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_contributions:i32 = 30;
    let contributions:i32 = selected_rows[0].get(12);
    assert_eq!(contributions,40);
    //update 
    let result = execution_handler.update_contributions(&new_contributions,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after update
    test_user_column_after_update(12 as usize,new_contributions.to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_update_banner_url(execution_handler:&mut ExecutionHandler,user_id:i32)->i32{
    println!("Testing update banner url");
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_banner_url = "test.com/test_banner_new".to_string();
    let banner_url:&str = selected_rows[0].get(13);
    assert_eq!(banner_url,"test.com/test_banner");
    //update
    let result = execution_handler.update_banner_url(new_banner_url,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    //check after update
    let select_row_result_second = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows_second = select_row_result_second.unwrap();
    test_user_column_after_update(13 as usize, "test.com/test_banner_new".to_owned(), &user_id,execution_handler).await;
    return user_id;
}

pub async fn test_update_user_name(execution_handler:&mut ExecutionHandler, user_id:i32)->i32{
    println!("Testing update banner url");
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();
    let new_user_name = "new_user445".to_string();
    let user_name:&str = selected_rows[0].get(3);
    assert_eq!(user_name,"ultimate_tester");
    //update
    let result = execution_handler.update_user_name(new_user_name,&user_id).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);
    //check after
    test_user_column_after_update(3 as usize,"new_user445".to_owned(),&user_id,execution_handler).await;
    return user_id;
}

pub async fn test_updating_entire_user(execution_handler:&mut ExecutionHandler){
    println!("Testing updating entire user");
    let initial_user:DBUser = gather_user_struct();
    let insert_result = execution_handler.insert_user(&initial_user).await;
    
    //assume insertion works due to previous tests ran before this one
    let user_id:i32 = insert_result.unwrap();
    let select_row_result = execution_handler.select_user_by_id(&user_id).await;
    let selected_rows = select_row_result.unwrap();

    //make sure the insertion worked right and the values are the same
    compare_user_to_db_user(&initial_user,&selected_rows[0]);

    //update the user
    let mut different_user:DBUser = gather_different_user_struct();
    different_user.id = user_id; // id has to match original row to update
    let result = execution_handler.update_entire_user(&different_user).await;
    let num_of_rows_updated = result.unwrap();
    assert_eq!(num_of_rows_updated,1);

    //make sure the user got updated
    let select_row_result_second = execution_handler.select_user_by_id(&different_user.id).await;
    let selected_rows_second = select_row_result_second.unwrap();
    compare_user_to_db_user(&different_user,&selected_rows_second[0]);
}

async fn test_user_column_after_update<T:std::fmt::Debug + std::cmp::PartialEq + for<'a> tokio_postgres::types::FromSql<'a>>(
    col_num:usize,
    value_expected:T,
    user_id:&i32,
    execution_handler:&mut ExecutionHandler){
        //test gathered column by the col num and checks against what is expected
        let select_row_result = execution_handler.select_user_by_id(user_id).await;
        let selected_rows:Vec<Row> = select_row_result.unwrap();
        let after_update_val:T = selected_rows[0].get(col_num);
        assert_eq!(after_update_val,value_expected);
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

fn gather_different_user_struct()->DBUser{
    let user:DBUser = DBUser{
        id:0,//doesn't matter in insertion
        display_name:"test12".to_string(),
        avatar_url:"test.com/avatar2".to_string(),
        user_name:"ultimate_tester2".to_string(),
        last_online:Utc::now().to_string(),
        github_id:"12".to_string(),
        discord_id:"22".to_string(),
        github_access_token:"232322".to_string(),
        discord_access_token:"293202".to_string(),
        banned:true,
        banned_reason:"ban evading2".to_string(),
        bio:"test2".to_string(),
        contributions:40,
        banner_url:"test.com/test_banner2".to_string(),
    };
    return user;
}