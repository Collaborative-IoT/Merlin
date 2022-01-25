/*
Handles all captures and handles duplicate logic for
invalid requests like a user trying to follow
the same user twice etc.
*/

use crate::communication::communication_types::{ScheduledRoomUpdate, UserProfileEdit};
use crate::data_store::db_models::{
    DBFollower, DBRoom, DBRoomBlock, DBRoomPermissions, DBScheduledRoom, DBScheduledRoomAttendance,
    DBUser, DBUserBlock,
};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use futures_util::Future;
use tokio_postgres::{row::Row, Error};

use super::communication_types::BaseUser;
use super::data_fetcher;

pub struct CaptureResult {
    pub desc: String,
    pub encountered_error: bool,
}

pub async fn capture_new_user(execution_handler: &mut ExecutionHandler, user: &DBUser) -> i32 {
    // No user has both a github id and discord id
    // We use -1 for which ever doesn't exist(which is already set in the DBUser)
    let user_already_exist_result = execution_handler
        .select_user_by_discord_or_github_id(user.discord_id.to_owned(), user.github_id.to_owned())
        .await;

    //  we haven't ran into db issues and our user doesn't exist
    if user_already_exist_result.is_ok() && user_already_exist_result.unwrap().len() == 0 {
        let insert_result = execution_handler.insert_user(user).await;
        if insert_result.is_ok() {
            let user_id: i32 = insert_result.unwrap();
            return user_id;
        } else {
            // unexpected error
            return -2 as i32;
        }
    } else {
        // duplicate
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
        let room_id = capture_room(future_for_execution).await;
        let req_results =
            handle_scheduled_room_capture_reqs(execution_handler, &room_id, user_id).await;
        if req_results == false {
            return room_id;
        } else {
            return -1;
        }
    } else {
        return -1;
    }
}

