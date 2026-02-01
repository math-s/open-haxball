mod common;

use common::{ball_is_in_bounds, create_game_with_players, simulate_ticks, InputBuilder};
use haxball_server::game::Game;
use haxball_server::physics::Vec2;
use haxball_server::protocol::{GameStatus, Team};

#[test]
fn test_full_game_1v1_goal_scored() {
    let mut game = create_game_with_players(1, 1);

    assert_eq!(game.status, GameStatus::Playing);
    assert_eq!(game.score.red, 0);
    assert_eq!(game.score.blue, 0);

    // Move ball into blue goal to score for red
    game.world.circles[game.ball_index].position = Vec2::new(815.0, 200.0);
    simulate_ticks(&mut game, 1);

    assert_eq!(game.score.red, 1);
    assert_eq!(game.status, GameStatus::Goal);
}

#[test]
fn test_player_chases_and_kicks_ball() {
    let mut game = create_game_with_players(1, 0);

    let player_id = "red_0".to_string();

    // Position player close to ball for reliable kick
    let ball_pos = game.world.circles[game.ball_index].position;
    let player_idx = game.players.get(&player_id).unwrap().circle_index;
    game.world.circles[player_idx].position = Vec2::new(ball_pos.x - 30.0, ball_pos.y);

    // Get initial ball position
    let ball_start = game.world.circles[game.ball_index].position;

    // Kick the ball
    game.set_player_input(&player_id, InputBuilder::new().kick().build());
    game.update(1.0 / 60.0);

    // Ball should have moved from being kicked
    let ball_end = game.world.circles[game.ball_index].position;
    let ball_vel = game.world.circles[game.ball_index].velocity;

    // Ball should have gained velocity (kicked by player)
    assert!(
        ball_vel.length() > 0.0 || ball_end.distance(ball_start) > 0.1,
        "Ball didn't move: start={:?}, end={:?}, vel={:?}",
        ball_start,
        ball_end,
        ball_vel
    );
}

#[test]
fn test_multiple_goals_score_tracking() {
    let mut game = create_game_with_players(1, 1);

    // Score multiple goals for red team
    for expected_score in 1..=3 {
        game.world.circles[game.ball_index].position = Vec2::new(815.0, 200.0);
        simulate_ticks(&mut game, 1);
        assert_eq!(game.score.red, expected_score);

        // Wait for reset
        simulate_ticks(&mut game, 150);
        assert_eq!(game.status, GameStatus::Playing);
    }

    // Score for blue team
    game.world.circles[game.ball_index].position = Vec2::new(-15.0, 200.0);
    simulate_ticks(&mut game, 1);
    assert_eq!(game.score.blue, 1);
    assert_eq!(game.score.red, 3);
}

#[test]
fn test_team_balancing_with_many_players() {
    let mut game = Game::new();

    // Add 6 players, should auto-balance
    for i in 0..6 {
        let team = game.get_auto_team();
        game.add_player(format!("p{}", i), format!("Player{}", i), team);
    }

    // Count teams
    let red_count = game.players.values().filter(|p| p.team == Team::Red).count();
    let blue_count = game.players.values().filter(|p| p.team == Team::Blue).count();

    // Should be balanced (3-3)
    assert_eq!(red_count, 3);
    assert_eq!(blue_count, 3);
}

#[test]
fn test_player_removal_mid_game() {
    let mut game = create_game_with_players(2, 2);

    assert_eq!(game.players.len(), 4);
    assert_eq!(game.status, GameStatus::Playing);

    // Remove a player
    game.remove_player("red_0");
    assert_eq!(game.players.len(), 3);
    assert_eq!(game.status, GameStatus::Playing);

    // Remove all but one
    game.remove_player("red_1");
    game.remove_player("blue_0");
    assert_eq!(game.players.len(), 1);
    assert_eq!(game.status, GameStatus::Playing);

    // Remove last player
    game.remove_player("blue_1");
    assert_eq!(game.players.len(), 0);
    assert_eq!(game.status, GameStatus::Waiting);
}

#[test]
fn test_ball_stays_in_bounds_under_stress() {
    let mut game = create_game_with_players(3, 3);

    // Give ball high velocity
    game.world.circles[game.ball_index].velocity = Vec2::new(500.0, 300.0);

    // All players kick constantly
    for _ in 0..600 {
        for i in 0..3 {
            game.set_player_input(&format!("red_{}", i), InputBuilder::new().kick().build());
            game.set_player_input(&format!("blue_{}", i), InputBuilder::new().kick().build());
        }
        game.update(1.0 / 60.0);

        assert!(ball_is_in_bounds(&game), "Ball went out of bounds");
    }
}

