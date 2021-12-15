use crate::data_store::db_models::{DBFollower};
use crate::data_store::sql_execution_handler::ExecutionHandler;

pub async fn test_follower_insertion_and_gather(execution_handler:&mut ExecutionHandler){
    //insert follower
    let new_follower:DBFollower = gather_db_follower();
    execution_handler.insert_follower(&new_follower).await;

    //make sure we have the correct results getting returned after insertion
    let gather_followers_result = execution_handler.select_all_followers_for_user(&new_follower.user_id).await;
    let selected_rows = gather_followers_result.unwrap();
    assert_eq!(selected_rows.len(),1);
    let follower_id:i32 = selected_rows[0].get(1);
    let user_id:i32 = selected_rows[0].get(2);
    assert_eq!(follower_id,new_follower.follower_id);
    assert_eq!(user_id,new_follower.user_id);
}

//gather all people the user is following
pub async fn test_gather_following(execution_handler:&mut ExecutionHandler){
    //we know this follower exist because of our previous test
    let target_follower_id:i32 = 22;
    let gather_results = execution_handler.select_all_following_for_user(&target_follower_id).await;
    let selected_rows = gather_results.unwrap();
    assert_eq!(selected_rows.len(),1);
    let user_id:i32 = selected_rows[0].get(2);
    assert_eq!(user_id,34);
}

pub async fn test_delete_following(execution_handler:&mut ExecutionHandler){
    let target_follower:i32 = 22;
    let target_user:i32 = 34;
    let result = execution_handler.delete_follower_for_user(&target_follower,&target_user).await;
    let rows_affected = result.unwrap();
    assert_eq!(rows_affected,1);
    let gather_followers_result = execution_handler.select_all_followers_for_user(&target_user).await;
    let selected_rows = gather_followers_result.unwrap();
    assert_eq!(selected_rows.len(),0);
}

fn gather_db_follower()->DBFollower{
    return DBFollower{
        id:0,
        follower_id:22,
        user_id:34
    };
}