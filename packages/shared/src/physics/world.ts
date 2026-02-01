import { Circle, Line } from './body.js';
import { mul, length } from './vector.js';
import {
  circleVsCircle,
  resolveCircleVsCircle,
  circleVsLine,
  resolveCircleVsLine,
} from './collision.js';

export const PHYSICS_TICK_RATE = 60;
export const PHYSICS_DT = 1 / PHYSICS_TICK_RATE;

const COLLISION_ITERATIONS = 3;
const MAX_VELOCITY = 1000;

export class PhysicsWorld {
  circles: Circle[] = [];
  lines: Line[] = [];

  addCircle(circle: Circle): void {
    this.circles.push(circle);
  }

  removeCircle(circle: Circle): void {
    const index = this.circles.indexOf(circle);
    if (index !== -1) {
      this.circles.splice(index, 1);
    }
  }

  addLine(line: Line): void {
    this.lines.push(line);
  }

  step(dt: number): void {
    // Apply friction and clamp velocities
    for (const circle of this.circles) {
      if (circle.isStatic) continue;

      // Apply friction (velocity damping)
      circle.velocity = mul(circle.velocity, circle.friction);

      // Clamp max velocity to prevent tunneling
      const speed = length(circle.velocity);
      if (speed > MAX_VELOCITY) {
        circle.velocity = mul(circle.velocity, MAX_VELOCITY / speed);
      }
    }

    // Use substeps for fast-moving objects
    const substeps = 2;
    const subDt = dt / substeps;

    for (let s = 0; s < substeps; s++) {
      // Update positions
      for (const circle of this.circles) {
        if (circle.isStatic) continue;
        circle.position.x += circle.velocity.x * subDt;
        circle.position.y += circle.velocity.y * subDt;

        // Safeguard against NaN positions
        if (isNaN(circle.position.x) || isNaN(circle.position.y)) {
          circle.position.x = 400;
          circle.position.y = 200;
          circle.velocity.x = 0;
          circle.velocity.y = 0;
        }
      }

      // Multiple collision iterations per substep
      for (let iter = 0; iter < COLLISION_ITERATIONS; iter++) {
        // Resolve circle vs line collisions first (walls are more important)
        for (const circle of this.circles) {
          for (const line of this.lines) {
            const result = circleVsLine(circle, line);
            if (result.collided) {
              resolveCircleVsLine(circle, line, result);
            }
          }
        }

        // Resolve circle vs circle collisions
        for (let i = 0; i < this.circles.length; i++) {
          for (let j = i + 1; j < this.circles.length; j++) {
            const a = this.circles[i];
            const b = this.circles[j];
            const result = circleVsCircle(a, b);
            if (result.collided) {
              resolveCircleVsCircle(a, b, result);
            }
          }
        }
      }
    }
  }
}
