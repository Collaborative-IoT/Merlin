mod server;
extern crate chrono;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use std::sync::{Arc, Mutex};
use std::{env, io::Error};
use tokio::net::TcpListener;
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    print_start();
    //trace!("a trace example");
    //debug!("deboogging");

    //warn!("o_O");
    //error!("boom");
}

fn print_start() {
    info!("Started....");
}
