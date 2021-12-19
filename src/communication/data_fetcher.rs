/*
abstracts usage of the sql execution handler 
by fetching and converts rows to correct response types.
*/
use futures_util::Future;
use tokio_postgres::{row::Row,Error};
use crate::communication::communication_types::{
    User
};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use std::hash::Hash;
use std::collections::HashSet;

pub async fn get_users(
    requester_user_id:i32,
    server_state:ServerState,
    execution_handler:&mut ExecutionHandler
    ){
        let user_ids:Vec<i32> = server_state.active_users.keys().cloned().collect();
}

pub async fn get_blocked_user_ids_for_user(
    execution_handler:&mut ExecutionHandler, user_id:&i32)->(bool,HashSet<i32>){
    let future_for_execution = execution_handler.select_all_blocked_for_user(user_id);
    let mut encountered_error = false;
    let blocked_users_result:(bool,HashSet<i32>) = get_single_column_of_all_rows_by_id(2,future_for_execution).await;
    return blocked_users_result;
}

pub async fn get_following_user_ids_for_user(
    execution_handler:&mut ExecutionHandler,user_id:&i32)->(bool,HashSet<i32>){
    let future_for_execution = execution_handler.select_all_following_for_user(user_id);
    let mut encountered_error = false;
    let following_users_result:(bool,HashSet<i32>) = get_single_column_of_all_rows_by_id(2, future_for_execution);
    return following_users_result;
}

pub async fn get_followers_user_ids_for_user(
    execution_handler:&mut ExecutionHandler,user_id:&i32)->(bool,HashSet<i32>){
    let future_for_execution = execution_handler.select_all_followers_for_user(user_id);
    let mut encountered_error = false;
    let followers_users_result:(bool,HashSet<i32>) = get_single_column_of_all_rows_by_id(2, future_for_execution);
    return followers_users_result;
}

/*
Constructs a User struct for each user id passed in,
filling in the data in relation to the requesting user.

If we had user:77 requesting all users in a room, we need to
fill out a user struct for each user specifically for user 77. Like the field
"i_blocked_them" needs to indicate if user 77 blocked each user etc.
*/
pub async fn gather_and_construct_users_for_user(
    execution_handler:&mut ExecutionHandler,
    requesting_user_id:&i32,
    user_ids_requesting_user_blocked:HashSet<i32>,
    users:Vec<i32>){
        let constructed_users:Vec<User> = Vec::new();
        //go through every user and construct
        for user in users{
            let user_gather_result = execution_handler.select_user_by_id(&user).await;
            //gather was successful
            if user_gather_result.is_ok(){
                let selected_rows = user_gather_result.unwrap();
                //make sure we get a result
                if selected_rows.len() == 1{

                }
            }
        }
    }


/*
Gathers all of the blockers,followers, for this specific user,
determines if this user blocked/followed the requesting user
for the fields "follows_you" and "they_blocked_you".

*/
pub async fn get_meta_data_for_user_and_construct(
    execution_handler:&mut ExecutionHandler,
    user_row:Row,
    blocked_by_requesting_user:bool,
    followed_by_requesting_user:bool,
    requesting_user_id:&i32,
    constructed_users_state:&mut Vec<User>)->(bool,User){
        let user_id:i32 = user_row.get(0);
        //get blocked and followed users for this user.
        let blocked_result:(bool,HashSet<i32>) = get_blocked_user_ids_for_user(execution_handler, &user_id).await;
        let following_result:(bool,HashSet<i32>) = get_following_user_ids_for_user(execution_handler, &user_id).await; 
        let followers_result:(bool,HashSet<i32>) = get_followers_user_ids_for_user(execution_handler, &user_id).await;
        let num_followers:i32 = followers_result.1.len().into();
        let user = construct_user(
            blocked_by_requesting_user, 
            followed_by_requesting_user, 
            following_result.1, 
            blocked_result.1, 
            user_row, 
            requesting_user_id,
            num_followers);
        if blocked_result.0 == true || following_result.0 == true{
            return (true,user);
        }
        else{
            return (false,user);
        }
    }   

pub fn construct_user(
    blocked_by_requesting_user:bool,
    followed_by_requesting_user:bool,
    following:HashSet<i32>,
    blocked:HashSet<i32>,
    user_row:Row,
    requesting_user_id:&i32,
    num_followers:i32)->User{
        return User{
            you_are_following:followed_by_requesting_user,
            username: user_row.get(3) as String,
            they_blocked_you:blocked.contains(requesting_user_id),
            num_following:following.len() as i32,
            num_followers:num_followers,
            last_online:user_row.get(4) as String,
            user_id:user_row.get(0) as i32,
            follows_you:following.contains(requesting_user_id),
            contributions:user_row.get(12) as i32,
            display_name:user_row.get(1) as String,
            bio:user_row.get(11) as String,
            avatar_url:user_row.get(2) as String,
            banner_url:user_row.get(13) as String,
            i_blocked_them:blocked_by_requesting_user
        };
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
    ColumnDataType:std::fmt::Debug + 
    std::cmp::PartialEq + 
    for<'a> tokio_postgres::types::FromSql<'a>+
    std::cmp::Eq+
    Hash>(
        col_index:usize,
        select_future:impl Future<Output = Result<Vec<Row>,Error>>)->(bool,HashSet<ColumnDataType>) {
            let mut data_set:HashSet<ColumnDataType> = HashSet::new();
            let mut encountered_error:bool = false;
            //execute select method and gather our rows
            let gather_result = select_future.await;
            if gather_result.is_ok(){
                //go through the rows and get the wanted column 
                //by using the col index
                let selected_rows = gather_result.unwrap();
                for row in selected_rows{
                    let column_data:ColumnDataType = row.get(col_index);
                    data_set.insert(column_data);
                }
            }
            else{
                encountered_error = true;
            }
            return (encountered_error, data_set);
}

pub async fn get_block