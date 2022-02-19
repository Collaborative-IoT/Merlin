use crate::state::state_types::{ActiveRooms, ActiveUsers, IoTServerConnections, PeerMap};

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
}
