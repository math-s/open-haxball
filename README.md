# Open HaxBall

Open-source HaxBall implementation with Rust WebSocket server and TypeScript client.

## Structure

```
packages/
├── server-rust/    # Rust WebSocket server with physics engine
├── server/         # TypeScript server (legacy/reference)
├── client/         # Browser client with rendering
└── shared/         # Shared TypeScript game logic
```

## Requirements

- **Rust** (1.70+)
- **Node.js** (18+)

## Quick Start

### Run Rust Server

```bash
cd packages/server-rust
cargo run --release
```

Server runs on `ws://127.0.0.1:8080`

### Run Client

```bash
npm install
cd packages/client
npm run dev
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
