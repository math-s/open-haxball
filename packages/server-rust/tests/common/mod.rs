use haxball_server::game::Game;
use haxball_server::protocol::{InputState, Team};

/// Creates a game with the specified number of players on each team
pub fn create_game_with_players(red: usize, blue: usize) -> Game {
    let mut game = Game::new();

    for i in 0..red {
        game.add_player(format!("red_{}", i), format!("RedPlayer{}", i), Team::Red);
    }

    for i in 0..blue {
        game.add_player(
            format!("blue_{}", i),
            format!("BluePlayer{}", i),
            Team::Blue,
        );
    }

    game
}

/// Simulates the given number of physics ticks
pub fn simulate_ticks(game: &mut Game, ticks: usize) {
    let dt = 1.0 / 60.0;
    for _ in 0..ticks {
        game.update(dt);
    }
}

/// Checks if the ball is within the map boundaries
pub fn ball_is_in_bounds(game: &Game) -> bool {
    let ball_pos = game.world.circles[game.ball_index].position;
    // Allow some tolerance for goal areas
    ball_pos.x >= -50.0 && ball_pos.x <= 850.0 && ball_pos.y >= -10.0 && ball_pos.y <= 410.0
}

/// Builder for creating InputState with fluent API
pub struct InputBuilder {
    state: InputState,
}

impl InputBuilder {
    pub fn new() -> Self {
        Self {
            state: InputState::default(),
        }
    }

    pub fn left(mut self) -> Self {
        self.state.left = true;
        self
    }

    pub fn right(mut self) -> Self {
        self.state.right = true;
        self
    }

    pub fn up(mut self) -> Self {
        self.state.up = true;
        self
    }

    pub fn down(mut self) -> Self {
        self.state.down = true;
        self
    }

    pub fn kick(mut self) -> Self {
        self.state.kick = true;
        self
    }

    pub fn build(self) -> InputState {
        self.state
    }
}

impl Default for InputBuilder {
    fn default() -> Self {
        Self::new()
    }
}
