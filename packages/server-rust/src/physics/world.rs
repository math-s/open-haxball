use super::collision::{
    circle_vs_circle, circle_vs_line, resolve_circle_vs_circle, resolve_circle_vs_line,
};
use super::{Circle, Line, Vec2};

pub const PHYSICS_TICK_RATE: u32 = 60;
pub const PHYSICS_DT: f32 = 1.0 / PHYSICS_TICK_RATE as f32;

const COLLISION_ITERATIONS: u32 = 3;
const MAX_VELOCITY: f32 = 1000.0;

pub struct PhysicsWorld {
    pub circles: Vec<Circle>,
    pub lines: Vec<Line>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            circles: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn add_circle(&mut self, circle: Circle) -> usize {
        let index = self.circles.len();
        self.circles.push(circle);
        index
    }

    pub fn remove_circle(&mut self, index: usize) {
        if index < self.circles.len() {
            self.circles.remove(index);
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    pub fn step(&mut self, dt: f32) {
        // Apply friction and clamp velocities
        for circle in &mut self.circles {
            if circle.is_static {
                continue;
            }

            // Apply friction (velocity damping)
            circle.velocity = circle.velocity.mul(circle.friction);

            // Clamp max velocity to prevent tunneling
            let speed = circle.velocity.length();
            if speed > MAX_VELOCITY {
                circle.velocity = circle.velocity.mul(MAX_VELOCITY / speed);
            }
        }

        // Use substeps for fast-moving objects
        let substeps = 2;
        let sub_dt = dt / substeps as f32;

        for _ in 0..substeps {
            // Update positions
            for circle in &mut self.circles {
                if circle.is_static {
                    continue;
                }
                circle.position.x += circle.velocity.x * sub_dt;
                circle.position.y += circle.velocity.y * sub_dt;

                // Safeguard against NaN positions
                if circle.position.x.is_nan() || circle.position.y.is_nan() {
                    circle.position = Vec2::new(400.0, 200.0);
                    circle.velocity = Vec2::default();
                }
            }

            // Multiple collision iterations per substep
            for _ in 0..COLLISION_ITERATIONS {
                // Resolve circle vs line collisions first (walls are more important)
                for circle_idx in 0..self.circles.len() {
                    for line in &self.lines {
                        let result = circle_vs_line(&self.circles[circle_idx], line);
                        if result.collided {
                            resolve_circle_vs_line(&mut self.circles[circle_idx], line, &result);
                        }
                    }
                }

                // Resolve circle vs circle collisions
                for i in 0..self.circles.len() {
                    for j in (i + 1)..self.circles.len() {
                        let result = circle_vs_circle(&self.circles[i], &self.circles[j]);
                        if result.collided {
                            // Need to split borrow
                            let (left, right) = self.circles.split_at_mut(j);
                            resolve_circle_vs_circle(&mut left[i], &mut right[0], &result);
                        }
                    }
                }
            }
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_circle(x: f32, y: f32, radius: f32) -> Circle {
        Circle::new(Vec2::new(x, y), radius, 1.0, 0.8, 0.99, false)
    }

    fn create_static_circle(x: f32, y: f32, radius: f32) -> Circle {
        Circle::new(Vec2::new(x, y), radius, 1.0, 0.8, 0.99, true)
    }

    #[test]
    fn test_physics_world_new_is_empty() {
        let world = PhysicsWorld::new();
        assert!(world.circles.is_empty());
        assert!(world.lines.is_empty());
    }

    #[test]
    fn test_physics_world_default_is_empty() {
        let world = PhysicsWorld::default();
        assert!(world.circles.is_empty());
        assert!(world.lines.is_empty());
    }

    #[test]
    fn test_add_circle_returns_index() {
        let mut world = PhysicsWorld::new();
        let idx1 = world.add_circle(create_circle(0.0, 0.0, 10.0));
        let idx2 = world.add_circle(create_circle(50.0, 50.0, 10.0));
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(world.circles.len(), 2);
    }

    #[test]
    fn test_remove_circle() {
        let mut world = PhysicsWorld::new();
        world.add_circle(create_circle(0.0, 0.0, 10.0));
        world.add_circle(create_circle(50.0, 50.0, 10.0));
        assert_eq!(world.circles.len(), 2);

        world.remove_circle(0);
        assert_eq!(world.circles.len(), 1);
    }

    #[test]
    fn test_remove_circle_out_of_bounds() {
        let mut world = PhysicsWorld::new();
        world.add_circle(create_circle(0.0, 0.0, 10.0));
        // Should not panic
        world.remove_circle(100);
        assert_eq!(world.circles.len(), 1);
    }

    #[test]
    fn test_add_line() {
        let mut world = PhysicsWorld::new();
        let line = Line::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 0.8);
        world.add_line(line);
        assert_eq!(world.lines.len(), 1);
    }

    #[test]
    fn test_step_updates_position() {
        let mut world = PhysicsWorld::new();
        let mut circle = create_circle(0.0, 0.0, 10.0);
        circle.velocity = Vec2::new(100.0, 0.0);
        world.add_circle(circle);

        let initial_x = world.circles[0].position.x;
        world.step(PHYSICS_DT);
        let final_x = world.circles[0].position.x;

        // Position should have moved
        assert!(final_x > initial_x);
    }

    #[test]
    fn test_step_applies_friction() {
        let mut world = PhysicsWorld::new();
        let mut circle = create_circle(0.0, 0.0, 10.0);
        circle.velocity = Vec2::new(100.0, 0.0);
        world.add_circle(circle);

        let initial_speed = world.circles[0].velocity.length();
        world.step(PHYSICS_DT);
        let final_speed = world.circles[0].velocity.length();

        // Velocity should decrease due to friction
        assert!(final_speed < initial_speed);
    }

    #[test]
    fn test_step_clamps_max_velocity() {
        let mut world = PhysicsWorld::new();
        let mut circle = create_circle(0.0, 0.0, 10.0);
        circle.velocity = Vec2::new(5000.0, 0.0); // Way above MAX_VELOCITY
        world.add_circle(circle);

        world.step(PHYSICS_DT);

        let speed = world.circles[0].velocity.length();
        assert!(speed <= MAX_VELOCITY);
    }

    #[test]
    fn test_step_static_circle_does_not_move() {
        let mut world = PhysicsWorld::new();
        let mut circle = create_static_circle(100.0, 100.0, 10.0);
        circle.velocity = Vec2::new(100.0, 100.0); // Try to give it velocity
        world.add_circle(circle);

        let initial_pos = world.circles[0].position;
        world.step(PHYSICS_DT);
        let final_pos = world.circles[0].position;

        assert_eq!(initial_pos.x, final_pos.x);
        assert_eq!(initial_pos.y, final_pos.y);
    }

    #[test]
    fn test_step_nan_position_recovery() {
        let mut world = PhysicsWorld::new();
        let mut circle = create_circle(f32::NAN, f32::NAN, 10.0);
        circle.velocity = Vec2::new(10.0, 10.0);
        world.add_circle(circle);

        world.step(PHYSICS_DT);

        // Position should be recovered to default
        assert!(!world.circles[0].position.x.is_nan());
        assert!(!world.circles[0].position.y.is_nan());
    }

    #[test]
    fn test_circle_vs_line_collision_in_step() {
        let mut world = PhysicsWorld::new();
        // Circle close to line, moving toward it
        let mut circle = create_circle(50.0, 8.0, 10.0);
        circle.velocity = Vec2::new(0.0, -100.0);
        world.add_circle(circle);

        // Horizontal line at y=0
        world.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 0.8));

        // After step, circle should have collided and bounced
        world.step(PHYSICS_DT);

        // After collision, the circle should be pushed away from line
        // and velocity should be reversed (positive y, moving up)
        assert!(
            world.circles[0].position.y > 0.0,
            "Circle should be above the line"
        );
    }

    #[test]
    fn test_circle_vs_circle_collision_in_step() {
        let mut world = PhysicsWorld::new();
        // Two circles moving toward each other
        let mut circle1 = create_circle(0.0, 0.0, 10.0);
        circle1.velocity = Vec2::new(100.0, 0.0);
        world.add_circle(circle1);

        let mut circle2 = create_circle(25.0, 0.0, 10.0);
        circle2.velocity = Vec2::new(-100.0, 0.0);
        world.add_circle(circle2);

        world.step(PHYSICS_DT);

        // Circles should have collided and separated
        let dist = world.circles[0]
            .position
            .distance(world.circles[1].position);
        assert!(dist >= 19.0); // Should be at least sum of radii (minus small tolerance)
    }

    #[test]
    fn test_multiple_circles_no_crash() {
        let mut world = PhysicsWorld::new();
        // Add several circles
        for i in 0..10 {
            let circle = create_circle(i as f32 * 30.0, i as f32 * 30.0, 10.0);
            world.add_circle(circle);
        }

        // Should not crash
        for _ in 0..60 {
            world.step(PHYSICS_DT);
        }

        assert_eq!(world.circles.len(), 10);
    }
}
