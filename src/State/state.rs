use futures_util::{SinkExt, StreamExt,stream::SplitSink};
use log::*;
use std::{net::SocketAddr, time::Duration, sync::{Arc, Mutex},collections::HashMap};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error,WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Result};

mod state_types;
use state_types::{IoTServerConnections,PeerMap,User,ActiveUsers,ActiveRooms};

pub struct ServerState{
    iot_server_connections : IoTServerConnections,
    peer_map : PeerMap,
}

struct Removal<K,V>
    map:HashMap<K,V>,
    value:String
}

//Holds all server memory state
impl struct ServerState{
    pub fn new()-> Self{
        Self{
            iot_server_connections:IoTServerConnections::new(Mutex::new(HashMap::new())),
            peer_map:PeerMap::new(Mutex::new(HashMap::new())),
            active_users:ActiveUsers::new(Mutex::new(HashMap::new())),
            rooms:ActiveRooms::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn remove<V>(&mut self, map:HashMap<String,V>, key:String){
        if map.contains_key(key){
            map.remove(key);
        }
        else{
            info!("Attempt to add remove a key that doesn't exist `{}`",key);
        }
    }

    pub fn add<V>(&mut self, map:HashMap<String,V>, key:String, data:V){
        if !map.contains_key(key){
            map.insert(key,data);
        }
        else{
            info!("There was an attempt to add a key that exists `{}`",key);
        }
    }
}