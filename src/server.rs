// #![deny(warnings)]
use futures_util::stream::SplitStream;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryFutureExt};
use std::net::SocketAddr;
use std::sync::{Arc};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use crate::state::state::ServerState;
use crate::communication::communication_router;
use crate::auth::oauth_locations;
use crate::auth::authentication_handler;
use crate::http::Uri;

pub async fn start_server<T:Into<SocketAddr>>(addr:T) {
    // Keep track of all connected users(websocket sender value).
    let server_state:Arc<RwLock<ServerState>>= Arc::new(RwLock::new(ServerState::new()));
    // Turn our "state" into a new Filter...
    setup_routes_and_serve(addr, server_state).await;
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
    communication_router::route_msg(msg.to_string(), current_user_id).await;
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

async fn setup_routes_and_serve<T:Into<SocketAddr>>(addr:T, server_state:Arc<RwLock<ServerState>>) {
    let server_state = warp::any().map(move || server_state.clone());
    //GET /user-api
    let user_api_route = warp::path("user-api")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(server_state)
        .map(|ws: warp::ws::Ws, server_state:Arc<RwLock<ServerState>>| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, server_state))
        });

    let discord_redirect_url:Uri = oauth_locations::discord().parse().unwrap();
    let discord_auth_callback_route:Uri = oauth_locations::save_tokens_location("test".to_owned(), "test".to_owned()).parse().unwrap();
    //GET /auth
    let discord_auth_route = 
        warp::path!("auth"/"discord")
        .map(move || {
            println!("redirecting");
            warp::redirect(discord_redirect_url.to_owned())});
    
    //GET /api/discord/auth-callback
    let discord_auth_callback_route = warp::path!("api"/"discord"/"auth-callback")
        .map(move ||warp::redirect(discord_auth_callback_route.to_owned()));

    let routes = 
        warp::get()
        .and(
            user_api_route
            .or(discord_auth_route)
            .or(discord_auth_callback_route)
     );
        
    warp::serve(routes).run(addr).await;
}
