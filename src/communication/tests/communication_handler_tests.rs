/*
The communication handler tests aims at
testing the logic that the handler enforces.

The testing is done through usage of the communication
router, which is responsible for routing requests to
the handler.

It doesn't test its consuming modules like the
execution handler, it makes sure
requests fail under certain circumstances,
state is being modified and messages are being
persisted to the voice server via RabbitMq.

This test isn't fully integration based, so we manually
grab the messages intended for the voice server after
publish and assert them.

As we get deeping into the tests we can no longer use
"mock users", mock users are users that don't have any
database user associated with it, and certain functionality
requires real users, like being able to block someone,
getting user previews and etc.

So at a specific point you will see the all the mock
users except for the owner will be removed and
new real users will take their place. The only user
that can remain a "mock user" is user 33 aka the owner of
room 3. This is because the functions that we are calling
assumes the requester is a real user, since technically
you have to be authenticated first before the server will
even hear your requests.

Throughout the entire test chain, all we are doing is:

1.spawning users and making them join a room
2.making requests on behalf on those users
3.we know if the tests should fail or not so we check
*/

use crate::communication::communication_router;
use crate::communication::communication_types::{
    BlockUserFromRoom, GenericRoomIdAndPeerId, VoiceServerClosePeer, VoiceServerCreateRoom,
};
use crate::communication::tests::communication_handler_test_helpers::helpers;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::rabbitmq::rabbit;
use crate::server::setup_execution_handler;
use crate::state::state::ServerState;
use futures::lock::Mutex;

use lapin::{options::*, types::FieldTable, Consumer};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

pub async fn tests() {
    //setup rabbit channels
    let connection = rabbit::setup_rabbit_connection().await.unwrap();
    let publish_channel: Arc<Mutex<lapin::Channel>> =
        Arc::new(Mutex::new(helpers::setup_channel(&connection).await));
    let consume_channel: lapin::Channel = helpers::setup_channel(&connection).await;
    let mut consumer = consume_channel
        .basic_consume(
            "voice_server_consume",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    //setup mock state/execution handler
    let mock_state: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState::new()));
    let execution_handler: Arc<Mutex<ExecutionHandler>> =
        Arc::new(Mutex::new(setup_execution_handler().await.unwrap()));
    //Setup mock inner user channels
    //
    //We use channels to direct messages
    //to tasks, after the message is
    //gathered by the task it is
    //forwarded to the user via
    //websocket connection.
    //
    //So we exclude the forwarding portion
    //and check the message being sent to make sure
    //our server is responding to requests correctly.
    let mut rx_user_one =
        helpers::create_and_add_new_user_channel_to_peer_map(33, &mock_state).await;
    let mut rx_user_two =
        helpers::create_and_add_new_user_channel_to_peer_map(34, &mock_state).await;
    helpers::insert_starting_user_state(&mock_state).await;
    test_creating_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
    )
    .await;
    //speaker
    test_joining_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
        "join-as-speaker",
        33,
    )
    .await;
    //listener
    test_joining_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut &mut rx_user_two,
        "join-as-new-peer",
        34,
    )
    .await;
    test_raising_and_lowering_hand(
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
        &mut rx_user_two,
        &mut consumer,
    )
    .await;
    test_adding_speaker(
        &mut rx_user_two,
        &mut consumer,
        &publish_channel,
        &execution_handler,
        &mock_state,
        &mut rx_user_one,
    )
    .await;
    test_removing_speaker(
        &mut consumer,
        &publish_channel,
        &execution_handler,
        &mock_state,
    )
    .await;
    //after this method there are no more
    //mock users in the room, all users
    //have a db user linked to it
    //because all the following tests
    //require real users.
    test_blocking_from_room(
        &publish_channel,
        &execution_handler,
        &mock_state,
        &mut rx_user_two,
        &mut consumer,
    )
    .await;
}

