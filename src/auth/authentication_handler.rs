use crate::reqwest;
use warp::reply::Reply;
use serde::{Deserialize, Serialize};
use std::env;
use crate::auth::oauth_locations;
use crate::warp::http::Uri;

pub struct Tokens {
    access: String,
    refresh: String,
}

#[derive(Deserialize, Serialize)]
pub struct CodeParams {
   pub code: String,
}

pub async fn authenticate_with_discord(token: String) -> bool {
    return true;
}

pub async fn authenticate_with_github(token: String) -> bool {
    return false;
}


pub async fn gather_tokens_and_save_url_for_discord(code: String) -> Uri{
    let base_url = "https://discordapp.com/api/oauth2/token";
    let client_id = env::var("DC_CLIENT_ID").unwrap();
    let client_secret = env::var("DC_CLIENT_SECRET").unwrap();
    let our_redirect_url = format!("{}/api/discord/auth-callback", base_url);
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("grant_type", "authorization_code".to_owned()),
        ("redirect_uri", our_redirect_url),
    ];

    let result = client
        .post(base_url)
        .query(&params)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let discord_auth_callback_route_url: Uri =
        oauth_locations::save_tokens_location("test".to_owned(), "test".to_owned())
            .parse()
            .unwrap();
    println!("{:?}", result);
    return discord_auth_callback_route_url;
}
