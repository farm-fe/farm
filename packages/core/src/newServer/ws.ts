import type { IncomingMessage, Server } from 'node:http';
import { STATUS_CODES, createServer as createHttpServer } from 'node:http';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import { createServer as createHttpsServer } from 'node:https';
import type { Socket } from 'node:net';
import path from 'node:path';
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRaw } from 'ws';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';
import { NormalizedServerConfig } from '../config/types.js';
import { HmrEngine } from '../server/hmr-engine.js';
import { WebSocket as WebSocketTypes } from '../types/ws.js';
import { ILogger, Logger } from '../utils/logger.js';
import { isObject } from '../utils/share.js';
import { HMRChannel } from './hmr.js';
import { CommonServerOptions } from './http.js';
import { HttpServer, ServerOptions } from './index.js';
import {
  CustomPayload,
  ErrorPayload,
  HMRPayload,
  InferCustomEventPayload
} from './type.js';

export interface WebSocketServer extends HMRChannel {
  /**
   * Listen on port and host
   */
  listen(): void;
  /**
   * Get all connected clients.
   */
  clients: Set<WebSocketClient>;
  /**
   * Disconnect all clients and terminate the server.
   */
  close(): Promise<void>;
  /**
   * Handle custom event emitted by `import.meta.hot.send`
   */
  on: WebSocketTypes.Server['on'] & {
    <T extends string>(
      event: T,
      listener: WebSocketCustomListener<InferCustomEventPayload<T>>
    ): void;
  };
  /**
   * Unregister event listener.
   */
  off: WebSocketTypes.Server['off'] & {
    // biome-ignore lint/complexity/noBannedTypes: <explanation>
    (event: string, listener: Function): void;
  };
}

export interface WebSocketClient {
  /**
   * Send event to the client
   */
  send(payload: HMRPayload): void;
  /**
   * Send custom event
   */
  send(event: string, payload?: CustomPayload['data']): void;
  /**
   * The raw WebSocket instance
   * @advanced
   */
  socket: WebSocketTypes;
}

const wsServerEvents = [
  'connection',
  'error',
  'headers',
  'listening',
  'message'
];

function noop() {
  // noop
}

const HMR_HEADER = 'farm_hmr';

export type WebSocketCustomListener<T> = (
  data: T,
  client: WebSocketClient
) => void;

const WebSocketServerRaw = process.versions.bun
  ? // @ts-expect-error: Bun defines `import.meta.require`
    import.meta.require('ws').WebSocketServer
  : WebSocketServerRaw_;

export class WsServer {
  public wss: WebSocketServerRaw_;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRaw, WebSocketClient>();
  public bufferedError: ErrorPayload | null = null;
  public logger: ILogger;
  public wsServer: Server;

  constructor(
    // private httpServer: HttpServer,
    // private config: ResolvedUserConfig,
    // private httpsOptions: HttpsServerOptions,
    // private publicPath: string,
    // private hmrEngine: HmrEngine,
    // logger?: ILogger
    private readonly app: any
  ) {
    this.logger = app.logger ?? new Logger();
    this.createWebSocketServer();
  }

