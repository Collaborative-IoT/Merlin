use chrono::{DateTime, Utc};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};
//.keys().cloned().collect::<Vec<_>>();

pub struct Board {
    pub room_id: String,
    pub owner_user_id: String,
    //Those granted permissions by the owner
    pub users_with_permission: HashSet<String>,
}

pub struct User {
    pub avatar_url: String,
    pub display_name: String,
    pub user_name: String,
    pub last_online: DateTime<Utc>,
    pub muted: bool,
    pub deaf: bool,
    pub ip: String,
    pub bio: String,
    pub banner_url: String,
}

pub struct Room {
    pub room_id: i32,
    pub muted: HashSet<i32>,
    pub voice_server_id: String,
    pub deaf: HashSet<i32>,
    pub user_ids: HashSet<i32>,
    pub public: bool,
    pub auto_speaker: bool,
}

//IoTServerConnectionId -> Permissions for the connection(represented as the board)
//Read the docs about the Board concept
pub type IoTServerConnections = HashMap<String, Board>;

//user id -> write connection.
//broadcasting requires you to acquire the lock of the mutex
//to access peer connections.
pub type PeerMap = HashMap<i32, mpsc::UnboundedSender<Message>>;

//current connected and authed users
pub type ActiveUsers = HashMap<i32, User>;

//room collection
pub type ActiveRooms = HashMap<i32, Room>;
