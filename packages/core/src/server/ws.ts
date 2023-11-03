import http from 'node:http';
// import path from 'node:path';
import { WebSocketServer as WebSocketServerRaw } from 'ws';
// import type { WebSocket } from 'ws';
// import { isObject } from '../index.js';

// const HMR_PATH = '__/hmr';

// interface WebSocketServer {
//   clients: Set<WebSocketClient>;
//   listen(): void;
//   send(payload: any): void;
//   send<T extends string>(event: T, payload?: any): void;
//   close(): Promise<void>;
//   on(event: string, listener: any): void;
//   off(event: string, listener: any): void;
// }

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

// class WebSocketServerImpl implements WebSocketServer {
export default class createFarmWsServer {
  public wss: any;
  // private hmr: any;
  constructor(private httpServer: http.Server, _config: any) {
    this.createWebSocketServer();
  }

  createWebSocketServer() {
    this.wss = new WebSocketServerRaw({ noServer: true });
    this.httpServer.on('upgrade', this.upgradeWsServer.bind(this));
  }

  upgradeWsServer(request: any, socket: any, head: any) {
    if (
      // request.url === config.hmr.path &&
      request.headers['sec-websocket-protocol'] === 'farm_hmr'
    ) {
      console.log(this);

      this.wss.handleUpgrade(request, socket, head, (ws: any) => {
        this.wss.emit('connection', ws, request);
      });
    }
  }

  listen() {
    // Start listening for WebSocket connections
    // wsHttpServer?.listen(port, host);
  }

  clients: Set<any> = new Set();

  send(_payload: any) {
    // Broadcast the payload to all connected clients
    // this.wss.clients.forEach((client) => {
    //   if (client.readyState === WebSocketServer.OPEN) {
    //     client.send(JSON.stringify(payload));
    //   }
    // });
  }

  sendCustomEvent<T extends string>(event: T, payload?: any) {
    // Send a custom event to all clients
    this.send({ type: 'custom', event, data: payload });
  }

  // close() {
  //   // Close the WebSocket server and terminate all client connections
  //   this.wss.close();
  // }

  on(event: string, listener: any) {
    // Register an event listener
    this.wss.on(event, listener);
  }

  // off(event: string, listener: any) {
  //   // Unregister an event listener
  //   // this.wss.removeListener(event, listener);
  // }

  // private handleCustomEvent(event: string, data: any) {
  //   // TODO Handle custom events here
  // }
}
