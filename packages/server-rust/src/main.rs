mod game;
mod map;
mod physics;
mod protocol;
mod room;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

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
            handle_connection(stream, addr, room).await;
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

async fn handle_connection(stream: TcpStream, addr: SocketAddr, room: Arc<Mutex<Room>>) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register client
    let client_id = {
        let mut room = room.lock().await;
        room.add_client(tx)
    };

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

    send_task.abort();
}
