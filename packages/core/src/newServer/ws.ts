import type { IncomingMessage, Server } from 'node:http';
import { STATUS_CODES, createServer as createHttpServer } from 'node:http';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import { createServer as createHttpsServer } from 'node:https';
import type { Socket } from 'node:net';
import path from 'node:path';
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRaw } from 'ws';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';
import { NormalizedServerConfig, ResolvedUserConfig } from '../config/types.js';
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
  public wss: WebSocketRaw;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRaw, WebSocketClient>();
  public bufferedError: ErrorPayload | null = null;
  public logger: ILogger;
  public wsServerOrHmrServer: Server;

  constructor(
    private httpServer: HttpServer,
    private config: ResolvedUserConfig,
    private httpsOptions: HttpsServerOptions,
    private publicPath: string,
    private hmrEngine: HmrEngine,
    logger?: ILogger
  ) {
    this.logger = logger ?? new Logger();
    this.createWebSocketServer();
  }

  createWebSocketServer() {
    const serverConfig = this.config.server as ServerOptions;
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
    let wss: WebSocketServerRaw_;
    let wsHttpServer: Server | undefined = undefined;

    const hmr = isObject(serverConfig.hmr) && serverConfig.hmr;
    const hmrServer = hmr && hmr.server;
    const hmrPort = hmr && hmr.port;
    const portsAreCompatible = !hmrPort || hmrPort === serverConfig.port;
    // @ts-ignore
    this.wsServerOrHmrServer =
      hmrServer || (portsAreCompatible && this.httpServer);
    let hmrServerWsListener: (
      req: InstanceType<typeof IncomingMessage>,
      socket: Duplex,
      head: Buffer
    ) => void;
    const port = hmrPort || 9000;
    const host = (hmr && hmr.host) || undefined;

    if (this.wsServerOrHmrServer) {
      let hmrBase = this.publicPath;
      const hmrPath = hmr ? hmr.path : undefined;
      if (hmrPath) {
        hmrBase = path.posix.join(hmrBase, hmrPath as string);
      }
      wss = new WebSocketServerRaw({ noServer: true });
      hmrServerWsListener = (req, socket, head) => {
        if (
          req.headers['sec-websocket-protocol'] === HMR_HEADER &&
          req.url === hmrBase
        ) {
          wss.handleUpgrade(req, socket as Socket, head, (ws) => {
            wss.emit('connection', ws, req);
          });
        }
      };
      this.wsServerOrHmrServer.on('upgrade', hmrServerWsListener);
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
      if (this.httpsOptions) {
        wsHttpServer = createHttpsServer(this.httpsOptions, route);
      } else {
        wsHttpServer = createHttpServer(route);
      }
      // vite dev server in middleware mode
      // need to call ws listen manually
      wss = new WebSocketServerRaw({ server: wsHttpServer });
    }
  }
}
