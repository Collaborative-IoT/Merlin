use crate::DataStore::db_models::{DBRoom,DBScheduledRoom,DBScheduledRoomAttendance};
use crate::DataStore::sql_execution_handler::ExecutionHandler;

pub async fn test_room_insert_and_gather(execution_handler:&mut ExecutionHandler)->i32{
    println!("testing room insert and gather");
    let new_room = gather_db_room();
    let result = execution_handler.insert_room(&new_room).await;
    let room_id = result.unwrap();
    //Check inserted data
    let gather_room_result = execution_handler.select_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let owner_id:i32 = selected_rows[0].get(1);
    let chat_mode:&str = selected_rows[0].get(2);
    assert_eq!(owner_id,8);
    assert_eq!(chat_mode,"off");
    return room_id;
}

pub async fn test_update_room_owner(execution_handler:&mut ExecutionHandler, room_id:i32)->i32{
    println!("testing updating room owner");
    //we know the room's current owner_id = 8 from previous testing
    let new_owner:i32 = 10;
    let update_result = execution_handler.update_room_owner(&room_id, &new_owner).await;
    let num_updated = update_result.unwrap();
    assert_eq!(num_updated,1);
    //check inserted data
    let gather_room_result = execution_handler.select_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let owner_id:i32 = selected_rows[0].get(1);
    assert_eq!(owner_id,10);
    return room_id;
}

pub async fn test_scheduled_room_insert_and_gather(execution_handler:&mut ExecutionHandler)->i32{
    println!("testing scheduled room insert and gather");
    let new_scheduled_room = gather_sch_db_room();
    let result = execution_handler.insert_scheduled_room(&new_scheduled_room).await;
    let room_id = result.unwrap();
    //check inserted data
    let gather_room_result =  execution_handler.select_scheduled_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let room_name:&str = selected_rows[0].get(1);
    let num_attending:i32 = selected_rows[0].get(2);
    let scheduled_for:&str = selected_rows[0].get(3);
    assert_eq!(room_name,"test");
    assert_eq!(num_attending,99);
    assert_eq!(scheduled_for,"test_val");
    return room_id;
}

pub async fn test_update_scheduled_room(execution_handler:&mut ExecutionHandler, room_id:i32)->i32{
    println!("testing update scheduled room");
    //we know the previous num_attending was 99 and scheduled_for was `test_val`
    let new_num_attending:i32 = 392;
    let scheduled_for:&str = "for";
    //update
    let update_result = execution_handler.update_scheduled_room(new_num_attending,scheduled_for,&room_id).await;
    let num_updated = update_result.unwrap();
    assert_eq!(392,num_attending);
    //check updated data
    let gather_room_result =  execution_handler.select_scheduled_room_by_id(&room_id).await;
    let selected_rows = gather_room_result.unwrap();
    let num_attending:i32 = selected_rows[0].get(2);
    let scheduled_for:&str = selected_rows[0].get(3);
    assert_eq!(num_attending,392);
    assert_eq!(scheduled_for,"for");
    return room_id;
}

pub async fn test_inserting_scheduled_room_attendance(execution_handler:&mut ExecutionHandler){
    println!("testing inserting scheduled room attendance");
    let new_sch_room_attendance:DBScheduledRoomAttendance = gather_sch_db_room_attendance();
    let insert_row_result = execution_handler.insert_scheduled_room_attendance(&new_sch_room_attendance).await;
    let gather_rows_result = execution_handler.select_all_attendance_for_scheduled_room(&new_sch_room_attendance.scheduled_room_id).await;
    let selected_rows = gather_rows_result.unwrap();
    assert_eq!(selected_rows.len(),1);
    let user_id:i32 = selected_rows[0].get(1);
    let is_owner:bool = selected_rows[0].get(3);
    assert_eq!(user_id,99);
    assert_eq!(is_owner,true);
}

fn gather_sch_db_room_attendance()->DBScheduledRoomAttendance{
    return DBScheduledRoomAttendance{
        id:0,
        user_id:99,
        scheduled_room_id:899,
        is_owner:true
    }
}

fn gather_db_room()->DBRoom{
    return DBRoom{
        id:0,
        owner_id:8,
        chat_mode:"off".to_string()
    };
}

fn gather_sch_db_room()->DBScheduledRoom{
    return DBScheduledRoom{
        id:0,
        room_name:"test".to_owned(),
        num_attending:99,
        scheduled_for:"test_val".to_string()
    };
}