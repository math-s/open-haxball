export interface Vec2 {
  x: number;
  y: number;
}

export function vec2(x: number, y: number): Vec2 {
  return { x, y };
}

export function add(a: Vec2, b: Vec2): Vec2 {
  return { x: a.x + b.x, y: a.y + b.y };
}

export function sub(a: Vec2, b: Vec2): Vec2 {
  return { x: a.x - b.x, y: a.y - b.y };
}

export function mul(v: Vec2, scalar: number): Vec2 {
  return { x: v.x * scalar, y: v.y * scalar };
}

export function div(v: Vec2, scalar: number): Vec2 {
  return { x: v.x / scalar, y: v.y / scalar };
}

export function dot(a: Vec2, b: Vec2): number {
  return a.x * b.x + a.y * b.y;
}

export function lengthSquared(v: Vec2): number {
  return v.x * v.x + v.y * v.y;
}

export function length(v: Vec2): number {
  return Math.sqrt(lengthSquared(v));
}

export function normalize(v: Vec2): Vec2 {
  const len = length(v);
  if (len === 0) return { x: 0, y: 0 };
  return div(v, len);
}

export function reflect(v: Vec2, normal: Vec2): Vec2 {
  const d = dot(v, normal);
  return sub(v, mul(normal, 2 * d));
}

export function distance(a: Vec2, b: Vec2): number {
  return length(sub(b, a));
}

export function distanceSquared(a: Vec2, b: Vec2): number {
  return lengthSquared(sub(b, a));
}

export function clone(v: Vec2): Vec2 {
  return { x: v.x, y: v.y };
}
