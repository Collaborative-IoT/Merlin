use crate::data_store::db_models::{
    DBRoom, DBRoomPermissions, DBScheduledRoom, DBScheduledRoomAttendance,
};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use tokio_postgres::{row::Row, Error};

//#rooms

pub async fn test_room_insert_and_gather(execution_handler: &mut ExecutionHandler) -> i32 {
    println!("testing room insert and gather");
    let new_room = gather_db_room();
    let result = execution_handler.insert_room(&new_room).await;
    let room_id = result.unwrap();
    //Check inserted data
    let gather_room_result = execution_handler.select_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let owner_id: i32 = selected_rows[0].get(1);
    let chat_mode: &str = selected_rows[0].get(2);
    assert_eq!(owner_id, 8);
    assert_eq!(chat_mode, "off");
    return room_id;
}

pub async fn test_update_room_owner(execution_handler: &mut ExecutionHandler, room_id: i32) -> i32 {
    println!("testing updating room owner");
    //we know the room's current owner_id = 8 from previous testing
    let new_owner: i32 = 10;
    let update_result = execution_handler
        .update_room_owner(&room_id, &new_owner)
        .await;
    let num_updated = update_result.unwrap();
    assert_eq!(num_updated, 1);
    //check inserted data
    let gather_room_result = execution_handler.select_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let owner_id: i32 = selected_rows[0].get(1);
    assert_eq!(owner_id, 10);
    return room_id;
}

pub async fn test_delete_room(execution_handler: &mut ExecutionHandler, room_id: i32) {
    println!("testing deleting room");
    //we know it exist based on previous tests
    let delete_rows_result = execution_handler.delete_room(&room_id).await;
    let rows_affected = delete_rows_result.unwrap();
    assert_eq!(rows_affected, 1);
    //check if they exist
    let gather_room_result = execution_handler.select_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    assert_eq!(selected_rows.len(), 0);
}

//#scheduled rooms

pub async fn test_scheduled_room_insert_and_gather(
    execution_handler: &mut ExecutionHandler,
) -> i32 {
    println!("testing scheduled room insert and gather");
    let new_scheduled_room = gather_sch_db_room();
    let result = execution_handler
        .insert_scheduled_room(&new_scheduled_room)
        .await;
    let room_id = result.unwrap();
    //check inserted data
    let gather_room_result = execution_handler
        .select_scheduled_room_by_id(&room_id)
        .await;
    let selected_rows = gather_room_result.unwrap();
    let room_name: &str = selected_rows[0].get(1);
    let num_attending: i32 = selected_rows[0].get(2);
    let scheduled_for: &str = selected_rows[0].get(3);
    assert_eq!(room_name, "test");
    assert_eq!(num_attending, 99);
    assert_eq!(scheduled_for, "test_val");
    return room_id;
}

pub async fn test_update_scheduled_room(
    execution_handler: &mut ExecutionHandler,
    room_id: i32,
) -> i32 {
    println!("testing updating scheduled room");
    //we know the previous num_attending was 99 and scheduled_for was `test_val`
    let new_num_attending: i32 = 392;
    let scheduled_for = "for".to_owned();
    //update
    let update_result = execution_handler
        .update_scheduled_room(&new_num_attending, scheduled_for, &room_id)
        .await;
    let num_updated = update_result.unwrap();
    assert_eq!(1, num_updated);
    //check updated data
    let gather_room_result = execution_handler
        .select_scheduled_room_by_id(&room_id)
        .await;
    let selected_rows = gather_room_result.unwrap();
    let num_attending: i32 = selected_rows[0].get(2);
    let scheduled_for: &str = selected_rows[0].get(3);
    assert_eq!(num_attending, 392);
    assert_eq!(scheduled_for, "for");
    return room_id;
}

pub async fn test_inserting_scheduled_room_attendance(execution_handler: &mut ExecutionHandler) {
    println!("testing inserting scheduled room attendance");
    let new_sch_room_attendance: DBScheduledRoomAttendance = gather_sch_db_room_attendance();
    execution_handler
        .insert_scheduled_room_attendance(&new_sch_room_attendance)
        .await.unwrap();
    let gather_rows_result = execution_handler
        .select_all_attendance_for_scheduled_room(&new_sch_room_attendance.scheduled_room_id)
        .await;
    let selected_rows = gather_rows_result.unwrap();
    assert_eq!(selected_rows.len(), 1);
    let user_id: i32 = selected_rows[0].get(1);
    let is_owner: bool = selected_rows[0].get(3);
    assert_eq!(user_id, 99);
    assert_eq!(is_owner, true);
}

pub async fn test_gathering_single_attendance(execution_handler: &mut ExecutionHandler){
    println!("testing gathering single attendance");
    let user_id:i32 = 99;
    let room_id:i32 = 899;
    let gather_result = execution_handler.select_single_room_attendance(&user_id,&room_id).await;
    let selected_rows = gather_result.unwrap();
    assert_eq!(selected_rows.len(),1);
}

pub async fn test_deleting_scheduled_room(execution_handler: &mut ExecutionHandler, room_id: i32) {
    println!("testing deleting scheduled room");
    let delete_result = execution_handler.delete_scheduled_room(&room_id).await;
    let rows_affected = delete_result.unwrap();
    assert_eq!(rows_affected, 1);
    let gather_room_result = execution_handler
        .select_scheduled_room_by_id(&room_id)
        .await;
    let selected_rows = gather_room_result.unwrap();
    assert_eq!(selected_rows.len(), 0);
}

