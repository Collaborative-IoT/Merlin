// #![deny(warnings)]
use futures_util::stream::SplitStream;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryFutureExt};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use crate::state::state::ServerState;

type Users = Arc<RwLock<HashMap<i32, mpsc::UnboundedSender<Message>>>>;

async fn start_server() {
    pretty_env_logger::init();

    // Keep track of all connected users(websocket sender value).
    let users:Arc<RwLock<ServerState>>= Arc::new(RwLock::new(ServerState::new()));
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    let main = warp::path("main")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users:Arc<RwLock<ServerState>>| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    warp::serve(main).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, server_state: Arc<RwLock<ServerState>>) {
    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    //authenticate and ensure auth passed
    let auth_result: (bool, i32) = handle_authentication(&mut user_ws_tx, &mut user_ws_rx).await;
    let auth_passed = auth_result.0;

    if auth_passed {
        //Make use of a mpsc channel for each user.
        let current_user_id = auth_result.1;
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("websocket send error: {}", e);
                    })
                    .await;
            }
        });
        server_state.write().await.peer_map.insert(current_user_id, tx);

        // Every time the user sends a message, broadcast it to
        // all other users...
        while let Some(result) = user_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", current_user_id, e);
                    break;
                }
            };
            user_message(&current_user_id, msg, &server_state).await;
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        user_disconnected(&current_user_id, &server_state).await;
    }
}

async fn user_message(current_user_id: &i32, msg: Message, server_state:&Arc<RwLock<ServerState>> ) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };
}

async fn broadcast_message(current_user_id: &i32, new_msg: String, server_state:&Arc<RwLock<ServerState>> ) {
    for (&uid, tx) in server_state.read().await.peer_map.iter() {
        if current_user_id.to_owned() != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                //user disconnection is handled in another task
            }
        }
    }
}

async fn user_disconnected(current_user_id: &i32, server_state: &Arc<RwLock<ServerState>>) {
    // Stream closed up, so remove from the user list
    server_state.write().await.peer_map.remove(current_user_id);
}

async fn handle_authentication(
    user_ws_tx: &mut SplitSink<WebSocket, Message>,
    user_ws_rx: &mut SplitStream<WebSocket>,
) -> (bool, i32) {
    return (true, 2 as i32);
}
