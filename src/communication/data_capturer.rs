/*
Handles all captures and handles duplicate logic for
invalid requests like a user trying to follow
the same user twice etc.
*/

use crate::data_store::db_models::{
    DBFollower, DBRoom, DBRoomBlock, DBScheduledRoom, DBScheduledRoomAttendance, DBUser,
    DBUserBlock,
};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use futures_util::Future;
use tokio_postgres::{row::Row, Error};

pub struct CaptureResult {
    pub desc: String,
    pub encountered_error: bool,
}

pub async fn capture_new_user(execution_handler: &mut ExecutionHandler, user: &DBUser) -> i32 {
    //no user has both a github id and discord id
    //we use -1 for which ever doesn't exist(which is already set in the DBUser)
    let user_already_exist_result = execution_handler
        .select_user_by_discord_or_github_id(user.discord_id.to_owned(), user.github_id.to_owned())
        .await;

    // we haven't ran into db issues and our user doesn't exist
    if user_already_exist_result.is_ok() && user_already_exist_result.unwrap().len() == 0 {
        let insert_result = execution_handler.insert_user(user).await;
        if insert_result.is_ok() {
            let user_id: i32 = insert_result.unwrap();
            return user_id;
        } else {
            return -1 as i32;
        }
    } else {
        return -1 as i32;
    }
}

pub async fn capture_new_room(execution_handler: &mut ExecutionHandler, room: &DBRoom) -> i32 {
    let insert_future_for_execution = execution_handler.insert_room(room);
    return capture_room(insert_future_for_execution).await;
}

pub async fn capture_new_scheduled_room(
    execution_handler: &mut ExecutionHandler,
    room: &DBScheduledRoom,
    user_id: &i32,
) -> i32 {
    if less_than_x_row_exists(
        execution_handler.select_all_owned_scheduled_rooms_for_user(user_id),
        3,
    )
    .await
    {
        let future_for_execution = execution_handler.insert_scheduled_room(room);
        return capture_room(future_for_execution).await;
    } else {
        return -1;
    }
}

