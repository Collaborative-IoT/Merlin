pub mod helpers {
    use crate::communication::communication_router;
    use crate::communication::communication_types::{
        BasicRequest, BasicResponse, BasicRoomCreation, GenericRoomIdAndPeerId,
        VoiceServerCreateRoom, VoiceServerRequest,
    };
    use crate::data_store::sql_execution_handler::ExecutionHandler;
    use crate::rabbitmq::rabbit;
    use crate::state::state::ServerState;
    use crate::state::state_types::User;
    use chrono::Utc;
    use futures::lock::Mutex;
    use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryFutureExt};
    use lapin::{options::*, types::FieldTable, Channel, Connection, Consumer};
    use serde::Serialize;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::sync::RwLock;
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use warp::ws::Message;
    //All users must be present in memory before operation
    //unless they are spawned apart of a test
    pub async fn insert_starting_user_state(server_state: &Arc<RwLock<ServerState>>) {
        insert_user_state(server_state, 33).await;
        insert_user_state(server_state, 34).await;
    }

    pub async fn insert_user_state(server_state: &Arc<RwLock<ServerState>>, user_id: i32) {
        let mut state = server_state.write().await;
        let user = User {
            last_online: Utc::now(),
            muted: true,
            deaf: true,
            ip: "test".to_string(),
            current_room_id: -1,
        };
        state.active_users.insert(user_id, user);
    }

    //starts rabbitmq connection channel
    pub async fn setup_channel(conn: &Connection) -> Channel {
        let publish_channel = conn.create_channel().await.unwrap();
        publish_channel
            .queue_declare(
                "voice_server_consume",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
        return publish_channel;
    }

    pub async fn consume_message(consumer: &mut Consumer) -> String {
        let delivery = consumer.next().await.unwrap().unwrap().1;
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
        let parsed_msg = rabbit::parse_message(delivery);
        return parsed_msg;
    }

    pub async fn create_and_add_new_user_channel_to_peer_map(
        mock_id: i32,
        mock_state: &Arc<RwLock<ServerState>>,
    ) -> UnboundedReceiverStream<Message> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);
        //add initial peer state to state
        //we will use th
        mock_state.write().await.peer_map.insert(mock_id, tx);
        return rx;
    }

    pub async fn grab_and_assert_request_response(
        rx: &mut UnboundedReceiverStream<Message>,
        op_code: &str,
        containing_data: &str,
    ) {
        let message = rx.next().await.unwrap().to_str().unwrap().to_owned();
        let parsed_json: BasicResponse = serde_json::from_str(&message).unwrap();
        assert_eq!(parsed_json.response_op_code, op_code);
        assert_eq!(parsed_json.response_containing_data, containing_data);
    }

    pub async fn grab_and_assert_message_to_voice_server<
        T: serde::de::DeserializeOwned + Serialize,
    >(
        consume_channel: &mut Consumer,
        d: String,
        uid: String,
        op: String,
    ) {
        let message = consume_message(consume_channel).await;
        let vs_message: VoiceServerRequest<T> = serde_json::from_str(&message).unwrap();
        assert_eq!(serde_json::to_string(&vs_message.d).unwrap(), d);
        assert_eq!(op, vs_message.op);
        assert_eq!(uid, vs_message.uid);
    }

    pub fn basic_request(op: String, data: String) -> String {
        let request = BasicRequest {
            request_op_code: op,
            request_containing_data: data,
        };
        return serde_json::to_string(&request).unwrap();
    }

    pub fn basic_hand_raise_or_lower(room_id: i32, peer_id: i32) -> String {
        let raise_or_lower = GenericRoomIdAndPeerId {
            roomId: room_id,
            peerId: peer_id,
        };
        return serde_json::to_string(&raise_or_lower).unwrap();
    }

    pub fn basic_room_creation() -> String {
        let room_creation = BasicRoomCreation {
            name: "test".to_owned(),
            desc: "test".to_owned(),
            public: true,
        };
        return serde_json::to_string(&room_creation).unwrap();
    }

    pub fn basic_voice_server_creation() -> String {
        let room_creation = VoiceServerCreateRoom {
            roomId: 3.to_string(),
        };
        return serde_json::to_string(&room_creation).unwrap();
    }

    pub fn basic_join_as(user_id: i32) -> String {
        let join = GenericRoomIdAndPeerId {
            roomId: 3,
            peerId: user_id,
        };
        return serde_json::to_string(&join).unwrap();
    }

    pub async fn send_create_or_join_room_request(
        state: &Arc<RwLock<ServerState>>,
        msg: String,
        publish_channel: &Arc<Mutex<lapin::Channel>>,
        execution_handler: &Arc<Mutex<ExecutionHandler>>,
        curr_room: i32,
        user_id: &i32,
    ) {
        let mut server_state = state.write().await;
        server_state
            .active_users
            .get_mut(user_id)
            .unwrap()
            .current_room_id = curr_room;
        drop(server_state);
        communication_router::route_msg(
            msg,
            user_id.clone(),
            state,
            publish_channel,
            execution_handler,
        )
        .await
        .unwrap();
    }

    //This is used to clear the messages that get fanned
    // to other mock users in a room for our tests.
    //
    //The way we do our tests, requires the user's
    //channel to be completely clear.
    pub async fn clear_message_that_was_fanned(rxs: Vec<&mut UnboundedReceiverStream<Message>>) {
        for rx in rxs {
            rx.next().await.unwrap();
        }
    }
}
