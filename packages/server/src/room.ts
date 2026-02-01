import { WebSocket } from 'ws';
import { Game, PHYSICS_DT, serializeGameState, InputState } from '@open-haxball/shared';
import { ServerMessage, serializeServerMessage } from './protocol.js';

interface Client {
  ws: WebSocket;
  playerId: string | null;
}

export class Room {
  private game: Game;
  private clients: Map<WebSocket, Client> = new Map();
  private gameLoopInterval: NodeJS.Timeout | null = null;
  private nextPlayerId = 1;

  constructor() {
    this.game = new Game();
  }

  start(): void {
    if (this.gameLoopInterval) return;

    this.gameLoopInterval = setInterval(() => {
      this.tick();
    }, PHYSICS_DT * 1000);

    console.log('Game loop started');
  }

  stop(): void {
    if (this.gameLoopInterval) {
      clearInterval(this.gameLoopInterval);
      this.gameLoopInterval = null;
    }
    console.log('Game loop stopped');
  }

  addClient(ws: WebSocket): void {
    this.clients.set(ws, { ws, playerId: null });
    console.log(`Client connected. Total clients: ${this.clients.size}`);
  }

  removeClient(ws: WebSocket): void {
    const client = this.clients.get(ws);
    if (client?.playerId) {
      this.game.removePlayer(client.playerId);
      this.broadcast({
        type: 'playerLeft',
        data: { playerId: client.playerId },
      });
      console.log(`Player ${client.playerId} left`);
    }
    this.clients.delete(ws);
    console.log(`Client disconnected. Total clients: ${this.clients.size}`);
  }

  handleJoin(ws: WebSocket, name: string): void {
    const client = this.clients.get(ws);
    if (!client) return;

    if (client.playerId) {
      this.send(ws, { type: 'error', data: { message: 'Already joined' } });
      return;
    }

    const playerId = `player_${this.nextPlayerId++}`;
    const team = this.game.getAutoTeam();
    const player = this.game.addPlayer(playerId, name, team);

    client.playerId = playerId;

    // Send join confirmation to the player
    this.send(ws, {
      type: 'joined',
      data: { playerId, team },
    });

    // Broadcast to all other clients
    this.broadcast(
      {
        type: 'playerJoined',
        data: { playerId, name, team },
      },
      ws
    );

    console.log(`Player ${name} (${playerId}) joined team ${team}`);
  }

  handleInput(ws: WebSocket, input: InputState): void {
    const client = this.clients.get(ws);
    if (!client?.playerId) return;

    this.game.setPlayerInput(client.playerId, input);
  }

  private tick(): void {
    this.game.update(PHYSICS_DT);

    // Broadcast state to all clients
    const state = serializeGameState(this.game.state);
    this.broadcast({ type: 'state', data: state });
  }

  private send(ws: WebSocket, message: ServerMessage): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(serializeServerMessage(message));
    }
  }

  private broadcast(message: ServerMessage, exclude?: WebSocket): void {
    const data = serializeServerMessage(message);
    for (const [ws] of this.clients) {
      if (ws !== exclude && ws.readyState === WebSocket.OPEN) {
        ws.send(data);
      }
    }
  }
}
