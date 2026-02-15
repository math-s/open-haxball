import { SerializedGameState, Team } from '@open-haxball/shared';
import { Renderer } from './renderer.js';
import { InputHandler } from './input.js';
import { NetworkClient } from './network.js';

class GameClient {
  private renderer: Renderer;
  private inputHandler: InputHandler;
  private network: NetworkClient;

  private gameState: SerializedGameState | null = null;
  private localPlayerId: string | null = null;
  private localTeam: Team | null = null;
  private playerName: string;
  private joined = false;

  constructor(canvas: HTMLCanvasElement, serverUrl: string, playerName: string) {
    this.renderer = new Renderer(canvas);
    this.inputHandler = new InputHandler();
    this.network = new NetworkClient(serverUrl);
    this.playerName = playerName;
  }

  start(): void {
    // Connect to server
    this.network.connect({
      onJoined: (playerId, team) => {
        console.log(`Joined as ${playerId} on team ${team}`);
        this.localPlayerId = playerId;
        this.localTeam = team;
        this.joined = true;
        this.updateUI();
      },
      onState: (state) => {
        this.gameState = state;
      },
      onPlayerJoined: (playerId, name, team) => {
        console.log(`${name} joined team ${team}`);
      },
      onPlayerLeft: (playerId) => {
        console.log(`Player ${playerId} left`);
      },
      onError: (message) => {
        console.error('Server error:', message);
      },
      onDisconnect: () => {
        console.log('Disconnected');
        this.gameState = null;
        this.joined = false;
        this.updateUI();
      },
    });

    // Start input handling
    const gameContainer = document.querySelector('.game-container') as HTMLElement;
    this.inputHandler.start((input) => {
      if (this.joined) {
        this.network.sendInput(input);
      }
    }, gameContainer);

    // Wait a bit for connection, then join
    setTimeout(() => {
      if (this.network.isConnected()) {
        this.network.join(this.playerName);
      } else {
        // Retry after connection
        const checkConnection = setInterval(() => {
          if (this.network.isConnected() && !this.joined) {
            this.network.join(this.playerName);
            clearInterval(checkConnection);
          }
        }, 500);
      }
    }, 500);

    // Start render loop
    this.renderLoop();
  }

  private renderLoop(): void {
    this.renderer.render(this.gameState, this.localPlayerId);
    requestAnimationFrame(() => this.renderLoop());
  }

  private updateUI(): void {
    const teamDisplay = document.getElementById('team-display');
    if (teamDisplay) {
      if (this.localTeam) {
        teamDisplay.textContent = `Team: ${this.localTeam.toUpperCase()}`;
        teamDisplay.style.color = this.localTeam === 'red' ? '#e74c3c' : '#3498db';
      } else {
        teamDisplay.textContent = 'Connecting...';
        teamDisplay.style.color = '#ffffff';
      }
    }
  }

  stop(): void {
    this.inputHandler.stop();
    this.network.disconnect();
  }
}

// Initialize game when DOM is ready
function init(): void {
  const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
  if (!canvas) {
    console.error('Canvas not found');
    return;
  }

  // Get player name
  const playerName = prompt('Enter your name:') || `Player${Math.floor(Math.random() * 1000)}`;

  // Check for custom server via URL params
  const params = new URLSearchParams(window.location.search);
  const customServer = params.get('server');

  // Determine server URL based on environment
  let serverUrl: string;
  let serverType: string;

  if (customServer) {
    // Custom server from URL param
    serverUrl = customServer.startsWith('ws') ? customServer : `wss://${customServer}`;
    serverType = 'Custom';
  } else if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    // Local development - always use Rust server
    const port = 3001;
    serverType = 'Rust';
    serverUrl = `ws://${window.location.hostname}:${port}`;
  } else {
    // Production - connect to fly.io (https://open-haxball.fly.dev)
    serverType = 'Rust (Fly.io)';
    serverUrl = 'wss://open-haxball.fly.dev';
  }

  console.log(`Connecting to ${serverType} server: ${serverUrl}`);

  // Update UI to show server type
  const serverDisplay = document.getElementById('server-display');
  if (serverDisplay) {
    serverDisplay.textContent = `Server: ${serverType}`;
    serverDisplay.style.color = '#4ecca3';  // Green for Rust
  }

  const client = new GameClient(canvas, serverUrl, playerName);
  client.start();

  // Handle page unload
  window.addEventListener('beforeunload', () => {
    client.stop();
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
