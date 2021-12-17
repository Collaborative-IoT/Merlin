mod server;
use server::Server;
extern crate chrono;
use std::{env, io::Error};
use std::sync::{Mutex,Arc};
use tokio::net::{TcpListener};
pub mod state{
    pub mod state;
    pub mod state_types;
}

pub mod data_store{
    pub mod db_models;
    pub mod delete_queries;
    pub mod creation_queries;
    pub mod insert_queries;
    pub mod select_queries;
    pub mod sql_execution_handler;
    pub mod update_queries;
    pub mod test;
    pub mod tests{
        pub mod blocks;
        pub mod follower;
        pub mod room;
        pub mod user;
    }
}

pub mod communication{
    pub mod communication_handler;
    pub mod communication_router;
    pub mod communication_types;
    pub mod db_to_json;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    
}
