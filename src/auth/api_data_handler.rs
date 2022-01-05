/*
Handles the json gathered from api end points
by using access tokens.
*/
use crate::communication::{data_capturer, data_fetcher};
use crate::data_store::db_models::DBUser;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

//grabs user api data and
//creates new user if it doesn't exist
pub async fn parse_and_capture_discord_user_data(
    data: serde_json::Value,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    access_token: String,
) {
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
        let avatar_url =
            construct_discord_image_url(fixed_dc_user_id.as_str(), fixed_dc_avatar_id.as_str());
        let mut handler = execution_handler.lock().unwrap();
        let pre_check_result = handler
            .select_user_by_discord_or_github_id(fixed_dc_user_id.to_owned(), "-1".to_string())
            .await;
        if pre_check_result.is_ok() && pre_check_result.unwrap().len() == 0 {
            //using discord username as our initial display name
            generate_and_capture_new_user(
                fixed_dc_user_id,
                "-1".to_string(),
                avatar_url,
                fixed_dc_username,
                "-1".to_owned(),
                access_token,
            )
        }
    }
}

pub async fn parse_and_capture_github_user_data(data: serde_json::Value) {}

pub fn construct_discord_image_url(discord_user_id: &str, discord_user_avatar_id: &str) -> String {
    return format!(
        "https://cdn.discordapp.com/avatars/{}/{}.png",
        discord_user_id, discord_user_avatar_id
    );
}

pub fn discord_user_request_is_valid(data: &serde_json::Value) -> bool {
    if data["avatar"] != serde_json::Value::Null
        && data["id"] != serde_json::Value::Null
        && data["username"] != serde_json::Value::Null
    {
        return true;
    } else {
        return false;
    }
}

pub fn generate_and_capture_new_user(
    discord_id: String,
    github_id: String,
    avatar_url: String,
    display_name: String,
    gh_access: String,
    dc_access: String,
) {
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
        banned_reason: "not banned",
        bio: "This user is a myth!".to_string(),
        contributions: 40,
        banner_url: "".to_string(),
    };
}
