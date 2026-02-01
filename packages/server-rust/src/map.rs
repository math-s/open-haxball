use crate::physics::{Line, Vec2};

pub const MAP_WIDTH: f32 = 800.0;
pub const MAP_HEIGHT: f32 = 400.0;
pub const GOAL_WIDTH: f32 = 120.0;
pub const GOAL_DEPTH: f32 = 30.0;

const WALL_RESTITUTION: f32 = 0.8;

pub struct Goal {
    pub min: Vec2,
    pub max: Vec2,
}

impl Goal {
    pub fn contains(&self, position: Vec2) -> bool {
        position.x >= self.min.x
            && position.x <= self.max.x
            && position.y >= self.min.y
            && position.y <= self.max.y
    }
}

pub struct GameMap {
    pub walls: Vec<Line>,
    pub red_goal: Goal,
    pub blue_goal: Goal,
    pub red_spawns: Vec<Vec2>,
    pub blue_spawns: Vec<Vec2>,
    pub ball_spawn: Vec2,
}

impl GameMap {
    pub fn new() -> Self {
        let width = MAP_WIDTH;
        let height = MAP_HEIGHT;
        let goal_width = GOAL_WIDTH;
        let goal_depth = GOAL_DEPTH;

        // Goal positions (centered vertically)
        let goal_top = (height - goal_width) / 2.0;
        let goal_bottom = (height + goal_width) / 2.0;

        let walls = vec![
            // Top wall
            Line::new(Vec2::new(0.0, 0.0), Vec2::new(width, 0.0), WALL_RESTITUTION),
            // Bottom wall
            Line::new(Vec2::new(0.0, height), Vec2::new(width, height), WALL_RESTITUTION),
            // Left wall (above goal)
            Line::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, goal_top), WALL_RESTITUTION),
            // Left wall (below goal)
            Line::new(Vec2::new(0.0, goal_bottom), Vec2::new(0.0, height), WALL_RESTITUTION),
            // Right wall (above goal)
            Line::new(Vec2::new(width, 0.0), Vec2::new(width, goal_top), WALL_RESTITUTION),
            // Right wall (below goal)
            Line::new(Vec2::new(width, goal_bottom), Vec2::new(width, height), WALL_RESTITUTION),
            // Red goal (left side) walls
            Line::new(Vec2::new(-goal_depth, goal_top), Vec2::new(0.0, goal_top), WALL_RESTITUTION),
            Line::new(Vec2::new(-goal_depth, goal_bottom), Vec2::new(0.0, goal_bottom), WALL_RESTITUTION),
            Line::new(Vec2::new(-goal_depth, goal_top), Vec2::new(-goal_depth, goal_bottom), WALL_RESTITUTION),
            // Blue goal (right side) walls
            Line::new(Vec2::new(width, goal_top), Vec2::new(width + goal_depth, goal_top), WALL_RESTITUTION),
            Line::new(Vec2::new(width, goal_bottom), Vec2::new(width + goal_depth, goal_bottom), WALL_RESTITUTION),
            Line::new(Vec2::new(width + goal_depth, goal_top), Vec2::new(width + goal_depth, goal_bottom), WALL_RESTITUTION),
        ];

        Self {
            walls,
            red_goal: Goal {
                min: Vec2::new(-goal_depth, goal_top),
                max: Vec2::new(0.0, goal_bottom),
            },
            blue_goal: Goal {
                min: Vec2::new(width, goal_top),
                max: Vec2::new(width + goal_depth, goal_bottom),
            },
            red_spawns: vec![
                Vec2::new(150.0, height / 2.0),
                Vec2::new(200.0, height / 2.0 - 80.0),
                Vec2::new(200.0, height / 2.0 + 80.0),
                Vec2::new(100.0, height / 2.0),
            ],
            blue_spawns: vec![
                Vec2::new(width - 150.0, height / 2.0),
                Vec2::new(width - 200.0, height / 2.0 - 80.0),
                Vec2::new(width - 200.0, height / 2.0 + 80.0),
                Vec2::new(width - 100.0, height / 2.0),
            ],
            ball_spawn: Vec2::new(width / 2.0, height / 2.0),
        }
    }
}

impl Default for GameMap {
    fn default() -> Self {
        Self::new()
    }
}
