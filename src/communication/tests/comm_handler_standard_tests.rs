use crate::communication::communication_router;
use crate::communication::communication_types::{
    GenericRoomId, GenericRoomIdAndPeerId, VoiceServerCreateRoom,
};
use crate::communication::tests::comm_handler_test_helpers::helpers;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use lapin::Consumer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn test_users_can_get_top_rooms(
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    consume_channel: &mut Consumer,
) {
    //Before we make this request we need to remove
    //all 'mock users', so there is only user in the room
    //which is the room owner aka 33.
    //
    //So we will remove user 33,add a real user,
    //make the top rooms request, and then add 33 back since
    //they are the room owner.
    //
    //The reason this owner must be
    //removed is because the function being called to get
    //user previews takes all users in a room and search the
    //database for their user row. Mock users don't have a row.
    println!("testing getting top rooms");
    helpers::clear_message_that_was_fanned(vec![speaker_rx]).await;
    let new_user_id = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "qopkqwepokqw1321241".to_string(),
        "fwieopjj29024nsdocikndv0".to_string(),
    )
    .await
    .0;
    state
        .write()
        .await
        .rooms
        .get_mut(&3)
        .unwrap()
        .user_ids
        .remove(&33);
    let request = helpers::basic_request("get_top_rooms".to_string(), "".to_string());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();

    //make sure the response is correct
    let mock_communication_room =
        helpers::construct_top_room_response_for_test(new_user_id, state).await;
    let mock_communication_room_str = serde_json::to_string(&mock_communication_room).unwrap();
    helpers::grab_and_assert_request_response(
        speaker_rx,
        "top_rooms",
        &mock_communication_room_str,
    )
    .await;
    //cleanup
    let mut write_state = state.write().await;
    let room = write_state.rooms.get_mut(&3).unwrap();
    room.user_ids.insert(33);
    room.user_ids.remove(&new_user_id);
}

pub async fn test_getting_all_users_in_room(
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    consume_channel: &mut Consumer,
) {
    //clear the only mock user
    state
        .write()
        .await
        .rooms
        .get_mut(&3)
        .unwrap()
        .user_ids
        .remove(&33);
    let mut new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "qopk13414qwepo242345kqw1321241".to_string(),
        "fwieopjj291323123234024nsdocikndv0".to_string(),
    )
    .await;

    //we only need them to be in the room
    //to check if we are getting their data.
    let new_second_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "31123123qopk1231213414qwepo242345kqw1321241".to_string(),
        "fwieopjj291323121231231233234024nsdocikndv0".to_string(),
    )
    .await;
    let data_for_request = GenericRoomId { room_id: 3 };
    let request = helpers::basic_request(
        "gather_all_users_in_room".to_owned(),
        serde_json::to_string(&data_for_request).unwrap(),
    );
    communication_router::route_msg(
        request,
        new_user.0,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    //the target user id is the first
    let mock_response = helpers::construct_user_response_for_test(new_second_user.0);
    helpers::grab_and_assert_request_response(
        &mut new_user.1,
        "all_users_for_room",
        &mock_response,
    )
    .await;
    //cleanup
    let mut write_state = state.write().await;
    let room = write_state.rooms.get_mut(&3).unwrap();
    room.user_ids.remove(&new_user.0);
    room.user_ids.remove(&new_second_user.0);
    room.user_ids.insert(33);
}

