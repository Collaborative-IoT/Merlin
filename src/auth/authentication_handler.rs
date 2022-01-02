use crate::reqwest;
use std::env;
pub struct Tokens {
    access: String,
    refresh: String,
}

pub async fn authenticate_with_discord(token: String) -> bool {
    return true;
}

pub async fn authenticate_with_github(token: String) -> bool {
    return false;
}

pub async fn gather_tokens_for_discord(code: String) {
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
    let result = client.post(base_url).query(&params).send().await.unwrap().text().await.unwrap();
    println!("{:?}",result);
}
