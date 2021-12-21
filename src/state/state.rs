use crate::state::state_types::{ActiveRooms, ActiveUsers, IoTServerConnections, PeerMap, User};
use std::collections::{HashMap, HashSet};

pub struct ServerState {
    pub iot_server_connections: IoTServerConnections,
    pub peer_map: PeerMap,
    pub rooms: ActiveRooms,
    pub active_users: ActiveUsers,
}

//Holds all server memory state
impl ServerState {
    pub fn new() -> Self {
        Self {
            iot_server_connections: IoTServerConnections::new(),
            peer_map: PeerMap::new(),
            active_users: ActiveUsers::new(),
            rooms: ActiveRooms::new(),
        }
    }

    pub fn remove_from_map<V>(&mut self, map: &mut HashMap<String, V>, key: String) -> bool {
        if map.contains_key(&key) {
            map.remove(&key);
            return true;
        } else {
            return false;
        }
    }

    pub fn add_to_map<V>(&mut self, map: &mut HashMap<String, V>, key: String, data: V) -> bool {
        if !map.contains_key(&key) {
            map.insert(key, data);
            return true;
        } else {
            return false;
        }
    }

    pub fn remove_from_set(&mut self, set: &mut HashSet<String>, key: String) -> bool {
        if set.contains(&key) {
            set.remove(&key);
            return true;
        } else {
            return false;
        }
    }

    pub fn add_to_set(&mut self, set: &mut HashSet<String>, key: String) -> bool {
        if !set.contains(&key) {
            set.insert(key);
            return true;
        } else {
            return false;
        }
    }
}
