use super::Vec2;

#[derive(Clone)]
pub struct Circle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    #[allow(dead_code)]
    pub mass: f32,
    pub inv_mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub is_static: bool,
}

impl Circle {
    pub fn new(
        position: Vec2,
        radius: f32,
        mass: f32,
        restitution: f32,
        friction: f32,
        is_static: bool,
    ) -> Self {
        let inv_mass = if is_static { 0.0 } else { 1.0 / mass };
        Self {
            position,
            velocity: Vec2::default(),
            radius,
            mass,
            inv_mass,
            restitution,
            friction,
            is_static,
        }
    }
}

#[derive(Clone)]
pub struct Line {
    pub p1: Vec2,
    pub p2: Vec2,
    pub restitution: f32,
}

impl Line {
    pub fn new(p1: Vec2, p2: Vec2, restitution: f32) -> Self {
        Self { p1, p2, restitution }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_new() {
        let circle = Circle::new(
            Vec2::new(10.0, 20.0),
            15.0,
            2.0,
            0.8,
            0.99,
            false,
        );

        assert_eq!(circle.position.x, 10.0);
        assert_eq!(circle.position.y, 20.0);
        assert_eq!(circle.radius, 15.0);
        assert_eq!(circle.mass, 2.0);
        assert_eq!(circle.inv_mass, 0.5); // 1/2
        assert_eq!(circle.restitution, 0.8);
        assert_eq!(circle.friction, 0.99);
        assert!(!circle.is_static);
        assert_eq!(circle.velocity.x, 0.0);
        assert_eq!(circle.velocity.y, 0.0);
    }

    #[test]
    fn test_circle_static_has_zero_inv_mass() {
        let circle = Circle::new(
            Vec2::new(0.0, 0.0),
            10.0,
            5.0,
            0.8,
            0.99,
            true, // static
        );

        assert!(circle.is_static);
        assert_eq!(circle.inv_mass, 0.0);
    }

    #[test]
    fn test_circle_clone() {
        let circle = Circle::new(
            Vec2::new(10.0, 20.0),
            15.0,
            2.0,
            0.8,
            0.99,
            false,
        );
        let cloned = circle.clone();

        assert_eq!(circle.position.x, cloned.position.x);
        assert_eq!(circle.position.y, cloned.position.y);
        assert_eq!(circle.radius, cloned.radius);
    }

    #[test]
    fn test_line_new() {
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 50.0),
            0.9,
        );

        assert_eq!(line.p1.x, 0.0);
        assert_eq!(line.p1.y, 0.0);
        assert_eq!(line.p2.x, 100.0);
        assert_eq!(line.p2.y, 50.0);
        assert_eq!(line.restitution, 0.9);
    }

    #[test]
    fn test_line_clone() {
        let line = Line::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 50.0),
            0.9,
        );
        let cloned = line.clone();

        assert_eq!(line.p1.x, cloned.p1.x);
        assert_eq!(line.p2.x, cloned.p2.x);
        assert_eq!(line.restitution, cloned.restitution);
    }
}
