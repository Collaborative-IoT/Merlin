/*
Handles the json gathered from api end points
by using access tokens.
*/
use crate::communication::data_capturer;
use crate::data_store::db_models::DBUser;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use chrono::Utc;
use futures::lock::Mutex;
use std::sync::Arc;
use tokio_postgres::row::Row;
use uuid::Uuid;

//grabs user api data and
//creates new user if it doesn't exist
//updates tokens if it does
pub async fn parse_and_capture_discord_user_data(
    data: serde_json::Value,
    execution_handler: Arc<Mutex<ExecutionHandler>>,
    access_token: String,
) -> bool {
    let mut action_was_successful: bool = true;
    if discord_user_request_is_valid(&data) {
        let discord_user_id: String = data["id"].to_string();
        let discord_avatar_id: String = data["avatar"].to_string();
        let discord_username: String = data["username"].to_string();

        //remove the trailing/begining double quotes
        //since the avatar id is formatted like "djweodiwdk"
        //instead of just djweodiwdk
        let fixed_dc_user_id = discord_user_id[1..discord_user_id.len() - 1].to_string();
        let fixed_dc_avatar_id = discord_avatar_id[1..discord_avatar_id.len() - 1].to_string();
        let fixed_dc_username = discord_username[1..discord_username.len() - 1].to_string();
        let fixed_dc_access = access_token[1..access_token.len()].to_string();
        let avatar_url =
            construct_discord_image_url(fixed_dc_user_id.as_str(), fixed_dc_avatar_id.as_str());
        let mut handler = execution_handler.lock().await;

        let user_id = generate_and_capture_new_user(
            fixed_dc_user_id.to_owned(),
            "-1".to_string(),
            avatar_url,
            fixed_dc_username,
            "-1".to_owned(),
            fixed_dc_access.to_owned(),
            &mut handler,
        )
        .await;
        check_user_id_and_continue(
            user_id,
            fixed_dc_access,
            "-1".to_owned(),
            &mut action_was_successful,
            &mut handler,
            fixed_dc_user_id,
            "-1".to_owned(),
        )
        .await;
    };
    return action_was_successful;
}

pub async fn parse_and_capture_github_user_data(
    data: serde_json::Value,
    execution_handler: Arc<Mutex<ExecutionHandler>>,
    access_token: String,
) -> bool {
    let mut action_was_successful = true;
    if github_user_request_is_valid(&data) {
        let github_avatar_url: String = data["avatar_url"].to_string();
        let github_name: String = data["name"].to_string();
        let github_id: String = data["id"].to_string();

        //remove the trailing/begining double quotes
        //since the id is formatted like "djweodiwdk"
        //instead of just djweodiwdk
        let fixed_avatar_url: String =
            github_avatar_url[1..github_avatar_url.len() - 1].to_string();
        let fixed_github_name: String = github_name[1..github_name.len() - 1].to_string();
        let fixed_github_id: String = github_id[1..github_id.len() - 1].to_string();
        let fixed_github_access = access_token[1..access_token.len()].to_string();
        let mut handler = execution_handler.lock().await;

        let user_id = generate_and_capture_new_user(
            "-1".to_owned(),
            fixed_github_id.to_owned(),
            fixed_avatar_url,
            fixed_github_name,
            fixed_github_access.to_owned(),
            "-1".to_owned(),
            &mut handler,
        )
        .await;

        check_user_id_and_continue(
            user_id,
            "-1".to_owned(),
            fixed_github_access,
            &mut action_was_successful,
            &mut handler,
            "-1".to_owned(),
            fixed_github_id,
        )
        .await;
    };
    return action_was_successful;
}

fn construct_discord_image_url(discord_user_id: &str, discord_user_avatar_id: &str) -> String {
    return format!(
        "https://cdn.discordapp.com/avatars/{}/{}.png",
        discord_user_id, discord_user_avatar_id
    );
}

fn discord_user_request_is_valid(data: &serde_json::Value) -> bool {
    if data["avatar"] != serde_json::Value::Null
        && data["id"] != serde_json::Value::Null
        && data["username"] != serde_json::Value::Null
    {
        return true;
    } else {
        return false;
    }
}

fn github_user_request_is_valid(data: &serde_json::Value) -> bool {
    if data["avatar_url"] != serde_json::Value::Null
        && data["name"] != serde_json::Value::Null
        && data["id"] != serde_json::Value::Null
    {
        return true;
    } else {
        return false;
    }
}

async fn generate_and_capture_new_user(
    discord_id: String,
    github_id: String,
    avatar_url: String,
    display_name: String,
    gh_access: String,
    dc_access: String,
    execution_handler: &mut ExecutionHandler,
) -> i32 {
    let user: DBUser = DBUser {
        id: -1, //doesn't matter in insertion
        display_name: display_name,
        avatar_url: avatar_url,
        user_name: Uuid::new_v4().to_string(),
        last_online: Utc::now().to_string(),
        github_id: github_id,
        discord_id: discord_id,
        github_access_token: gh_access,
        discord_access_token: dc_access,
        banned: false,
        banned_reason: "not banned".to_string(),
        bio: "This user is a myth!".to_string(),
        contributions: 0,
        banner_url: "".to_string(),
    };
    let user_id = data_capturer::capture_new_user(execution_handler, &user).await;
    return user_id;
}

async fn check_user_id_and_continue(
    user_id: i32,
    dc_access_token: String,
    gh_access_token: String,
    action_was_successful: &mut bool,
    execution_handler: &mut ExecutionHandler,
    discord_id: String,
    github_id: String,
) {
    // if we get a -1 that means that the already exist
    // so we need to grab this user and update its access token
    if user_id == -1 {
        let user_result = execution_handler
            .select_user_by_discord_or_github_id(discord_id, github_id)
            .await;

        if user_result.is_ok() {
            let user_rows = user_result.unwrap();
            grab_user_and_update_tokens(
                user_rows,
                dc_access_token,
                gh_access_token,
                action_was_successful,
                execution_handler,
            )
            .await;
        } else {
            *action_was_successful = false;
        }

    //if we get a number that isn't a -2 or -1 as
    //the user_id, that means we have successfully
    //inserted a new user with the initial access token
    } else if user_id != -2 {
        *action_was_successful = true;
    }
    //-2 means we have ran into an unexpected error.
    else {
        *action_was_successful = false;
    }
}

async fn grab_user_and_update_tokens(
    user_rows: Vec<Row>,
    dc_access_token: String,
    gh_access_token: String,
    action_was_successful: &mut bool,
    execution_handler: &mut ExecutionHandler,
) {
    //make sure there is only one user
    if user_rows.len() == 1 {
        //extract the user id and update the tokens
        let user_id: i32 = user_rows[0].get(0);
        let update_dc_token_result = execution_handler
            .update_discord_access_token(dc_access_token, &user_id)
            .await;
        let update_gh_token_result = execution_handler
            .update_github_access_token(gh_access_token, &user_id)
            .await;
        //ensure the tokens get updated
        if update_dc_token_result.is_ok()
            && update_gh_token_result.is_ok()
            && update_dc_token_result.unwrap() == 1
            && update_gh_token_result.unwrap() == 1
        {
            *action_was_successful = true;
        } else {
            *action_was_successful = false;
        }
    } else {
        *action_was_successful = false;
    }
}