#[test]
fn test_simultaneous_kicks() {
    let mut game = create_game_with_players(2, 0);

    // Position both players near the ball
    let ball_pos = game.world.circles[game.ball_index].position;

    for player in game.players.values() {
        let idx = player.circle_index;
        game.world.circles[idx].position = Vec2::new(ball_pos.x - 25.0, ball_pos.y);
    }

    // Both players kick
    game.set_player_input("red_0", InputBuilder::new().kick().build());
    game.set_player_input("red_1", InputBuilder::new().kick().build());

    simulate_ticks(&mut game, 1);

    // Ball should have moved
    let new_ball_vel = game.world.circles[game.ball_index].velocity;
    assert!(new_ball_vel.length() > 0.0);
}

#[test]
fn test_goal_during_player_removal() {
    let mut game = create_game_with_players(2, 2);

    // Score a goal
    game.world.circles[game.ball_index].position = Vec2::new(815.0, 200.0);
    simulate_ticks(&mut game, 1);
    assert_eq!(game.status, GameStatus::Goal);

    // Remove a player during goal state
    game.remove_player("red_0");

    // Continue simulation - should not crash
    simulate_ticks(&mut game, 150);

    assert_eq!(game.status, GameStatus::Playing);
    assert_eq!(game.players.len(), 3);
}

#[test]
fn test_empty_game_no_crash() {
    let mut game = Game::new();

    // Update empty game
    for _ in 0..60 {
        game.update(1.0 / 60.0);
    }

    assert_eq!(game.status, GameStatus::Waiting);
}

#[test]
fn test_all_players_leaving_mid_game() {
    let mut game = create_game_with_players(2, 2);
    assert_eq!(game.status, GameStatus::Playing);

    // Remove all players
    let ids: Vec<String> = game.players.keys().cloned().collect();
    for id in ids {
        game.remove_player(&id);
    }

    assert_eq!(game.status, GameStatus::Waiting);
    assert!(game.players.is_empty());

    // Game should still be updateable
    simulate_ticks(&mut game, 60);
}

#[test]
fn test_player_movement_all_directions() {
    let mut game = create_game_with_players(1, 0);
    let player_id = "red_0";
    let player_idx = game.players.get(player_id).unwrap().circle_index;

    // Test each direction
    let directions = [
        (InputBuilder::new().left().build(), "left"),
        (InputBuilder::new().right().build(), "right"),
        (InputBuilder::new().up().build(), "up"),
        (InputBuilder::new().down().build(), "down"),
    ];

    for (input, name) in directions {
        // Reset velocity
        game.world.circles[player_idx].velocity = Vec2::default();

        game.set_player_input(player_id, input);
        game.update(1.0 / 60.0);

        let vel = game.world.circles[player_idx].velocity;
        assert!(
            vel.length() > 0.0,
            "Player didn't move when pressing {}",
            name
        );
    }
}

#[test]
fn test_serialize_state_after_multiple_updates() {
    let mut game = create_game_with_players(2, 2);

    simulate_ticks(&mut game, 60);

    let state = game.serialize_state();

    assert_eq!(state.players.len(), 4);
    assert_eq!(state.status, GameStatus::Playing);
    assert!(state.ball.radius > 0.0);
}

#[test]
fn test_kick_exact_edge_of_range() {
    let mut game = create_game_with_players(1, 0);

    let player_id = "red_0";
    let player_idx = game.players.get(player_id).unwrap().circle_index;
    let player_radius = game.world.circles[player_idx].radius;
    let ball_radius = game.world.circles[game.ball_index].radius;

    // Position player at exact kick range (plus a tiny bit inside)
    let ball_pos = game.world.circles[game.ball_index].position;
    let kick_range = player_radius + ball_radius + 30.0; // KICK_DISTANCE = 30.0
    game.world.circles[player_idx].position = Vec2::new(ball_pos.x - kick_range + 0.1, ball_pos.y);

    game.set_player_input(player_id, InputBuilder::new().kick().build());
    game.update(1.0 / 60.0);

    // Ball should have been kicked
    let ball_vel = game.world.circles[game.ball_index].velocity;
    assert!(ball_vel.length() > 0.0, "Ball should be kicked at edge of range");
}

#[test]
fn test_goal_at_exact_boundary() {
    let mut game = create_game_with_players(1, 0);

    // Position ball exactly at goal boundary
    game.world.circles[game.ball_index].position = Vec2::new(800.0, 200.0);
    simulate_ticks(&mut game, 1);

    // Should score (red goal min x is 800.0)
    assert_eq!(game.score.red, 1);
}
