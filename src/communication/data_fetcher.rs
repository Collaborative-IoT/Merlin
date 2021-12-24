/*
abstracts usage of the sql execution handler
by fetching and converts rows to correct response types.
*/
use crate::communication::communication_types::{RoomPermissions, User, UserPreview};
use crate::data_store::db_models::DBScheduledRoom;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use futures_util::Future;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use tokio_postgres::{row::Row, Error};

/*
Gathers user structs in direct relation
to the requester, including relationship specific
data like if the requester is blocked by
each user etc.
*/
pub async fn get_users_for_user(
    requester_user_id: i32,
    user_ids: Vec<i32>,
    execution_handler: &mut ExecutionHandler,
) -> (bool, Vec<User>) {
    let blocked_user_ids = get_blocked_user_ids_for_user(execution_handler, &requester_user_id)
        .await
        .1;
    let following_user_ids = get_following_user_ids_for_user(execution_handler, &requester_user_id)
        .await
        .1;
    return gather_and_construct_users_for_user(
        execution_handler,
        &requester_user_id,
        blocked_user_ids,
        following_user_ids,
        user_ids,
    )
    .await;
}

/*
Constructs a User struct for each user id passed in,
filling in the data in relation to the requesting user.

If we had user:77 requesting all users in a room, we need to
fill out a user struct for each user specifically for user 77. Like the field
"i_blocked_them" needs to indicate if user 77 blocked each user etc.
*/
pub async fn gather_and_construct_users_for_user(
    execution_handler: &mut ExecutionHandler,
    requesting_user_id: &i32,
    user_ids_requesting_user_blocked: HashSet<i32>,
    user_ids_requesting_user_follows: HashSet<i32>,
    users: Vec<i32>,
) -> (bool, Vec<User>) {
    let mut error_encountered = false;
    let mut constructed_users: Vec<User> = Vec::new();
    for user in users {
        let user_gather_result = execution_handler.select_user_by_id(&user).await;
        //only gather user is db selection was successful
        if user_gather_result.is_ok() {
            let selected_rows = user_gather_result.unwrap();
            //use user data to construct user
            if selected_rows.len() == 1 {
                let user_result = get_meta_data_for_user_and_construct(
                    execution_handler,
                    &selected_rows[0],
                    user_ids_requesting_user_blocked.contains(&user),
                    user_ids_requesting_user_follows.contains(&user),
                    requesting_user_id,
                )
                .await;
                check_user_result_and_handle_error(
                    user_result,
                    &mut constructed_users,
                    &mut error_encountered,
                );
            }
        }
    }
    return (error_encountered, constructed_users);
}

/*
Gathers all of the blockers,followers, for this specific user,
determines if this user blocked/followed the requesting user
for the fields "follows_you" and "they_blocked_you".
*/
async fn get_meta_data_for_user_and_construct(
    execution_handler: &mut ExecutionHandler,
    user_row: &Row,
    blocked_by_requesting_user: bool,
    followed_by_requesting_user: bool,
    requesting_user_id: &i32,
) -> (bool, User) {
    let user_id: i32 = user_row.get(0);
    //get blocked and followed users for this user.
    let blocked_result: (bool, HashSet<i32>) =
        get_blocked_user_ids_for_user(execution_handler, &user_id).await;
    let following_result: (bool, HashSet<i32>) =
        get_following_user_ids_for_user(execution_handler, &user_id).await;
    let followers_result: (bool, HashSet<i32>) =
        get_follower_user_ids_for_user(execution_handler, &user_id).await;
    let num_followers: i32 = followers_result.1.len() as i32;
    let user = construct_user(
        blocked_by_requesting_user,
        followed_by_requesting_user,
        following_result.1,
        blocked_result.1,
        user_row,
        requesting_user_id,
        num_followers,
    );
    if blocked_result.0 == true || following_result.0 == true {
        return (true, user);
    } else {
        return (false, user);
    }
}

