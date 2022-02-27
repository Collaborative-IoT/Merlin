use crate::common::response_logic::send_to_requester_channel;
use crate::communication::types::{BasicResponse, GetFollowListResponse};
use crate::communication::types::{CommunicationRoom, RoomDetails, UserPreview};
use crate::state::state::ServerState;
use crate::state::types::Room;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub fn parse_peer_and_room_id(
    peer_id: &String,
    room_id: &String,
) -> Result<(i32, i32), std::num::ParseIntError> {
    let peer: i32 = peer_id.parse()?;
    let room: i32 = room_id.parse()?;
    return Ok((peer, room));
}

pub async fn send_follow_list(
    target: (bool, HashSet<i32>),
    server_state: &Arc<RwLock<ServerState>>,
    requester_id: i32,
    peer_id: i32,
) {
    let mut write_state = server_state.write().await;
    // if we encountered error getting the follow list from the db
    if target.0 == true {
        send_to_requester_channel(
            "issue with request".to_owned(),
            requester_id,
            &mut write_state,
            "invalid_request".to_owned(),
        );
    } else {
        let vec_user_ids: Vec<i32> = target.1.into_iter().collect();
        let response = GetFollowListResponse {
            user_ids: vec_user_ids,
            for_user: peer_id,
        };
        let response_str = serde_json::to_string(&response).unwrap();
        send_to_requester_channel(
            response_str,
            requester_id,
            &mut write_state,
            "follow_list_response".to_owned(),
        );
    }
}

/// Constructs communication rooms
/// communication rooms are rooms that differ from state
/// by containing user previews and other small pieces of data.
pub fn construct_communication_room(
    previews: HashMap<i32, UserPreview>,
    room_state: &Room,
    holder: &mut Vec<CommunicationRoom>,
    creator_id: i32,
    chat_mode: String,
) {
    let new_communication_room_details = RoomDetails {
        name: room_state.name.to_owned(),
        description: room_state.desc.to_owned(),
        chat_throttle: room_state.chat_throttle,
        is_private: room_state.public == false,
    };

    let new_communication_room = CommunicationRoom {
        details: new_communication_room_details,
        room_id: room_state.room_id,
        num_of_people_in_room: room_state.amount_of_users,
        voice_server_id: room_state.voice_server_id.to_owned(),
        creator_id: creator_id,
        people_preview_data: previews,
        auto_speaker_setting: room_state.auto_speaker,
        created_at: room_state.created_at.to_owned(),
        chat_mode: chat_mode,
    };

    holder.push(new_communication_room);
}
