import { InputState, SerializedGameState, Team } from '@open-haxball/shared';

// Client -> Server messages
export type ClientMessage =
  | { type: 'join'; data: { name: string } }
  | { type: 'input'; data: InputState };

// Server -> Client messages
export type ServerMessage =
  | { type: 'joined'; data: { playerId: string; team: Team } }
  | { type: 'state'; data: SerializedGameState }
  | { type: 'playerJoined'; data: { playerId: string; name: string; team: Team } }
  | { type: 'playerLeft'; data: { playerId: string } }
  | { type: 'error'; data: { message: string } };

export function parseClientMessage(data: string): ClientMessage | null {
  try {
    const message = JSON.parse(data);
    if (typeof message !== 'object' || !message.type) {
      return null;
    }
    return message as ClientMessage;
  } catch {
    return null;
  }
}

export function serializeServerMessage(message: ServerMessage): string {
  return JSON.stringify(message);
}
