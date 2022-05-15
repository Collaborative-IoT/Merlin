use std::collections::HashSet;

use serde_json::Value;

use crate::{
    communication::types::{BasicResponse, NewIoTServer, PassiveData},
    state::state::ServerState,
    ws_fan,
};

pub async fn route_msg(msg: String, state: &mut ServerState) {
    println!("{}", msg);
    let msg: serde_json::Value = serde_json::from_str(&msg).unwrap();
    if msg["passed_auth"] != Value::Null {
        check_auth_and_insert(msg, state).await;
    } else if msg["category"] != Value::Null {
        // using .to_string includes the trailing quotes, so your string be like
        // "example" instead of just -> example
        let category = msg["category"].to_string();
        let category_corrected = category[1..category.len() - 1].to_string();

        if category_corrected == "passive_data" {
            // Let the room know there is a new IoT Server
            let external_id = msg["server_id"].to_string();
            let external_id = external_id[1..external_id.len() - 1].to_string();

            let actual_passive_data = msg["data"].to_string();
            let actual_passive_data =
                actual_passive_data[1..actual_passive_data.len() - 1].to_string();
            //construct and notify everyone in that room of the new
            //passive data snapshot from the server.
            let passive_data = serde_json::to_string(&BasicResponse {
                response_op_code: "passive_data".to_owned(),
                response_containing_data: serde_json::to_string(&PassiveData {
                    external_id: external_id.clone(),
                    passive_data: actual_passive_data.clone(),
                })
                .unwrap(),
            })
            .unwrap();

            //should always be Some, but just extra safety
            if let Some(room_id) = state.external_servers.get(&external_id) {
                let cloned_room_id = room_id.clone();
                insert_new_passive_snapshot(
                    state,
                    actual_passive_data,
                    cloned_room_id,
                    external_id,
                );
                ws_fan::fan::broadcast_message_to_room(passive_data, state, cloned_room_id).await;
            }
        } else if msg["category"].to_string() == "disconnected" {
            let external_id = msg["server_id"].to_string();
            if let Some(room_id) = state.external_servers.get(&external_id) {
                if let Some(room) = state.rooms.get_mut(room_id) {
                    let cloned_room_id = room_id.clone();
                    drop(room_id);
                    room.iot_server_connections.remove(&external_id);
                    let basic_response = BasicResponse {
                        response_op_code: "hoi_server_disconnected".to_owned(),
                        response_containing_data: external_id.clone(),
                    };
                    // Let the room know a server was disconnected
                    ws_fan::fan::broadcast_message_to_room(
                        serde_json::to_string(&basic_response).unwrap(),
                        state,
                        cloned_room_id,
                    )
                    .await;
                    state.external_servers.remove(&external_id);
                }
            }
        }
    }
}

pub fn insert_new_passive_snapshot(
    state: &mut ServerState,
    passive_data: String,
    room_id: i32,
    external_id: String,
) {
    if let Some(room) = state.rooms.get_mut(&room_id) {
        if let Some(board) = room.iot_server_connections.get_mut(&external_id) {
            board.passive_data_snapshot = Some(passive_data);
        }
    }
}

pub async fn check_auth_and_insert(msg: serde_json::Value, state: &mut ServerState) {
    let passed_auth = msg["passed_auth"].as_bool();
    if let Some(passed) = passed_auth {
        // If we passed auth insert our new server connection
        if passed {
            let external_server_id = msg["server_id"].to_string();
            let external_server_id =
                external_server_id[1..external_server_id.len() - 1].to_string();
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
                                passive_data_snapshot: None,
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
