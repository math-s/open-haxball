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

#[derive(Clone, Default, Debug, Serialize)]
pub struct Score {
    pub red: u32,
    pub blue: u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Team tests
    #[test]
    fn test_team_serialize() {
        let red = Team::Red;
        let blue = Team::Blue;

        assert_eq!(serde_json::to_string(&red).unwrap(), "\"red\"");
        assert_eq!(serde_json::to_string(&blue).unwrap(), "\"blue\"");
    }

    #[test]
    fn test_team_deserialize() {
        let red: Team = serde_json::from_str("\"red\"").unwrap();
        let blue: Team = serde_json::from_str("\"blue\"").unwrap();

        assert_eq!(red, Team::Red);
        assert_eq!(blue, Team::Blue);
    }

    #[test]
    fn test_team_equality() {
        assert_eq!(Team::Red, Team::Red);
        assert_eq!(Team::Blue, Team::Blue);
        assert_ne!(Team::Red, Team::Blue);
    }

    // GameStatus tests
    #[test]
    fn test_game_status_serialize() {
        assert_eq!(
            serde_json::to_string(&GameStatus::Waiting).unwrap(),
            "\"waiting\""
        );
        assert_eq!(
            serde_json::to_string(&GameStatus::Playing).unwrap(),
            "\"playing\""
        );
        assert_eq!(
            serde_json::to_string(&GameStatus::Goal).unwrap(),
            "\"goal\""
        );
        assert_eq!(
            serde_json::to_string(&GameStatus::Finished).unwrap(),
            "\"finished\""
        );
    }

    #[test]
    fn test_game_status_deserialize() {
        let waiting: GameStatus = serde_json::from_str("\"waiting\"").unwrap();
        let playing: GameStatus = serde_json::from_str("\"playing\"").unwrap();

        assert_eq!(waiting, GameStatus::Waiting);
        assert_eq!(playing, GameStatus::Playing);
    }

    // InputState tests
    #[test]
    fn test_input_state_default() {
        let input = InputState::default();

        assert!(!input.left);
        assert!(!input.right);
        assert!(!input.up);
        assert!(!input.down);
        assert!(!input.kick);
    }

    #[test]
    fn test_input_state_deserialize() {
        let json = r#"{"left":true,"right":false,"up":true,"down":false,"kick":true}"#;
        let input: InputState = serde_json::from_str(json).unwrap();

        assert!(input.left);
        assert!(!input.right);
        assert!(input.up);
        assert!(!input.down);
        assert!(input.kick);
    }

    #[test]
    fn test_input_state_deserialize_complete() {
        // All fields must be present (no default attribute on InputState)
        let json = r#"{"left":true,"right":false,"up":false,"down":false,"kick":false}"#;
        let input: InputState = serde_json::from_str(json).unwrap();

        assert!(input.left);
        assert!(!input.right);
    }

    // Score tests
    #[test]
    fn test_score_default() {
        let score = Score::default();

        assert_eq!(score.red, 0);
        assert_eq!(score.blue, 0);
    }

    #[test]
    fn test_score_serialize() {
        let score = Score { red: 3, blue: 2 };
        let json = serde_json::to_string(&score).unwrap();

        assert!(json.contains("\"red\":3"));
        assert!(json.contains("\"blue\":2"));
    }

    // ClientMessage tests
    #[test]
    fn test_client_message_join_deserialize() {
        let json = r#"{"type":"join","data":{"name":"TestPlayer"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Join { name } => assert_eq!(name, "TestPlayer"),
            _ => panic!("Expected Join message"),
        }
    }

    #[test]
    fn test_client_message_input_deserialize() {
        let json = r#"{"type":"input","data":{"left":true,"right":false,"up":false,"down":true,"kick":false}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Input(input) => {
                assert!(input.left);
                assert!(!input.right);
                assert!(!input.up);
                assert!(input.down);
                assert!(!input.kick);
            }
            _ => panic!("Expected Input message"),
        }
    }

    // ServerMessage tests
    #[test]
    fn test_server_message_joined_serialize() {
        let msg = ServerMessage::Joined {
            player_id: "player_1".to_string(),
            team: Team::Red,
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"joined\""));
        assert!(json.contains("\"playerId\":\"player_1\""));
        assert!(json.contains("\"team\":\"red\""));
    }

    #[test]
    fn test_server_message_player_left_serialize() {
        let msg = ServerMessage::PlayerLeft {
            player_id: "player_1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"playerLeft\""));
        assert!(json.contains("\"playerId\":\"player_1\""));
    }

    #[test]
    fn test_server_message_error_serialize() {
        let msg = ServerMessage::Error {
            message: "Test error".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"Test error\""));
    }

    #[test]
    fn test_server_message_state_serialize() {
        let state = SerializedGameState {
            players: vec![],
            ball: SerializedBall {
                position: Vec2::new(400.0, 200.0),
                velocity: Vec2::new(0.0, 0.0),
                radius: 10.0,
            },
            score: Score { red: 1, blue: 2 },
            status: GameStatus::Playing,
            last_goal_team: Some(Team::Blue),
        };
        let msg = ServerMessage::State(state);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"state\""));
        assert!(json.contains("\"status\":\"playing\""));
        assert!(json.contains("\"lastGoalTeam\":\"blue\""));
    }

    // SerializedPlayer tests
    #[test]
    fn test_serialized_player() {
        let player = SerializedPlayer {
            id: "p1".to_string(),
            name: "TestPlayer".to_string(),
            position: Vec2::new(100.0, 200.0),
            velocity: Vec2::new(10.0, 20.0),
            radius: 15.0,
            team: Team::Red,
        };
        let json = serde_json::to_string(&player).unwrap();

        assert!(json.contains("\"id\":\"p1\""));
        assert!(json.contains("\"name\":\"TestPlayer\""));
        assert!(json.contains("\"radius\":15"));
        assert!(json.contains("\"team\":\"red\""));
    }

    // SerializedBall tests
    #[test]
    fn test_serialized_ball() {
        let ball = SerializedBall {
            position: Vec2::new(400.0, 200.0),
            velocity: Vec2::new(50.0, -30.0),
            radius: 10.0,
        };
        let json = serde_json::to_string(&ball).unwrap();

        assert!(json.contains("\"radius\":10"));
    }

    // Malformed JSON tests
    #[test]
    fn test_malformed_json_handling() {
        let result: Result<ClientMessage, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_message_type() {
        let json = r#"{"type":"invalid","data":{}}"#;
        let result: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"type":"join","data":{}}"#;
        let result: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
