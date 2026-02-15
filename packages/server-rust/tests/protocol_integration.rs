use haxball_server::physics::Vec2;
use haxball_server::protocol::{
    ClientMessage, GameStatus, InputState, Score, SerializedBall, SerializedGameState,
    SerializedPlayer, ServerMessage, Team,
};

#[test]
fn test_client_message_round_trip_join() {
    let json = r#"{"type":"join","data":{"name":"TestPlayer"}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::Join { name } => {
            assert_eq!(name, "TestPlayer");
        }
        _ => panic!("Expected Join message"),
    }
}

#[test]
fn test_client_message_round_trip_input() {
    let json =
        r#"{"type":"input","data":{"left":true,"right":false,"up":true,"down":false,"kick":true}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::Input(input) => {
            assert!(input.left);
            assert!(!input.right);
            assert!(input.up);
            assert!(!input.down);
            assert!(input.kick);
        }
        _ => panic!("Expected Input message"),
    }
}

#[test]
fn test_server_message_state_serialization() {
    let state = SerializedGameState {
        players: vec![
            SerializedPlayer {
                id: "player_1".to_string(),
                name: "Alice".to_string(),
                position: Vec2::new(100.0, 200.0),
                velocity: Vec2::new(10.0, -5.0),
                radius: 15.0,
                team: Team::Red,
            },
            SerializedPlayer {
                id: "player_2".to_string(),
                name: "Bob".to_string(),
                position: Vec2::new(700.0, 200.0),
                velocity: Vec2::new(-10.0, 5.0),
                radius: 15.0,
                team: Team::Blue,
            },
        ],
        ball: SerializedBall {
            position: Vec2::new(400.0, 200.0),
            velocity: Vec2::new(50.0, 30.0),
            radius: 10.0,
        },
        score: Score { red: 2, blue: 1 },
        status: GameStatus::Playing,
        last_goal_team: Some(Team::Red),
        match_time_remaining: Some(120.0),
        is_host: false,
        intermission_time_remaining: None,
    };

    let msg = ServerMessage::State(state);
    let json = serde_json::to_string(&msg).unwrap();

    // Verify key fields are present
    assert!(json.contains("\"type\":\"state\""));
    assert!(json.contains("\"player_1\""));
    assert!(json.contains("\"player_2\""));
    assert!(json.contains("\"Alice\""));
    assert!(json.contains("\"Bob\""));
    assert!(json.contains("\"red\":2"));
    assert!(json.contains("\"blue\":1"));
    assert!(json.contains("\"status\":\"playing\""));
    assert!(json.contains("\"lastGoalTeam\":\"red\""));
}

#[test]
fn test_malformed_json_handling() {
    // Various malformed inputs
    let invalid_inputs = vec![
        "",
        "not json",
        "{}",
        "{\"type\":\"unknown\"}",
        "{\"type\":\"join\"}",             // Missing data
        "{\"type\":\"join\",\"data\":{}}", // Missing name
        "{\"type\":\"input\"}",            // Missing data
        "null",
        "[]",
        "123",
        "true",
    ];

    for input in invalid_inputs {
        let result: Result<ClientMessage, _> = serde_json::from_str(input);
        assert!(result.is_err(), "Should have failed to parse: {}", input);
    }
}

#[test]
fn test_input_state_requires_all_fields() {
    // InputState requires all fields to be present (no serde default)
    let json = r#"{"left":true}"#;
    let result: Result<InputState, _> = serde_json::from_str(json);

    // Should fail because not all fields are present
    assert!(result.is_err());
}

#[test]
fn test_input_state_all_false() {
    let json = r#"{"left":false,"right":false,"up":false,"down":false,"kick":false}"#;
    let input: InputState = serde_json::from_str(json).unwrap();

    assert!(!input.left);
    assert!(!input.right);
    assert!(!input.up);
    assert!(!input.down);
    assert!(!input.kick);
}

#[test]
fn test_input_state_all_true() {
    let json = r#"{"left":true,"right":true,"up":true,"down":true,"kick":true}"#;
    let input: InputState = serde_json::from_str(json).unwrap();

    assert!(input.left);
    assert!(input.right);
    assert!(input.up);
    assert!(input.down);
    assert!(input.kick);
}

#[test]
fn test_server_message_joined_round_trip() {
    let msg = ServerMessage::Joined {
        player_id: "player_123".to_string(),
        team: Team::Blue,
    };

    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"joined\""));
    assert!(json.contains("\"playerId\":\"player_123\""));
    assert!(json.contains("\"team\":\"blue\""));
}