pub async fn capture_new_scheduled_room_attendance(
    execution_handler: &mut ExecutionHandler,
    attendance: &DBScheduledRoomAttendance,
) -> CaptureResult {
    // we know the attender exist but we don't know the scheduled room exist, so check
    if row_exists(execution_handler.select_scheduled_room_by_id(&attendance.scheduled_room_id))
        .await
    {
        let select_future = execution_handler
            .select_single_room_attendance(&attendance.user_id, &attendance.scheduled_room_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        try_to_increase_num_attending_for_sch_room(
            &attendance.scheduled_room_id,
            execution_handler,
        )
        .await;
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
    // we know the follower exist but we don't know the followee exist, so check
    if row_exists(execution_handler.select_user_by_id(&follower.user_id)).await {
        let select_future =
            execution_handler.select_single_follow(&follower.follower_id, &follower.user_id);
        let will_be_duplicate = insert_will_be_duplicate(select_future).await;
        println!("{}", will_be_duplicate);
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
    // we know the blocker exists, but we don't know the blockee exist, so check
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
    return handle_removal_or_update_capture(
        "User block successfully removed".to_owned(),
        "Unexpected error removing user block".to_owned(),
        1,
        deletion_result,
    );
}

pub async fn capture_room_block_removal(
    execution_handler: &mut ExecutionHandler,
    owner_id: &i32,
    blocked_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler
        .delete_room_block_for_user(owner_id, blocked_id)
        .await;
    return handle_removal_or_update_capture(
        "Room block successfully removed".to_owned(),
        "Unexpected error removing room block".to_owned(),
        1,
        deletion_result,
    );
}

pub async fn capture_follower_removal(
    execution_handler: &mut ExecutionHandler,
    follower_id: &i32,
    user_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler
        .delete_follower_for_user(follower_id, user_id)
        .await;
    return handle_removal_or_update_capture(
        "Sucessfully unfollowed user".to_owned(),
        "Unexpected error unfollowing user".to_owned(),
        1,
        deletion_result,
    );
}

pub async fn capture_room_removal(
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> CaptureResult {
    let deletion_result = execution_handler.delete_room(room_id).await;
    return handle_removal_or_update_capture(
        "Room Removed".to_owned(),
        "Unexpected error removing room".to_owned(),
        1,
        deletion_result,
    );
}

pub async fn capture_user_update(
    execution_handler: &mut ExecutionHandler,
    user_id: &i32,
    edit: UserProfileEdit,
) -> CaptureResult {
    let current_user_or_not: Option<BaseUser> =
        data_fetcher::gather_base_user(execution_handler, user_id).await;
    if current_user_or_not.is_some() {
        let mut current_user = current_user_or_not.unwrap();
        merge_updates_with_current_user(&mut current_user, edit, execution_handler).await;
        let update_result = execution_handler
            .update_base_user_fields(&current_user)
            .await;
        return handle_removal_or_update_capture(
            "Fields Successfully Updated".to_owned(),
            "Error Updating fields".to_owned(),
            1,
            update_result,
        );
    };
    return generic_error_capture_result();
}

pub async fn capture_scheduled_room_update(
    user_id: &i32,
    update: &ScheduledRoomUpdate,
    execution_handler: &mut ExecutionHandler,
) -> CaptureResult {
    let owned_rooms_result = execution_handler
        .select_all_owned_scheduled_rooms_for_user(user_id)
        .await;
    if owned_rooms_result.is_ok() {
        let selected_rows = owned_rooms_result.unwrap();

        // go through owned rooms, find the match and update.
        for row in selected_rows {
            let room_id: i32 = row.get(2);
            if room_id == update.room_id {
                let update_result = execution_handler
                    .update_scheduled_room(
                        update.scheduled_for.to_owned(),
                        &update.room_id,
                        update.description.to_owned(),
                        update.name.to_owned(),
                    )
                    .await;
                return handle_removal_or_update_capture(
                    "Room Successfully Updated".to_owned(),
                    "Issue Updating Room".to_owned(),
                    1,
                    update_result,
                );
            }
        }
    };
    return CaptureResult {
        desc: "Issue Updating Room".to_owned(),
        encountered_error: true,
    };
}

pub async fn capture_new_room_owner_update(
    room_id: &i32,
    new_owner_id: &i32,
    execution_handler: &mut ExecutionHandler,
) -> CaptureResult {
    let room_update_result = execution_handler
        .update_room_owner(room_id, new_owner_id)
        .await;
    return handle_removal_or_update_capture(
        "Room owner updated successfully".to_owned(),
        "Issue updating room owner".to_owned(),
        1,
        room_update_result,
    );
}

pub async fn capture_new_room_permissions(
    permissions: &DBRoomPermissions,
    execution_handler: &mut ExecutionHandler,
) -> bool {
    let permissions_for_user =
        data_fetcher::get_room_permissions_for_users(&permissions.room_id, execution_handler).await;
    // We didn't run into an error grabbing permissions and
    // We didn't find any for this room user.
    if permissions_for_user.0 == false && !permissions_for_user.1.contains_key(&permissions.user_id)
    {
        let insert_result = execution_handler.insert_room_permission(permissions).await;
        if insert_result.is_ok() {
            return false;
        } else {
            return true;
        }
    } else {
        return true;
    }
}

pub async fn capture_new_room_permissions_update(
    permissions: &DBRoomPermissions,
    execution_handler: &mut ExecutionHandler,
) -> CaptureResult {
    let update_result = execution_handler
        .update_entire_room_permissions(permissions)
        .await;
    return handle_removal_or_update_capture(
        "Permissions Successfully Updated".to_owned(),
        "Issue Updating Permissions".to_owned(),
        1,
        update_result,
    );
}

/// Makes sure a x amount of row were successfully deleted/updated
fn handle_removal_or_update_capture(
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

/// Checks to see if a select future execution has rows or not
///     useful for checking if a user exist before trying to
///     follow them etc.
async fn row_exists(select_future: impl Future<Output = Result<Vec<Row>, Error>>) -> bool {
    let select_result = select_future.await;
    if select_result.is_ok() && select_result.unwrap().len() == 1 {
        return true;
    } else {
        return false;
    }
}

/// Used for limiting the amount of
///     entries a user can make to a specific table.
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

/// only executes insert future if the insertion
/// won't be a duplicate.
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

/// Handles logic for room capture
///     takes in future generated by room insert methods
///     in the sql_execution_handler.
///
/// Could be more generic to handle other similar cases.
async fn capture_room(future_exc: impl Future<Output = Result<i32, Error>>) -> i32 {
    let insert_result = future_exc.await;
    if insert_result.is_ok() {
        let room_id: i32 = insert_result.unwrap();
        return room_id;
    } else {
        return -1 as i32;
    }
}

/// Handles captures that doesn't require
///     any data to be returned from it.
///
/// returns whether or not issues occured
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

/// Checks select future to determine if there is
///     a row with the same credentials already present
///     before insertion.
async fn insert_will_be_duplicate(
    future_exc: impl Future<Output = Result<Vec<Row>, Error>>,
) -> bool {
    let future_result = future_exc.await;
    if future_result.is_ok() {
        let selected_rows = future_result.unwrap();
        if selected_rows.len() > 0 {
            return true;
        } else {
            return false;
        }
    } else {
        // Return will be duplicate to signify
        // issue with future execution
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

fn generic_error_capture_result() -> CaptureResult {
    return CaptureResult {
        desc: "Unexpected Error".to_owned(),
        encountered_error: false,
    };
}

async fn merge_updates_with_current_user(
    current_user: &mut BaseUser,
    updates: UserProfileEdit,
    execution_handler: &mut ExecutionHandler,
) {
    if updates.display_name.is_some() {
        let target_update = updates.display_name.unwrap();
        if field_is_long_enough(&target_update, 20 as usize, 3 as usize) {
            current_user.display_name = target_update;
        }
    };
    if updates.username.is_some() {
        let target_update = updates.username.unwrap();
        if !username_already_exist(&target_update, execution_handler).await {
            current_user.username = target_update;
        };
    };
    if updates.bio.is_some() {
        let target_update = updates.bio.unwrap();
        if field_is_long_enough(&target_update, 40 as usize, 1 as usize) {
            current_user.bio = target_update;
        }
    };
    if updates.avatar_url.is_some() {
        current_user.avatar_url = updates.avatar_url.unwrap();
    }
    if updates.banner_url.is_some() {
        current_user.banner_url = updates.banner_url.unwrap();
    }
}

///  Non fatal operation, if failure occurs no big deal since it only captures how
///  many users will attend.
async fn try_to_increase_num_attending_for_sch_room(
    room_id: &i32,
    execution_handler: &mut ExecutionHandler,
) {
    let sch_room_result = execution_handler.select_scheduled_room_by_id(room_id).await;
    if sch_room_result.is_ok() {
        let selected_rows = sch_room_result.unwrap();
        if selected_rows.len() == 1 {
            let row = &selected_rows[0];
            let old_num_attending: i32 = row.get(2);
            let new_num_attending = old_num_attending + 1;
            execution_handler
                .update_num_attending_sch_room(&new_num_attending, room_id)
                .await;
        }
    };
}

fn field_is_long_enough(data: &String, max_expected_len: usize, min_expected_len: usize) -> bool {
    let data_count = data.chars().count();
    if data_count >= min_expected_len && data_count <= max_expected_len {
        return true;
    } else {
        return false;
    }
}

async fn username_already_exist(
    username: &String,
    execution_handler: &mut ExecutionHandler,
) -> bool {
    let search_result = execution_handler.select_user_by_username(username).await;
    if search_result.is_ok() {
        let selected_rows = search_result.unwrap();
        if selected_rows.len() != 0 {
            return true;
        } else {
            return false;
        }
    } else {
        return true;
    }
}

/// Atempts to insert the room creator's attendance as the owner
///     and increases the sch room attendance number(apart od sch room attendance).
async fn handle_scheduled_room_capture_reqs(
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
    user_id: &i32,
) -> bool {
    let attendance = DBScheduledRoomAttendance {
        id: -1,
        scheduled_room_id: room_id.clone(),
        user_id: user_id.clone(),
        is_owner: true,
    };
    let capture_res = capture_new_scheduled_room_attendance(execution_handler, &attendance).await;
    return capture_res.encountered_error;
}
