extern crate chrono;
extern crate reqwest;
extern crate warp;
use std::env;
#[allow(dead_code)]
mod test;

pub mod state {
    pub mod owner_queue;
    pub mod state;
    pub mod types;
}

pub mod common {
    pub mod response_logic;
}

pub mod data_store {
    pub mod creation_queries;
    pub mod db_models;
    pub mod delete_queries;
    pub mod insert_queries;
    pub mod select_queries;
    pub mod sql_execution_handler;
    pub mod test;
    pub mod update_queries;
    pub mod tests {
        pub mod blocks;
        pub mod follower;
        pub mod room;
        pub mod user;
    }
}

pub mod communication {
    pub mod data_capturer;
    pub mod data_fetcher;
    pub mod handler;
    pub mod helpers;
    pub mod router;
    pub mod test;
    pub mod types;
    pub mod tests {
        pub mod capture_and_fetch;
        pub mod hand_tests;
        pub mod helpers;
        pub mod mod_tests;
        pub mod owner_tests;
        pub mod standard_tests;
        pub mod tests;
    }
}

pub mod auth {
    pub mod api_data_handler;
    pub mod authentication_handler;
    pub mod oauth_locations;
    pub mod ws_auth_handler;
}

pub mod server;
pub mod ws_fan {
    pub mod fan;
}
pub mod rabbitmq {
    pub mod rabbit;
    pub mod test;
}
pub mod rooms {
    pub mod handler;
    pub mod permission_configs;
}

pub mod vs_response {
    pub mod handler;
    pub mod router;
    pub mod types;
}

pub mod logging {
    pub mod console;
}

#[tokio::main]
async fn main() {
    //make a clean slate for our ascii art
    logging::console::clear_terminal();

    //print our ascii across win systems and unix systems
    let unix_path =
        env::current_dir().unwrap().to_str().unwrap().to_owned() + "/src/ascii_art/knightascii.txt";
    let unix_result = logging::console::log_start(unix_path);
    if unix_result.is_err() {
        let win_path = env::current_dir().unwrap().to_str().unwrap().to_owned()
            + r#"\src\ascii_art\knightascii.txt"#;
        logging::console::log_start(win_path).unwrap();
    }
    //start
    server::start_server(([127, 0, 0, 1], 3030)).await;
}