pub async fn get_room_permissions_for_users(
    room_id: &i32,
    execution_handler: &mut ExecutionHandler,
) -> (bool, HashMap<i32, RoomPermissions>) {
    let mut permissions: HashMap<i32, RoomPermissions> = HashMap::new();
    let gather_result = execution_handler
        .select_all_room_permissions_for_room(room_id)
        .await;
    if gather_result.is_ok() {
        let selected_rows = gather_result.unwrap();
        for row in selected_rows {
            let user_id: i32 = row.get(1);
            let is_mod: bool = row.get(3);
            let is_speaker: bool = row.get(4);
            let asked_to_speak: bool = row.get(5);
            let user_permission = RoomPermissions {
                is_mod: is_mod,
                is_speaker: is_speaker,
                asked_to_speak: asked_to_speak,
            };
            permissions.insert(user_id, user_permission);
        }
        return (false, permissions);
    } else {
        return (true, permissions);
    }
}

pub async fn get_user_previews_for_users(
    user_ids: Vec<i32>,
    execution_handler: &mut ExecutionHandler,
) -> (bool, HashMap<i32, UserPreview>) {
    let mut user_previews: HashMap<i32, UserPreview> = HashMap::new();
    let mut encountered_error: bool = false;
    for user_id in user_ids {
        let gather_result = execution_handler.select_user_preview_by_id(&user_id).await;
        if gather_result.is_ok() {
            let selected_rows = gather_result.unwrap();
            let display_name: String = selected_rows[0].get(2);
            let avatar_url: String = selected_rows[0].get(1);
            let user_preview = UserPreview {
                display_name: display_name,
                avatar_url: avatar_url,
            };
            user_previews.insert(user_id, user_preview);
        } else {
            encountered_error = true;
        }
    }
    return (encountered_error, user_previews);
}

pub async fn get_scheduled_rooms(
    room_ids: Vec<i32>,
    execution_handler: &mut ExecutionHandler,
) -> (bool, Vec<DBScheduledRoom>) {
    let mut scheduled_rooms: Vec<DBScheduledRoom> = Vec::new();
    for room_id in room_ids {
        let room_gather_result = execution_handler
            .select_scheduled_room_by_id(&room_id)
            .await;
        if room_gather_result.is_ok() {
            let selected_rows = room_gather_result.unwrap();
            for row in selected_rows {
                let scheduled_room = construct_scheduled_room(&row);
                scheduled_rooms.push(scheduled_room);
            }
            return (false, scheduled_rooms);
        } else {
            return (true, scheduled_rooms);
        }
    }
    return (false, scheduled_rooms);
}

pub async fn get_blocked_user_ids_for_user(
    execution_handler: &mut ExecutionHandler,
    user_id: &i32,
) -> (bool, HashSet<i32>) {
    let future_for_execution = execution_handler.select_all_blocked_for_user(user_id);
    let blocked_users_result: (bool, HashSet<i32>) =
        get_single_column_of_all_rows_by_id(2, future_for_execution).await;
    return blocked_users_result;
}

pub async fn get_following_user_ids_for_user(
    execution_handler: &mut ExecutionHandler,
    user_id: &i32,
) -> (bool, HashSet<i32>) {
    let future_for_execution = execution_handler.select_all_following_for_user(user_id);
    let following_users_result: (bool, HashSet<i32>) =
        get_single_column_of_all_rows_by_id(2, future_for_execution).await;
    return following_users_result;
}

pub async fn get_follower_user_ids_for_user(
    execution_handler: &mut ExecutionHandler,
    user_id: &i32,
) -> (bool, HashSet<i32>) {
    let future_for_execution = execution_handler.select_all_followers_for_user(user_id);
    let followers_users_result: (bool, HashSet<i32>) =
        get_single_column_of_all_rows_by_id(2, future_for_execution).await;
    return followers_users_result;
}

