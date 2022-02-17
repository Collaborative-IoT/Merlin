/*When a user connects to the server via ws
after they authenticate using the
 auth endpoints, we need to authenticate them
with their access/refresh tokens that they have
*/
use crate::auth::authentication_handler;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use futures::lock::Mutex;
use std::sync::Arc;
use tokio_postgres::Row;

pub async fn gather_user_id_using_discord_id(
    refresh: String,
    access: String,
    handler: &Arc<Mutex<ExecutionHandler>>,
) -> Option<i32> {
    let gather_result = authentication_handler::gather_user_basic_data_discord(access).await;
    if gather_result.is_ok() {
        let response_data = gather_result.unwrap();
        //if we found an id in the response go and
        //get the user from the db
        let user_id = get_id_from_response(response_data, handler, "dc").await?;
        return Some(user_id);
    } else {
        let result =
            authentication_handler::exchange_discord_refresh_token_for_access(refresh).await;
        if result.is_ok() {
            let response_data = result.unwrap();
            let user_id = get_id_from_response(response_data, handler, "dc").await?;
            return Some(user_id);
        }
    }
    return None;
}

pub async fn gather_user_id_using_github_id(
    access: String,
    handler: &Arc<Mutex<ExecutionHandler>>,
) -> Option<i32> {
    let gather_result = authentication_handler::gather_user_basic_data_github(access).await;
    if gather_result.is_ok() {
        let response_data = gather_result.unwrap();
        let user_id = get_id_from_response(response_data, handler, "gh").await?;
        return Some(user_id);
    }

    return None;
}

//selects by discord id or github id based on the passed in type
async fn get_id_from_response(
    response_data: serde_json::Value,
    handler: &Arc<Mutex<ExecutionHandler>>,
    type_of_select: &str,
) -> Option<i32> {
    if response_data["id"] != serde_json::Value::Null {
        let mut execution_handler = handler.lock().await;
        let result = select_dc_or_gh(&mut execution_handler, type_of_select, response_data).await;
        if result.is_ok() {
            let selected_rows = result.unwrap();
            if selected_rows.len() == 1 {
                let user_id: i32 = selected_rows[0].get(0);
                return Some(user_id);
            };
        };
    };
    return None;
}

async fn select_dc_or_gh(
    execution_handler: &mut ExecutionHandler,
    type_of_select: &str,
    response_data: serde_json::Value,
) -> Result<Vec<Row>, tokio_postgres::Error> {
    if type_of_select == "dc" {
        return execution_handler
            .select_user_by_discord_or_github_id(response_data["id"].to_string(), "-1".to_owned())
            .await;
    }
    return execution_handler
        .select_user_by_discord_or_github_id("-1".to_owned(), response_data["id"].to_string())
        .await;
}