//webrtc requests are dynamic because
//they send browser connection information
//to the voice server. So we will only test
//the invalid case, a webrtc request is invalid
//when there is no peerid and room id associated.
pub async fn test_invalid_webrtc_request<T: Serialize + DeserializeOwned>(
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
    incorrect_data: T,
) {
    //The three invalid cases include:
    //1.When a user sends a webrtc request for a different user
    //2.When a user doesn't include both peer/room id in their webrtc request
    //3.When a user sends a request for a room they aren't in
    //To be completely clear, user 33(we are mocking requests for)
    //is in room 3.
    println!("Testing webrtc invalid requests");
    let basic_request = helpers::basic_request(
        "@get-recv-tracks".to_string(),
        serde_json::to_string(&incorrect_data).unwrap(),
    );
    communication_router::route_msg(basic_request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(speaker_rx, "invalid_request", "issue with request")
        .await;
}

pub async fn test_creating_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
) {
    // Make sure users cannot create a room if they
    // are currently in a room.
    //
    // Each user has a current room id, which helps us
    // not have to search all rooms to find a user which
    // is inefficent. If a user has a room value of -1,
    // they are not in a room, but if they have a non-negative
    // room number they are in a room. This is handled by the
    // communication handler internally.

    // Set the mock user's room as 2(even though room 2
    // doesn't exist).
    //
    // This should make the request to
    // create a room fail.
    println!("testing creating room");
    let create_room_msg =
        helpers::basic_request("create_room".to_owned(), helpers::basic_room_creation());
    helpers::send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        2,
        &33,
    )
    .await;
    // Check that user is getting an error response
    // to their task channel.
    helpers::grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request")
        .await;

    // Set the user's room state back to -1, signifying
    // that they aren't in a room, which means they
    // can successfully create a room
    helpers::send_create_or_join_room_request(
        state,
        create_room_msg,
        publish_channel,
        execution_handler,
        -1,
        &33,
    )
    .await;
    // The second attempt for room creation should be successful,
    // resulting in a new room in state and a message to the voice
    // server via RabbitMQ. So, we can check these side effects.
    helpers::grab_and_assert_message_to_voice_server::<VoiceServerCreateRoom>(
        consume_channel,
        helpers::basic_voice_server_creation(),
        "33".to_owned(),
        "create-room".to_owned(),
    )
    .await;

    //Check the server state after the successful creations etc.
    let server_state = state.read().await;
    assert_eq!(server_state.rooms.len(), 1);
    assert_eq!(server_state.rooms.contains_key(&3), true);
}

pub async fn test_unfollowing_or_following_user(){
    
}

pub async fn test_joining_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
    type_of_join: &str,
    user_id: i32,
) {
    println!("testing joining room(type_op_code:{})", type_of_join);
    // Based on the previous tests for data capture/execution handler we know
    // the room id->3 exists.
    // Set user to a fake room to test illegal requests,
    // no user can join a room if they are already in a room.

    let create_room_msg = helpers::basic_request(
        type_of_join.to_owned(),
        helpers::generic_room_and_peer_id(user_id.clone(), 3),
    );
    helpers::send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        2,
        &user_id,
    )
    .await;

    // Check the channel and make sure there is an error.
    helpers::grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request")
        .await;

    // The second attempt should pass due to the room being changed to -1
    // which means the user isn't in a room.
    helpers::send_create_or_join_room_request(
        state,
        create_room_msg.clone(),
        publish_channel,
        execution_handler,
        -1,
        &user_id,
    )
    .await;

    // Ensure that we published the correct message
    helpers::grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        helpers::generic_room_and_peer_id(user_id, 3),
        user_id.to_string(),
        type_of_join.to_owned(),
    )
    .await;

    //Check:The user in the room?
    //Check:There only one user in the room?
    //Check:The user's current room state is updated?
    let server_state = state.read().await;

    // We know there is no one in the room when we pass in
    // join as speaker from this test
    //
    // After the speaker joins there should be one
    // then when the listner join it should be 2
    let num;
    if type_of_join == "join-as-speaker" {
        num = 1;
    } else {
        num = 2
    }
    assert_eq!(server_state.rooms.get(&3).unwrap().user_ids.len(), num);
    assert_eq!(
        server_state
            .rooms
            .get(&3)
            .unwrap()
            .user_ids
            .contains(&user_id),
        true
    );
    assert_eq!(
        server_state
            .active_users
            .get(&user_id)
            .unwrap()
            .current_room_id,
        3
    );
}
