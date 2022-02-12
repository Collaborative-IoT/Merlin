use crate::communication::communication_types::BasicResponse;
use crate::state::state::ServerState;
use crate::vs_response::vs_response_types::VoiceServerResponse;
use crate::ws_fan::fan;

//used for basic events where
//the user_id is the only containing data
//needed for the other users in the room
//but the main details is needed to be sent
//to the user themselves.
pub async fn notify_user_and_room(
    response: VoiceServerResponse,
    state: &mut ServerState,
    op_code_for_other_users: String,
) {
    let room_id: i32 = response.d["roomId"].to_string().parse().unwrap();
    let user_id: i32 = response.uid.parse().unwrap();
    let basic_response_for_user = BasicResponse {
        response_op_code: response.op,
        response_containing_data: response.d.to_string(),
    };
    let basic_response_for_room = BasicResponse {
        response_op_code: op_code_for_other_users,
        response_containing_data: user_id.to_string(),
    };
    fan::broadcast_message_to_single_user(
        serde_json::to_string(&basic_response_for_user).unwrap(),
        state,
        &user_id,
    )
    .await;
    fan::broadcast_message_to_room_excluding_user(
        serde_json::to_string(&basic_response_for_room).unwrap(),
        state,
        room_id,
        user_id,
    )
    .await;
}

pub async fn notify_user_only(response: VoiceServerResponse, state: &mut ServerState) {
    let basic_response_for_user = BasicResponse {
        response_op_code: response.op,
        response_containing_data: response.d.to_string(),
    };
    let user_id: i32 = response.uid.parse().unwrap();
    fan::broadcast_message_to_single_user(
        serde_json::to_string(&basic_response_for_user).unwrap(),
        state,
        &user_id,
    )
    .await;
}

pub async fn notify_entire_room(response: serde_json::Value, state: &mut ServerState){
    let basic_response_for_user = BasicResponse {
        response_op_code: response["op"].to_string(),
        response_containing_data: response["d"].to_string(),
    };
    let room_id:i32 = response["rid"].to_string().parse().unwrap();

    fan::broadcast_message_to_room(serde_json::to_string(&basic_response_for_user).unwrap(), state, room_id).await;
}
