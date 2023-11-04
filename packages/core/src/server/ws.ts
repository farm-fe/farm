import http from 'node:http';
// import path from 'node:path';
import type { IncomingMessage } from 'node:http';
// import type { Socket } from 'node:net'
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRawType } from 'ws';

import { WebSocketServer as WebSocketServerRaw } from 'ws';
import { DefaultLogger, Logger, red } from '../index.js';
// import type { WebSocket } from 'ws';
// import { isObject } from '../index.js';

// const HMR_PATH = '__/hmr';
const HMR_HEADER = 'farm_hmr';

export interface IWebSocketServer {
  clients: Set<WebSocketClient>;
  listen(): void;
  send(payload: any): void;
  send<T extends string>(event: T, payload?: any): void;
  close(): Promise<void>;
  on(event: string, listener: any): void;
  off(event: string, listener: any): void;
}

const wsServerEvents = [
  'connection',
  'error',
  'headers',
  'listening',
  'message'
];
export type WebSocketCustomListener<T> = (
  data: T,
  client: WebSocketClient
) => void;
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
  socket: WebSocketRawType;
}

// class WebSocketServerImpl implements WebSocketServer {
export default class WsServer implements IWebSocketServer {
  public wss: WebSocketServerRaw;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRawType, WebSocketClient>();
  public bufferedError: any = null;
  public logger: Logger;
  // private hmr: any;
  constructor(
    private httpServer: http.Server,
    private config: any,
    public isFarmHmrEvent: boolean = false,
    logger?: Logger
  ) {
    this.logger = logger ?? new DefaultLogger();
    this.createWebSocketServer();
  }

  private createWebSocketServer() {
    this.wss = new WebSocketServerRaw({ noServer: true });
    // TODO IF not have httpServer
    this.httpServer.on('upgrade', this.upgradeWsServer.bind(this));
  }

  private upgradeWsServer(
    request: IncomingMessage,
    socket: Duplex,
    head: Buffer
  ) {
    if (this.isHMRRequest(request)) {
      this.handleHMRUpgrade(request, socket, head);
    }
  }

  listen() {
    // Start listening for WebSocket connections
    // if alone with use httpServer we need start this function
    // wsHttpServer?.listen(port, host);
    // TODO IF not have httpServer
  }

  // clients: Set<any> = new Set();

  send(_payload: any) {
    // Broadcast the payload to all connected clients
    // this.wss.clients.forEach((client) => {
    //   if (client.readyState === WebSocketServer.OPEN) {
    //     client.send(JSON.stringify(payload));
    //   }
    // });
  }

  private isHMRRequest(request: IncomingMessage): boolean {
    return (
      request.url === this.config.hmr.path &&
      request.headers['sec-websocket-protocol'] === HMR_HEADER
    );
  }

  private handleHMRUpgrade(
    request: IncomingMessage,
    socket: Duplex,
    head: Buffer
  ) {
    this.wss.handleUpgrade(request, socket, head, (ws: WebSocketRawType) => {
      this.wss.emit('connection', ws, request);
    });
  }

  // sendCustomEvent<T extends string>(event: T, payload?: any) {
  //   // Send a custom event to all clients
  //   this.send({ type: 'custom', event, data: payload });
  // }

  // // close() {
  // //   // Close the WebSocket server and terminate all client connections
  // //   this.wss.close();
  // // }

  // on(event: string, listener: () => void) {
  //   // Register an event listener
  //   if (wsServerEvents.includes(event)) this.wss.on(event, listener);
  //   else {
  //     if (!this.customListeners.has(event)) {
  //       this.customListeners.set(event, new Set());
  //     }
  //     this.customListeners.get(event).add(listener);
  //   }
  // }

  // off(event: string, listener: () => void) {
  //   if (wsServerEvents.includes(event)) {
  //     this.wss.off(event, listener);
  //   } else {
  //     this.customListeners.get(event)?.delete(listener);
  //   }
  // }

  get clients(): Set<any> {
    return new Set(
      Array.from(this.wss.clients).map(this.getSocketClient.bind(this))
    );
  }

  // private handleCustomEvent(event: string, data: any) {
  //   // TODO Handle custom events here
  // }

  // close() {
  //   if (this.upgradeWsServer && this.httpServer) {
  //     this.httpServer.off('upgrade', this.upgradeWsServer);
  //   }
  //   return new Promise((resolve, reject) => {
  //     this.wss.clients.forEach((client) => {
  //       client.terminate();
  //     });
  //     this.wss.close((err) => {
  //       if (err) {
  //         reject(err);
  //       } else {
  //         // TODO if not have httpServer
  //         resolve(true);
  //       }
  //     });
  //   });
  // }

