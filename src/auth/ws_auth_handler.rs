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

//We possibly renewed the AuthCredentials
//so we need to return them.
pub struct UserIdAndNewAuthCredentials {
    pub user_id: i32,
    pub refresh: Option<String>,
    pub access: Option<String>,
}

pub async fn gather_user_id_using_discord_id(
    refresh: String,
    access: String,
    handler: &Arc<Mutex<ExecutionHandler>>,
) -> Option<UserIdAndNewAuthCredentials> {
    let gather_result = authentication_handler::gather_user_basic_data_discord(access).await;
    if let Ok(response_data) = gather_result {
        //if we found an id in the response go and
        //get the user from the db
        let user_id = get_id_from_response(response_data, handler, "dc").await?;
        return Some(UserIdAndNewAuthCredentials {
            user_id,
            refresh: None,
            access: None,
        });
    }
    let result = authentication_handler::exchange_discord_refresh_token_for_access(refresh).await;
    if let Ok(response_data) = result {
        let user_id = get_id_from_response(response_data.clone(), handler, "dc").await?;
        // if we can't find user
        // try to exchange old refresh for new set
        if user_id != -1 {
            let new_access = response_data["access_token"].to_string();
            let new_refresh = response_data["refresh_token"].to_string();
            //remove extra quotes
            let fixed_new_access = new_access[1..new_access.len() - 1].to_string();
            let fixed_new_refresh = new_access[1..new_refresh.len() - 1].to_string();
            return Some(UserIdAndNewAuthCredentials {
                user_id,
                access: Some(fixed_new_access),
                refresh: Some(fixed_new_refresh),
            });
        }
        return Some(UserIdAndNewAuthCredentials {
            user_id,
            access: None,
            refresh: None,
        });
    }
    return None;
}

pub async fn gather_user_id_using_github_id(
    access: String,
    handler: &Arc<Mutex<ExecutionHandler>>,
) -> Option<UserIdAndNewAuthCredentials> {
    let gather_result = authentication_handler::gather_user_basic_data_github(access).await;
    if gather_result.is_ok() {
        let response_data = gather_result.unwrap();
        let user_id = get_id_from_response(response_data, handler, "gh").await?;
        return Some(UserIdAndNewAuthCredentials {
            user_id,
            access: None,
            refresh: None,
        });
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
        if let Ok(selected_rows) = result {
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
    let dc_or_gh_id = response_data["id"].to_string();
    let fixed_dc_or_gh_id = dc_or_gh_id[1..dc_or_gh_id.len() - 1].to_string();
    if type_of_select == "dc" {
        return execution_handler
            .select_user_by_discord_or_github_id(fixed_dc_or_gh_id, "-1".to_owned())
            .await;
    }
    return execution_handler
        .select_user_by_discord_or_github_id("-1".to_owned(), fixed_dc_or_gh_id)
        .await;
}