pub async fn get_blocked_user_ids_for_room(
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> (bool, HashSet<i32>) {
    let future_for_execution = execution_handler.select_all_blocked_users_for_room(room_id);
    let blocked_users_result: (bool, HashSet<i32>) =
        get_single_column_of_all_rows_by_id(2, future_for_execution).await;
    return blocked_users_result;
}

pub async fn get_room_owner_and_settings(
    execution_handler: &mut ExecutionHandler,
    room_id: &i32,
) -> (bool, i32, String) {
    let gather_result = execution_handler.select_room_by_id(room_id).await;
    if gather_result.is_ok() {
        let selected_rows = gather_result.unwrap();
        if selected_rows.len() == 1 {
            let owner_id: i32 = selected_rows[0].get(1);
            let chat_mode: String = selected_rows[0].get(2);
            return (false, owner_id, chat_mode);
        } else {
            return (true, -1 as i32, "".to_owned());
        }
    } else {
        return (true, -2 as i32, "".to_owned());
    }
}

fn construct_user(
    blocked_by_requesting_user: bool,
    followed_by_requesting_user: bool,
    following: HashSet<i32>,
    blocked: HashSet<i32>,
    user_row: &Row,
    requesting_user_id: &i32,
    num_followers: i32,
) -> User {
    let username: String = user_row.get(3);
    let num_following: i32 = following.len() as i32;
    let last_online: String = user_row.get(4);
    let user_id: i32 = user_row.get(0);
    let contributions: i32 = user_row.get(12);
    let display_name: String = user_row.get(1);
    let bio: String = user_row.get(11);
    let avatar_url: String = user_row.get(2);
    let banner_url: String = user_row.get(13);

    return User {
        you_are_following: followed_by_requesting_user,
        username: username,
        they_blocked_you: blocked.contains(requesting_user_id),
        num_following: num_following,
        num_followers: num_followers,
        last_online: last_online,
        user_id: user_id,
        follows_you: following.contains(requesting_user_id),
        contributions: contributions,
        display_name: display_name,
        bio: bio,
        avatar_url: avatar_url,
        banner_url: banner_url,
        i_blocked_them: blocked_by_requesting_user,
    };
}

fn check_user_result_and_handle_error(
    user_result: (bool, User),
    constructed_users_state: &mut Vec<User>,
    error_encountered: &mut bool,
) {
    //mark encountered error for user reporting
    if user_result.0 == true {
        *error_encountered = true;
    }
    constructed_users_state.push(user_result.1);
}

/*
1.executes future to get rows(the method passed in the `select_future` parameter)
2.selects a specifc column of the rows based on col_index
3.collects all of the columns' data and stores in a hashset
4.returns error status and hashset.

TODO: Simplify this function, it shouldn't require any long comments
and should self document.
*/
pub async fn get_single_column_of_all_rows_by_id<
    ColumnDataType: std::fmt::Debug
        + std::cmp::PartialEq
        + for<'a> tokio_postgres::types::FromSql<'a>
        + std::cmp::Eq
        + Hash,
>(
    col_index: usize,
    select_future: impl Future<Output = Result<Vec<Row>, Error>>,
) -> (bool, HashSet<ColumnDataType>) {
    let mut data_set: HashSet<ColumnDataType> = HashSet::new();
    let mut encountered_error: bool = false;
    //execute select method and gather our rows
    let gather_result = select_future.await;
    if gather_result.is_ok() {
        //go through the rows and get the wanted column
        //by using the col index
        let selected_rows = gather_result.unwrap();
        for row in selected_rows {
            let column_data: ColumnDataType = row.get(col_index);
            data_set.insert(column_data);
        }
    } else {
        encountered_error = true;
    }
    return (encountered_error, data_set);
}

fn construct_scheduled_room(row: &Row) -> DBScheduledRoom {
    let room_id: i32 = row.get(0);
    let room_name: String = row.get(1);
    let num_attending: i32 = row.get(2);
    let scheduled_for: String = row.get(3);

    let scheduled_room = DBScheduledRoom {
        id: room_id,
        room_name: room_name,
        num_attending: num_attending,
        scheduled_for: scheduled_for,
    };
    return scheduled_room;
}
