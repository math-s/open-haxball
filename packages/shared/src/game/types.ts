import { Circle } from '../physics/body.js';
import { Vec2 } from '../physics/vector.js';

export type Team = 'red' | 'blue';
export type GameStatus = 'waiting' | 'playing' | 'goal' | 'finished';

export interface InputState {
  left: boolean;
  right: boolean;
  up: boolean;
  down: boolean;
  kick: boolean;
}

export interface Player {
  id: string;
  name: string;
  body: Circle;
  team: Team;
  input: InputState;
}

export interface Score {
  red: number;
  blue: number;
}

export interface GameState {
  players: Map<string, Player>;
  ball: Circle;
  score: Score;
  status: GameStatus;
  lastGoalTeam: Team | null;
  goalTimer: number;
}

export interface SerializedPlayer {
  id: string;
  name: string;
  position: Vec2;
  velocity: Vec2;
  radius: number;
  team: Team;
}

export interface SerializedGameState {
  players: SerializedPlayer[];
  ball: {
    position: Vec2;
    velocity: Vec2;
    radius: number;
  };
  score: Score;
  status: GameStatus;
  lastGoalTeam: Team | null;
  matchTimeRemaining?: number;
  isHost: boolean;
  intermissionTimeRemaining?: number;
}

export function serializeGameState(state: GameState): SerializedGameState {
  const players: SerializedPlayer[] = [];
  for (const [, player] of state.players) {
    players.push({
      id: player.id,
      name: player.name,
      position: { x: player.body.position.x, y: player.body.position.y },
      velocity: { x: player.body.velocity.x, y: player.body.velocity.y },
      radius: player.body.radius,
      team: player.team,
    });
  }

  return {
    players,
    ball: {
      position: { x: state.ball.position.x, y: state.ball.position.y },
      velocity: { x: state.ball.velocity.x, y: state.ball.velocity.y },
      radius: state.ball.radius,
    },
    score: { ...state.score },
    status: state.status,
    lastGoalTeam: state.lastGoalTeam,
    isHost: false,
  };
}

export function createEmptyInput(): InputState {
  return {
    left: false,
    right: false,
    up: false,
    down: false,
    kick: false,
  };
}
