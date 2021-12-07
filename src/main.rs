mod server;
use server::Server;

use std::{env, io::Error};

use futures_util::{future, StreamExt, TryStreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};


#[tokio::main]
async fn main() -> Result<(), Error> {

    let _ = env_logger::try_init();
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(Server.accept_connection(stream));
    }
    Ok(())
}
