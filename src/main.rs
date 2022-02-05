extern crate chrono;
extern crate reqwest;
extern crate warp;
#[allow(dead_code)]
mod test;

pub mod state {
    pub mod state;
    pub mod state_helpers;
    pub mod state_types;
}

pub mod common {
    pub mod common_response_logic;
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
    pub mod communication_handler;
    pub mod communication_handler_helpers;
    pub mod communication_router;
    pub mod communication_types;
    pub mod data_capturer;
    pub mod data_fetcher;
    pub mod test;
    pub mod tests {
        pub mod capture_and_fetch;
        pub mod communication_handler_test_helpers;
        pub mod communication_handler_tests;
    }
}

pub mod auth {
    pub mod api_data_handler;
    pub mod authentication_handler;
    pub mod oauth_locations;
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
    pub mod permission_configs;
    pub mod room_handler;
}

#[tokio::main]
async fn main() {
    print_start();
    server::start_server(([127, 0, 0, 1], 3030)).await;
    //trace!("a trace example");
    //debug!("deboogging");

    //warn!("o_O");
    //error!("boom");
}

fn print_start() {
    //info!("Started....");
}
