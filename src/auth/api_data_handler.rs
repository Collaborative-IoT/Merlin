/*
Handles the json gathered from api end points 
by using access tokens.
*/

use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::communication::{data_capturer, data_fetcher};

pub async fn parse_and_capture_discord_user_data(data:serde_json::Value){

    
}

pub async fn parse_and_capture_github_user_data(data:serde_json::Value){

}