  createWebSocketServer() {
    const self = this;
    const { resolvedUserConfig: config } = this.app;
    const serverConfig = config.server as unknown as ServerOptions;
    if (serverConfig.ws === false) {
      return {
        name: 'ws',
        get clients() {
          return new Set<WebSocketClient>();
        },
        async close() {
          // noop
        },
        on: noop as any as WebSocketServer['on'],
        off: noop as any as WebSocketServer['off'],
        listen: noop,
        send: noop
      };
    }
    let wsHttpServer: Server | undefined = undefined;

    const hmr = isObject(serverConfig.hmr) && serverConfig.hmr;
    const hmrServer = hmr && hmr.server;
    const hmrPort = hmr && hmr.port;
    const portsAreCompatible = !hmrPort || hmrPort === serverConfig.port;
    // @ts-ignore
    this.wsServer = hmrServer || (portsAreCompatible && this.app.httpServer);
    let hmrServerWsListener: (
      req: InstanceType<typeof IncomingMessage>,
      socket: Duplex,
      head: Buffer
    ) => void;
    const port = hmrPort || 9000;
    const host = (hmr && hmr.host) || undefined;

    if (this.wsServer) {
      let hmrBase = this.app.publicPath;

      const hmrPath = hmr ? hmr.path : undefined;
      if (hmrPath) {
        hmrBase = path.posix.join(hmrBase, hmrPath as string);
      }
      this.wss = new WebSocketServerRaw({ noServer: true });
      hmrServerWsListener = (req, socket, head) => {
        if (
          req.headers['sec-websocket-protocol'] === HMR_HEADER &&
          req.url === hmrBase
        ) {
          this.wss.handleUpgrade(req, socket as Socket, head, (ws) => {
            this.wss.emit('connection', ws, req);
          });
        }
      };
      this.wsServer.on('upgrade', hmrServerWsListener);
    } else {
      // http server request handler keeps the same with
      // https://github.com/websockets/ws/blob/45e17acea791d865df6b255a55182e9c42e5877a/lib/websocket-server.js#L88-L96
      const route = ((_, res) => {
        const statusCode = 426;
        const body = STATUS_CODES[statusCode];
        if (!body)
          throw new Error(
            `No body text found for the ${statusCode} status code`
          );

        res.writeHead(statusCode, {
          'Content-Length': body.length,
          'Content-Type': 'text/plain'
        });
        res.end(body);
      }) as Parameters<typeof createHttpServer>[1];

      if (this.app.httpsOptions) {
        wsHttpServer = createHttpsServer(this.app.httpsOptions, route);
      } else {
        wsHttpServer = createHttpServer(route);
      }
      // vite dev server in middleware mode
      // need to call ws listen manually
      this.wss = new WebSocketServerRaw({ server: wsHttpServer });
    }

    this.wss.on('connection', (socket) => {
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
          this.app.hmrEngine.hmrUpdate(parsed.path, true);
          return;
        }
        if (!parsed || parsed.type !== 'custom' || !parsed.event) return;
        const listeners = this.customListeners.get(parsed.event);
        if (!listeners?.size) return;
        const client = this.getSocketClient(socket);
        listeners.forEach((listener) => listener(parsed.data, client));
      });
      socket.on('error', (err) => {
        throw new Error(`ws error:\n${err.stack}`);
      });

      socket.send(JSON.stringify({ type: 'connected' }));

      if (this.bufferedError) {
        socket.send(JSON.stringify(this.bufferedError));
        this.bufferedError = null;
      }
    });

    this.wss.on('error', (e: Error & { code: string }) => {
      if (e.code === 'EADDRINUSE') {
        throw new Error('WebSocket server error: Port is already in use');
      } else {
        throw new Error(`WebSocket server error ${e.stack || e.message}`);
      }
    });

    return {
      name: 'ws',
      listen: () => {
        // @ts-ignore
        wsHttpServer?.listen(port, host);
      },
      on: ((event: string, fn: () => void) => {
        if (wsServerEvents.includes(event)) this.wss.on(event, fn);
        else {
          if (!this.customListeners.has(event)) {
            this.customListeners.set(event, new Set());
          }
          this.customListeners.get(event).add(fn);
        }
      }) as WebSocketServer['on'],
      off: ((event: string, fn: () => void) => {
        if (wsServerEvents.includes(event)) {
          this.wss.off(event, fn);
        } else {
          this.customListeners.get(event)?.delete(fn);
        }
      }) as WebSocketServer['off'],

      get clients() {
        return new Set(
          Array.from(this.wss.clients).map((socket: any) =>
            self.getSocketClient(socket)
          )
        );
      },

      send(...args: any[]) {
        let payload: HMRPayload;
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
        this.wss.clients.forEach((client: any) => {
          // readyState 1 means the connection is open
          if (client.readyState === 1) {
            client.send(stringified);
          }
        });
      },

      close() {
        // should remove listener if hmr.server is set
        // otherwise the old listener swallows all WebSocket connections
        if (hmrServerWsListener && this.wsServer) {
          this.wsServer.off('upgrade', hmrServerWsListener);
        }
        return new Promise<void>((resolve, reject) => {
          this.wss.clients.forEach((client: any) => {
            client.terminate();
          });
          this.wss.close((err: any) => {
            if (err) {
              reject(err);
            } else {
              if (wsHttpServer) {
                wsHttpServer.close((err) => {
                  if (err) {
                    reject(err);
                  } else {
                    resolve();
                  }
                });
              } else {
                resolve();
              }
            }
          });
        });
      }
    };
  }

  send(...args: any[]) {
    let payload: HMRPayload;
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
    this.wss.clients.forEach((client: any) => {
      // readyState 1 means the connection is open
      if (client.readyState === 1) {
        client.send(stringified);
      }
    });
  }

  getSocketClient(socket: WebSocketRaw) {
    if (!this.clientsMap.has(socket)) {
      this.clientsMap.set(socket, {
        send: (...args) => {
          let payload: HMRPayload;
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
        },
        // @ts-ignore
        rawSend: (str: string) => socket.send(str),
        socket
      });
    }
    return this.clientsMap.get(socket);
  }
}
