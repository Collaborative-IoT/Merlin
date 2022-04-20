use std::collections::HashMap;

use crate::state::types::{ActiveRooms, ActiveUsers, IoTServerConnections, PeerMap};

use super::owner_queue::OwnerQueue;

pub struct ServerState {
    pub iot_server_connections: IoTServerConnections,
    pub peer_map: PeerMap,
    pub rooms: ActiveRooms,
    pub active_users: ActiveUsers,
    pub owner_queues: HashMap<i32, OwnerQueue>,
}

//Holds all server memory state
impl ServerState {
    pub fn new() -> Self {
        Self {
            iot_server_connections: IoTServerConnections::new(),
            peer_map: PeerMap::new(),
            active_users: ActiveUsers::new(),
            rooms: ActiveRooms::new(),
            owner_queues: HashMap::new(),
        }
    }
}
