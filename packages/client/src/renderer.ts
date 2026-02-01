import { SerializedGameState, Team, GameStatus } from '@open-haxball/shared';

const FIELD_COLOR = '#3a8c3a';
const LINE_COLOR = '#ffffff';
const RED_TEAM_COLOR = '#e74c3c';
const BLUE_TEAM_COLOR = '#3498db';
const BALL_COLOR = '#ffffff';
const GOAL_COLOR = '#888888';

export class Renderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private width = 800;
  private height = 400;
  private goalWidth = 120;
  private goalDepth = 30;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Could not get canvas context');
    this.ctx = ctx;

    // Set canvas size with goal areas
    this.canvas.width = this.width + this.goalDepth * 2;
    this.canvas.height = this.height;
  }

  render(state: SerializedGameState | null, localPlayerId: string | null): void {
    this.ctx.save();

    // Offset for goal areas
    this.ctx.translate(this.goalDepth, 0);

    this.drawField();
    this.drawGoals();

    if (state) {
      this.drawBall(state.ball);
      this.drawPlayers(state.players, localPlayerId);
      this.drawScore(state.score);
      this.drawStatus(state.status, state.lastGoalTeam);
    } else {
      this.drawConnecting();
    }

    this.ctx.restore();
  }

  private drawField(): void {
    const { ctx, width, height } = this;

    // Field background
    ctx.fillStyle = FIELD_COLOR;
    ctx.fillRect(0, 0, width, height);

    // Field border
    ctx.strokeStyle = LINE_COLOR;
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, width, height);

    // Center line
    ctx.beginPath();
    ctx.moveTo(width / 2, 0);
    ctx.lineTo(width / 2, height);
    ctx.stroke();

    // Center circle
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 50, 0, Math.PI * 2);
    ctx.stroke();

    // Center dot
    ctx.fillStyle = LINE_COLOR;
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  private drawGoals(): void {
    const { ctx, height, goalWidth, goalDepth } = this;
    const goalTop = (height - goalWidth) / 2;

    // Red goal (left)
    ctx.fillStyle = GOAL_COLOR;
    ctx.globalAlpha = 0.3;
    ctx.fillRect(-goalDepth, goalTop, goalDepth, goalWidth);
    ctx.globalAlpha = 1;
    ctx.strokeStyle = RED_TEAM_COLOR;
    ctx.lineWidth = 3;
    ctx.strokeRect(-goalDepth, goalTop, goalDepth, goalWidth);

    // Blue goal (right)
    ctx.fillStyle = GOAL_COLOR;
    ctx.globalAlpha = 0.3;
    ctx.fillRect(this.width, goalTop, goalDepth, goalWidth);
    ctx.globalAlpha = 1;
    ctx.strokeStyle = BLUE_TEAM_COLOR;
    ctx.lineWidth = 3;
    ctx.strokeRect(this.width, goalTop, goalDepth, goalWidth);
  }

  private drawBall(ball: { position: { x: number; y: number }; radius: number }): void {
    const { ctx } = this;

    ctx.fillStyle = BALL_COLOR;
    ctx.strokeStyle = '#000000';
    ctx.lineWidth = 2;

    ctx.beginPath();
    ctx.arc(ball.position.x, ball.position.y, ball.radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  }

  private drawPlayers(
    players: Array<{
      id: string;
      name: string;
      position: { x: number; y: number };
      radius: number;
      team: Team;
    }>,
    localPlayerId: string | null
  ): void {
    const { ctx } = this;

    for (const player of players) {
      const isLocal = player.id === localPlayerId;
      const color = player.team === 'red' ? RED_TEAM_COLOR : BLUE_TEAM_COLOR;

      // Player circle
      ctx.fillStyle = color;
      ctx.strokeStyle = isLocal ? '#ffffff' : '#000000';
      ctx.lineWidth = isLocal ? 3 : 2;

      ctx.beginPath();
      ctx.arc(player.position.x, player.position.y, player.radius, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // Player name
      ctx.fillStyle = '#ffffff';
      ctx.font = '12px Arial';
      ctx.textAlign = 'center';
      ctx.fillText(player.name, player.position.x, player.position.y - player.radius - 5);
    }
  }

  private drawScore(score: { red: number; blue: number }): void {
    const { ctx, width } = this;

    ctx.font = 'bold 36px Arial';
    ctx.textAlign = 'center';

    // Red score
    ctx.fillStyle = RED_TEAM_COLOR;
    ctx.fillText(score.red.toString(), width / 2 - 40, 40);

    // Separator
    ctx.fillStyle = '#ffffff';
    ctx.fillText('-', width / 2, 40);

    // Blue score
    ctx.fillStyle = BLUE_TEAM_COLOR;
    ctx.fillText(score.blue.toString(), width / 2 + 40, 40);
  }

  private drawStatus(status: GameStatus, lastGoalTeam: Team | null): void {
    const { ctx, width, height } = this;

    if (status === 'goal' && lastGoalTeam) {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
      ctx.fillRect(0, 0, width, height);

      const color = lastGoalTeam === 'red' ? RED_TEAM_COLOR : BLUE_TEAM_COLOR;
      ctx.fillStyle = color;
      ctx.font = 'bold 48px Arial';
      ctx.textAlign = 'center';
      ctx.fillText('GOAL!', width / 2, height / 2);

      ctx.font = '24px Arial';
      ctx.fillText(`${lastGoalTeam.toUpperCase()} TEAM SCORES!`, width / 2, height / 2 + 40);
    } else if (status === 'waiting') {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
      ctx.fillRect(0, 0, width, height);

      ctx.fillStyle = '#ffffff';
      ctx.font = '24px Arial';
      ctx.textAlign = 'center';
      ctx.fillText('Waiting for players...', width / 2, height / 2);
    }
  }

  private drawConnecting(): void {
    const { ctx, width, height } = this;

    ctx.fillStyle = '#333333';
    ctx.fillRect(0, 0, width, height);

    ctx.fillStyle = '#ffffff';
    ctx.font = '24px Arial';
    ctx.textAlign = 'center';
    ctx.fillText('Connecting...', width / 2, height / 2);
  }
}