pub async fn capture_new_scheduled_room_attendance(
    execution_handler: &mut ExecutionHandler,
    attendance: &DBScheduledRoomAttendance,
) -> CaptureResult {
    //we know the attender exist but we don't know the scheduled room exist, so check
    if row_exists(execution_handler.select_scheduled_room_by_id(&attendance.scheduled_room_id))
        .await
    {
        let select_future = execution_handler
            .select_single_room_attendance(&attendance.user_id, &attendance.scheduled_room_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        let insert_future = execution_handler.insert_scheduled_room_attendance(attendance);
        return ensure_no_duplicates_exist_and_capture(
            will_be_duplicate,
            insert_future,
            "You already declared you are attending this room!".to_owned(),
        )
        .await;
    } else {
        return too_many_insertions_exist_capture_result();
    }
}

pub async fn capture_new_follower(
    execution_handler: &mut ExecutionHandler,
    follower: &DBFollower,
) -> CaptureResult {
    //we know the follower exist but we don't know the followee exist, so check
    if row_exists(execution_handler.select_user_by_id(&follower.user_id)).await {
        let select_future =
            execution_handler.select_single_follow(&follower.follower_id, &follower.user_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        let insert_future = execution_handler.insert_follower(follower);
        return ensure_no_duplicates_exist_and_capture(
            will_be_duplicate,
            insert_future,
            "You are already following this user!".to_owned(),
        )
        .await;
    } else {
        return row_does_not_exist_capture_result();
    }
}

pub async fn capture_new_user_block(
    execution_handler: &mut ExecutionHandler,
    user_block: &DBUserBlock,
) -> CaptureResult {
    //we know the blocker exists, but we don't know the blockee exist, so check
    if row_exists(execution_handler.select_user_by_id(&user_block.blocked_user_id)).await {
        let select_future = execution_handler
            .select_single_user_block(&user_block.owner_user_id, &user_block.blocked_user_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        let insert_future = execution_handler.insert_user_block(user_block);
        return ensure_no_duplicates_exist_and_capture(
            will_be_duplicate,
            insert_future,
            "This user is already blocked!".to_owned(),
        )
        .await;
    } else {
        return row_does_not_exist_capture_result();
    }
}

pub async fn capture_new_room_block(
    execution_handler: &mut ExecutionHandler,
    room_block: &DBRoomBlock,
) -> CaptureResult {
    if row_exists(execution_handler.select_user_by_id(&room_block.blocked_user_id)).await {
        let select_future = execution_handler
            .select_single_room_block(&room_block.owner_room_id, &room_block.blocked_user_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        let insert_future = execution_handler.insert_room_block(room_block);
        return ensure_no_duplicates_exist_and_capture(
            will_be_duplicate,
            insert_future,
            "This user is already blocked for this room!".to_owned(),
        )
        .await;
    } else {
        return row_does_not_exist_capture_result();
    }
}

pub async fn capture_user_block_removal(
    execution_handler: &mut ExecutionHandler,
    owner_id: &i32,
    blocked_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler
        .delete_block_for_user(owner_id, blocked_id)
        .await;
    return handle_removal_capture(
        "User block successfully removed".to_owned(),
        "Unexpected error removing user block".to_owned(),
        1,
        deletion_result,
    )
    .await;
}

pub async fn capture_room_block_removal(
    execution_handler: &mut ExecutionHandler,
    owner_id: &i32,
    blocked_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler
        .delete_room_block_for_user(owner_id, blocked_id)
        .await;
    return handle_removal_capture(
        "Room block successfully removed".to_owned(),
        "Unexpected error removing room block".to_owned(),
        1,
        deletion_result,
    )
    .await;
}

pub async fn capture_follower_removal(
    execution_handler: &mut ExecutionHandler,
    follower_id: &i32,
    user_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler
        .delete_follower_for_user(follower_id, user_id)
        .await;
    return handle_removal_capture(
        "Sucessfully unfollowed user".to_owned(),
        "Unexpected error unfollowing user".to_owned(),
        1,
        deletion_result,
    )
    .await;
}

pub async fn capture_room_removal(
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler.delete_room(room_id).await;
    return handle_removal_capture(
        "Room Removed".to_owned(),
        "Unexpected error removing room".to_owned(),
        1,
        deletion_result,
    )
    .await;
}

//makes sure a x amount of row were successfully deleted
async fn handle_removal_capture(
    success_msg: String,
    error_msg: String,
    expected_amount: u64,
    result: Result<u64, Error>,
) -> CaptureResult {
    if result.is_ok() && result.unwrap() == expected_amount {
        return CaptureResult {
            desc: success_msg,
            encountered_error: false,
        };
    } else {
        return CaptureResult {
            desc: error_msg,
            encountered_error: true,
        };
    }
}

//checks to see if a select future execution has rows or not
//useful for checking if a user exist before trying to
//follow them etc.
async fn row_exists(select_future: impl Future<Output = Result<Vec<Row>, Error>>) -> bool {
    let select_result = select_future.await;
    if select_result.is_ok() && select_result.unwrap().len() == 1 {
        return true;
    } else {
        return false;
    }
}

//used for limiting the amount of
//entries a user can make to a specific
//table.
async fn less_than_x_row_exists(
    select_future: impl Future<Output = Result<Vec<Row>, Error>>,
    x: usize,
) -> bool {
    let select_result = select_future.await;
    if select_result.is_ok() && select_result.unwrap().len() < x {
        return true;
    } else {
        return false;
    }
}

//only executes insert future if the insertion
//won't be a duplicate.
async fn ensure_no_duplicates_exist_and_capture(
    will_be_duplicate: bool,
    insert_future: impl Future<Output = Result<(), Error>>,
    error_message: String,
) -> CaptureResult {
    if will_be_duplicate {
        return CaptureResult {
            desc: error_message,
            encountered_error: true,
        };
    }
    return handle_basic_insert_with_no_returning(insert_future).await;
}

//handles logic for room capture
//takes in future generated by room insert methods
//in the sql_execution_handler.
//could be more generic to handle other similar cases.
async fn capture_room(future_exc: impl Future<Output = Result<i32, Error>>) -> i32 {
    let insert_result = future_exc.await;
    if insert_result.is_ok() {
        let room_id: i32 = insert_result.unwrap();
        return room_id;
    } else {
        return -1 as i32;
    }
}

//handles captures that doesn't require
//any data to be returned from it
//returns whether or not issues occured
async fn handle_basic_insert_with_no_returning(
    future_exc: impl Future<Output = Result<(), Error>>,
) -> CaptureResult {
    let insert_result = future_exc.await;
    if insert_result.is_ok() {
        return CaptureResult {
            desc: "Action Successful".to_owned(),
            encountered_error: false,
        };
    } else {
        return CaptureResult {
            desc: "Issue with execution".to_owned(),
            encountered_error: true,
        };
    }
}

//checks select future to determine if there is
//a row with the same credentials already present
//before insertion
async fn insert_will_be_duplicate(
    future_exc: impl Future<Output = Result<Vec<Row>, Error>>,
) -> bool {
    let future_result = future_exc.await;
    if future_result.is_ok() {
        let selected_rows = future_result.unwrap();
        if selected_rows.len() != 1 {
            return true;
        } else {
            return false;
        }
    } else {
        //return will be duplicate to signify
        //issue with future execution
        return true;
    }
}

fn row_does_not_exist_capture_result() -> CaptureResult {
    return CaptureResult {
        desc: "Invalid request!".to_owned(),
        encountered_error: true,
    };
}

fn too_many_insertions_exist_capture_result() -> CaptureResult {
    return CaptureResult {
        desc: "Capture Limit Reached!".to_owned(),
        encountered_error: false,
    };
}
