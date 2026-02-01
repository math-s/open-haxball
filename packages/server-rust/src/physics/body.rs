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
