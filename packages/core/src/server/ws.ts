import path from 'node:path';
import { WebSocketServer } from 'ws';
import type { WebSocket } from 'ws';
import { isObject } from '../index.js';

const HMR_PATH = '__/hmr';

interface WebSocketServer {
  clients: Set<WebSocketClient>;
  listen(): void;
  send(payload: any): void;
  send<T extends string>(event: T, payload?: any): void;
  close(): Promise<void>;
  on(event: string, listener: Function): void;
  off(event: string, listener: Function): void;
}

export interface WebSocketClient {
  /**
   * Send event to the client
   */
  send(payload: any): void;
  /**
   * Send custom event
   */
  send(event: string, payload?: any['data']): void;
  /**
   * The raw WebSocket instance
   * @advanced
   */
  socket: any;
}

class WebSocketServerImpl implements WebSocketServer {
  private wss: WebSocketServer;
  private hmr: any;
  constructor(httpServer: any, config: any) {
    this.hmr = isObject(config.server.hmr) && config.server.hmr;
    this.wss = new WebSocketServer({ noServer: true });
    httpServer.on('upgrade', (request, socket, head) => {
      if (request.headers['sec-websocket-protocol'] === 'vite-hmr') {
        this.wss.handleUpgrade(request, socket, head, (ws) => {
          this.wss.emit('connection', ws, request);
        });
      }
    });

    this.wss.on('connection', (ws: any) => {
      ws.on('message', (message: any) => {
        // Handle incoming messages here
        const data = JSON.parse(message.toString());
        if (data.type === 'custom') {
          this.handleCustomEvent(data.event, data.data);
        }
      });

      // Send a connected message to the client
      ws.send(JSON.stringify({ type: 'connected' }));
    });
  }

  listen() {
    // Start listening for WebSocket connections
    wsHttpServer?.listen(port, host);
  }

  clients: Set<any> = new Set();

  send(payload: HMRPayload) {
    // Broadcast the payload to all connected clients
    this.wss.clients.forEach((client) => {
      if (client.readyState === WebSocketServer.OPEN) {
        client.send(JSON.stringify(payload));
      }
    });
  }

  sendCustomEvent<T extends string>(event: T, payload?: any) {
    // Send a custom event to all clients
    this.send({ type: 'custom', event, data: payload });
  }

  close() {
    // Close the WebSocket server and terminate all client connections
    this.wss.close();
  }

  on(event: string, listener: Function) {
    // Register an event listener
    this.wss.on(event, listener);
  }

  off(event: string, listener: Function) {
    // Unregister an event listener
    this.wss.removeListener(event, listener);
  }

  private handleCustomEvent(event: string, data: any) {
    // TODO Handle custom events here
  }
}
