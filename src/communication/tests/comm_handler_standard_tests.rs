use crate::communication::communication_types::{
    BasicRequest, DeafAndMuteStatus, DeafAndMuteStatusUpdate, GenericRoomId,
    GenericRoomIdAndPeerId, GenericUserId, RoomUpdate, VoiceServerClosePeer, VoiceServerCreateRoom,
    VoiceServerDestroyRoom,
};
use crate::communication::tests::comm_handler_test_helpers::helpers;
use crate::communication::{communication_router, data_fetcher};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use futures::lock::Mutex;
use lapin::Consumer;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashSet;
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
    println!("testing webrtc invalid requests");
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

pub async fn test_unfollowing_and_following_user(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    println!("testing following and unfollowing user");
    //create 2 new users
    let mut new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "311231212111111111111wepo242345kqw1321241".to_string(),
        "fwij2428959478239825833234024nsdocikndv0".to_string(),
    )
    .await;

    let new_second_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "31!1231212111111111111wepo242345kqw1321241".to_string(),
        "fwij!2428959478239825833234024nsdocikndv0".to_string(),
    )
    .await;

    let follow_request = GenericUserId {
        user_id: new_second_user.0.to_owned(),
    };

    //use user 1 to follow user 2
    communication_router::route_msg(
        helpers::basic_request(
            "follow_user".to_owned(),
            serde_json::to_string(&follow_request).unwrap(),
        ),
        new_user.0,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut new_user.1,
        "user_follow_successful",
        &new_second_user.0.to_string(),
    )
    .await;
    //get user 1 from the perspective of user 2 after follow
    let mut temp_lock = execution_handler.lock().await;
    let result = data_fetcher::get_users_for_user(
        new_second_user.0.to_owned(),
        vec![new_user.0.to_owned()],
        &mut temp_lock,
    )
    .await;
    assert_eq!(result.1.len(), 1);
    assert_eq!(result.1[0].follows_you, true);
    drop(temp_lock);
    //use user 1 to unfollow user 2
    communication_router::route_msg(
        helpers::basic_request(
            "unfollow_user".to_owned(),
            serde_json::to_string(&follow_request).unwrap(),
        ),
        new_user.0,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut new_user.1,
        "user_unfollow_successful",
        &new_second_user.0.to_string(),
    )
    .await;
    //get user 1 from the perspective of user 2 after unfollow
    let mut temp_lock = execution_handler.lock().await;
    let result = data_fetcher::get_users_for_user(
        new_second_user.0.to_owned(),
        vec![new_user.0.to_owned()],
        &mut temp_lock,
    )
    .await;
    assert_eq!(result.1.len(), 1);
    assert_eq!(result.1[0].follows_you, false);
}

pub async fn test_blocking_and_unblocking_user(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    println!("testing blocking and unblocking user");
    //create 2 new users
    let mut new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "3@#$11111111wepo242345kqw1321241".to_string(),
        "%$$$$$$$$833234024nsdocikndv0".to_string(),
    )
    .await;

    let new_second_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "31#$%11111111wepo242345kqw1321241".to_string(),
        "#@9478239825833234024nsdocikndv0".to_string(),
    )
    .await;

    //use user one to block user two
    helpers::trigger_block_or_unblock(
        publish_channel,
        state,
        execution_handler,
        &mut new_user,
        &new_second_user,
        "block_user".to_owned(),
        "user_personally_blocked",
    )
    .await;

    //get user one from user two perspective
    let mut temp_lock = execution_handler.lock().await;
    let result = data_fetcher::get_users_for_user(
        new_second_user.0.to_owned(),
        vec![new_user.0.to_owned()],
        &mut temp_lock,
    )
    .await;
    assert_eq!(result.1[0].they_blocked_you, true);
    drop(temp_lock);

    //use user two to unblock user two
    helpers::trigger_block_or_unblock(
        publish_channel,
        state,
        execution_handler,
        &mut new_user,
        &new_second_user,
        "unblock_user".to_owned(),
        "user_personally_unblocked",
    )
    .await;

    //get user one from user two perspective
    let mut temp_lock = execution_handler.lock().await;
    let result = data_fetcher::get_users_for_user(
        new_second_user.0.to_owned(),
        vec![new_user.0.to_owned()],
        &mut temp_lock,
    )
    .await;
    assert_eq!(result.1[0].they_blocked_you, false);
}

pub async fn test_blocking_and_unblocking_user_invalid(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    gh_id: String,
    dc_id: String,
    op_code: String,
) {
    println!("testing blocking or unblocking user invalid({})", op_code);
    let mut new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        gh_id,
        dc_id,
    )
    .await;

    //test blocking user that doesn't exist
    let block_request = helpers::basic_request(
        op_code,
        serde_json::to_string(&GenericUserId { user_id: 9939239 }).unwrap(),
    );

    communication_router::route_msg(
        block_request,
        new_user.0.to_owned(),
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();

    helpers::grab_and_assert_request_response(
        &mut new_user.1,
        "invalid_request",
        "issue with request",
    )
    .await;
}

