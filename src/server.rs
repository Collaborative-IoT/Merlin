use futures_util::{SinkExt, StreamExt,stream::SplitSink};
use log::*;
use std::{net::SocketAddr, time::Duration, sync::{Arc, Mutex},collections::HashMap};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error,WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Result};
use crate::State::state::ServerState;
pub struct Server;

impl Server{

    pub async fn accept_connection(stream: TcpStream,state: Arc<Mutex<ServerState>>){
        if let Err(e) = Server::handle_connection(stream,state).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {}", err),
            }
        }
    }

    async fn handle_connection(stream: TcpStream,state:Arc<Mutex<ServerState>>) -> Result<()> {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let mut interval = tokio::time::interval(Duration::from_millis(1000));
        state.lock().unwrap().peer_map.lock().unwrap().insert("test".to_string(), ws_sender);
        // Echo incoming WebSocket messages and send a message periodically every second.

        loop {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            let msg = msg?;
                            if msg.is_text(){
                                let data = msg.to_text().unwrap().to_string();
                                
                                //ws_sender.send(msg).await?;
                            } else if msg.is_close() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }

}