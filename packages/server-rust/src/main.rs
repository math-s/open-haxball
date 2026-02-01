mod game;
mod map;
mod physics;
mod protocol;
mod room;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_tungstenite::tungstenite::Message;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

use crate::physics::world::PHYSICS_DT;
use crate::protocol::ClientMessage;
use crate::room::Room;

const PORT: u16 = 3001;

#[tokio::main]
async fn main() {
    let room = Arc::new(Mutex::new(Room::new()));

    // Start game loop (60Hz tick)
    let room_clone = room.clone();
    tokio::spawn(async move {
        game_loop(room_clone).await;
    });

    // Start WebSocket server
    let addr = format!("0.0.0.0:{}", PORT);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("Rust WebSocket server listening on port {}", PORT);

    while let Ok((stream, addr)) = listener.accept().await {
        let room = room.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| {
                let room = room.clone();
                async move { handle_request(req, addr, room).await }
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn game_loop(room: Arc<Mutex<Room>>) {
    let mut interval = tokio::time::interval(Duration::from_secs_f32(PHYSICS_DT));

    loop {
        interval.tick().await;
        let mut room = room.lock().await;
        room.tick();
    }
}

async fn handle_request(
    mut req: Request<Incoming>,
    addr: SocketAddr,
    room: Arc<Mutex<Room>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Health check endpoint
    if req.uri().path() == "/" || req.uri().path() == "/health" {
        if req.method() == hyper::Method::GET {
            println!("Health check from {}", addr);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("OK")))
                .unwrap());
        }
    }

    // Check for WebSocket upgrade
    if hyper_tungstenite::is_upgrade_request(&req) {
        let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None).unwrap();

        tokio::spawn(async move {
            if let Err(e) = handle_websocket(websocket, addr, room).await {
                eprintln!("Error in websocket connection: {}", e);
            }
        });

        return Ok(response.map(|_| Full::new(Bytes::new())));
    }

    // Not found
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("Not Found")))
        .unwrap())
}

async fn handle_websocket(
    websocket: hyper_tungstenite::HyperWebsocket,
    addr: SocketAddr,
    room: Arc<Mutex<Room>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = websocket.await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register client
    let client_id = {
        let mut room = room.lock().await;
        room.add_client(tx)
    };

    println!("WebSocket client {} connected", addr);

    // Task to forward outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let mut room = room.lock().await;
                    room.handle_message(client_id, client_msg);
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Cleanup
    {
        let mut room = room.lock().await;
        room.remove_client(client_id);
    }

    println!("WebSocket client {} disconnected", addr);
    send_task.abort();

    Ok(())
}
