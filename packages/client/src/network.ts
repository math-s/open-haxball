import { InputState, SerializedGameState, Team } from '@open-haxball/shared';

type ServerMessage =
  | { type: 'joined'; data: { playerId: string; team: Team } }
  | { type: 'state'; data: SerializedGameState }
  | { type: 'playerJoined'; data: { playerId: string; name: string; team: Team } }
  | { type: 'playerLeft'; data: { playerId: string } }
  | { type: 'error'; data: { message: string } };

type ClientMessage =
  | { type: 'join'; data: { name: string } }
  | { type: 'input'; data: InputState };

export interface NetworkCallbacks {
  onJoined: (playerId: string, team: Team) => void;
  onState: (state: SerializedGameState) => void;
  onPlayerJoined: (playerId: string, name: string, team: Team) => void;
  onPlayerLeft: (playerId: string) => void;
  onError: (message: string) => void;
  onDisconnect: () => void;
}

export class NetworkClient {
  private ws: WebSocket | null = null;
  private callbacks: NetworkCallbacks | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private serverUrl: string;

  constructor(serverUrl: string) {
    this.serverUrl = serverUrl;
  }

  connect(callbacks: NetworkCallbacks): void {
    this.callbacks = callbacks;
    this.reconnectAttempts = 0;
    this.createConnection();
  }

  private createConnection(): void {
    try {
      this.ws = new WebSocket(this.serverUrl);

      this.ws.onopen = () => {
        console.log('Connected to server');
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          const message: ServerMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (e) {
          console.error('Failed to parse message:', e);
        }
      };

      this.ws.onclose = () => {
        console.log('Disconnected from server');
        this.callbacks?.onDisconnect();
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      this.attemptReconnect();
    }
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 10000);
      console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
      setTimeout(() => this.createConnection(), delay);
    }
  }

  private handleMessage(message: ServerMessage): void {
    if (!this.callbacks) return;

    switch (message.type) {
      case 'joined':
        this.callbacks.onJoined(message.data.playerId, message.data.team);
        break;
      case 'state':
        this.callbacks.onState(message.data);
        break;
      case 'playerJoined':
        this.callbacks.onPlayerJoined(
          message.data.playerId,
          message.data.name,
          message.data.team
        );
        break;
      case 'playerLeft':
        this.callbacks.onPlayerLeft(message.data.playerId);
        break;
      case 'error':
        this.callbacks.onError(message.data.message);
        break;
    }
  }

  private send(message: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  join(name: string): void {
    this.send({ type: 'join', data: { name } });
  }

  sendInput(input: InputState): void {
    this.send({ type: 'input', data: input });
  }

  disconnect(): void {
    this.maxReconnectAttempts = 0;
    this.ws?.close();
    this.ws = null;
    this.callbacks = null;
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}
