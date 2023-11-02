import { WebSocketServer } from 'ws';
import http from 'http';
import fs from 'fs';
import path from 'path';
// 创建一个独立的WebSocket服务器，不直接侦听HTTP请求
const wsServer = new WebSocketServer({ noServer: true });
const filePath = path.join(process.cwd(), './index.html');
// 创建HTTP服务器
const httpServer = http.createServer((req, res) => {
  // 处理HTTP请求
  // 读取HTML文件并将其发送给客户端
  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/html' });
      res.end('Not Found');
    } else {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      console.log(data);
      res.end(data);
    }
  });
});

// 在HTTP服务器上监听端口
httpServer.listen(8080, () => {
  console.log('http://localhost:8080/websocket');
});

// 当收到HTTP服务器的"upgrade"事件时，处理WebSocket升级请求
httpServer.on('upgrade', (request, socket, head) => {
  if (request.url === '/websocket') {
    wsServer.handleUpgrade(request, socket, head, (ws) => {
      wsServer.emit('connection', ws, request);
    });
  } else {
    socket.destroy();
  }
});

// WebSocket连接处理逻辑
wsServer.on('connection', (ws, request) => {
  // 处理WebSocket连接
  ws.on('message', (message) => {
    console.log(`Received: ${message}`);
  });
});
