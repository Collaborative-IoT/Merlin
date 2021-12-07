use futures_util::{SinkExt, StreamExt,stream::SplitSink};
use log::*;
use std::{net::SocketAddr, time::Duration, sync::{Arc, Mutex},collections::{HashSet,HashMap}};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error,WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Result};

pub struct Board{
    room_id:String,
    owner_user_id:String,
    //Those granted permissions by the owner
    users_with_permission: HashSet<String>
}

pub struct User{
    avatar_url:String,
    followers:HashSet<String>,
    following:HashSet<String>,
    blocked_users:HashSet<String>
}

pub struct Room{
    room_id:String,
    owner_user_id:String,
    speakers:HashSet<String>,
    user_ids:HashSet<String>,
    public:bool,
    blocked_users:HashSet<String>,
    blocked_ips:HashSet<String>
}

//IoTServerConnectionId -> Permissions for the connection(represented as the board)
//Read the docs about the Board concept
pub type IoTServerConnections = Arc<Mutex<HashMap<String,Board>>>;

//user id -> write connection.
//broadcasting requires you to acquire the lock of the mutex
//to access peer connections.
pub type PeerMap =  Arc<Mutex<HashMap<String,
    SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>>>>;

//current connected and authed users
pub type ActiveUsers = Arc<Mutex<HashMap<String,User>>>;

//room collection
pub type ActiveRooms = Arc<Mutex<HashMap<String,Room>>>