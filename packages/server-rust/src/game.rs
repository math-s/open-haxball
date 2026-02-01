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
        if self.players.len() >= 1 && self.status == GameStatus::Waiting {
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

        let to_ball = ball_pos.sub(player_pos);
        let dist = to_ball.length();
        let kick_range = player_radius + ball_radius + KICK_DISTANCE;

        if dist <= kick_range && dist > 0.0 {
            let kick_dir = to_ball.normalize();
            let kick_impulse = kick_dir.mul(KICK_FORCE);

            let ball = &mut self.world.circles[self.ball_index];
            ball.velocity = ball.velocity.add(kick_impulse);
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
