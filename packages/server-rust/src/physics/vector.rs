use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::default()
        } else {
            self / len
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_vec2_default() {
        let v = Vec2::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn test_vec2_add() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        let result = a + b;
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    #[test]
    fn test_vec2_sub() {
        let a = Vec2::new(5.0, 7.0);
        let b = Vec2::new(2.0, 3.0);
        let result = a - b;
        assert_eq!(result.x, 3.0);
        assert_eq!(result.y, 4.0);
    }

    #[test]
    fn test_vec2_mul() {
        let v = Vec2::new(2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    #[test]
    fn test_vec2_div() {
        let v = Vec2::new(6.0, 8.0);
        let result = v / 2.0;
        assert_eq!(result.x, 3.0);
        assert_eq!(result.y, 4.0);
    }

    #[test]
    fn test_vec2_dot_product() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        let result = a.dot(b);
        assert_eq!(result, 11.0); // 1*3 + 2*4 = 11
    }

    #[test]
    fn test_vec2_dot_perpendicular_is_zero() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        let result = a.dot(b);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_vec2_length_squared() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length_squared(), 25.0); // 3^2 + 4^2 = 25
    }

    #[test]
    fn test_vec2_length() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0); // sqrt(25) = 5
    }

    #[test]
    fn test_vec2_length_zero_vector() {
        let v = Vec2::default();
        assert_eq!(v.length(), 0.0);
    }

    #[test]
    fn test_vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let normalized = v.normalize();
        // Length should be 1
        let len = normalized.length();
        assert!((len - 1.0).abs() < 0.0001);
        // Direction should be preserved
        assert!((normalized.x - 0.6).abs() < 0.0001);
        assert!((normalized.y - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_vec2_normalize_zero_vector_returns_zero() {
        let v = Vec2::default();
        let normalized = v.normalize();
        assert_eq!(normalized.x, 0.0);
        assert_eq!(normalized.y, 0.0);
    }

    #[test]
    fn test_vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.distance(b), 5.0);
    }

    #[test]
    fn test_vec2_distance_same_point_is_zero() {
        let a = Vec2::new(5.0, 7.0);
        let b = Vec2::new(5.0, 7.0);
        assert_eq!(a.distance(b), 0.0);
    }

    #[test]
    fn test_vec2_distance_is_symmetric() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(4.0, 6.0);
        assert_eq!(a.distance(b), b.distance(a));
    }
}
