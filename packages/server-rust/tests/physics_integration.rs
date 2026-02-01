use haxball_server::physics::{Circle, Line, PhysicsWorld, Vec2};

fn create_circle(x: f32, y: f32, radius: f32) -> Circle {
    Circle::new(Vec2::new(x, y), radius, 1.0, 0.8, 0.99, false)
}

fn create_static_circle(x: f32, y: f32, radius: f32) -> Circle {
    Circle::new(Vec2::new(x, y), radius, 1.0, 0.8, 0.99, true)
}

const DT: f32 = 1.0 / 60.0;

#[test]
fn test_multiple_circles_collision_chain() {
    let mut world = PhysicsWorld::new();

    // Create a chain of circles
    world.add_circle(create_circle(0.0, 0.0, 10.0));
    world.add_circle(create_circle(25.0, 0.0, 10.0));
    world.add_circle(create_circle(50.0, 0.0, 10.0));

    // Give the first circle some velocity
    world.circles[0].velocity = Vec2::new(200.0, 0.0);

    // Simulate for a while
    for _ in 0..120 {
        world.step(DT);
    }

    // The last circle should have gained some velocity (momentum transfer)
    assert!(world.circles[2].velocity.x > 0.0);
}

#[test]
fn test_circle_bounces_in_box() {
    let mut world = PhysicsWorld::new();

    // Create walls forming a box
    world.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 1.0)); // Top
    world.add_line(Line::new(
        Vec2::new(0.0, 200.0),
        Vec2::new(200.0, 200.0),
        1.0,
    )); // Bottom
    world.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 200.0), 1.0)); // Left
    world.add_line(Line::new(
        Vec2::new(200.0, 0.0),
        Vec2::new(200.0, 200.0),
        1.0,
    )); // Right

    // Create a circle in the center with velocity
    let mut circle = create_circle(100.0, 100.0, 10.0);
    circle.velocity = Vec2::new(150.0, 100.0);
    world.add_circle(circle);

    // Simulate
    for _ in 0..300 {
        world.step(DT);

        // Circle should stay within bounds (with some tolerance for radius)
        let pos = world.circles[0].position;
        assert!(pos.x >= 5.0 && pos.x <= 195.0, "x out of bounds: {}", pos.x);
        assert!(pos.y >= 5.0 && pos.y <= 195.0, "y out of bounds: {}", pos.y);
    }
}

#[test]
fn test_high_speed_circle_does_not_tunnel() {
    let mut world = PhysicsWorld::new();

    // Create a wall
    world.add_line(Line::new(
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 200.0),
        1.0,
    ));

    // Create a circle with very high velocity toward the wall
    let mut circle = create_circle(50.0, 100.0, 10.0);
    circle.velocity = Vec2::new(800.0, 0.0); // High speed
    world.add_circle(circle);

    // Simulate
    for _ in 0..60 {
        world.step(DT);
    }

    // Circle should not have passed through the wall
    // It should either be on the left side or have bounced back
    let pos = world.circles[0].position;
    // Due to velocity clamping and collision resolution, it should not tunnel
    assert!(
        pos.x <= 100.0,
        "Circle tunneled through wall: x = {}",
        pos.x
    );
}

#[test]
fn test_friction_slows_circle() {
    let mut world = PhysicsWorld::new();

    let mut circle = create_circle(100.0, 100.0, 10.0);
    circle.velocity = Vec2::new(100.0, 0.0);
    circle.friction = 0.95; // Strong friction
    world.add_circle(circle);

    let initial_speed = world.circles[0].velocity.length();

    // Simulate for a while
    for _ in 0..120 {
        world.step(DT);
    }

    let final_speed = world.circles[0].velocity.length();

    // Circle should have slowed down significantly
    assert!(
        final_speed < initial_speed * 0.5,
        "Friction did not slow circle enough: {} -> {}",
        initial_speed,
        final_speed
    );
}

#[test]
fn test_static_circle_blocks_dynamic() {
    let mut world = PhysicsWorld::new();

    // Static circle in the center
    world.add_circle(create_static_circle(100.0, 100.0, 20.0));

    // Dynamic circle moving toward static
    let mut dynamic = create_circle(50.0, 100.0, 10.0);
    dynamic.velocity = Vec2::new(200.0, 0.0);
    world.add_circle(dynamic);

    // Simulate
    for _ in 0..60 {
        world.step(DT);
    }

    // Static circle should not have moved
    let static_pos = world.circles[0].position;
    assert_eq!(static_pos.x, 100.0);
    assert_eq!(static_pos.y, 100.0);

    // Dynamic circle should have bounced
    let dynamic_vel = world.circles[1].velocity;
    // After collision, x velocity should be negative or significantly reduced
    assert!(dynamic_vel.x < 100.0);
}

