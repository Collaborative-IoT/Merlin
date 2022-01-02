extern crate chrono;
extern crate warp;
mod test;

pub mod state {
    pub mod state;
    pub mod state_types;
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
    pub mod communication_router;
    pub mod communication_types;
    pub mod data_capturer;
    pub mod data_fetcher;
    pub mod permission_configs;
    pub mod test;
    pub mod tests {
        pub mod capture_and_fetch;
    }
}

pub mod auth{
    pub mod authentication_handler;
    pub mod oauth_locations;
}

pub mod server;

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
