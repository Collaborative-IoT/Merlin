/*
abstracts usage of the sql execution handler 
by fetching and converts rows to correct response types.
*/

use tokio_postgres::{row::Row,Error};
use crate::communication::communication_types::{
    User
};
use crate::data_store::{sql_execution_handler::ExecutionHandler};
use crate::state::state::ServerState;
use std::hash::Hash;
use std::collections::HashSet;
use std::future::Future;

pub async fn get_users(
    requester_user_id:i32,
    server_state:ServerState
    ){

}

/*
1.executes method to get rows(the method passed in the `select_method` parameter)
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
    Hash, 
    IdType,
    Fut:Future<Output = Result<Vec<Row>,Error>>,
    Func:Fn(IdType)->Fut>(
        id:IdType, 
        execution_handler:&mut ExecutionHandler,
        col_index:usize,
        select_method:Func)->(bool,HashSet<ColumnDataType>) {
            let mut data_set:HashSet<ColumnDataType> = HashSet::new();
            let mut encountered_error:bool = false;
            //execute select method and gather our rows
            let gather_result = select_method(id).await;
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