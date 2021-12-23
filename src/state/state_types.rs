use chrono::{DateTime, Utc};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{Message, Result};
use tokio_tungstenite::WebSocketStream;
//.keys().cloned().collect::<Vec<_>>();

pub struct Board {
    room_id: String,
    owner_user_id: String,
    //Those granted permissions by the owner
    users_with_permission: HashSet<String>,
}

pub struct User {
    avatar_url: String,
    display_name: String,
    user_name: String,
    last_online: DateTime<Utc>,
    muted: bool,
    deaf: bool,
    ip: String,
    bio: String,
    banner_url: String,
}

pub struct Room {
    room_id: i32,
    muted: HashSet<String>,
    voice_server_id: String,
    deaf: HashSet<String>,
    user_ids: HashSet<String>,
    public: bool,
    auto_speaker: bool
}

//IoTServerConnectionId -> Permissions for the connection(represented as the board)
//Read the docs about the Board concept
pub type IoTServerConnections = HashMap<String, Board>;

//user id -> write connection.
//broadcasting requires you to acquire the lock of the mutex
//to access peer connections.
pub type PeerMap = HashMap<String, SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>>;

//current connected and authed users
pub type ActiveUsers = HashMap<i32, User>;

//room collection
pub type ActiveRooms = HashMap<i32, Room>;
