use std::collections::HashMap;

use crate::state::types::{ActiveRooms, ActiveUsers, PeerMap};

use super::owner_queue::OwnerQueue;

pub struct ServerState {
    pub peer_map: PeerMap,
    pub rooms: ActiveRooms,
    pub active_users: ActiveUsers,
    pub owner_queues: HashMap<i32, OwnerQueue>,
    /// maps external iot server ids
    /// to their local rooms
    pub external_servers: HashMap<String, i32>,
}

//Holds all server memory state
impl ServerState {
    pub fn new() -> Self {
        Self {
            peer_map: PeerMap::new(),
            active_users: ActiveUsers::new(),
            rooms: ActiveRooms::new(),
            owner_queues: HashMap::new(),
            external_servers: HashMap::new(),
        }
    }
}
