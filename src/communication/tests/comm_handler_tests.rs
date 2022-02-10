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

use crate::communication::communication_types::{GenericRoomId, GenericRoomIdAndPeerId};
use crate::communication::tests::comm_handler_test_helpers::helpers;
use crate::communication::tests::{
    comm_handler_hand_tests, comm_handler_mod_tests, comm_handler_owner_tests,
    comm_handler_standard_tests,
};
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
    comm_handler_standard_tests::test_creating_room(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
        &mut rx_user_one,
    )
    .await;
    //speaker
    comm_handler_standard_tests::test_joining_room(
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
    comm_handler_standard_tests::test_joining_room(
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

    //run this test a second time to get a hand raise
    //since we know it should be successful. This test
    //was originally run apart of lower/raise hand.
    //we need a hand raise for "test_adding_speakers"
    //which is called right after this
    comm_handler_hand_tests::users_in_room_as_listener_can_raise(
        &mut rx_user_two,
        &publish_channel,
        &execution_handler,
        &mock_state,
        &mut rx_user_one,
    )
    .await;
    test_adding_speaker(
        &mut consumer,
        &publish_channel,
        &execution_handler,
        &mock_state,
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

    comm_handler_standard_tests::test_users_can_get_top_rooms(
        &execution_handler,
        &mock_state,
        &mut rx_user_one,
        &publish_channel,
        &mut consumer,
    )
    .await;
    comm_handler_standard_tests::test_getting_all_users_in_room(
        &execution_handler,
        &mock_state,
        &publish_channel,
        &mut consumer,
    )
    .await;
    comm_handler_standard_tests::test_invalid_webrtc_request(
        &execution_handler,
        &mock_state,
        &publish_channel,
        &mut rx_user_one,
        GenericRoomIdAndPeerId {
            roomId: 3,
            peerId: 34,
        },
    )
    .await;
    comm_handler_standard_tests::test_invalid_webrtc_request(
        &execution_handler,
        &mock_state,
        &publish_channel,
        &mut rx_user_one,
        GenericRoomId { room_id: 3 },
    )
    .await;
    comm_handler_standard_tests::test_invalid_webrtc_request(
        &execution_handler,
        &mock_state,
        &publish_channel,
        &mut rx_user_one,
        GenericRoomIdAndPeerId {
            roomId: 2,
            peerId: 34,
        },
    )
    .await;
    comm_handler_standard_tests::test_unfollowing_and_following_user(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
    )
    .await;
    comm_handler_standard_tests::test_blocking_and_unblocking_user(
        &mut consumer,
        &publish_channel,
        &mock_state,
        &execution_handler,
    ).await;
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
    comm_handler_hand_tests::users_not_in_room_cannot_make_requests(
        publish_channel,
        execution_handler,
        state,
    )
    .await;
    comm_handler_hand_tests::users_in_room_as_listener_can_raise(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
    comm_handler_mod_tests::non_mods_can_not_lower_hands(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
    )
    .await;
    comm_handler_mod_tests::mods_can_lower_hands(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
    comm_handler_hand_tests::users_can_lower_their_own_hand(
        listener_rx,
        publish_channel,
        execution_handler,
        state,
        speaker_rx,
    )
    .await;
}

async fn test_adding_speaker(
    consume_channel: &mut Consumer,
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
) {
    println!("testing adding speaker");
    comm_handler_mod_tests::non_mods_can_not_bring_up_speakers(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
    )
    .await;
    comm_handler_mod_tests::mods_can_bring_up_speakers(
        consume_channel,
        publish_channel,
        execution_handler,
        state,
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
    comm_handler_mod_tests::non_mods_can_not_remove_speaker(
        publish_channel,
        execution_handler,
        state,
        consume_channel,
    )
    .await;
    comm_handler_mod_tests::mods_can_remove_speaker(
        consume_channel,
        publish_channel,
        execution_handler,
        state,
    )
    .await;
}

async fn test_blocking_from_room(
    publish_channel: &Arc<Mutex<lapin::Channel>>,
    execution_handler: &Arc<Mutex<ExecutionHandler>>,
    state: &Arc<RwLock<ServerState>>,
    listener_rx: &mut UnboundedReceiverStream<Message>,
    consume_channel: &mut Consumer,
) {
    println!("testing room blocking");
    comm_handler_owner_tests::non_owner_can_not_block_from_room(
        publish_channel,
        execution_handler,
        state,
        listener_rx,
    )
    .await;
    comm_handler_owner_tests::owner_can_block_from_room(
        consume_channel,
        publish_channel,
        execution_handler,
        state,
    )
    .await;
}
