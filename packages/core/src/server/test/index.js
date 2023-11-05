// 服务器端 (server.js)

import { WebSocketServer } from 'ws';
import http from 'http';
import fs from 'fs';
const wsPort = 6000; // WebSocket server port
const httpPort = 8000; // HTTP server port

// 创建 HTTP 服务器，用于提供 HTML 页面
const httpServer = http.createServer((req, res) => {
  if (req.url === '/') {
    const html = fs.readFileSync('index.html', 'utf8');
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(html);
  }
});

httpServer.listen(httpPort, () => {
  console.log(`HTTP server is listening on port ${httpPort}`);
  console.log('Open http://localhost:8000 in your browser');
});

// 创建 WebSocket 服务器
const wss = new WebSocketServer({ noServer: true });

wss.on('connection', (ws) => {
  console.log('WebSocket client connected');

  ws.on('message', (message) => {
    console.log(`Received message from client: ${message}`);
    // 向客户端发送响应消息
    ws.send(`Server received: ${message}`);
  });
});

httpServer.on('upgrade', (request, socket, head) => {
  if (request.url === '/websocket') {
    wss.handleUpgrade(request, socket, head, (ws) => {
      wss.emit('connection', ws, request);
    });
  }
});
