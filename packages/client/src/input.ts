import { InputState, createEmptyInput } from '@open-haxball/shared';
import { TouchControls, isTouchDevice } from './touch-controls.js';

export class InputHandler {
  private state: InputState = createEmptyInput();
  private onChange: ((input: InputState) => void) | null = null;
  private touchControls: TouchControls | null = null;

  constructor() {
    this.handleKeyDown = this.handleKeyDown.bind(this);
    this.handleKeyUp = this.handleKeyUp.bind(this);
  }

  start(onChange: (input: InputState) => void, gameContainer?: HTMLElement): void {
    this.onChange = onChange;
    window.addEventListener('keydown', this.handleKeyDown);
    window.addEventListener('keyup', this.handleKeyUp);

    if (isTouchDevice() && gameContainer) {
      this.touchControls = new TouchControls(gameContainer, {
        onDirectionChange: (left, right, up, down) => {
          const changed =
            this.state.left !== left ||
            this.state.right !== right ||
            this.state.up !== up ||
            this.state.down !== down;
          if (changed) {
            this.state.left = left;
            this.state.right = right;
            this.state.up = up;
            this.state.down = down;
            this.notifyChange();
          }
        },
        onKickChange: (kick) => {
          if (this.state.kick !== kick) {
            this.state.kick = kick;
            this.notifyChange();
          }
        },
      });
      this.touchControls.show();
    }
  }

  stop(): void {
    window.removeEventListener('keydown', this.handleKeyDown);
    window.removeEventListener('keyup', this.handleKeyUp);
    this.touchControls?.destroy();
    this.touchControls = null;
    this.onChange = null;
  }

  private handleKeyDown(e: KeyboardEvent): void {
    if (this.updateKey(e.key, true)) {
      e.preventDefault();
      this.notifyChange();
    }
  }

  private handleKeyUp(e: KeyboardEvent): void {
    if (this.updateKey(e.key, false)) {
      e.preventDefault();
      this.notifyChange();
    }
  }

  private updateKey(key: string, pressed: boolean): boolean {
    const prevState = { ...this.state };

    switch (key.toLowerCase()) {
      case 'w':
      case 'arrowup':
        this.state.up = pressed;
        break;
      case 's':
      case 'arrowdown':
        this.state.down = pressed;
        break;
      case 'a':
      case 'arrowleft':
        this.state.left = pressed;
        break;
      case 'd':
      case 'arrowright':
        this.state.right = pressed;
        break;
      case ' ':
      case 'x':
        this.state.kick = pressed;
        break;
      default:
        return false;
    }

    // Return true if state changed
    return (
      prevState.up !== this.state.up ||
      prevState.down !== this.state.down ||
      prevState.left !== this.state.left ||
      prevState.right !== this.state.right ||
      prevState.kick !== this.state.kick
    );
  }

  private notifyChange(): void {
    if (this.onChange) {
      this.onChange({ ...this.state });
    }
  }

  getState(): InputState {
    return { ...this.state };
  }
}
