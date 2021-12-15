use crate::data_store::db_models::{DBUserBlock,DBRoomBlock};
use crate::data_store::sql_execution_handler::ExecutionHandler;

pub async fn test_insert_and_gather_user_blocks(execution_handler:&mut ExecutionHandler){
    println!("testing inserting user block");
    let user_block:DBUserBlock = gather_user_block();
    execution_handler.insert_user_block(&user_block).await;
    let gather_result = execution_handler.select_all_blocked_for_user(&user_block.owner_user_id).await;
    let selected_rows = gather_result.unwrap();
    assert_eq!(selected_rows.len(),1);
    let owner_user_id:i32 = selected_rows[0].get(1);
    let blocked_user_id:i32 = selected_rows[0].get(2);
    assert_eq!(owner_user_id,user_block.owner_user_id);
    assert_eq!(blocked_user_id,user_block.blocked_user_id);
}

pub async fn test_get_blockers_for_user(execution_handler:&mut ExecutionHandler){
    println!("testing getting user blockers");
    let user_id:i32 = 33;
    let gather_result = execution_handler.select_all_blockers_for_user(&user_id).await;
    let selected_rows = gather_result.unwrap();
    let owner_user_id:i32 = selected_rows[0].get(1);
    assert_eq!(owner_user_id,22);
}

pub async fn test_insert_and_gather_room_blocks(execution_handler:&mut ExecutionHandler){
    println!("testing inserting room block");
    let room_block:DBRoomBlock = gather_room_block();
    execution_handler.insert_room_block(&room_block).await.unwrap();
    let gather_result = execution_handler.select_all_blocked_users_for_room(&room_block.owner_room_id).await;
    let selected_rows = gather_result.unwrap();
    assert_eq!(selected_rows.len(),1);
    let owner_room_id:i32 = selected_rows[0].get(1);
    let blocked_user_id:i32 = selected_rows[0].get(2);
    assert_eq!(owner_room_id,room_block.owner_room_id);
    assert_eq!(blocked_user_id,room_block.blocked_user_id);
}

pub async fn test_remove_user_block(execution_handler:&mut ExecutionHandler){
    println!("testing remove user block");
    let target_user:i32 = 22;
    let target_blocked_user:i32 = 33;
    let remove_result = execution_handler.delete_block_for_user(&target_user,&target_blocked_user).await;
    let rows_affected = remove_result.unwrap();
    assert_eq!(rows_affected,1);
}

pub async fn test_remove_room_block(execution_handler:&mut ExecutionHandler){
    println!("testing remove room block");
    let target_room:i32 = 22;
    let target_user:i32 = 33;
    let remove_result = execution_handler.delete_room_block_for_user(&target_room,&target_user).await;
    let rows_affected = remove_result.unwrap();
    assert_eq!(rows_affected,1);
}

fn gather_user_block()->DBUserBlock{
    return DBUserBlock{
        id:0,
        owner_user_id:22,
        blocked_user_id:33
    };
}

fn gather_room_block()->DBRoomBlock{
    return DBRoomBlock{
        id:0,
        owner_room_id:22,
        blocked_user_id:33
    };
}