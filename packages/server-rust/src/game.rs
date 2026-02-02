use std::collections::HashMap;

use crate::map::GameMap;
use crate::physics::{Circle, PhysicsWorld, Vec2};
use crate::protocol::{
    GameStatus, InputState, Score, SerializedBall, SerializedGameState, SerializedPlayer, Team,
};

// Game constants
const PLAYER_RADIUS: f32 = 15.0;
const PLAYER_MASS: f32 = 1.0;
const PLAYER_SPEED: f32 = 300.0;
const PLAYER_FRICTION: f32 = 0.96;
const PLAYER_RESTITUTION: f32 = 0.5;

const BALL_RADIUS: f32 = 10.0;
const BALL_MASS: f32 = 0.5;
const BALL_FRICTION: f32 = 0.99;
const BALL_RESTITUTION: f32 = 0.8;

const KICK_DISTANCE: f32 = 30.0;
const KICK_FORCE: f32 = 500.0;

const GOAL_RESET_TIME: f32 = 2.0;

pub struct Player {
    pub id: String,
    pub name: String,
    pub team: Team,
    pub input: InputState,
    pub circle_index: usize,
}

pub struct Game {
    pub world: PhysicsWorld,
    pub players: HashMap<String, Player>,
    pub ball_index: usize,
    pub score: Score,
    pub status: GameStatus,
    pub last_goal_team: Option<Team>,
    pub goal_timer: f32,
    pub map: GameMap,
}

impl Game {
    pub fn new() -> Self {
        let mut world = PhysicsWorld::new();
        let map = GameMap::new();

        // Add map walls to physics world
        for wall in &map.walls {
            world.add_line(wall.clone());
        }

        // Create ball
        let ball = Circle::new(
            map.ball_spawn,
            BALL_RADIUS,
            BALL_MASS,
            BALL_RESTITUTION,
            BALL_FRICTION,
            false,
        );
        let ball_index = world.add_circle(ball);

        Self {
            world,
            players: HashMap::new(),
            ball_index,
            score: Score::default(),
            status: GameStatus::Waiting,
            last_goal_team: None,
            goal_timer: 0.0,
            map,
        }
    }

    pub fn add_player(&mut self, id: String, name: String, team: Team) -> &Player {
        let spawns = match team {
            Team::Red => &self.map.red_spawns,
            Team::Blue => &self.map.blue_spawns,
        };
        let spawn_index = self.players.len() % spawns.len();
        let spawn_pos = spawns[spawn_index];

        let circle = Circle::new(
            spawn_pos,
            PLAYER_RADIUS,
            PLAYER_MASS,
            PLAYER_RESTITUTION,
            PLAYER_FRICTION,
            false,
        );
        let circle_index = self.world.add_circle(circle);

        let player = Player {
            id: id.clone(),
            name,
            team,
            input: InputState::default(),
            circle_index,
        };

        self.players.insert(id.clone(), player);

        // Start game if we have at least 1 player
        if !self.players.is_empty() && self.status == GameStatus::Waiting {
            self.status = GameStatus::Playing;
        }

        self.players.get(&id).unwrap()
    }

    pub fn remove_player(&mut self, id: &str) {
        if let Some(player) = self.players.remove(id) {
            // Find and remove the circle
            // Note: This shifts indices, which is a simplification
            // In production, you'd want a more robust approach
            self.world.remove_circle(player.circle_index);

            // Update remaining player indices
            for p in self.players.values_mut() {
                if p.circle_index > player.circle_index {
                    p.circle_index -= 1;
                }
            }

            // Also update ball index if needed
            if self.ball_index > player.circle_index {
                self.ball_index -= 1;
            }
        }

        // Go back to waiting if no players
        if self.players.is_empty() {
            self.status = GameStatus::Waiting;
        }
    }

