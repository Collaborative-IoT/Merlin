// #![deny(warnings)]
use crate::auth::authentication_handler;
use crate::auth::authentication_handler::CodeParams;
use crate::auth::oauth_locations;
use crate::communication::communication_router;
use crate::data_store::sql_execution_handler::ExecutionHandler;
use crate::state::state::ServerState;
use crate::warp::http::Uri;
use futures::lock::Mutex;
use futures_util::stream::SplitStream;
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryFutureExt};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_postgres::{Error, NoTls};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

pub async fn start_server<T: Into<SocketAddr>>(addr: T) {
    // Keep track of all connected users(websocket sender value).
    let server_state: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState::new()));
    let execution_handler: Arc<Mutex<ExecutionHandler>> =
        Arc::new(Mutex::new(setup_execution_handler().await.unwrap()));
    // Turn our "state" into a new Filter...
    setup_routes_and_serve(addr, server_state, execution_handler).await;
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
        server_state
            .write()
            .await
            .peer_map
            .insert(current_user_id, tx);

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

async fn user_message(
    current_user_id: &i32,
    msg: Message,
    server_state: &Arc<RwLock<ServerState>>,
) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };
    communication_router::route_msg(msg.to_string(), current_user_id).await;
}

async fn broadcast_message(
    current_user_id: &i32,
    new_msg: String,
    server_state: &Arc<RwLock<ServerState>>,
) {
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

async fn setup_routes_and_serve<T: Into<SocketAddr>>(
    addr: T,
    server_state: Arc<RwLock<ServerState>>,
    execution_handler: Arc<Mutex<ExecutionHandler>>,
) {
    let server_state = warp::any().map(move || server_state.clone());
    let execution_handler = warp::any().map(move || execution_handler.clone());

    //GET /user-api
    let user_api_route = warp::path("user-api")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(server_state)
        .and(execution_handler.clone())
        .map(
            |ws: warp::ws::Ws,
             server_state: Arc<RwLock<ServerState>>,
             execution_handler: Arc<Mutex<ExecutionHandler>>| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| user_connected(socket, server_state))
            },
        );

    //GET /auth/discord
    let discord_redirect_url: Uri = oauth_locations::discord().parse().unwrap();
    let discord_auth_route = warp::path!("auth" / "discord")
        .map(move || warp::redirect::redirect(discord_redirect_url.to_owned()));

    //GET /api/discord/auth-callback
    let discord_auth_callback_route = warp::path!("api" / "discord" / "auth-callback")
        .and(execution_handler.clone())
        .and(warp::query::<CodeParams>())
        .then(
            |execution_handler: Arc<Mutex<ExecutionHandler>>, code: CodeParams| async {
                let token_url_result =
                    authentication_handler::gather_tokens_and_construct_save_url_discord(
                        code.code,
                        execution_handler,
                    )
                    .await;
                if token_url_result.is_ok() {
                    let url: Uri = token_url_result.unwrap();
                    warp::redirect::redirect(url)
                } else {
                    let url: Uri = oauth_locations::error_auth_location().parse().unwrap();
                    warp::redirect::redirect(url)
                }
            },
        );

    //GET auth/github
    let github_redirect_url: Uri = oauth_locations::github().parse().unwrap();
    let github_auth_route = warp::path!("auth" / "github")
        .map(move || warp::redirect::redirect(github_redirect_url.to_owned()));

    //GET /api/github/auth-callback
    let github_auth_callback_route = warp::path!("api" / "github" / "auth-callback")
        .and(execution_handler.clone())
        .and(warp::query::<CodeParams>())
        .then(
            |execution_handler: Arc<Mutex<ExecutionHandler>>, code: CodeParams| async {
                let token_url_result =
                    authentication_handler::gather_tokens_and_construct_save_url_github(
                        code.code,
                        execution_handler,
                    )
                    .await;
                if token_url_result.is_ok() {
                    let url: Uri = token_url_result.unwrap();
                    warp::redirect::redirect(url)
                } else {
                    let url: Uri = oauth_locations::error_auth_location().parse().unwrap();
                    warp::redirect::redirect(url)
                }
            },
        );
    let routes = warp::get().and(
        user_api_route
            .or(discord_auth_route)
            .or(discord_auth_callback_route)
            .or(github_auth_route)
            .or(github_auth_callback_route),
    );

    warp::serve(routes).run(addr).await;
}

async fn setup_execution_handler() -> Result<ExecutionHandler, Error> {
    //"host=localhost user=postgres port=5432 password=password"
    let host = env::var("PG_HOST").unwrap();
    let user = env::var("PG_USER").unwrap();
    let port = env::var("PG_PORT").unwrap();
    let password = env::var("PG_PASSWORD").unwrap();
    let config: String = format!(
        "host={} user={} port={} password={}",
        host, user, port, password
    );

    let (client, connection) = tokio_postgres::connect(&config, NoTls).await?;
    //TODO: handle connection error
    tokio::spawn(async move { if let Err(e) = connection.await {} });
    let mut handler = ExecutionHandler::new(client);
    handler.create_all_tables_if_needed().await?;
    return Ok(handler);
}
