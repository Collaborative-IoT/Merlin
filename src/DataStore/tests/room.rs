use crate::DataStore::db_models::{DBRoom,DBScheduledRoom};
use crate::DataStore::sql_execution_handler::ExecutionHandler;

pub async fn test_room_insert_and_gather(execution_handler:&mut ExecutionHandler)->i32{
    println!("testing room insert and gather");
    let new_room = gather_db_room();
    let result = execution_handler.insert_room(new_room).await;
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

pub async fn test_scheduled_room_insert_and_gather(execution_handler:&mut ExecutionHandler)->i32{
    println!("testing scheduled room insert and gather");
    let new_scheduled_room = gather_sch_db_room();
    let result = execution_handler.insert_scheduled_room(new_scheduled_room).await;
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