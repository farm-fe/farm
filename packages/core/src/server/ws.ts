import type { IncomingMessage } from 'node:http';
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRawType } from 'ws';

import { WebSocket, WebSocketServer as WebSocketServerRaw } from 'ws';
import { Logger, NormalizedServerConfig, red } from '../index.js';
import { HmrEngine } from './hmr-engine.js';
import { Server } from './type.js';

import type { ILogger } from '../index.js';
import { resolveHostname, resolveServerUrls } from '../utils/http.js';

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
  send(payload: any): void;
  send(event: string, payload?: any['data']): void;
  rawSend(str: string): void;
  socket: WebSocketRawType;
}

export default class WsServer implements IWebSocketServer {
  public wss: WebSocketServerRaw;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRawType, WebSocketClient>();
  public bufferedError: any = null;
  public logger: ILogger;
  private hmrOrigins: string[];
  constructor(
    private httpServer: Server,
    private config: NormalizedServerConfig,
    private hmrEngine: HmrEngine,
    logger?: ILogger
  ) {
    this.logger = logger ?? new Logger();
    this.hmrOrigins = this.generateHMROrigins(config);
    this.createWebSocketServer();
  }

  private generateHMROrigins(config: NormalizedServerConfig): string[] {
    const { protocol, hostname, port } = config;
    const origins = [];

    // Add localhost with configured port
    const urls = resolveServerUrls(this.httpServer, config);
    const localUrls = [...(urls.local || []), ...(urls.network || [])];

    for (const url of localUrls) {
      origins.push(url);
    }

    // Add non-localhost origin
    const configuredOrigin = `${protocol}://${hostname.name}:${port}`;

    if (
      hostname &&
      hostname.name &&
      localUrls.every((url) => url !== configuredOrigin)
    ) {
      origins.push(configuredOrigin);
    }

    if (this.config.host !== this.config.hmr.host) {
      const hmrHostname = resolveHostname(this.config.hmr.host);
      origins.push(
        `${this.config.hmr?.protocol || protocol}://${hmrHostname.name}:${this.config.hmr?.port || this.config.port}`
      );
    }

    return origins;
  }

  private createWebSocketServer() {
    try {
      const WebSocketServer = process.versions.bun
        ? // @ts-expect-error: Bun defines `import.meta.require`
          import.meta.require('ws').WebSocketServer
        : WebSocketServerRaw;

      if (this.config.host === this.config.hmr.host) {
        this.wss = new WebSocketServer({ noServer: true });
        this.connection();
        this.httpServer.on('upgrade', this.upgradeWsServer.bind(this));
      } else {
        const hmrHostname = resolveHostname(this.config.hmr.host);
        this.wss = new WebSocketServer({
          host: hmrHostname.name,
          port: this.config.hmr?.port || this.config.port
        });
        this.connection();
      }
    } catch (err) {
      this.handleSocketError(err);
    }
  }

  private upgradeWsServer(
    request: IncomingMessage,
    socket: Duplex,
    head: Buffer
  ) {
    if (this.isHMRRequest(request)) {
      this.handleHMRUpgrade(request, socket, head);
    } else {
      this.logger.error(
        `HMR upgrade failed because of invalid HMR path, header or host. The HMR connection is only allowed on hosts: ${this.hmrOrigins.join(
          ', '
        )}. You can try set server.host or server.allowedHosts to allow the connection.`
      );
    }
  }

  listen() {
    // TODO alone with use httpServer we need start this function
    // Start listening for WebSocket connections
  }

  // Farm uses the `sendMessage` method in hmr and
  // the send method is reserved for migration vite
  send(...args: any[]) {
    let payload: any;
    if (typeof args[0] === 'string') {
      payload = {
        type: 'custom',
        event: args[0],
        data: args[1]
      };
    } else {
      payload = args[0];
    }

    if (payload.type === 'error' && !this.wss.clients.size) {
      this.bufferedError = payload;
      return;
    }

    const stringified = JSON.stringify(payload);
    this.wss.clients.forEach((client) => {
      // readyState 1 means the connection is open
      if (client.readyState === 1) {
        client.send(stringified);
      }
    });
  }

  private isHMRRequest(request: IncomingMessage): boolean {
    const origin = request.headers['origin'];

    return (
      request.url === this.config.hmr.path &&
      request.headers['sec-websocket-protocol'] === HMR_HEADER &&
      (this.hmrOrigins.includes(origin) ||
        this.config.allowedHosts?.includes(origin))
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

  get clients(): Set<WebSocketClient> {
    return new Set(
      Array.from(this.wss.clients).map(this.getSocketClient.bind(this))
    );
  }

  // a custom method defined by farm to send custom events
  public sendCustomEvent<T extends string>(event: T, payload?: any) {
    // Send a custom event to all clients
    this.send({ type: 'custom', event, data: payload });
  }

  public on(event: string, listener: (...args: any[]) => void) {
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
          this.logger.error('Failed to parse WebSocket message: ' + raw);
        }
        // transform vite js-update to farm update
        if (parsed?.type === 'js-update' && parsed?.path) {
          this.hmrEngine.hmrUpdate(parsed.path, true);
          return;
        }

        if (!parsed || parsed.type !== 'custom' || !parsed.event) return;
        const listeners = this.customListeners.get(parsed.event);
        if (!listeners?.size) return;
        const client = this.getSocketClient(socket);
        listeners.forEach((listener) => listener(parsed.data, client));
      });

      socket.on('error', (err: Error & { code: string }) => {
        return this.handleSocketError(err);
      });

      socket.send(JSON.stringify({ type: 'connected' }));

      if (this.bufferedError) {
        socket.send(JSON.stringify(this.bufferedError));
        this.bufferedError = null;
      }
    });
  }

  public async close() {
    if (this.upgradeWsServer && this.httpServer) {
      this.httpServer.off('upgrade', this.upgradeWsServer);
    }
    await this.terminateAllClients();
    await this.closeWebSocketServer();
    // TODO if not have httpServer we need close httpServer
  }

  private terminateAllClients() {
    const terminatePromises = Array.from(this.wss.clients).map((client) => {
      return new Promise((resolve) => {
        if (client.readyState === WebSocket.OPEN) {
          client.send(JSON.stringify({ type: 'closing' }));
          client.close(1000, 'Server shutdown');
        }
        // Temporarily remove the direct shutdown of ws
        // client.terminate();
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
        send: (...args) => this.sendMessage(socket, ...args),
        socket,
        rawSend: (str) => socket.send(str)
      });
    }
    return this.clientsMap.get(socket);
  }

  private sendMessage(socket: WebSocketRawType, ...args: any[]) {
    let payload: any;
    if (typeof args[0] === 'string') {
      payload = {
        type: 'custom',
        event: args[0],
        data: args[1]
      };
    } else {
      payload = args[0];
    }
    socket.send(JSON.stringify(payload));
  }

  private handleSocketError(err: Error & { code: string }) {
    if (err.code === 'EADDRINUSE') {
      this.logger.error(red(`WebSocket server error: Port is already in use`), {
        error: err
      });
    } else {
      this.logger.error(
        red(`WebSocket server error:\n${err.stack || err.message}`),
        {
          error: err
        }
      );
    }
  }
}