#[test]
fn test_server_message_player_joined_round_trip() {
    let msg = ServerMessage::PlayerJoined {
        player_id: "player_456".to_string(),
        name: "TestPlayer".to_string(),
        team: Team::Red,
    };

    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"playerJoined\""));
    assert!(json.contains("\"playerId\":\"player_456\""));
    assert!(json.contains("\"name\":\"TestPlayer\""));
    assert!(json.contains("\"team\":\"red\""));
}

#[test]
fn test_server_message_player_left_round_trip() {
    let msg = ServerMessage::PlayerLeft {
        player_id: "player_789".to_string(),
    };

    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"playerLeft\""));
    assert!(json.contains("\"playerId\":\"player_789\""));
}

#[test]
fn test_server_message_error_round_trip() {
    let msg = ServerMessage::Error {
        message: "Something went wrong!".to_string(),
    };

    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("\"message\":\"Something went wrong!\""));
}

#[test]
fn test_team_serialize_deserialize() {
    // Serialize
    assert_eq!(serde_json::to_string(&Team::Red).unwrap(), "\"red\"");
    assert_eq!(serde_json::to_string(&Team::Blue).unwrap(), "\"blue\"");

    // Deserialize
    let red: Team = serde_json::from_str("\"red\"").unwrap();
    let blue: Team = serde_json::from_str("\"blue\"").unwrap();
    assert_eq!(red, Team::Red);
    assert_eq!(blue, Team::Blue);
}

#[test]
fn test_game_status_serialize_deserialize() {
    let statuses = [
        (GameStatus::Waiting, "\"waiting\""),
        (GameStatus::Playing, "\"playing\""),
        (GameStatus::Goal, "\"goal\""),
        (GameStatus::Finished, "\"finished\""),
    ];

    for (status, expected_json) in statuses {
        // Serialize
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected_json);

        // Deserialize
        let deserialized: GameStatus = serde_json::from_str(expected_json).unwrap();
        assert_eq!(deserialized, status);
    }
}

#[test]
fn test_serialized_game_state_with_no_players() {
    let state = SerializedGameState {
        players: vec![],
        ball: SerializedBall {
            position: Vec2::new(400.0, 200.0),
            velocity: Vec2::new(0.0, 0.0),
            radius: 10.0,
        },
        score: Score { red: 0, blue: 0 },
        status: GameStatus::Waiting,
        last_goal_team: None,
        match_time_remaining: None,
        is_host: true,
        intermission_time_remaining: None,
    };

    let msg = ServerMessage::State(state);
    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"players\":[]"));
    assert!(json.contains("\"lastGoalTeam\":null"));
}

#[test]
fn test_serialized_player_fields() {
    let player = SerializedPlayer {
        id: "p1".to_string(),
        name: "Player Name".to_string(),
        position: Vec2::new(123.45, 678.90),
        velocity: Vec2::new(-10.5, 20.25),
        radius: 15.0,
        team: Team::Red,
    };

    let json = serde_json::to_string(&player).unwrap();

    assert!(json.contains("\"id\":\"p1\""));
    assert!(json.contains("\"name\":\"Player Name\""));
    assert!(json.contains("\"radius\":15"));
    assert!(json.contains("\"team\":\"red\""));
}

#[test]
fn test_score_serialization() {
    let score = Score { red: 10, blue: 7 };
    let json = serde_json::to_string(&score).unwrap();

    assert!(json.contains("\"red\":10"));
    assert!(json.contains("\"blue\":7"));
}

#[test]
fn test_vec2_serialization() {
    let vec = Vec2::new(123.456, -789.012);
    let json = serde_json::to_string(&vec).unwrap();

    // Vec2 should serialize to an object with x and y fields
    assert!(json.contains("\"x\":"));
    assert!(json.contains("\"y\":"));
}

#[test]
fn test_join_message_with_special_characters() {
    let json = r#"{"type":"join","data":{"name":"Test \"Player\" <script>"}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::Join { name } => {
            assert_eq!(name, "Test \"Player\" <script>");
        }
        _ => panic!("Expected Join message"),
    }
}

#[test]
fn test_join_message_with_unicode() {
    let json = r#"{"type":"join","data":{"name":"玩家一号🎮"}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::Join { name } => {
            assert_eq!(name, "玩家一号🎮");
        }
        _ => panic!("Expected Join message"),
    }
}

#[test]
fn test_join_message_with_empty_name() {
    let json = r#"{"type":"join","data":{"name":""}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::Join { name } => {
            assert_eq!(name, "");
        }
        _ => panic!("Expected Join message"),
    }
}
