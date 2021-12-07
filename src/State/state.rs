use futures_util::{SinkExt, StreamExt,stream::SplitSink};
use log::*;
use std::{net::SocketAddr, time::Duration, sync::{Arc, Mutex},collections::HashMap};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error,WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Result};

mod state_types;
use state_types::{IoTServerConnections,PeerMap,User};

pub struct ServerState{
    iot_server_connections : IoTServerConnections,
    peer_map : PeerMap,
}

impl struct ServerState{
    pub fn new()-> Self{
        Self{
            iot_server_connections:IoTServerConnections::new(Mutex::new(HashMap::new())),
            peer_map:PeerMap::new(Mutex::new(HashMap::new())),
            active_users:User::new(Mutex::new(HashMap::new)));
        }
    }
}