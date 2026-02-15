import type { Team } from '@open-haxball/shared';

interface ChatMessage {
  playerId: string;
  name: string;
  text: string;
  element: HTMLDivElement;
  timeout: number;
}

export class Chat {
  private messagesContainer: HTMLDivElement;
  private input: HTMLInputElement;
  private messages: ChatMessage[] = [];
  private onSendMessage: (text: string) => void;
  private getPlayerTeam: (playerId: string) => Team | null;

  constructor(
    onSendMessage: (text: string) => void,
    getPlayerTeam: (playerId: string) => Team | null
  ) {
    this.onSendMessage = onSendMessage;
    this.getPlayerTeam = getPlayerTeam;

    // Create messages container
    this.messagesContainer = document.createElement('div');
    this.messagesContainer.className = 'chat-messages';

    // Get existing input element
    this.input = document.getElementById('chat-input') as HTMLInputElement;

    // Add messages container to game container
    const gameContainer = document.querySelector('.game-container');
    if (gameContainer) {
      gameContainer.appendChild(this.messagesContainer);
    }

    this.setupInputHandlers();
  }

  private setupInputHandlers(): void {
    // Handle Enter key globally
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        if (document.activeElement === this.input) {
          // Input is focused - send message if there's text
          const text = this.input.value.trim();
          if (text) {
            this.onSendMessage(text);
            this.input.value = '';
          }
          this.input.blur();
        } else {
          // Input not focused - focus it
          this.input.focus();
        }
        e.preventDefault();
      } else if (e.key === 'Escape') {
        if (document.activeElement === this.input) {
          this.input.blur();
          e.preventDefault();
        }
      }
    });
  }

  addMessage(playerId: string, name: string, text: string): void {
    // Create message element
    const messageElement = document.createElement('div');
    messageElement.className = 'chat-message';

    // Get team color
    const team = this.getPlayerTeam(playerId);
    const nameColor = this.getTeamColor(team);

    // Set message content with colored name
    messageElement.innerHTML = `<span style="color: ${nameColor}; font-weight: bold;">${this.escapeHtml(name)}</span>: ${this.escapeHtml(text)}`;

    // Add to container
    this.messagesContainer.appendChild(messageElement);

    // Set timeout to remove after 30 seconds
    const timeout = window.setTimeout(() => {
      this.removeMessage(playerId, messageElement);
    }, 30000);

    // Store message
    this.messages.push({
      playerId,
      name,
      text,
      element: messageElement,
      timeout,
    });

    // Scroll to bottom
    this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
  }

  private removeMessage(playerId: string, element: HTMLDivElement): void {
    // Remove from DOM
    if (element.parentElement) {
      element.parentElement.removeChild(element);
    }

    // Remove from messages array
    const index = this.messages.findIndex((m) => m.element === element);
    if (index !== -1) {
      clearTimeout(this.messages[index].timeout);
      this.messages.splice(index, 1);
    }
  }

  private getTeamColor(team: Team | null): string {
    switch (team) {
      case 'red':
        return '#e74c3c';
      case 'blue':
        return '#3498db';
      default:
        return '#cccccc';
    }
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  destroy(): void {
    // Clear all timeouts
    this.messages.forEach((msg) => clearTimeout(msg.timeout));
    this.messages = [];

    // Remove messages container
    if (this.messagesContainer.parentElement) {
      this.messagesContainer.parentElement.removeChild(this.messagesContainer);
    }
  }
}
