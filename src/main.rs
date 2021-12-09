mod server;
use server::Server;
extern crate chrono;
use std::{env, io::Error};
use futures_util::{future, StreamExt, TryStreamExt};
use log::info;
use std::sync::{Mutex,Arc};
use tokio::net::{TcpListener, TcpStream};
pub mod State{
    pub mod state;
    pub mod state_types;
}

#[tokio::main]
async fn main() -> Result<(), Error> {

    let _ = env_logger::try_init();
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);
    let mut state_holder = Arc::new(Mutex::new(State::state::ServerState::new()));
    
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(Server::accept_connection(stream,state_holder.clone()));
    }
    Ok(())
}
