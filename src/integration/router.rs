use std::collections::HashSet;

use serde_json::Value;

use crate::{
    communication::types::{BasicResponse, NewIoTServer},
    state::state::ServerState,
    ws_fan,
};

pub async fn route_msg(msg: String, state: &mut ServerState) {
    println!("{}", msg);
    let msg: serde_json::Value = serde_json::from_str(&msg).unwrap();
    if msg["passed_auth"] != Value::Null {
        check_auth_and_insert(msg, state).await;
    }
}

pub async fn check_auth_and_insert(msg: serde_json::Value, state: &mut ServerState) {
    let passed_auth = msg["passed_auth"].as_bool();
    if let Some(passed) = passed_auth {
        // If we passed auth insert our new server connection
        if passed {
            let external_server_id = msg["server_id"].to_string();
            let user_id: Result<i32, _> = msg["user_id"].to_string().parse();
            if let Ok(user_id) = user_id {
                if let Some(user) = state.active_users.get(&user_id) {
                    let room_id = user.current_room_id.clone();
                    if let Some(room) = state.rooms.get_mut(&room_id) {
                        room.iot_server_connections.insert(
                            external_server_id.clone(),
                            crate::state::types::Board {
                                room_id: room_id,
                                owner_user_id: user_id.clone(),
                                users_with_permission: HashSet::new(),
                                external_server_id: external_server_id.clone(),
                            },
                        );
                        state
                            .external_servers
                            .insert(external_server_id.clone(), room.room_id.clone());
                        let basic_response = BasicResponse {
                            response_op_code: "new_iot_server".to_owned(),
                            response_containing_data: serde_json::to_string(&NewIoTServer {
                                external_id: external_server_id,
                                owner_id: user_id,
                            })
                            .unwrap(),
                        };
                        // Let the room know there is a new IoT Server
                        ws_fan::fan::broadcast_message_to_room(
                            serde_json::to_string(&basic_response).unwrap(),
                            state,
                            room_id,
                        )
                        .await;
                    }
                }
            }
        }
    }
}
