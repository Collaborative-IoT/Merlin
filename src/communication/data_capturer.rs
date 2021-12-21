use crate::data_store::db_models::
{DBUser};
use crate::data_store::sql_execution_handler::ExecutionHandler;


pub async fn capture_new_user(
    execution_handler:&mut ExecutionHandler,
    user:&DBUser)->(bool,String){
        let selected_rows = execution_handler

}
