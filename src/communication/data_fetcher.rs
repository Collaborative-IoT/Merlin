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
    execution_handler:&mut ExecutionHandler, 
    this_user_id:&i32)->(bool,HashSet<i32>){
    //construct
    let future_for_execution = execution_handler.select_all_blocked_for_user(this_user_id);
    let mut encountered_error = false;
    //get who this current user blocked
    let blocked_users_result:(bool,HashSet<i32>) = get_single_column_of_all_rows_by_id(2,future_for_execution).await;
    return blocked_users_result;
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