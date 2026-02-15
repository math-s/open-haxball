use std::collections::HashMap;
use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;

use crate::game::Game;
use crate::physics::world::PHYSICS_DT;
use crate::protocol::{ClientMessage, InputState, ServerMessage, Team};

// Kick players after 5 minutes of no input
const IDLE_TIMEOUT_SECS: u64 = 300;

pub struct Client {
    pub sender: UnboundedSender<String>,
    pub player_id: Option<String>,
    pub last_activity: Instant,
    pub join_timestamp: Instant,
}

pub struct Room {
    pub game: Game,
    pub clients: HashMap<usize, Client>,
    next_player_id: u32,
    next_client_id: usize,
}

impl Room {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            clients: HashMap::new(),
            next_player_id: 1,
            next_client_id: 1,
        }
    }

    pub fn add_client(&mut self, sender: UnboundedSender<String>) -> usize {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let now = Instant::now();
        self.clients.insert(
            client_id,
            Client {
                sender,
                player_id: None,
                last_activity: now,
                join_timestamp: now,
            },
        );

        println!(
            "Client {} connected. Total clients: {}",
            client_id,
            self.clients.len()
        );
        client_id
    }

    pub fn remove_client(&mut self, client_id: usize) {
        if let Some(client) = self.clients.remove(&client_id) {
            if let Some(player_id) = &client.player_id {
                self.game.remove_player(player_id);
                self.broadcast(
                    ServerMessage::PlayerLeft {
                        player_id: player_id.clone(),
                    },
                    Some(client_id),
                );
                println!("Player {} left", player_id);
            }
        }
        println!(
            "Client {} disconnected. Total clients: {}",
            client_id,
            self.clients.len()
        );
    }

    pub fn handle_message(&mut self, client_id: usize, message: ClientMessage) {
        match message {
            ClientMessage::Join { name } => self.handle_join(client_id, name),
            ClientMessage::Input(input) => self.handle_input(client_id, input),
            ClientMessage::Chat { text } => self.handle_chat(client_id, text),
            ClientMessage::SwitchTeam { team } => self.handle_switch_team(client_id, team),
            ClientMessage::RestartMatch => self.handle_restart_match(client_id),
        }
    }

    fn handle_join(&mut self, client_id: usize, name: String) {
        let client = match self.clients.get_mut(&client_id) {
            Some(c) => c,
            None => return,
        };

        if client.player_id.is_some() {
            self.send(
                client_id,
                ServerMessage::Error {
                    message: "Already joined".to_string(),
                },
            );
            return;
        }

        let player_id = format!("player_{}", self.next_player_id);
        self.next_player_id += 1;

        let team = self.game.get_auto_team();
        self.game.add_player(player_id.clone(), name.clone(), team);

        // Update client
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.player_id = Some(player_id.clone());
        }

        // Send join confirmation
        self.send(
            client_id,
            ServerMessage::Joined {
                player_id: player_id.clone(),
                team,
            },
        );

        // Broadcast to others
        self.broadcast(
            ServerMessage::PlayerJoined {
                player_id: player_id.clone(),
                name: name.clone(),
                team,
            },
            Some(client_id),
        );

        println!("Player {} ({}) joined team {:?}", name, player_id, team);
    }

    fn handle_input(&mut self, client_id: usize, input: InputState) {
        let player_id = match self.clients.get_mut(&client_id) {
            Some(c) => {
                c.last_activity = Instant::now();
                c.player_id.clone()
            }
            None => return,
        };

        if let Some(player_id) = player_id {
            self.game.set_player_input(&player_id, input);
        }
    }

    fn handle_chat(&mut self, client_id: usize, text: String) {
        // Update last activity
        let player_id = match self.clients.get_mut(&client_id) {
            Some(c) => {
                c.last_activity = Instant::now();
                c.player_id.clone()
            }
            None => return,
        };

        // Look up player info
        let (player_id, name) = match player_id {
            Some(pid) => match self.game.players.get(&pid) {
                Some(player) => (pid, player.name.clone()),
                None => return,
            },
            None => return,
        };

        // Truncate text to 200 chars
        let text: String = text.chars().take(200).collect();

        // Broadcast to all clients
        self.broadcast(
            ServerMessage::Chat {
                player_id,
                name,
                text,
            },
            None,
        );
    }

    fn handle_switch_team(&mut self, client_id: usize, team: Team) {
        let player_id = match self.clients.get_mut(&client_id) {
            Some(c) => {
                c.last_activity = Instant::now();
                c.player_id.clone()
            }
            None => return,
        };

        if let Some(pid) = player_id {
            self.game.switch_player_team(&pid, team);
        }
    }

    fn handle_restart_match(&mut self, client_id: usize) {
        // Update last activity and get player ID
        let player_id = match self.clients.get_mut(&client_id) {
            Some(c) => {
                c.last_activity = Instant::now();
                c.player_id.clone()
            }
            None => return,
        };

        // Verify player is host
        let host_id = self.get_host_player_id();
        if player_id.as_ref() != host_id.as_ref() {
            self.send(
                client_id,
                ServerMessage::Error {
                    message: "Only the host can restart the match".to_string(),
                },
            );
            return;
        }

        self.game.restart_match();
    }

    fn get_host_player_id(&self) -> Option<String> {
        self.clients
            .values()
            .filter(|c| c.player_id.is_some())
            .min_by_key(|c| c.join_timestamp)
            .and_then(|c| c.player_id.clone())
    }

    pub fn tick(&mut self) {
        // Check for idle players and collect IDs to remove
        let idle_clients: Vec<usize> = self
            .clients
            .iter()
            .filter(|(_, client)| {
                client.player_id.is_some()
                    && client.last_activity.elapsed().as_secs() > IDLE_TIMEOUT_SECS
            })
            .map(|(id, _)| *id)
            .collect();

        // Kick idle players
        for client_id in idle_clients {
            println!("Kicking idle client {}", client_id);
            self.remove_client(client_id);
        }

        self.game.update(PHYSICS_DT);

        // Get host player ID
        let host_id = self.get_host_player_id();

        // Send state to each client with their is_host status
        for (client_id, client) in &self.clients {
            let is_host =
                client.player_id.as_ref() == host_id.as_ref() && client.player_id.is_some();
            let state = self.game.serialize_state(is_host);
            self.send(*client_id, ServerMessage::State(state));
        }
    }

    fn send(&self, client_id: usize, message: ServerMessage) {
        if let Some(client) = self.clients.get(&client_id) {
            if let Ok(json) = serde_json::to_string(&message) {
                let _ = client.sender.send(json);
            }
        }
    }

    fn broadcast(&self, message: ServerMessage, exclude: Option<usize>) {
        if let Ok(json) = serde_json::to_string(&message) {
            for (id, client) in &self.clients {
                if Some(*id) != exclude {
                    let _ = client.sender.send(json.clone());
                }
            }
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