async fn test_creating_room(
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

async fn test_joining_room(
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

// Raising/lowering your
// hand causes no interaction
// with the voice server so we
// just check the broadcasted
// messages.
async fn test_raising_and_lowering_hand(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    state: &Arc<RwLock<ServerState>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
    consume_channel: &mut Consumer,
) {
    println!("Testing raising/lowering hand");
    users_not_in_room_cannot_make_requests(publish_channel, execution_handler, state).await;
    users_in_room_as_listener_can_raise(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
    non_mods_can_not_lower_hands(publish_channel, execution_handler, state, consume_channel).await;
    mods_can_lower_hands(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
    users_can_lower_their_own_hand(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
}

async fn test_adding_speaker(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    println!("testing adding speaker");
    non_mods_can_not_bring_up_speakers(publish_channel, execution_handler, state, consume_channel)
        .await;
    mods_can_bring_up_speakers(
        listener_rx,
        consume_channel,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
}

//NOTE:MISSING TEST CASE OF REMOVING
//OTHER MODS.
async fn test_removing_speaker(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    println!("testing removing speaker");
    non_mods_can_not_remove_speaker(publish_channel, execution_handler, state, consume_channel)
        .await;
    mods_can_remove_speaker(consume_channel, publish_channel, execution_handler, state).await;
}

async fn test_blocking_from_room(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
    consume_channel: &mut Consumer,
) {
    println!("testing room blocking");
    non_owner_can_not_block_from_room(publish_channel, execution_handler, state, listener_rx).await;
    owner_can_block_from_room(consume_channel, publish_channel, execution_handler, state).await;
}

//| INNER LOGIC/HELPERS FROM THIS POINT FORWARD|
//
//| INNER LOGIC/HELPERS FROM THIS POINT FORWARD|

async fn owner_can_block_from_room(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    //clear room and add new real user
    //(see begining of file for real vs mock users)
    helpers::clear_all_users_except_owner(state).await;
    let new_real_user_id = helpers::spawn_new_real_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
    )
    .await;

    let data = serde_json::to_string(&BlockUserFromRoom {
        user_id: new_real_user_id.clone(),
        room_id: 3,
    })
    .unwrap();
    let request = helpers::basic_request("block_user_from_room".to_string(), data.clone());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    //check result
    helpers::grab_and_assert_message_to_voice_server::<VoiceServerClosePeer>(
        consume_channel,
        helpers::generic_close_peer(new_real_user_id, 3),
        new_real_user_id.to_string(),
        "close-peer".to_owned(),
    )
    .await;
}

async fn non_owner_can_not_block_from_room(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
) {
    //User 34 is not the owner, so this should fail
    let data = serde_json::to_string(&BlockUserFromRoom {
        user_id: 38,
        room_id: 3,
    })
    .unwrap();
    let request = helpers::basic_request("block_user_from_room".to_string(), data);
    communication_router::route_msg(request, 34, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "issue_blocking_user", "38").await;
}

async fn mods_can_remove_speaker(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("remove_speaker".to_string(), data.clone());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        data,
        "34".to_owned(),
        "remove-speaker".to_owned(),
    )
    .await;
}

async fn non_mods_can_not_remove_speaker(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    let mut mock_user_rx = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        38,
        consume_channel,
    )
    .await;
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("remove_speaker".to_string(), data);
    communication_router::route_msg(request, 38, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(&mut mock_user_rx, "issue_removing_speaker", "34")
        .await;
}

async fn non_mods_can_not_bring_up_speakers(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    //joins room 3 as a listener
    let mut mock_user_rx = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        37,
        consume_channel,
    )
    .await;
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("add_speaker".to_owned(), data.clone());
    communication_router::route_msg(request, 37, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_request_response(&mut mock_user_rx, "issue_adding_speaker", "34")
        .await;
}

async fn mods_can_bring_up_speakers(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    //TESTCASE - Mods can bring up speakers

    //run this test a second time to get a hand raise
    //since we know it should be successful. This test
    //was originally run apart of lower/raise hand.
    users_in_room_as_listener_can_raise(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
    //our actual test
    let data = helpers::generic_room_and_peer_id(34, 3);
    let request = helpers::basic_request("add_speaker".to_owned(), data.clone());
    communication_router::route_msg(request, 33, state, publish_channel, execution_handler)
        .await
        .unwrap();
    helpers::grab_and_assert_message_to_voice_server::<GenericRoomIdAndPeerId>(
        consume_channel,
        data,
        "34".to_string(),
        "add-speaker".to_string(),
    )
    .await;
}

async fn users_not_in_room_cannot_make_requests(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    // TESTCASE - USERS NOT IN THE ROOM CAN'T MAKE LOWER/RAISE REQUESTS
    // Make sure no user not in the room
    // can make a raise hand request.
    // We create a new user that has no current room
    // and make the request.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 35),
    );
    let mut mock_temp_user_rx =
        helpers::create_and_add_new_user_channel_to_peer_map(35, state).await;
    communication_router::route_msg(
        raise_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;

    //make sure no one not in the room can
    //make a lower hand request
    let raise_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 35),
    );
    communication_router::route_msg(
        raise_hand_message,
        35,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx,
        "invalid_request",
        "issue with request",
    )
    .await;
}

async fn users_in_room_as_listener_can_raise(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - USERS IN THE ROOM AS A LISTENER CAN REQUEST TO SPEAK
    // Make sure hand raising works for users in the room as listeners.
    // From previous tests, we know user number 34 is a listener.
    // The listener rx is user num 34.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;
    helpers::clear_message_that_was_fanned(vec![speaker_rx]).await;
}

async fn non_mods_can_not_lower_hands(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    consume_channel: &mut Consumer,
) {
    // TESTCASE - NON MODS CAN'T LOWER HANDS
    // Make sure a non-mod user can't lower another user's hand
    // Make another user join as a listenr
    // and try to lower user 34's hand who is requesting.

    //joins room 3 as a listener
    let mut mock_temp_user_rx_two = helpers::spawn_new_user_and_join_room(
        publish_channel,
        execution_handler,
        state,
        36,
        consume_channel,
    )
    .await;
    //try to lower 34's hand(non mod) as 36(non mod)
    let raise_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        36,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(
        &mut mock_temp_user_rx_two,
        "invalid_request",
        "issue with request",
    )
    .await;
}

async fn users_can_lower_their_own_hand(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - USERS CAN LOWER THEIR OWN HAND
    // Make sure user who was declined to speak can request again
    // and lower their own hand.
    let raise_hand_message = helpers::basic_request(
        "raise_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        raise_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_asking_to_speak", "34").await;

    let lower_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        lower_hand_message,
        34,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(listener_rx, "user_hand_lowered", "34").await;
    helpers::clear_message_that_was_fanned(vec![speaker_rx]).await;
}

async fn mods_can_lower_hands(
    listener_rx: &mut UnboundedReceiverStream<Message>,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    speaker_rx: &mut UnboundedReceiverStream<Message>,
) {
    // TESTCASE - MODS CAN LOWER HANDS
    // Make sure the room owner can lower the hand of 34 ,
    // the speaker rx is user num 33 aka the owner
    let lower_hand_message = helpers::basic_request(
        "lower_hand".to_owned(),
        helpers::basic_hand_raise_or_lower(3, 34),
    );
    communication_router::route_msg(
        lower_hand_message,
        33,
        state,
        publish_channel,
        execution_handler,
    )
    .await
    .unwrap();
    helpers::grab_and_assert_request_response(speaker_rx, "user_hand_lowered", "34").await;
    helpers::clear_message_that_was_fanned(vec![listener_rx]).await;
}
