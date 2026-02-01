use super::{Circle, Line, Vec2};

const EPSILON: f32 = 0.0001;
const SEPARATION_BUFFER: f32 = 0.5;

pub struct CollisionResult {
    pub collided: bool,
    pub normal: Vec2,
    pub penetration: f32,
}

impl Default for CollisionResult {
    fn default() -> Self {
        Self {
            collided: false,
            normal: Vec2::default(),
            penetration: 0.0,
        }
    }
}

pub fn circle_vs_circle(a: &Circle, b: &Circle) -> CollisionResult {
    let diff = b.position.sub(a.position);
    let dist = diff.length();
    let min_dist = a.radius + b.radius;

    if dist >= min_dist || dist < EPSILON {
        return CollisionResult::default();
    }

    let normal = diff.normalize();
    let penetration = min_dist - dist;

    CollisionResult {
        collided: true,
        normal,
        penetration,
    }
}

pub fn resolve_circle_vs_circle(a: &mut Circle, b: &mut Circle, result: &CollisionResult) {
    if !result.collided {
        return;
    }

    let total_inv_mass = a.inv_mass + b.inv_mass;
    if total_inv_mass == 0.0 {
        return;
    }

    // Separate circles
    let correction = result.normal.mul(result.penetration / total_inv_mass);
    a.position = a.position.sub(correction.mul(a.inv_mass));
    b.position = b.position.add(correction.mul(b.inv_mass));

    // Calculate relative velocity
    let relative_velocity = b.velocity.sub(a.velocity);
    let velocity_along_normal = relative_velocity.dot(result.normal);

    // Don't resolve if velocities are separating
    if velocity_along_normal > 0.0 {
        return;
    }

    // Restitution
    let e = a.restitution.min(b.restitution);

    // Impulse magnitude
    let j = -(1.0 + e) * velocity_along_normal / total_inv_mass;

    // Apply impulse
    let impulse = result.normal.mul(j);
    a.velocity = a.velocity.sub(impulse.mul(a.inv_mass));
    b.velocity = b.velocity.add(impulse.mul(b.inv_mass));
}

pub fn circle_vs_line(circle: &Circle, line: &Line) -> CollisionResult {
    let line_vec = line.p2.sub(line.p1);
    let line_length = line_vec.length();
    let line_dir = line_vec.normalize();

    // Project circle center onto line
    let to_circle = circle.position.sub(line.p1);
    let projection = to_circle.dot(line_dir);

    let closest_point = if projection <= 0.0 {
        line.p1
    } else if projection >= line_length {
        line.p2
    } else {
        line.p1.add(line_dir.mul(projection))
    };

    let dist = circle.position.distance(closest_point);

    if dist >= circle.radius || dist < EPSILON {
        return CollisionResult::default();
    }

    let normal = circle.position.sub(closest_point).normalize();
    let penetration = circle.radius - dist;

    CollisionResult {
        collided: true,
        normal,
        penetration,
    }
}