pub async fn test_deleting_scheduled_room_attendance(execution_handler: &mut ExecutionHandler) {
    println!("testing deleting scheduled room attendance");
    let user_id: i32 = 99;
    let scheduled_room_id: i32 = 899;
    let delete_result = execution_handler
        .delete_user_room_attendance(&user_id, &scheduled_room_id)
        .await;
    let rows_affected = delete_result.unwrap();
    assert_eq!(rows_affected, 1);
    let gather_rows_result = execution_handler
        .select_all_attendance_for_scheduled_room(&scheduled_room_id)
        .await;
    let selected_rows = gather_rows_result.unwrap();
    assert_eq!(selected_rows.len(), 0);
}

pub async fn test_deleting_all_scheduled_room_attendance(execution_handler: &mut ExecutionHandler) {
    let new_sch_room_attendance: DBScheduledRoomAttendance = gather_sch_db_room_attendance();
    //insert two of the same attendances
    execution_handler
        .insert_scheduled_room_attendance(&new_sch_room_attendance)
        .await.unwrap();
    execution_handler
        .insert_scheduled_room_attendance(&new_sch_room_attendance)
        .await.unwrap();
    //delete
    let delete_result = execution_handler
        .delete_all_scheduled_room_attendance(&new_sch_room_attendance.scheduled_room_id)
        .await;
    let rows_affected = delete_result.unwrap();
    assert_eq!(rows_affected, 2);
    //check
    let gather_rows_result = execution_handler
        .select_all_attendance_for_scheduled_room(&new_sch_room_attendance.scheduled_room_id)
        .await;
    let selected_rows = gather_rows_result.unwrap();
    assert_eq!(selected_rows.len(), 0);
}

//#permissions

pub async fn test_room_permission_insert_and_gather(execution_handler: &mut ExecutionHandler) {
    let new_permissions: DBRoomPermissions = gather_permissions();
    execution_handler
        .insert_room_permission(&new_permissions)
        .await.unwrap();
    let gather_result = execution_handler
        .select_all_room_permissions_for_user(&new_permissions.user_id, &new_permissions.room_id)
        .await;
    assert_permissions( &new_permissions, gather_result);

    let gather_all_result = execution_handler
        .select_all_room_permissions_for_room(&new_permissions.room_id)
        .await;
    assert_permissions( &new_permissions, gather_all_result);
}

pub async fn test_update_room_permission_for_user(execution_handler: &mut ExecutionHandler) {
    //we already inserted a row with these credentials in previous tests
    //the row id doesn't matter in the update, so generate the same
    //object and modify the bool fields(real permissions)
    let mut mock_obj: DBRoomPermissions = gather_permissions();
    mock_obj.is_mod = false;
    mock_obj.is_speaker = false;
    mock_obj.asked_to_speak = false;

    let update_result = execution_handler
        .update_entire_room_permissions(&mock_obj)
        .await;
    let num_modified = update_result.unwrap();
    assert_eq!(num_modified, 1);
    let gather_result = execution_handler
        .select_all_room_permissions_for_user(&mock_obj.user_id, &mock_obj.room_id)
        .await;
    assert_permissions( &mock_obj, gather_result);
}

pub async fn test_delete_room_permissions(execution_handler: &mut ExecutionHandler) {
    //we already inserted a permission under this room_id in previous tests
    let target_room_id: i32 = 2000;
    let deletion_result = execution_handler
        .delete_all_room_permissions(&target_room_id)
        .await;
    let rows_affected = deletion_result.unwrap();
    assert_eq!(rows_affected, 1);
    //this user/room permission entry did exist at one point due to previous tests
    let target_user_id: i32 = 99;
    let gather_result = execution_handler
        .select_all_room_permissions_for_user(&target_room_id, &target_user_id)
        .await;
    let selected_rows = gather_result.unwrap();
    assert_eq!(selected_rows.len(), 0);
}

fn assert_permissions(
    new_permissions: &DBRoomPermissions,
    gather_result: Result<Vec<Row>, Error>,
) {
    let selected_rows = gather_result.unwrap();
    assert_eq!(selected_rows.len(), 1);
    let user_id: i32 = selected_rows[0].get(1);
    let room_id: i32 = selected_rows[0].get(2);
    let is_mod: bool = selected_rows[0].get(3);
    let is_speaker: bool = selected_rows[0].get(4);
    let asked_to_speak: bool = selected_rows[0].get(5);
    assert_eq!(user_id, new_permissions.user_id);
    assert_eq!(room_id, new_permissions.room_id);
    assert_eq!(is_mod, new_permissions.is_mod);
    assert_eq!(is_speaker, new_permissions.is_speaker);
    assert_eq!(asked_to_speak, new_permissions.asked_to_speak);
}

fn gather_permissions() -> DBRoomPermissions {
    return DBRoomPermissions {
        id: 0,
        user_id: 99,
        room_id: 2000,
        is_mod: true,
        is_speaker: true,
        asked_to_speak: true,
    };
}

fn gather_sch_db_room_attendance() -> DBScheduledRoomAttendance {
    return DBScheduledRoomAttendance {
        id: 0,
        user_id: 99,
        scheduled_room_id: 899,
        is_owner: true,
    };
}

fn gather_db_room() -> DBRoom {
    return DBRoom {
        id: 0,
        owner_id: 8,
        chat_mode: "off".to_string(),
    };
}

fn gather_sch_db_room() -> DBScheduledRoom {
    return DBScheduledRoom {
        id: 0,
        room_name: "test".to_owned(),
        num_attending: 99,
        scheduled_for: "test_val".to_string(),
    };
}