  public sendCustomEvent<T extends string>(event: T, payload?: any) {
    // Send a custom event to all clients
    this.send({ type: 'custom', event, data: payload });
  }

  public on(event: string, listener: () => void) {
    if (wsServerEvents.includes(event)) {
      this.wss.on(event, listener);
    } else {
      this.addCustomEventListener(event, listener);
    }
  }

  public off(event: string, listener: () => void) {
    if (wsServerEvents.includes(event)) {
      this.wss.off(event, listener);
    } else {
      this.removeCustomEventListener(event, listener);
    }
  }

  connection() {
    this.wss.on('connection', (socket: WebSocketRawType) => {
      socket.on('message', (raw) => {
        if (!this.customListeners.size) return;
        let parsed: any;
        try {
          parsed = JSON.parse(String(raw));
        } catch {
          // logger.error('Failed to parse WebSocket message: ' + raw);
        }
        if (!parsed || parsed.type !== 'custom' || !parsed.event) return;
        const listeners = this.customListeners.get(parsed.event);
        if (!listeners?.size) return;
        const client = this.getSocketClient(socket);
        listeners.forEach((listener) => listener(parsed.data, client));
      });
      socket.on('error', (err) => {
        this.logger.error(`${red(`ws error:`)}\n${err.stack}`, {
          timestamp: true,
          error: err
        });
      });
      socket.send(JSON.stringify({ type: 'connected' }));
      if (this.bufferedError) {
        socket.send(JSON.stringify(this.bufferedError));
        this.bufferedError = null;
      }
    });
  }

  // getSocketClient(socket: WebSocketRawType) {
  //   if (!this.clientsMap.has(socket)) {
  //     this.clientsMap.set(socket, {
  //       send: (...args) => {
  //         // HMR payload
  //         let payload: any;
  //         if (typeof args[0] === 'string') {
  //           payload = {
  //             type: 'custom',
  //             event: args[0],
  //             data: args[1]
  //           };
  //         } else {
  //           payload = args[0];
  //         }
  //         socket.send(JSON.stringify(payload));
  //       },
  //       socket
  //     });
  //   }
  //   return this.clientsMap.get(socket);
  // }

  // public get clients() {
  //   return new Set(Array.from(this.wss.clients).map(this.getSocketClient));
  // }

  public async close() {
    if (this.upgradeWsServer && this.httpServer) {
      this.httpServer.off('upgrade', this.upgradeWsServer);
    }
    await this.terminateAllClients();
    await this.closeWebSocketServer();
    // TODO if not have httpServer
  }

  private terminateAllClients() {
    const terminatePromises = Array.from(this.wss.clients).map((client) => {
      return new Promise((resolve) => {
        client.terminate();
        client.once('close', () => resolve(true));
      });
    });
    return Promise.all(terminatePromises);
  }

  private closeWebSocketServer() {
    return new Promise((resolve, reject) => {
      this.wss.close((err) => {
        if (err) {
          reject(err);
        } else {
          // TODO if not have httpServer
          resolve(true);
        }
      });
    });
  }

  private addCustomEventListener(event: string, listener: () => void) {
    if (!this.customListeners.has(event)) {
      this.customListeners.set(event, new Set());
    }
    this.customListeners.get(event).add(listener);
  }

  private removeCustomEventListener(event: string, listener: () => void) {
    this.customListeners.get(event)?.delete(listener);
  }

  private getSocketClient(socket: WebSocketRawType) {
    if (!this.clientsMap.has(socket)) {
      this.clientsMap.set(socket, {
        send: (...args) =>
          this.sendMessage(socket, this.isFarmHmrEvent, ...args),
        socket
      });
    }
    return this.clientsMap.get(socket);
  }

  private sendMessage(
    socket: WebSocketRawType,
    isFarmHmrEvent: boolean,
    ...args: any[]
  ) {
    let payload: any;
    if (typeof args[0] === 'string' && !isFarmHmrEvent) {
      payload = {
        type: 'cus撒打算大撒打算大tom',
        event: args[0],
        data: args[1]
      };
    } else {
      payload = args[0];
    }
    socket.send(payload);
  }

  // private error() {
  //   this.wss.on('error', (e: Error & { code: string }) => {
  //     if (e.code === 'EADDRINUSE') {
  //       this.logger.error(
  //         red(`WebSocket server error: Port is already in use`),
  //         {
  //           error: e
  //         }
  //       );
  //     } else {
  //       this.logger.error(
  //         red(`WebSocket server error:\n${e.stack || e.message}`),
  //         {
  //           error: e
  //         }
  //       );
  //     }
  //   });
  // }
}