pub fn resolve_circle_vs_line(circle: &mut Circle, line: &Line, result: &CollisionResult) {
    if !result.collided || circle.is_static {
        return;
    }

    // Separate circle from line with a small buffer
    circle.position = circle.position.add(result.normal.mul(result.penetration + SEPARATION_BUFFER));

    // Reflect velocity
    let velocity_along_normal = circle.velocity.dot(result.normal);

    // Only resolve if moving towards the line
    if velocity_along_normal >= 0.0 {
        return;
    }

    let e = circle.restitution.min(line.restitution);
    let impulse = -(1.0 + e) * velocity_along_normal;

    circle.velocity = circle.velocity.add(result.normal.mul(impulse));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_circle(x: f32, y: f32, radius: f32) -> Circle {
        Circle::new(
            Vec2::new(x, y),
            radius,
            1.0,       // mass
            0.8,       // restitution
            0.99,      // friction
            false,     // is_static
        )
    }

    fn create_static_circle(x: f32, y: f32, radius: f32) -> Circle {
        Circle::new(
            Vec2::new(x, y),
            radius,
            1.0,
            0.8,
            0.99,
            true,
        )
    }

    // Circle vs Circle collision detection tests
    #[test]
    fn test_circle_vs_circle_no_collision_far_apart() {
        let a = create_circle(0.0, 0.0, 10.0);
        let b = create_circle(100.0, 0.0, 10.0);
        let result = circle_vs_circle(&a, &b);
        assert!(!result.collided);
    }

    #[test]
    fn test_circle_vs_circle_collision_overlapping() {
        let a = create_circle(0.0, 0.0, 10.0);
        let b = create_circle(15.0, 0.0, 10.0);
        let result = circle_vs_circle(&a, &b);
        assert!(result.collided);
        assert!(result.penetration > 0.0);
        // Normal should point from a to b
        assert!(result.normal.x > 0.0);
    }

    #[test]
    fn test_circle_vs_circle_exactly_touching() {
        let a = create_circle(0.0, 0.0, 10.0);
        let b = create_circle(20.0, 0.0, 10.0);
        let result = circle_vs_circle(&a, &b);
        // Exactly touching means dist == min_dist, which is not collision
        assert!(!result.collided);
    }

    #[test]
    fn test_circle_vs_circle_same_position_no_crash() {
        let a = create_circle(50.0, 50.0, 10.0);
        let b = create_circle(50.0, 50.0, 10.0);
        let result = circle_vs_circle(&a, &b);
        // Same position means dist < EPSILON, so no collision to prevent NaN
        assert!(!result.collided);
    }

    #[test]
    fn test_circle_vs_circle_diagonal_collision() {
        let a = create_circle(0.0, 0.0, 10.0);
        let b = create_circle(10.0, 10.0, 10.0);
        let result = circle_vs_circle(&a, &b);
        assert!(result.collided);
        // Normal should be normalized diagonal
        let expected_len = result.normal.length();
        assert!((expected_len - 1.0).abs() < 0.0001);
    }

    // Circle vs Circle resolution tests
    #[test]
    fn test_resolve_circle_vs_circle_separates_circles() {
        let mut a = create_circle(0.0, 0.0, 10.0);
        let mut b = create_circle(15.0, 0.0, 10.0);
        let result = circle_vs_circle(&a, &b);

        let initial_dist = a.position.distance(b.position);
        resolve_circle_vs_circle(&mut a, &mut b, &result);
        let final_dist = a.position.distance(b.position);

        // After resolution, circles should be further apart
        assert!(final_dist >= initial_dist);
    }

    #[test]
    fn test_resolve_circle_vs_circle_velocity_change() {
        let mut a = create_circle(0.0, 0.0, 10.0);
        a.velocity = Vec2::new(100.0, 0.0); // Moving right
        let mut b = create_circle(15.0, 0.0, 10.0);
        b.velocity = Vec2::new(0.0, 0.0); // Stationary

        let result = circle_vs_circle(&a, &b);
        resolve_circle_vs_circle(&mut a, &mut b, &result);

        // After collision, a should slow down and b should speed up
        assert!(a.velocity.x < 100.0);
        assert!(b.velocity.x > 0.0);
    }

    #[test]
    fn test_resolve_circle_vs_circle_no_resolution_when_separating() {
        let mut a = create_circle(0.0, 0.0, 10.0);
        a.velocity = Vec2::new(-100.0, 0.0); // Moving away
        let mut b = create_circle(15.0, 0.0, 10.0);
        b.velocity = Vec2::new(100.0, 0.0); // Moving away

        let initial_vel_a = a.velocity;
        let initial_vel_b = b.velocity;

        let result = circle_vs_circle(&a, &b);
        resolve_circle_vs_circle(&mut a, &mut b, &result);

        // Velocities should not change (they're already separating)
        // Note: positions may still be adjusted for overlap
        assert!((a.velocity.x - initial_vel_a.x).abs() < 0.001);
        assert!((b.velocity.x - initial_vel_b.x).abs() < 0.001);
    }

    #[test]
    fn test_resolve_circle_vs_circle_static_circle_does_not_move() {
        let mut a = create_circle(0.0, 0.0, 10.0);
        a.velocity = Vec2::new(100.0, 0.0);
        let mut b = create_static_circle(15.0, 0.0, 10.0);

        let initial_pos = b.position;

        let result = circle_vs_circle(&a, &b);
        resolve_circle_vs_circle(&mut a, &mut b, &result);

        // Static circle should not move
        assert_eq!(b.position.x, initial_pos.x);
        assert_eq!(b.position.y, initial_pos.y);
    }

    // Circle vs Line collision detection tests
    #[test]
    fn test_circle_vs_line_no_collision() {
        let circle = create_circle(50.0, 50.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );
        let result = circle_vs_line(&circle, &line);
        assert!(!result.collided);
    }

    #[test]
    fn test_circle_vs_line_collision_perpendicular() {
        let circle = create_circle(50.0, 5.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );
        let result = circle_vs_line(&circle, &line);
        assert!(result.collided);
        // Normal should point up (away from line)
        assert!(result.normal.y > 0.0);
    }

    #[test]
    fn test_circle_vs_line_collision_near_endpoint() {
        let circle = create_circle(5.0, 5.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );
        let result = circle_vs_line(&circle, &line);
        assert!(result.collided);
    }

    #[test]
    fn test_circle_vs_line_vertical_line() {
        let circle = create_circle(5.0, 50.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 100.0),
            0.8,
        );
        let result = circle_vs_line(&circle, &line);
        assert!(result.collided);
        // Normal should point right
        assert!(result.normal.x > 0.0);
    }

    #[test]
    fn test_circle_vs_line_at_line_endpoint() {
        let circle = create_circle(105.0, 0.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );
        let result = circle_vs_line(&circle, &line);
        assert!(result.collided);
    }

    // Circle vs Line resolution tests
    #[test]
    fn test_resolve_circle_vs_line_separates_circle() {
        let mut circle = create_circle(50.0, 5.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );

        let result = circle_vs_line(&circle, &line);
        let initial_y = circle.position.y;
        resolve_circle_vs_line(&mut circle, &line, &result);

        // Circle should be pushed away from line
        assert!(circle.position.y > initial_y);
    }

    #[test]
    fn test_resolve_circle_vs_line_reflects_velocity() {
        let mut circle = create_circle(50.0, 5.0, 10.0);
        circle.velocity = Vec2::new(0.0, -100.0); // Moving toward line
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );

        let result = circle_vs_line(&circle, &line);
        resolve_circle_vs_line(&mut circle, &line, &result);

        // Velocity should now point away from line
        assert!(circle.velocity.y > 0.0);
    }

    #[test]
    fn test_resolve_circle_vs_line_static_circle_does_not_move() {
        let mut circle = create_static_circle(50.0, 5.0, 10.0);
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            0.8,
        );

        let initial_pos = circle.position;
        let result = circle_vs_line(&circle, &line);
        resolve_circle_vs_line(&mut circle, &line, &result);

        // Static circle should not move
        assert_eq!(circle.position.x, initial_pos.x);
        assert_eq!(circle.position.y, initial_pos.y);
    }

    #[test]
    fn test_collision_result_default() {
        let result = CollisionResult::default();
        assert!(!result.collided);
        assert_eq!(result.penetration, 0.0);
    }
}
