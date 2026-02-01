import { Circle, Line } from './body.js';
import { Vec2, sub, add, mul, dot, length, normalize, distance } from './vector.js';

const EPSILON = 0.0001;

export interface CollisionResult {
  collided: boolean;
  normal: Vec2;
  penetration: number;
}

export function circleVsCircle(a: Circle, b: Circle): CollisionResult {
  const diff = sub(b.position, a.position);
  const dist = length(diff);
  const minDist = a.radius + b.radius;

  if (dist >= minDist || dist < EPSILON) {
    return { collided: false, normal: { x: 0, y: 0 }, penetration: 0 };
  }

  const normal = normalize(diff);
  const penetration = minDist - dist;

  return { collided: true, normal, penetration };
}

export function resolveCircleVsCircle(a: Circle, b: Circle, result: CollisionResult): void {
  if (!result.collided) return;

  const { normal, penetration } = result;

  // Separate circles
  const totalInvMass = a.invMass + b.invMass;
  if (totalInvMass === 0) return;

  const correction = mul(normal, penetration / totalInvMass);
  a.position = sub(a.position, mul(correction, a.invMass));
  b.position = add(b.position, mul(correction, b.invMass));

  // Calculate relative velocity
  const relativeVelocity = sub(b.velocity, a.velocity);
  const velocityAlongNormal = dot(relativeVelocity, normal);

  // Don't resolve if velocities are separating
  if (velocityAlongNormal > 0) return;

  // Restitution
  const e = Math.min(a.restitution, b.restitution);

  // Impulse magnitude
  const j = -(1 + e) * velocityAlongNormal / totalInvMass;

  // Apply impulse
  const impulse = mul(normal, j);
  a.velocity = sub(a.velocity, mul(impulse, a.invMass));
  b.velocity = add(b.velocity, mul(impulse, b.invMass));
}

export function circleVsLine(circle: Circle, line: Line): CollisionResult {
  const { p1, p2 } = line;
  const lineVec = sub(p2, p1);
  const lineLength = length(lineVec);
  const lineDir = normalize(lineVec);

  // Project circle center onto line
  const toCircle = sub(circle.position, p1);
  const projection = dot(toCircle, lineDir);

  let closestPoint: Vec2;

  if (projection <= 0) {
    closestPoint = p1;
  } else if (projection >= lineLength) {
    closestPoint = p2;
  } else {
    closestPoint = add(p1, mul(lineDir, projection));
  }

  const dist = distance(circle.position, closestPoint);

  if (dist >= circle.radius || dist < EPSILON) {
    return { collided: false, normal: { x: 0, y: 0 }, penetration: 0 };
  }

  const normal = normalize(sub(circle.position, closestPoint));
  const penetration = circle.radius - dist;

  return { collided: true, normal, penetration };
}

const SEPARATION_BUFFER = 0.5;

export function resolveCircleVsLine(circle: Circle, line: Line, result: CollisionResult): void {
  if (!result.collided || circle.isStatic) return;

  const { normal, penetration } = result;

  // Separate circle from line with a small buffer to prevent sticking
  circle.position = add(circle.position, mul(normal, penetration + SEPARATION_BUFFER));

  // Reflect velocity
  const velocityAlongNormal = dot(circle.velocity, normal);

  // Only resolve if moving towards the line
  if (velocityAlongNormal >= 0) return;

  const e = Math.min(circle.restitution, line.restitution);
  const impulse = -(1 + e) * velocityAlongNormal;

  circle.velocity = add(circle.velocity, mul(normal, impulse));
}
