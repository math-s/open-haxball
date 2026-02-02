# Open HaxBall

Open-source HaxBall implementation with Rust WebSocket server and TypeScript client.

## Structure

```
packages/
├── server-rust/    # Rust WebSocket server with physics engine
├── client/         # Browser client with rendering
└── shared/         # Shared TypeScript game logic
```

## Requirements

- **Rust** (1.70+)
- **Node.js** (18+)

## Quick Start

```bash
# Install dependencies
npm install

# Run both server and client
npm run dev
```

The Rust server runs on `ws://127.0.0.1:3001` and the client dev server runs on `http://localhost:8080`.

### Run Components Separately

**Server only:**
```bash
npm run dev:server
# Or directly: cd packages/server-rust && cargo run
```

**Client only:**
```bash
npm run dev:client
```

## Features

- Real-time multiplayer physics
- WebSocket communication with binary protocol
- Collision detection and resolution
- Player movement with keyboard input
- Canvas rendering

## Protocol

Binary WebSocket protocol with MessagePack serialization:
- `Join` - Client joins room
- `Input` - Player keyboard state
- `GameState` - Server broadcasts physics state (60 tick/s)

## License

MIT
