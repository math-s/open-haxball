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

#[cfg(test)]
mod tests {
    use super::*;

    // Goal tests
    #[test]
    fn test_goal_contains_inside() {
        let goal = Goal {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(100.0, 100.0),
        };

        assert!(goal.contains(Vec2::new(50.0, 50.0)));
        assert!(goal.contains(Vec2::new(0.0, 0.0))); // Edge
        assert!(goal.contains(Vec2::new(100.0, 100.0))); // Edge
    }

    #[test]
    fn test_goal_contains_outside() {
        let goal = Goal {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(100.0, 100.0),
        };

        assert!(!goal.contains(Vec2::new(-1.0, 50.0)));
        assert!(!goal.contains(Vec2::new(101.0, 50.0)));
        assert!(!goal.contains(Vec2::new(50.0, -1.0)));
        assert!(!goal.contains(Vec2::new(50.0, 101.0)));
    }

    #[test]
    fn test_goal_contains_on_boundary() {
        let goal = Goal {
            min: Vec2::new(10.0, 20.0),
            max: Vec2::new(30.0, 40.0),
        };

        // On boundaries should be inside (inclusive)
        assert!(goal.contains(Vec2::new(10.0, 30.0)));
        assert!(goal.contains(Vec2::new(30.0, 30.0)));
        assert!(goal.contains(Vec2::new(20.0, 20.0)));
        assert!(goal.contains(Vec2::new(20.0, 40.0)));
    }

    // GameMap tests
    #[test]
    fn test_game_map_new() {
        let map = GameMap::new();

        assert!(!map.walls.is_empty());
        assert!(!map.red_spawns.is_empty());
        assert!(!map.blue_spawns.is_empty());
    }

    #[test]
    fn test_game_map_default() {
        let map = GameMap::default();

        assert!(!map.walls.is_empty());
    }

    #[test]
    fn test_game_map_has_spawn_points() {
        let map = GameMap::new();

        assert!(map.red_spawns.len() >= 1);
        assert!(map.blue_spawns.len() >= 1);
    }

    #[test]
    fn test_game_map_ball_spawn_is_center() {
        let map = GameMap::new();

        assert_eq!(map.ball_spawn.x, MAP_WIDTH / 2.0);
        assert_eq!(map.ball_spawn.y, MAP_HEIGHT / 2.0);
    }

    #[test]
    fn test_game_map_red_spawns_on_left() {
        let map = GameMap::new();

        for spawn in &map.red_spawns {
            assert!(spawn.x < MAP_WIDTH / 2.0);
        }
    }

    #[test]
    fn test_game_map_blue_spawns_on_right() {
        let map = GameMap::new();

        for spawn in &map.blue_spawns {
            assert!(spawn.x > MAP_WIDTH / 2.0);
        }
    }

    #[test]
    fn test_game_map_goals_positioned_correctly() {
        let map = GameMap::new();

        // Red goal is on left (x <= 0)
        assert!(map.red_goal.max.x <= 0.0);

        // Blue goal is on right (x >= MAP_WIDTH)
        assert!(map.blue_goal.min.x >= MAP_WIDTH);
    }

    #[test]
    fn test_game_map_goals_vertically_centered() {
        let map = GameMap::new();

        let goal_center_y = (map.red_goal.min.y + map.red_goal.max.y) / 2.0;
        let map_center_y = MAP_HEIGHT / 2.0;

        assert!((goal_center_y - map_center_y).abs() < 0.001);
    }

    #[test]
    fn test_game_map_has_enough_walls() {
        let map = GameMap::new();

        // Should have at least:
        // - Top and bottom walls
        // - Left wall segments (above and below goal)
        // - Right wall segments (above and below goal)
        // - Goal walls (3 per goal)
        assert!(map.walls.len() >= 12);
    }

    #[test]
    fn test_game_map_wall_restitution() {
        let map = GameMap::new();

        for wall in &map.walls {
            assert!(wall.restitution > 0.0);
            assert!(wall.restitution <= 1.0);
        }
    }

    #[test]
    fn test_game_map_spawns_inside_boundaries() {
        let map = GameMap::new();

        for spawn in &map.red_spawns {
            assert!(spawn.x >= 0.0 && spawn.x <= MAP_WIDTH);
            assert!(spawn.y >= 0.0 && spawn.y <= MAP_HEIGHT);
        }

        for spawn in &map.blue_spawns {
            assert!(spawn.x >= 0.0 && spawn.x <= MAP_WIDTH);
            assert!(spawn.y >= 0.0 && spawn.y <= MAP_HEIGHT);
        }
    }
}
