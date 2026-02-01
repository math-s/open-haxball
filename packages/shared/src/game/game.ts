import { PhysicsWorld, PHYSICS_DT } from '../physics/world.js';
import { Circle, createCircle } from '../physics/body.js';
import { Vec2, vec2, sub, length, normalize, mul, add } from '../physics/vector.js';
import { GameState, Player, Team, InputState, createEmptyInput } from './types.js';
import { GameMap, createDefaultMap, isInGoal } from './map.js';

// Game constants
const PLAYER_RADIUS = 15;
const PLAYER_MASS = 1;
const PLAYER_SPEED = 300;
const PLAYER_FRICTION = 0.96;

const BALL_RADIUS = 10;
const BALL_MASS = 0.5;
const BALL_FRICTION = 0.99;

const KICK_DISTANCE = 30;
const KICK_FORCE = 500;

const GOAL_RESET_TIME = 2; // seconds

export class Game {
  world: PhysicsWorld;
  state: GameState;
  map: GameMap;

  constructor() {
    this.world = new PhysicsWorld();
    this.map = createDefaultMap();

    // Add map walls to physics world
    for (const wall of this.map.walls) {
      this.world.addLine(wall);
    }

    // Create ball
    const ball = createCircle({
      position: this.map.ballSpawn,
      radius: BALL_RADIUS,
      mass: BALL_MASS,
      friction: BALL_FRICTION,
      restitution: 0.8,
    });
    this.world.addCircle(ball);

    this.state = {
      players: new Map(),
      ball,
      score: { red: 0, blue: 0 },
      status: 'waiting',
      lastGoalTeam: null,
      goalTimer: 0,
    };
  }

  addPlayer(id: string, name: string, team: Team): Player {
    const spawns = team === 'red' ? this.map.redSpawns : this.map.blueSpawns;
    const spawnIndex = this.state.players.size % spawns.length;
    const spawnPos = spawns[spawnIndex];

    const body = createCircle({
      position: vec2(spawnPos.x, spawnPos.y),
      radius: PLAYER_RADIUS,
      mass: PLAYER_MASS,
      friction: PLAYER_FRICTION,
      restitution: 0.5,
    });

    const player: Player = {
      id,
      name,
      body,
      team,
      input: createEmptyInput(),
    };

    this.state.players.set(id, player);
    this.world.addCircle(body);

    // Start game if we have at least 1 player
    if (this.state.players.size >= 1 && this.state.status === 'waiting') {
      this.state.status = 'playing';
    }

    return player;
  }

  removePlayer(id: string): void {
    const player = this.state.players.get(id);
    if (player) {
      this.world.removeCircle(player.body);
      this.state.players.delete(id);
    }

    // Go back to waiting if no players
    if (this.state.players.size === 0) {
      this.state.status = 'waiting';
    }
  }

  setPlayerInput(id: string, input: InputState): void {
    const player = this.state.players.get(id);
    if (player) {
      player.input = { ...input };
    }
  }

  update(dt: number): void {
    if (this.state.status === 'goal') {
      this.state.goalTimer -= dt;
      if (this.state.goalTimer <= 0) {
        this.resetPositions();
        this.state.status = 'playing';
      }
      return;
    }

    if (this.state.status !== 'playing') {
      return;
    }

    // Process player inputs
    for (const [, player] of this.state.players) {
      this.processPlayerInput(player, dt);
    }

    // Step physics
    this.world.step(dt);

    // Check for goals
    this.checkGoals();
  }

  private processPlayerInput(player: Player, dt: number): void {
    const { input, body } = player;

    // Movement
    let moveX = 0;
    let moveY = 0;

    if (input.left) moveX -= 1;
    if (input.right) moveX += 1;
    if (input.up) moveY -= 1;
    if (input.down) moveY += 1;

    // Normalize diagonal movement
    if (moveX !== 0 || moveY !== 0) {
      const moveLen = Math.sqrt(moveX * moveX + moveY * moveY);
      moveX /= moveLen;
      moveY /= moveLen;

      body.velocity.x += moveX * PLAYER_SPEED * dt;
      body.velocity.y += moveY * PLAYER_SPEED * dt;
    }

    // Kick
    if (input.kick) {
      this.tryKick(player);
    }
  }

  private tryKick(player: Player): void {
    const { body } = player;
    const ball = this.state.ball;

    const toBall = sub(ball.position, body.position);
    const dist = length(toBall);
    const kickRange = body.radius + ball.radius + KICK_DISTANCE;

    if (dist <= kickRange && dist > 0) {
      const kickDir = normalize(toBall);
      const kickImpulse = mul(kickDir, KICK_FORCE);
      ball.velocity = add(ball.velocity, kickImpulse);
    }
  }

  private checkGoals(): void {
    const ballPos = this.state.ball.position;

    // Check red goal (blue scores)
    if (isInGoal(ballPos, this.map.redGoal)) {
      this.state.score.blue++;
      this.state.status = 'goal';
      this.state.lastGoalTeam = 'blue';
      this.state.goalTimer = GOAL_RESET_TIME;
      return;
    }

    // Check blue goal (red scores)
    if (isInGoal(ballPos, this.map.blueGoal)) {
      this.state.score.red++;
      this.state.status = 'goal';
      this.state.lastGoalTeam = 'red';
      this.state.goalTimer = GOAL_RESET_TIME;
      return;
    }
  }

  private resetPositions(): void {
    // Reset ball
    this.state.ball.position = vec2(this.map.ballSpawn.x, this.map.ballSpawn.y);
    this.state.ball.velocity = vec2(0, 0);

    // Reset players
    let redIndex = 0;
    let blueIndex = 0;

    for (const [, player] of this.state.players) {
      const spawns = player.team === 'red' ? this.map.redSpawns : this.map.blueSpawns;
      const index = player.team === 'red' ? redIndex++ : blueIndex++;
      const spawn = spawns[index % spawns.length];

      player.body.position = vec2(spawn.x, spawn.y);
      player.body.velocity = vec2(0, 0);
    }
  }

  getAutoTeam(): Team {
    let redCount = 0;
    let blueCount = 0;

    for (const [, player] of this.state.players) {
      if (player.team === 'red') redCount++;
      else blueCount++;
    }

    return redCount <= blueCount ? 'red' : 'blue';
  }
}