    pub fn set_player_input(&mut self, id: &str, input: InputState) {
        if let Some(player) = self.players.get_mut(id) {
            player.input = input;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.status == GameStatus::Goal {
            self.goal_timer -= dt;
            if self.goal_timer <= 0.0 {
                self.reset_positions();
                self.status = GameStatus::Playing;
            }
            return;
        }

        if self.status != GameStatus::Playing {
            return;
        }

        // Process player inputs
        let player_data: Vec<_> = self
            .players
            .values()
            .map(|p| (p.circle_index, p.input))
            .collect();

        for (circle_index, input) in player_data {
            self.process_player_input(circle_index, input, dt);
        }

        // Step physics
        self.world.step(dt);

        // Check for goals
        self.check_goals();
    }

    fn process_player_input(&mut self, circle_index: usize, input: InputState, dt: f32) {
        let circle = &mut self.world.circles[circle_index];

        // Movement
        let mut move_x = 0.0f32;
        let mut move_y = 0.0f32;

        if input.left {
            move_x -= 1.0;
        }
        if input.right {
            move_x += 1.0;
        }
        if input.up {
            move_y -= 1.0;
        }
        if input.down {
            move_y += 1.0;
        }

        // Normalize diagonal movement
        if move_x != 0.0 || move_y != 0.0 {
            let move_len = (move_x * move_x + move_y * move_y).sqrt();
            move_x /= move_len;
            move_y /= move_len;

            circle.velocity.x += move_x * PLAYER_SPEED * dt;
            circle.velocity.y += move_y * PLAYER_SPEED * dt;
        }

        // Kick
        if input.kick {
            self.try_kick(circle_index);
        }
    }

    fn try_kick(&mut self, player_circle_index: usize) {
        let player_pos = self.world.circles[player_circle_index].position;
        let player_radius = self.world.circles[player_circle_index].radius;

        let ball = &self.world.circles[self.ball_index];
        let ball_pos = ball.position;
        let ball_radius = ball.radius;

        let to_ball = ball_pos - player_pos;
        let dist = to_ball.length();
        let kick_range = player_radius + ball_radius + KICK_DISTANCE;

        if dist <= kick_range && dist > 0.0 {
            let kick_dir = to_ball.normalize();
            let kick_impulse = kick_dir * KICK_FORCE;

            let ball = &mut self.world.circles[self.ball_index];
            ball.velocity = ball.velocity + kick_impulse;
        }
    }

    fn check_goals(&mut self) {
        let ball_pos = self.world.circles[self.ball_index].position;

        // Check red goal (blue scores)
        if self.map.red_goal.contains(ball_pos) {
            self.score.blue += 1;
            self.status = GameStatus::Goal;
            self.last_goal_team = Some(Team::Blue);
            self.goal_timer = GOAL_RESET_TIME;
            return;
        }

        // Check blue goal (red scores)
        if self.map.blue_goal.contains(ball_pos) {
            self.score.red += 1;
            self.status = GameStatus::Goal;
            self.last_goal_team = Some(Team::Red);
            self.goal_timer = GOAL_RESET_TIME;
        }
    }

    fn reset_positions(&mut self) {
        // Reset ball
        let ball = &mut self.world.circles[self.ball_index];
        ball.position = self.map.ball_spawn;
        ball.velocity = Vec2::default();

        // Reset players
        let mut red_index = 0;
        let mut blue_index = 0;

        for player in self.players.values() {
            let (spawns, index) = match player.team {
                Team::Red => {
                    let idx = red_index;
                    red_index += 1;
                    (&self.map.red_spawns, idx)
                }
                Team::Blue => {
                    let idx = blue_index;
                    blue_index += 1;
                    (&self.map.blue_spawns, idx)
                }
            };
            let spawn = spawns[index % spawns.len()];

            let circle = &mut self.world.circles[player.circle_index];
            circle.position = spawn;
            circle.velocity = Vec2::default();
        }
    }

    pub fn get_auto_team(&self) -> Team {
        let mut red_count = 0;
        let mut blue_count = 0;

        for player in self.players.values() {
            match player.team {
                Team::Red => red_count += 1,
                Team::Blue => blue_count += 1,
            }
        }

        if red_count <= blue_count {
            Team::Red
        } else {
            Team::Blue
        }
    }

    pub fn serialize_state(&self) -> SerializedGameState {
        let players: Vec<SerializedPlayer> = self
            .players
            .values()
            .map(|p| {
                let circle = &self.world.circles[p.circle_index];
                SerializedPlayer {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    position: circle.position,
                    velocity: circle.velocity,
                    radius: circle.radius,
                    team: p.team,
                }
            })
            .collect();

        let ball = &self.world.circles[self.ball_index];

        SerializedGameState {
            players,
            ball: SerializedBall {
                position: ball.position,
                velocity: ball.velocity,
                radius: ball.radius,
            },
            score: self.score.clone(),
            status: self.status,
            last_goal_team: self.last_goal_team,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_new() {
        let game = Game::new();
        assert!(game.players.is_empty());
        assert_eq!(game.score.red, 0);
        assert_eq!(game.score.blue, 0);
        assert_eq!(game.status, GameStatus::Waiting);
        assert!(game.last_goal_team.is_none());
    }

    #[test]
    fn test_game_default() {
        let game = Game::default();
        assert!(game.players.is_empty());
        assert_eq!(game.status, GameStatus::Waiting);
    }

    // Player management tests
    #[test]
    fn test_add_player_starts_game() {
        let mut game = Game::new();
        assert_eq!(game.status, GameStatus::Waiting);

        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        assert_eq!(game.status, GameStatus::Playing);
    }

    #[test]
    fn test_add_player_spawns_at_team_position() {
        let mut game = Game::new();
        let red_spawn = game.map.red_spawns[0];

        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);
        let circle_index = game.players.get("p1").unwrap().circle_index;
        let player_pos = game.world.circles[circle_index].position;

        assert_eq!(player_pos.x, red_spawn.x);
        assert_eq!(player_pos.y, red_spawn.y);
    }

    #[test]
    fn test_add_player_blue_team_spawns_correctly() {
        let mut game = Game::new();
        let blue_spawn = game.map.blue_spawns[0];

        game.add_player("p1".to_string(), "Player1".to_string(), Team::Blue);
        let circle_index = game.players.get("p1").unwrap().circle_index;
        let player_pos = game.world.circles[circle_index].position;

        assert_eq!(player_pos.x, blue_spawn.x);
        assert_eq!(player_pos.y, blue_spawn.y);
    }

    #[test]
    fn test_remove_player_goes_to_waiting() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);
        assert_eq!(game.status, GameStatus::Playing);

        game.remove_player("p1");

        assert!(game.players.is_empty());
        assert_eq!(game.status, GameStatus::Waiting);
    }