#[test]
fn test_two_static_circles_no_movement() {
    let mut world = PhysicsWorld::new();

    // Two static circles overlapping
    world.add_circle(create_static_circle(100.0, 100.0, 20.0));
    world.add_circle(create_static_circle(110.0, 100.0, 20.0));

    let pos1_before = world.circles[0].position;
    let pos2_before = world.circles[1].position;

    // Simulate
    for _ in 0..60 {
        world.step(DT);
    }

    // Neither should have moved
    assert_eq!(world.circles[0].position.x, pos1_before.x);
    assert_eq!(world.circles[1].position.x, pos2_before.x);
}

#[test]
fn test_circle_with_zero_velocity_stays_put() {
    let mut world = PhysicsWorld::new();

    let circle = create_circle(100.0, 100.0, 10.0);
    // Velocity is already 0.0 by default
    world.add_circle(circle);

    let pos_before = world.circles[0].position;

    // Simulate
    for _ in 0..60 {
        world.step(DT);
    }

    let pos_after = world.circles[0].position;

    // Should not have moved (within floating point tolerance)
    assert!((pos_after.x - pos_before.x).abs() < 0.001);
    assert!((pos_after.y - pos_before.y).abs() < 0.001);
}

#[test]
fn test_many_circles_stress_test() {
    let mut world = PhysicsWorld::new();

    // Create a box
    world.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(500.0, 0.0), 0.9));
    world.add_line(Line::new(
        Vec2::new(0.0, 500.0),
        Vec2::new(500.0, 500.0),
        0.9,
    ));
    world.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 500.0), 0.9));
    world.add_line(Line::new(
        Vec2::new(500.0, 0.0),
        Vec2::new(500.0, 500.0),
        0.9,
    ));

    // Add many circles with random-ish velocities
    for i in 0..20 {
        let x = 50.0 + (i % 5) as f32 * 80.0;
        let y = 50.0 + (i / 5) as f32 * 80.0;
        let mut circle = create_circle(x, y, 8.0);
        circle.velocity = Vec2::new(
            ((i * 17) % 200) as f32 - 100.0,
            ((i * 31) % 200) as f32 - 100.0,
        );
        world.add_circle(circle);
    }

    // Simulate for many ticks without crashing
    for _ in 0..600 {
        world.step(DT);

        // All circles should have valid positions
        for circle in &world.circles {
            assert!(!circle.position.x.is_nan(), "NaN position detected");
            assert!(!circle.position.y.is_nan(), "NaN position detected");
        }
    }
}

#[test]
fn test_restitution_affects_bounce() {
    // Test with high restitution
    let mut world_bouncy = PhysicsWorld::new();
    world_bouncy.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 1.0));
    let mut circle_bouncy = create_circle(100.0, 50.0, 10.0);
    circle_bouncy.velocity = Vec2::new(0.0, -100.0);
    circle_bouncy.restitution = 1.0;
    world_bouncy.add_circle(circle_bouncy);

    // Test with low restitution
    let mut world_damp = PhysicsWorld::new();
    world_damp.add_line(Line::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), 0.3));
    let mut circle_damp = create_circle(100.0, 50.0, 10.0);
    circle_damp.velocity = Vec2::new(0.0, -100.0);
    circle_damp.restitution = 0.3;
    world_damp.add_circle(circle_damp);

    // Simulate both
    for _ in 0..30 {
        world_bouncy.step(DT);
        world_damp.step(DT);
    }

    // Bouncy circle should have higher velocity magnitude
    let speed_bouncy = world_bouncy.circles[0].velocity.length();
    let speed_damp = world_damp.circles[0].velocity.length();

    assert!(
        speed_bouncy > speed_damp,
        "Bouncy: {}, Damp: {}",
        speed_bouncy,
        speed_damp
    );
}

#[test]
fn test_diagonal_collision() {
    let mut world = PhysicsWorld::new();

    // Create two circles that will collide diagonally
    let mut circle1 = create_circle(0.0, 0.0, 10.0);
    circle1.velocity = Vec2::new(100.0, 100.0);
    world.add_circle(circle1);

    let circle2 = create_circle(50.0, 50.0, 10.0);
    world.add_circle(circle2);

    // Simulate until they collide
    for _ in 0..60 {
        world.step(DT);
    }

    // Second circle should have gained velocity
    let vel2 = world.circles[1].velocity;
    assert!(vel2.length() > 0.0);
}
