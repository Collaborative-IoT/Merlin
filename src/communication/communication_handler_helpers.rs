use crate::state::state::ServerState;

pub fn web_rtc_request_is_valid(
    server_state: &ServerState,
    request_data: &serde_json::Value,
    requester_id: &i32,
) -> bool {
    //make sure the user in the request value is the user requesting
    //make sure the room exists
    //make sure the user requesting is in the room their requesting for
    if request_data["peerId"] == serde_json::Value::Null
        || request_data["roomId"] == serde_json::Value::Null
    {
        return false;
    }

    let user_id_from_request_result = request_data["peerId"].to_string().parse();
    let room_id_from_request_result = request_data["roomId"].to_string().parse();

    if !user_id_from_request_result.is_ok() || !room_id_from_request_result.is_ok() {
        return false;
    }

    let user_id_from_request: i32 = user_id_from_request_result.unwrap();
    let room_id_from_request: i32 = room_id_from_request_result.unwrap();

    if &user_id_from_request == requester_id
        && server_state.rooms.contains_key(&room_id_from_request)
        && server_state
            .rooms
            .get(&room_id_from_request)
            .unwrap()
            .user_ids
            .contains(&user_id_from_request)
    {
        return true;
    }

    return false;
}