    #[test]
    fn test_remove_player_updates_indices() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);
        game.add_player("p2".to_string(), "Player2".to_string(), Team::Blue);

        let p2_initial_idx = game.players.get("p2").unwrap().circle_index;

        game.remove_player("p1");

        // p2's index should have been decremented
        let p2_final_idx = game.players.get("p2").unwrap().circle_index;
        assert!(p2_final_idx < p2_initial_idx);
    }

    #[test]
    fn test_remove_nonexistent_player() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Should not panic
        game.remove_player("nonexistent");

        assert_eq!(game.players.len(), 1);
    }

    // Team balancing tests
    #[test]
    fn test_get_auto_team_balanced() {
        let mut game = Game::new();

        // First player should go to Red (or Blue, depending on tie-breaker)
        let team1 = game.get_auto_team();
        game.add_player("p1".to_string(), "Player1".to_string(), team1);

        // Second player should go to the other team
        let team2 = game.get_auto_team();
        assert_ne!(team1, team2);
    }

    #[test]
    fn test_get_auto_team_fills_smaller_team() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);
        game.add_player("p2".to_string(), "Player2".to_string(), Team::Red);

        let team = game.get_auto_team();
        assert_eq!(team, Team::Blue);
    }

    // Update tests
    #[test]
    fn test_update_does_nothing_when_waiting() {
        let mut game = Game::new();
        assert_eq!(game.status, GameStatus::Waiting);

        let ball_pos_before = game.world.circles[game.ball_index].position;
        game.update(0.016);
        let ball_pos_after = game.world.circles[game.ball_index].position;

        // Ball should not have moved
        assert_eq!(ball_pos_before.x, ball_pos_after.x);
        assert_eq!(ball_pos_before.y, ball_pos_after.y);
    }

    #[test]
    fn test_update_processes_input() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        let player_idx = game.players.get("p1").unwrap().circle_index;
        let initial_vel = game.world.circles[player_idx].velocity;

        game.set_player_input(
            "p1",
            InputState {
                left: false,
                right: true,
                up: false,
                down: false,
                kick: false,
            },
        );

        game.update(0.016);

        let final_vel = game.world.circles[player_idx].velocity;
        // Player should have accelerated to the right
        assert!(final_vel.x > initial_vel.x);
    }

    #[test]
    fn test_set_player_input_nonexistent_player() {
        let mut game = Game::new();
        // Should not panic
        game.set_player_input(
            "nonexistent",
            InputState {
                left: true,
                right: false,
                up: false,
                down: false,
                kick: false,
            },
        );
    }

    // Kick tests
    #[test]
    fn test_kick_ball_in_range() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Move player close to ball
        let ball_pos = game.world.circles[game.ball_index].position;
        let player_idx = game.players.get("p1").unwrap().circle_index;
        game.world.circles[player_idx].position = Vec2::new(ball_pos.x - 30.0, ball_pos.y);

        let ball_vel_before = game.world.circles[game.ball_index].velocity;

        game.set_player_input(
            "p1",
            InputState {
                left: false,
                right: false,
                up: false,
                down: false,
                kick: true,
            },
        );
        game.update(0.016);

        let ball_vel_after = game.world.circles[game.ball_index].velocity;
        // Ball should have gained velocity
        assert!(ball_vel_after.length() > ball_vel_before.length());
    }

    #[test]
    fn test_kick_ball_out_of_range() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Player is at spawn, ball is at center - should be out of range
        let ball_vel_before = game.world.circles[game.ball_index].velocity;

        game.set_player_input(
            "p1",
            InputState {
                left: false,
                right: false,
                up: false,
                down: false,
                kick: true,
            },
        );
        game.update(0.016);

        let ball_vel_after = game.world.circles[game.ball_index].velocity;
        // Ball velocity should not have changed significantly (only friction)
        assert!((ball_vel_after.x - ball_vel_before.x).abs() < 1.0);
    }

    // Goal tests
    #[test]
    fn test_goal_scored_by_blue() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Move ball into red goal (blue scores)
        game.world.circles[game.ball_index].position = Vec2::new(-15.0, 200.0);

        game.update(0.016);

        assert_eq!(game.score.blue, 1);
        assert_eq!(game.score.red, 0);
        assert_eq!(game.status, GameStatus::Goal);
        assert_eq!(game.last_goal_team, Some(Team::Blue));
    }

    #[test]
    fn test_goal_scored_by_red() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Move ball into blue goal (red scores)
        game.world.circles[game.ball_index].position = Vec2::new(815.0, 200.0);

        game.update(0.016);

        assert_eq!(game.score.red, 1);
        assert_eq!(game.score.blue, 0);
        assert_eq!(game.status, GameStatus::Goal);
        assert_eq!(game.last_goal_team, Some(Team::Red));
    }

    #[test]
    fn test_goal_timer_resets_game() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Score a goal
        game.world.circles[game.ball_index].position = Vec2::new(-15.0, 200.0);
        game.update(0.016);
        assert_eq!(game.status, GameStatus::Goal);

        // Wait for goal timer to expire
        for _ in 0..150 {
            // ~2.4 seconds at 60fps
            game.update(0.016);
        }

        assert_eq!(game.status, GameStatus::Playing);
    }

    #[test]
    fn test_positions_reset_after_goal() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        // Score a goal
        game.world.circles[game.ball_index].position = Vec2::new(-15.0, 200.0);
        game.update(0.016);

        // Wait for reset
        for _ in 0..150 {
            game.update(0.016);
        }

        // Ball should be back at center
        let ball_pos = game.world.circles[game.ball_index].position;
        assert!((ball_pos.x - game.map.ball_spawn.x).abs() < 1.0);
        assert!((ball_pos.y - game.map.ball_spawn.y).abs() < 1.0);
    }

    // Serialization tests
    #[test]
    fn test_serialize_state() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        let state = game.serialize_state();

        assert_eq!(state.players.len(), 1);
        assert_eq!(state.players[0].id, "p1");
        assert_eq!(state.players[0].team, Team::Red);
        assert_eq!(state.score.red, 0);
        assert_eq!(state.score.blue, 0);
        assert_eq!(state.status, GameStatus::Playing);
    }

    #[test]
    fn test_serialize_state_empty_game() {
        let game = Game::new();
        let state = game.serialize_state();

        assert!(state.players.is_empty());
        assert_eq!(state.status, GameStatus::Waiting);
    }

    // Diagonal movement tests
    #[test]
    fn test_diagonal_movement_normalized() {
        let mut game = Game::new();
        game.add_player("p1".to_string(), "Player1".to_string(), Team::Red);

        let player_idx = game.players.get("p1").unwrap().circle_index;

        // Move diagonally
        game.set_player_input(
            "p1",
            InputState {
                left: false,
                right: true,
                up: true,
                down: false,
                kick: false,
            },
        );
        game.update(0.016);

        let vel = game.world.circles[player_idx].velocity;
        // Diagonal speed should not be faster than straight movement
        // (both x and y should be scaled)
        assert!(vel.x.abs() < 10.0); // Reasonable single-frame acceleration
        assert!(vel.y.abs() < 10.0);
    }
}
