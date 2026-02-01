import { Vec2, vec2 } from '../physics/vector.js';
import { Line, createLine } from '../physics/body.js';

export interface GameMap {
  width: number;
  height: number;
  walls: Line[];
  goalWidth: number;
  goalDepth: number;
  redGoal: { min: Vec2; max: Vec2 };
  blueGoal: { min: Vec2; max: Vec2 };
  redSpawns: Vec2[];
  blueSpawns: Vec2[];
  ballSpawn: Vec2;
}

export function createDefaultMap(): GameMap {
  const width = 800;
  const height = 400;
  const goalWidth = 120;
  const goalDepth = 30;

  // Goal positions (centered vertically)
  const goalTop = (height - goalWidth) / 2;
  const goalBottom = (height + goalWidth) / 2;

  // Wall restitution
  const wallBounce = 0.8;

  const walls: Line[] = [
    // Top wall
    createLine(vec2(0, 0), vec2(width, 0), wallBounce),
    // Bottom wall
    createLine(vec2(0, height), vec2(width, height), wallBounce),

    // Left wall (above goal)
    createLine(vec2(0, 0), vec2(0, goalTop), wallBounce),
    // Left wall (below goal)
    createLine(vec2(0, goalBottom), vec2(0, height), wallBounce),

    // Right wall (above goal)
    createLine(vec2(width, 0), vec2(width, goalTop), wallBounce),
    // Right wall (below goal)
    createLine(vec2(width, goalBottom), vec2(width, height), wallBounce),

    // Red goal (left side) walls
    createLine(vec2(-goalDepth, goalTop), vec2(0, goalTop), wallBounce),
    createLine(vec2(-goalDepth, goalBottom), vec2(0, goalBottom), wallBounce),
    createLine(vec2(-goalDepth, goalTop), vec2(-goalDepth, goalBottom), wallBounce),

    // Blue goal (right side) walls
    createLine(vec2(width, goalTop), vec2(width + goalDepth, goalTop), wallBounce),
    createLine(vec2(width, goalBottom), vec2(width + goalDepth, goalBottom), wallBounce),
    createLine(vec2(width + goalDepth, goalTop), vec2(width + goalDepth, goalBottom), wallBounce),
  ];

  return {
    width,
    height,
    walls,
    goalWidth,
    goalDepth,
    redGoal: {
      min: vec2(-goalDepth, goalTop),
      max: vec2(0, goalBottom),
    },
    blueGoal: {
      min: vec2(width, goalTop),
      max: vec2(width + goalDepth, goalBottom),
    },
    redSpawns: [
      vec2(150, height / 2),
      vec2(200, height / 2 - 80),
      vec2(200, height / 2 + 80),
      vec2(100, height / 2),
    ],
    blueSpawns: [
      vec2(width - 150, height / 2),
      vec2(width - 200, height / 2 - 80),
      vec2(width - 200, height / 2 + 80),
      vec2(width - 100, height / 2),
    ],
    ballSpawn: vec2(width / 2, height / 2),
  };
}

export function isInGoal(position: Vec2, goal: { min: Vec2; max: Vec2 }): boolean {
  return (
    position.x >= goal.min.x &&
    position.x <= goal.max.x &&
    position.y >= goal.min.y &&
    position.y <= goal.max.y
  );
}