//without cleanup means the room should still be alive
//after we force users to leave.
pub async fn test_leaving_room_without_cleanup(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
) {
    println!("testing leaving room without cleanup");
    let new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "3@#$12342341111111wepo242345kqw1321241".to_string(),
        "%12312$$$$$$$$833234024nsdocikndv0".to_string(),
    )
    .await;
    let request = helpers::basic_request(
        "leave_room".to_owned(),
        serde_json::to_string(&GenericRoomId { room_id: 3 }).unwrap(),
    );
    communication_router::route_msg(
        request,
        new_user.0.clone(),
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    let close_peer = VoiceServerClosePeer {
        roomId: "3".to_owned(),
        peerId: new_user.0.to_string(),
        kicked: false,
    };
    helpers::grab_and_assert_message_to_voice_server::<VoiceServerClosePeer>(
        consume_channel,
        serde_json::to_string(&close_peer).unwrap(),
        new_user.0.to_string(),
        "close-peer".to_owned(),
    )
    .await;
    //user no longer in room state
    assert!(
        state
            .read()
            .await
            .rooms
            .get(&3)
            .unwrap()
            .user_ids
            .contains(&new_user.0)
            == false
    );
    assert!(
        state
            .read()
            .await
            .active_users
            .get(&new_user.0)
            .unwrap()
            .current_room_id
            == -1
    );
}

pub async fn test_leaving_room_with_cleanup(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
) {
    println!("testing leaving room with cleanup");
    let mut write_state = state.write().await;
    let room = write_state.rooms.get_mut(&3).unwrap();

    room.user_ids = HashSet::new();
    room.amount_of_users = 1;
    room.user_ids.insert(33);
    drop(write_state);
    //send the request with the wrong room
    let request = helpers::basic_request(
        "leave_room".to_owned(),
        serde_json::to_string(&GenericRoomId { room_id: 4 }).unwrap(),
    );
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request")
        .await;

    //send the request with the correct room
    let request = helpers::basic_request(
        "leave_room".to_owned(),
        serde_json::to_string(&GenericRoomId { room_id: 3 }).unwrap(),
    );
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    let destroy = VoiceServerDestroyRoom {
        roomId: "3".to_string(),
    };
    //check voice server msg
    helpers::grab_and_assert_message_to_voice_server::<VoiceServerDestroyRoom>(
        consume_channel,
        serde_json::to_string(&destroy).unwrap(),
        "-1".to_string(),
        "destroy-room".to_owned(),
    )
    .await;
    //check server state
    let read_state = state.read().await;
    assert_eq!(read_state.rooms.contains_key(&3), false);
    assert_eq!(
        read_state.active_users.get(&33).unwrap().current_room_id,
        -1
    );
}

pub async fn test_updating_room_meta_data(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
) {
    //test invalid update
    //this person is not a mod so it should fail
    println!("testing invalid/valid room metadata update");
    let mut new_user = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
        "3@#$12342341111111wepo242345kqw1321241".to_string(),
        "%12312$$$$$$$$833234024nsdocikndv0".to_string(),
    )
    .await;

    let room_update: RoomUpdate = RoomUpdate {
        name: "test90432840".to_owned(),
        public: false,
        chat_throttle: 2000,
        description: "for the best".to_owned(),
        auto_speaker: true,
    };
    let basic_request = helpers::basic_request(
        "update_room_meta".to_owned(),
        serde_json::to_string(&room_update).unwrap(),
    );

    communication_router::route_msg(
        basic_request,
        new_user.0,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();

    helpers::grab_and_assert_request_response(
        &mut new_user.1,
        "invalid_request",
        "issue with request",
    )
    .await;

    //test valid update
    //user 33 is the owner and is a mod so it should complete
    let room_update: RoomUpdate = RoomUpdate {
        name: "test90432840!!!!!".to_owned(),
        public: true,
        chat_throttle: 3000,
        description: "for the bes333".to_owned(),
        auto_speaker: true,
    };
    let basic_request = helpers::basic_request(
        "update_room_meta".to_owned(),
        serde_json::to_string(&room_update).unwrap(),
    );

    communication_router::route_msg(basic_request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();

    helpers::grab_and_assert_request_response(
        user_one_rx,
        "room_meta_update",
        &serde_json::to_string(&room_update).unwrap(),
    )
    .await;

    let read_state = state.read().await;
    let room = read_state.rooms.get(&3).unwrap();
    assert_eq!(room.chat_throttle, room_update.chat_throttle);
    assert_eq!(room.public, room_update.public);
    assert_eq!(room.desc, room_update.description);
    assert_eq!(room.auto_speaker, room_update.auto_speaker);
    assert_eq!(room.name, room_update.name);
}

pub async fn test_updating_muted_and_deaf(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    user_one_rx: &mut UnboundedReceiverStream<Message>,
) {
    println!("testing updating muted and deaf status");
    let mut room_death_and_mute = DeafAndMuteStatus {
        deaf: true,
        muted: true,
    };
    let mut room_death_and_mute_response = DeafAndMuteStatusUpdate {
        deaf: true,
        muted: true,
        user_id: 33,
    };

    let basic_request = helpers::basic_request(
        "update_deaf_and_mute".to_owned(),
        serde_json::to_string(&room_death_and_mute).unwrap(),
    );

    // Test invalid case
    // no user who isn't in a room can
    // change their death/mute status
    // we will set user 33's room to -1
    // and expect it to fail.
    state
        .write()
        .await
        .active_users
        .get_mut(&33)
        .unwrap()
        .current_room_id = -1;
    communication_router::route_msg(
        basic_request.clone(),
        33,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(user_one_rx, "invalid_request", "issue with request")
        .await;

    //test the valid case
    state
        .write()
        .await
        .active_users
        .get_mut(&33)
        .unwrap()
        .current_room_id = 3;
    communication_router::route_msg(basic_request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();

    helpers::grab_and_assert_request_response(
        user_one_rx,
        "user_mute_and_deaf_update",
        &serde_json::to_string(&room_death_and_mute_response).unwrap(),
    )
    .await;

    let read_state = state.read().await;
    let user = read_state.active_users.get(&33).unwrap();
    assert_eq!(user.deaf, room_death_and_mute.deaf);
    assert_eq!(user.muted, room_death_and_mute.muted);
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
