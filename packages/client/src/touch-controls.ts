export function isTouchDevice(): boolean {
  return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
}

export interface TouchInputCallback {
  onDirectionChange(left: boolean, right: boolean, up: boolean, down: boolean): void;
  onKickChange(kick: boolean): void;
}

const DEAD_ZONE = 0.3;
const BASE_SIZE = 120;
const THUMB_SIZE = 50;
const KICK_SIZE = 80;
const DIRECTION_THRESHOLD = Math.cos(Math.PI * 3 / 8); // ~0.38

export class TouchControls {
  private container: HTMLDivElement;
  private joystickBase: HTMLDivElement;
  private joystickThumb: HTMLDivElement;
  private kickButton: HTMLDivElement;
  private callback: TouchInputCallback;

  private joystickTouchId: number | null = null;
  private kickTouchId: number | null = null;
  private baseCenterX = 0;
  private baseCenterY = 0;
  private maxRadius = BASE_SIZE / 2;

  private handleTouchStart: (e: TouchEvent) => void;
  private handleTouchMove: (e: TouchEvent) => void;
  private handleTouchEnd: (e: TouchEvent) => void;

  constructor(gameContainer: HTMLElement, callback: TouchInputCallback) {
    this.callback = callback;

    // Create container
    this.container = document.createElement('div');
    this.container.id = 'touch-controls';
    Object.assign(this.container.style, {
      position: 'fixed',
      bottom: '0',
      left: '0',
      right: '0',
      height: '180px',
      zIndex: '1000',
      pointerEvents: 'none',
    });

    // Create joystick
    this.joystickBase = document.createElement('div');
    Object.assign(this.joystickBase.style, {
      position: 'absolute',
      bottom: '20px',
      left: '20px',
      width: `${BASE_SIZE}px`,
      height: `${BASE_SIZE}px`,
      borderRadius: '50%',
      background: 'rgba(255,255,255,0.15)',
      border: '2px solid rgba(255,255,255,0.3)',
      pointerEvents: 'auto',
      touchAction: 'none',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    });

    this.joystickThumb = document.createElement('div');
    Object.assign(this.joystickThumb.style, {
      width: `${THUMB_SIZE}px`,
      height: `${THUMB_SIZE}px`,
      borderRadius: '50%',
      background: 'rgba(255,255,255,0.4)',
      transition: 'none',
    });
    this.joystickBase.appendChild(this.joystickThumb);

    // Create kick button
    this.kickButton = document.createElement('div');
    this.kickButton.textContent = 'KICK';
    Object.assign(this.kickButton.style, {
      position: 'absolute',
      bottom: '40px',
      right: '30px',
      width: `${KICK_SIZE}px`,
      height: `${KICK_SIZE}px`,
      borderRadius: '50%',
      background: 'rgba(78,204,163,0.3)',
      border: '2px solid rgba(78,204,163,0.6)',
      color: '#ffffff',
      fontSize: '14px',
      fontWeight: 'bold',
      fontFamily: 'Arial, sans-serif',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      pointerEvents: 'auto',
      touchAction: 'none',
      userSelect: 'none',
      webkitUserSelect: 'none',
    });

    this.container.appendChild(this.joystickBase);
    this.container.appendChild(this.kickButton);
    gameContainer.appendChild(this.container);

    // Bind event handlers
    this.handleTouchStart = this.onTouchStart.bind(this);
    this.handleTouchMove = this.onTouchMove.bind(this);
    this.handleTouchEnd = this.onTouchEnd.bind(this);

    document.addEventListener('touchstart', this.handleTouchStart, { passive: false });
    document.addEventListener('touchmove', this.handleTouchMove, { passive: false });
    document.addEventListener('touchend', this.handleTouchEnd, { passive: false });
    document.addEventListener('touchcancel', this.handleTouchEnd, { passive: false });
  }

  show(): void {
    this.container.style.display = '';
  }

  hide(): void {
    this.container.style.display = 'none';
  }

  destroy(): void {
    document.removeEventListener('touchstart', this.handleTouchStart);
    document.removeEventListener('touchmove', this.handleTouchMove);
    document.removeEventListener('touchend', this.handleTouchEnd);
    document.removeEventListener('touchcancel', this.handleTouchEnd);
    this.container.remove();
  }

  private isInsideElement(touch: Touch, element: HTMLElement): boolean {
    const rect = element.getBoundingClientRect();
    return (
      touch.clientX >= rect.left &&
      touch.clientX <= rect.right &&
      touch.clientY >= rect.top &&
      touch.clientY <= rect.bottom
    );
  }

  private onTouchStart(e: TouchEvent): void {
    for (let i = 0; i < e.changedTouches.length; i++) {
      const touch = e.changedTouches[i];

      if (this.joystickTouchId === null && this.isInsideElement(touch, this.joystickBase)) {
        e.preventDefault();
        this.joystickTouchId = touch.identifier;
        const rect = this.joystickBase.getBoundingClientRect();
        this.baseCenterX = rect.left + rect.width / 2;
        this.baseCenterY = rect.top + rect.height / 2;
        this.updateJoystick(touch.clientX, touch.clientY);
      } else if (this.kickTouchId === null && this.isInsideElement(touch, this.kickButton)) {
        e.preventDefault();
        this.kickTouchId = touch.identifier;
        this.kickButton.style.background = 'rgba(78,204,163,0.6)';
        this.callback.onKickChange(true);
      }
    }
  }

  private onTouchMove(e: TouchEvent): void {
    for (let i = 0; i < e.changedTouches.length; i++) {
      const touch = e.changedTouches[i];

      if (touch.identifier === this.joystickTouchId) {
        e.preventDefault();
        this.updateJoystick(touch.clientX, touch.clientY);
      }
    }
  }

  private onTouchEnd(e: TouchEvent): void {
    for (let i = 0; i < e.changedTouches.length; i++) {
      const touch = e.changedTouches[i];

      if (touch.identifier === this.joystickTouchId) {
        e.preventDefault();
        this.joystickTouchId = null;
        this.resetJoystickThumb();
        this.callback.onDirectionChange(false, false, false, false);
      }

      if (touch.identifier === this.kickTouchId) {
        e.preventDefault();
        this.kickTouchId = null;
        this.kickButton.style.background = 'rgba(78,204,163,0.3)';
        this.callback.onKickChange(false);
      }
    }
  }

  private updateJoystick(clientX: number, clientY: number): void {
    let dx = clientX - this.baseCenterX;
    let dy = clientY - this.baseCenterY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    // Clamp thumb position to base radius
    const clampedDist = Math.min(distance, this.maxRadius);
    let thumbX = 0;
    let thumbY = 0;
    if (distance > 0) {
      thumbX = (dx / distance) * clampedDist;
      thumbY = (dy / distance) * clampedDist;
    }

    this.joystickThumb.style.transform = `translate(${thumbX}px, ${thumbY}px)`;

    // Map to boolean directions
    const normalized = distance / this.maxRadius;
    if (normalized < DEAD_ZONE) {
      this.callback.onDirectionChange(false, false, false, false);
      return;
    }

    const angle = Math.atan2(dy, dx);
    const cosA = Math.cos(angle);
    const sinA = Math.sin(angle);

    this.callback.onDirectionChange(
      cosA < -DIRECTION_THRESHOLD,  // left
      cosA > DIRECTION_THRESHOLD,   // right
      sinA < -DIRECTION_THRESHOLD,  // up
      sinA > DIRECTION_THRESHOLD,   // down
    );
  }

  private resetJoystickThumb(): void {
    this.joystickThumb.style.transform = 'translate(0px, 0px)';
  }
}
