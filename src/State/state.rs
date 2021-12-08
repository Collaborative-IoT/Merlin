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

    pub fn remove_from_map<V>(&mut self, map:&mut HashMap<String,V>, key:String)->bool{
        if map.contains_key(key){
            map.remove(key);
            return true;
        }
        else{
            warn!("Attempt to add remove a key that doesn't exist `{}`",key);
            return false;
        }
    }

    pub fn add_to_map<V>(&mut self, map:&mut HashMap<String,V>, key:String, data:V)->bool{
        if !map.contains_key(key){
            map.insert(key,data);
            return true;
        }
        else{
            warn!("There was an attempt to add a key that exists `{}`",key);
            return false;
        }
    }

    pub fn remove_from_set(&mut self, set:&mut HashSet<String>, key:String)->bool{
        if set.contains(key){
            set.remove(key);
            return true;
        }
        else{
            warn!("There was an attempt to remove a key that doesn't exist");
            return false;
        }
    }

    pub fn add_to_set(&mut self,set:&mut HashSet<String>, key:String)->bool{
        if !set.contains(key){
            set.insert(key);
            return true;
        }
        else{
            info("There was an attempt to add a key that exists");
            return false;
        }
    }
}