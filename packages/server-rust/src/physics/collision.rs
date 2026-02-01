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
