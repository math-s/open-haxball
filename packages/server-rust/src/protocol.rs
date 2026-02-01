use crate::physics::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    Waiting,
    Playing,
    Goal,
    Finished,
}

#[derive(Clone, Copy, Default, Debug, Deserialize)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub kick: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Score {
    pub red: u32,
    pub blue: u32,
}

impl Default for Score {
    fn default() -> Self {
        Self { red: 0, blue: 0 }
    }
}

// Client -> Server messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    #[serde(rename = "join")]
    Join { name: String },
    #[serde(rename = "input")]
    Input(InputState),
}

// Server -> Client messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    #[serde(rename = "joined")]
    Joined {
        #[serde(rename = "playerId")]
        player_id: String,
        team: Team,
    },
    #[serde(rename = "state")]
    State(SerializedGameState),
    #[serde(rename = "playerJoined")]
    PlayerJoined {
        #[serde(rename = "playerId")]
        player_id: String,
        name: String,
        team: Team,
    },
    #[serde(rename = "playerLeft")]
    PlayerLeft {
        #[serde(rename = "playerId")]
        player_id: String,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Clone, Debug, Serialize)]
pub struct SerializedPlayer {
    pub id: String,
    pub name: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub team: Team,
}

#[derive(Clone, Debug, Serialize)]
pub struct SerializedBall {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SerializedGameState {
    pub players: Vec<SerializedPlayer>,
    pub ball: SerializedBall,
    pub score: Score,
    pub status: GameStatus,
    #[serde(rename = "lastGoalTeam")]
    pub last_goal_team: Option<Team>,
}
