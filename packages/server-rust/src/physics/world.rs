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
