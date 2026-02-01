import { InputState, createEmptyInput } from '@open-haxball/shared';

export class InputHandler {
  private state: InputState = createEmptyInput();
  private onChange: ((input: InputState) => void) | null = null;

  constructor() {
    this.handleKeyDown = this.handleKeyDown.bind(this);
    this.handleKeyUp = this.handleKeyUp.bind(this);
  }

  start(onChange: (input: InputState) => void): void {
    this.onChange = onChange;
    window.addEventListener('keydown', this.handleKeyDown);
    window.addEventListener('keyup', this.handleKeyUp);
  }

  stop(): void {
    window.removeEventListener('keydown', this.handleKeyDown);
    window.removeEventListener('keyup', this.handleKeyUp);
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
