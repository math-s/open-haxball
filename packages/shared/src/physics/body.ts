import { Vec2, vec2 } from './vector.js';

export interface Circle {
  position: Vec2;
  velocity: Vec2;
  radius: number;
  mass: number;
  invMass: number;
  restitution: number;
  friction: number;
  isStatic: boolean;
}

export interface Line {
  p1: Vec2;
  p2: Vec2;
  restitution: number;
}

export function createCircle(options: {
  position: Vec2;
  radius: number;
  mass?: number;
  restitution?: number;
  friction?: number;
  isStatic?: boolean;
  velocity?: Vec2;
}): Circle {
  const mass = options.mass ?? 1;
  const isStatic = options.isStatic ?? false;
  return {
    position: { ...options.position },
    velocity: options.velocity ? { ...options.velocity } : vec2(0, 0),
    radius: options.radius,
    mass: mass,
    invMass: isStatic ? 0 : 1 / mass,
    restitution: options.restitution ?? 0.5,
    friction: options.friction ?? 0.99,
    isStatic: isStatic,
  };
}

export function createLine(p1: Vec2, p2: Vec2, restitution: number = 0.8): Line {
  return {
    p1: { ...p1 },
    p2: { ...p2 },
    restitution,
  };
}
