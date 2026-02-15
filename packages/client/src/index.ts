import { SerializedGameState, Team } from '@open-haxball/shared';
import { Renderer } from './renderer.js';
import { InputHandler } from './input.js';
import { NetworkClient } from './network.js';
import { Chat } from './chat.js';

class GameClient {
  private renderer: Renderer;
  private inputHandler: InputHandler;
  private network: NetworkClient;
  private chat: Chat;

  private gameState: SerializedGameState | null = null;
  private localPlayerId: string | null = null;
  private localTeam: Team | null = null;
  private playerName: string;
  private joined = false;
  private isHost = false;
  private matchTimeRemaining: number | null = null;

  constructor(canvas: HTMLCanvasElement, serverUrl: string, playerName: string) {
    this.renderer = new Renderer(canvas);
    this.inputHandler = new InputHandler();
    this.network = new NetworkClient(serverUrl);
    this.playerName = playerName;

    // Initialize chat with callback to send messages and get player team
    this.chat = new Chat(
      (text: string) => {
        if (this.joined) {
          this.network.sendChat(text);
        }
      },
      (playerId: string) => this.getPlayerTeam(playerId)
    );
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
        this.isHost = state.isHost;
        this.matchTimeRemaining = state.matchTimeRemaining ?? null;

        // Debug log when status changes to finished
        if (state.status === 'finished' && this.gameState?.status !== 'finished') {
          console.log('Match finished!', { isHost: state.isHost });
        }

        this.updateMatchUI();
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
      onChat: (playerId, name, text) => {
        this.chat.addMessage(playerId, name, text);
      },
    });

    // Start input handling
    const gameContainer = document.querySelector('.game-container') as HTMLElement;
    this.inputHandler.start((input) => {
      if (this.joined) {
        this.network.sendInput(input);
      }
    }, gameContainer);

    // Team switch button
    const teamSwitchBtn = document.getElementById('team-switch-btn');
    if (teamSwitchBtn) {
      teamSwitchBtn.addEventListener('click', () => {
        if (!this.joined || !this.localTeam) return;

        const newTeam: Team = this.localTeam === 'red' ? 'blue' : 'red';
        const teamName = newTeam === 'red' ? 'RED' : 'BLUE';

        if (confirm(`Switch to ${teamName} team?`)) {
          this.network.switchTeam(newTeam);
          this.localTeam = newTeam; // Optimistic update
          this.updateUI();
        }
      });
    }

    // Restart match button
    const restartBtn = document.getElementById('restart-match-btn');
    if (restartBtn) {
      restartBtn.addEventListener('click', () => {
        console.log('Restart button clicked', { isHost: this.isHost, status: this.gameState?.status });
        if (!this.isHost) {
          console.warn('Not the host, cannot restart');
          return;
        }
        if (confirm('Restart the match?')) {
          console.log('Sending restart match command');
          this.network.restartMatch();
        }
      });
    }

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

    // Update team switch button state
    const teamSwitchBtn = document.getElementById('team-switch-btn') as HTMLButtonElement;
    if (teamSwitchBtn) {
      teamSwitchBtn.disabled = !this.joined;
    }
  }

  private updateMatchUI(): void {
    // Update timer
    const timerEl = document.getElementById('match-timer');
    if (timerEl && this.matchTimeRemaining !== null) {
      const minutes = Math.floor(this.matchTimeRemaining / 60);
      const seconds = Math.floor(this.matchTimeRemaining % 60);
      timerEl.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;

      // Color based on time remaining
      if (this.matchTimeRemaining < 30) {
        timerEl.style.color = '#e74c3c'; // Red
      } else if (this.matchTimeRemaining < 60) {
        timerEl.style.color = '#f39c12'; // Orange
      } else {
        timerEl.style.color = '#4ecca3'; // Green
      }
    } else if (timerEl) {
      timerEl.textContent = '--:--';
      timerEl.style.color = '#888';
    }

    // Show/hide restart button (host can restart early during intermission)
    const restartBtn = document.getElementById('restart-match-btn') as HTMLButtonElement;
    if (restartBtn) {
      const showRestart = this.isHost && this.gameState?.status === 'finished';
      restartBtn.style.display = showRestart ? 'block' : 'none';
      if (showRestart) {
        restartBtn.textContent = 'Restart Now';
      }
    }
  }

  private getPlayerTeam(playerId: string): Team | null {
    if (!this.gameState) return null;
    const player = this.gameState.players.find((p) => p.id === playerId);
    return player ? player.team : null;
  }

  stop(): void {
    this.inputHandler.stop();
    this.network.disconnect();
    this.chat.destroy();
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
