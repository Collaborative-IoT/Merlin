use crate::auth::api_data_handler;
use crate::auth::oauth_locations;
use crate::reqwest;
use crate::warp::http::Uri;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::env;

use crate::data_store::sql_execution_handler::ExecutionHandler;
use futures::lock::Mutex;
use std::sync::Arc;

/*
Handles the authentication logic for gathering basic data
and constructing urls for callbacks.

Logic could be reduced, but is better to clearly show endpoints.
*/
#[derive(Deserialize, Serialize)]
pub struct CodeParams {
    pub code: String,
}

pub async fn gather_tokens_and_construct_save_url_discord(
    code: String,
    execution_handler: Arc<Mutex<ExecutionHandler>>,
) -> Result<Uri, Error> {
    let base_url = "https://discordapp.com/api/oauth2/token";
    let base_api_url = env::var("BASE_API_URL").unwrap();
    let client_id = env::var("DC_CLIENT_ID").unwrap();
    let client_secret = env::var("DC_CLIENT_SECRET").unwrap();
    let our_redirect_url = format!("{}/api/discord/auth-callback", base_api_url);
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("grant_type", "authorization_code".to_owned()),
        ("redirect_uri", our_redirect_url),
    ];

    //send request to get access/refresh tokens
    let result = client
        .post(base_url)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;
    let failed_auth_location: Uri = oauth_locations::error_auth_location().parse().unwrap();

    if discord_token_gather_is_valid(&result) {
        let access_token: String = result["access_token"].to_owned().to_string();
        let refresh_token: String = result["refresh_token"].to_owned().to_string();

        //make api request and grab user json data
        //
        //remove double quotes from each side of the access token, 
        //it gets parsed as "fdsfsfdf" instead of fdsfsfdf
        let basic_data_gather_result =
            gather_user_basic_data_discord(access_token[1..access_token.len() - 1].to_string()).await;

        //if the json data was good, meaning no errors from api call
        if basic_data_gather_result.is_ok() {
            let basic_data = basic_data_gather_result.unwrap();
            let action_was_successful = api_data_handler::parse_and_capture_discord_user_data(
                basic_data,
                execution_handler,
                access_token.to_owned(),
            )
            .await;
            //if did not successfully create a new user/update the old user's access token
            if action_was_successful == false {
                return Ok(failed_auth_location);
            }
        } else {
            return Ok(failed_auth_location);
        }
        //we encountered 0 issues, we want to save the new set of tokens
        //on the client side by redirecting them to a
        //page made for stripping and saving access tokens
        let discord_auth_callback_route_url: Uri =
            oauth_locations::save_tokens_location(access_token, refresh_token)
                .parse()
                .unwrap();
        return Ok(discord_auth_callback_route_url);
    } else {
        return Ok(failed_auth_location);
    }
}

pub async fn gather_tokens_and_construct_save_url_github(
    code: String,
    execution_handler: Arc<Mutex<ExecutionHandler>>,
) -> Result<Uri, Error> {
    let base_url = "https://github.com/login/oauth/access_token";
    let client_id = env::var("GH_CLIENT_ID").unwrap();
    let client_secret = env::var("GH_CLIENT_SECRET").unwrap();
    let base_api_url = env::var("BASE_API_URL").unwrap();
    let our_redirect_url = format!("{}/api/github/auth-callback", base_api_url);
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", our_redirect_url),
    ];

    let result = client
        .post(base_url)
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?
        .text()
        .await?;
    //comes with surrounding double quotes
    let json_value: serde_json::Value = serde_json::from_str(&result).unwrap();
    let failed_auth_location: Uri = oauth_locations::error_auth_location().parse().unwrap();

    if github_token_gather_is_valid(&json_value) {
        let access_token: String = json_value["access_token"].to_owned().to_string();
        let refresh_token: String = " invalidforplatform ".to_owned().to_string();

        //make api request and grab user json data
        //
        //remove double quotes from each side of the access token, 
        //it gets parsed as "fdsfsfdf" instead of fdsfsfdf
        let basic_data_gather_result = gather_user_basic_data_github(access_token[1..access_token.len() - 1].to_string()).await;

        //if the json data was good, meaning no errors from api call
        if basic_data_gather_result.is_ok() {
            let basic_data = basic_data_gather_result.unwrap();
            let action_was_successful = api_data_handler::parse_and_capture_github_user_data(
                basic_data,
                execution_handler,
                access_token.to_owned(),
            )
            .await;

            //if did not successfully create a new user/update the old user's access token
            if action_was_successful == false {
                return Ok(failed_auth_location);
            };
        } else {
            return Ok(failed_auth_location);
        }

        //we encountered 0 issues, we want to save the new set of tokens
        //on the client side by redirecting them to a
        //page made for stripping and saving access tokens
        let github_auth_callback_route_url: Uri =
            oauth_locations::save_tokens_location(access_token, refresh_token)
                .parse()
                .unwrap();
        return Ok(github_auth_callback_route_url);
    } else {
        return Ok(failed_auth_location);
    }
}

pub async fn exchange_discord_refresh_token_for_access(
    refresh_token: String,
) -> Result<serde_json::Value, Error> {
    //https://discord.com/api/v8/oauth2/token
    let base_url = "https://discord.com/api/v8/oauth2/token";
    let client_id = env::var("DC_CLIENT_ID").unwrap();
    let client_secret = env::var("DC_CLIENT_SECRET").unwrap();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "refresh_token".to_owned()),
        (
            "refresh_token",
            refresh_token,
        ),
    ];
    let client = reqwest::Client::new();
    //send request to get access/refresh tokens
    let result = client
        .post(base_url)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", result);
    return Ok(result);
}

pub async fn gather_user_basic_data_discord(
    access_token: String,
) -> Result<serde_json::Value, Error> {
    let base_url = "https://discord.com/api/v6/users/@me";
    let bearer_token: String = format!("Bearer {}", access_token); 
    println!("{}",bearer_token);
    let client = reqwest::Client::new();
    let result: serde_json::Value = client
        .get(base_url)
        .header("Authorization", bearer_token)
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", result);
    return Ok(result);
}

pub async fn gather_user_basic_data_github(
    access_token: String,
) -> Result<serde_json::Value, Error> {
    let base_url = "https://api.github.com/user";
    let bearer_token: String = format!("token {}", access_token); //removes double quotes
    let client = reqwest::Client::new();
    let result = client
        .get(base_url)
        .header("Authorization", bearer_token)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10; Win64; x64; rv:60.0) Gecko/20100101 Firefox/60.0",
        )
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", result);
    return Ok(result);
}

pub fn discord_token_gather_is_valid(result: &serde_json::Value) -> bool {
    if result["access_token"] != serde_json::Value::Null
        && result["refresh_token"] != serde_json::Value::Null
        && result["error"] == serde_json::Value::Null
    {
        return true;
    } else {
        return false;
    }
}

pub fn github_token_gather_is_valid(result: &serde_json::Value) -> bool {
    if result["access_token"] != serde_json::Value::Null {
        return true;
    } else {
        return false;
    }
}
