use crate::auth::oauth_locations;
use crate::reqwest;
use crate::warp::http::Uri;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]

/*
Handles the authentication logic for gathering basic data
and constructing urls for callbacks.

Logic could be reduced, but is better to clearly show endpoints.
*/

pub struct CodeParams {
    pub code: String,
}

pub async fn gather_tokens_and_construct_save_url_discord(code: String) -> Result<Uri, Error> {
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

    //make sure response is correct and construct url for client side saving
    if discord_token_gather_is_valid(&result) {
        let access_token: String = result["access_token"].to_owned().to_string();
        let refresh_token: String = result["refresh_token"].to_owned().to_string();
        let discord_auth_callback_route_url: Uri =
            oauth_locations::save_tokens_location(access_token, refresh_token)
                .parse()
                .unwrap();
        return Ok(discord_auth_callback_route_url);
    } else {
        let failed_auth_location: Uri = oauth_locations::error_auth_location().parse().unwrap();
        return Ok(failed_auth_location);
    }
}

pub async fn gather_tokens_and_construct_save_url_github(code: String) -> Result<Uri, Error> {
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

    if github_token_gather_is_valid(&json_value) {
        let access_token: String = json_value["access_token"].to_owned().to_string();
        let refresh_token: String = " invalidforplatform ".to_owned().to_string();
        let github_auth_callback_route_url: Uri =
            oauth_locations::save_tokens_location(access_token, refresh_token)
                .parse()
                .unwrap();
        return Ok(github_auth_callback_route_url);
    } else {
        let failed_auth_location: Uri = oauth_locations::error_auth_location().parse().unwrap();
        return Ok(failed_auth_location);
    }
}

pub async fn gather_user_basic_data_discord(
    access_token: String,
) -> Result<serde_json::Value, Error> {
    let base_url = "https://discord.com/api/v6/users/@me";
    let bearer_token: String = format!("Bearer {}", &access_token[1..access_token.len() - 1]); //removes double quotes
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
) -> Result<(), Error> {
    let base_url = "https://api.github.com/user";
    let bearer_token: String = format!("token {}", &access_token[1..access_token.len() - 1]); //removes double quotes
    let client = reqwest::Client::new();
    let result = client
        .get(base_url)
        .header("Authorization", bearer_token)
        .header("Accept", "application/json")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10; Win64; x64; rv:60.0) Gecko/20100101 Firefox/60.0")
        .send()
        .await?
        .text()
        .await?;
    println!("{:?}", result);
    return Ok(());
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
