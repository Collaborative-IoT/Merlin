
use crate::vs_response::vs_response_types::{VoiceServerResponse};
use crate::state::state::ServerState;
use crate::communication::communication_types::BasicResponse;
use crate::ws_fan::fan;

pub async fn notify_user_left(
    response:VoiceServerResponse, 
    state: &mut ServerState){
        let room_id:i32 = response.d["roomId"].to_string().parse().unwrap();
        let user_id:i32 = response.uid.parse().unwrap();
        let basic_response_for_user = BasicResponse{
            response_op_code: response.op,
            response_containing_data: response.d.to_string()
        };
        let basic_response_for_room = BasicResponse{
            response_op_code:"user_left_room".to_owned(),
            response_containing_data:user_id.to_string()
        };
        fan::broadcast_message_to_single_user(serde_json::to_string(&basic_response_for_user).unwrap(), state, &user_id).await;
        fan::broadcast_message_to_room_excluding_user(serde_json::to_string(&basic_response_for_room).unwrap(), state, room_id, user_id).await;
}
