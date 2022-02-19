use crate::communication::communication_types::BasicResponse;
use crate::state::state::ServerState;
use warp::ws::Message;

pub fn send_to_requester_channel(
    response_data: String,
    requester_id: i32,
    server_state: &mut ServerState,
    op_code: String,
) {
    let response = BasicResponse {
        response_op_code: op_code,
        response_containing_data: response_data,
    };
    // TODO:handle error
    let gather_result = server_state.peer_map.get(&requester_id);
    if gather_result.is_some() {
        gather_result
            .unwrap()
            .send(Message::text(serde_json::to_string(&response).unwrap()))
            .unwrap_or_default();
    }
}
