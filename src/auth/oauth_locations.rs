use std::env;

pub fn discord()->String{
    let client_id = env::var("DC_CLIENT_ID").unwrap();
    //could be http://localhost:3000 or production domain for api
    let base_url = env::var("BASE_API_URL").unwrap();
    let our_redirect_url = format!("{}/api/discord/auth-callback",base_url);
    let discord_redirect_url = format!("https://discord.com/api/oauth2/authorize?client_id=${}&redirect_uri=${}&response_type=code&scope=identify",
        client_id, our_redirect_url);

    return discord_redirect_url;
}

//where the api redirects you on the frontend client
//to save the tokens in the browser
//before you authenticate via websocket
pub fn save_tokens_location(access:String, refresh:String)->String{
    let base_ui_url = env::var("BASE_UI_URL").unwrap();
    let ui_url:String = format!("{}/save_info?refresh={}&access={}", base_ui_url,refresh,access);
    return ui_url;
}