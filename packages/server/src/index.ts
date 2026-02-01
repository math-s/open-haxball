import { WebSocketServer, WebSocket } from 'ws';
import { Room } from './room.js';
import { parseClientMessage } from './protocol.js';

const PORT = parseInt(process.env.PORT || '3000', 10);

const wss = new WebSocketServer({ port: PORT });
const room = new Room();

room.start();

wss.on('connection', (ws: WebSocket) => {
  room.addClient(ws);

  ws.on('message', (data: Buffer) => {
    const message = parseClientMessage(data.toString());
    if (!message) return;

    switch (message.type) {
      case 'join':
        room.handleJoin(ws, message.data.name);
        break;
      case 'input':
        room.handleInput(ws, message.data);
        break;
    }
  });

  ws.on('close', () => {
    room.removeClient(ws);
  });

  ws.on('error', (error) => {
    console.error('WebSocket error:', error);
    room.removeClient(ws);
  });
});

console.log(`WebSocket server listening on port ${PORT}`);

process.on('SIGINT', () => {
  console.log('Shutting down...');
  room.stop();
  wss.close();
  process.exit(0);
});
