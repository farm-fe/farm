import { STATUS_CODES, createServer as createHttpServer } from 'node:http';
import { createServer as createHttpsServer } from 'node:https';
import path from 'node:path';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';

import { ILogger, Logger } from '../utils/logger.js';
import { isObject } from '../utils/share.js';
import { HMRChannel } from './hmr.js';
import { ServerOptions, newServer } from './index.js';

import type { IncomingMessage, Server } from 'node:http';
import type { Socket } from 'node:net';
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRaw } from 'ws';
import type { WebSocket as WebSocketTypes } from '../types/ws.js';

import {
  CustomPayload,
  ErrorPayload,
  HMRPayload,
  InferCustomEventPayload
} from './type.js';

const WS_CONNECTED_MESSAGE = JSON.stringify({ type: 'connected' });
const WS_CUSTOM_EVENT_TYPE = 'custom';

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

// TODO return 出来的值 最后需要跟 ws 保持一致 不需要在包装一层 ws 📦
export class WsServer {
  public wss: WebSocketServerRaw_;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRaw, WebSocketClient>();
  public bufferedError: ErrorPayload | null = null;
  public logger: ILogger;
  public wsServer: any;
  wsHttpServer: Server | undefined;
  private serverConfig: ServerOptions;
  private port: number;
  private host: string | undefined;
  private hmrServerWsListener: (
    req: InstanceType<typeof IncomingMessage>,
    socket: Duplex,
    head: Buffer
  ) => void;
  constructor(private readonly app: any) {
    this.logger = app.logger ?? new Logger();
    this.serverConfig = app.resolvedUserConfig.server as ServerOptions;
    this.#createWebSocketServer();
  }

  get name(): string {
    return 'ws';
  }

  #createWebSocketServer() {
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

    const hmr = isObject(serverConfig.hmr) ? serverConfig.hmr : undefined;
    const hmrServer = hmr?.server;
    const hmrPort = hmr?.port;
    const portsAreCompatible = !hmrPort || hmrPort === serverConfig.port;
    this.wsServer = hmrServer || (portsAreCompatible && this.app.httpServer);

    this.port = (hmrPort as number) || 9000;
    this.host = ((hmr && hmr.host) as string) || undefined;

    if (this.wsServer) {
      let hmrBase = this.app.publicPath;

      const hmrPath = hmr?.path;
      if (hmrPath) {
        hmrBase = path.posix.join(hmrBase, hmrPath as string);
      }

      this.wss = new WebSocketServerRaw({ noServer: true });
      this.hmrServerWsListener = (req, socket, head) => {
        // TODO 这里需要处理一下 normalizePublicPath 的问题  hmrBase 路径匹配不到 req.url 的问题
        if (
          req.headers['sec-websocket-protocol'] === HMR_HEADER &&
          req.url === hmrBase
        ) {
          this.wss.handleUpgrade(req, socket as Socket, head, (ws) => {
            this.wss.emit('connection', ws, req);
          });
        }
      };
      this.wsServer.on('upgrade', this.hmrServerWsListener);
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
        this.wsHttpServer = createHttpsServer(this.app.httpsOptions, route);
      } else {
        this.wsHttpServer = createHttpServer(route);
      }
      // vite dev server in middleware mode
      // need to call ws listen manually
      this.wss = new WebSocketServerRaw({ server: this.wsHttpServer });
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
        if (!parsed || parsed.type !== WS_CUSTOM_EVENT_TYPE || !parsed.event)
          return;
        const listeners = this.customListeners.get(parsed.event);
        if (!listeners?.size) return;
        const client = this.#getSocketClient(socket);
        listeners.forEach((listener) => listener(parsed.data, client));
      });
      socket.on('error', (err) => {
        throw new Error(`WebSocket error: \n${err.stack}`);
      });

      socket.send(WS_CONNECTED_MESSAGE);

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
  }

  listen() {
    this.wsHttpServer?.listen(this.port, this.host);
  }

  on(event: string, fn: () => void) {
    if (wsServerEvents.includes(event)) {
      this.wss.on(event, fn);
    } else {
      if (!this.customListeners.has(event)) {
        this.customListeners.set(event, new Set());
      }
      this.customListeners.get(event).add(fn);
    }
  }

  off(event: string, fn: () => void) {
    if (wsServerEvents.includes(event)) {
      this.wss.off(event, fn);
    } else {
      this.customListeners.get(event)?.delete(fn);
    }
  }

  get clients() {
    return new Set(
      Array.from(this.wss.clients).map((socket: any) =>
        this.#getSocketClient(socket)
      )
    );
  }

  send(...args: any[]) {
    const payload: HMRPayload = this.#createPayload(...args);
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

  async close() {
    // should remove listener if hmr.server is set
    // otherwise the old listener swallows all WebSocket connections
    if (this.hmrServerWsListener && this.wsServer) {
      this.wsServer.off('upgrade', this.hmrServerWsListener);
    }
    try {
      this.wss.clients.forEach((client: any) => {
        client.terminate();
      });
      await new Promise<void>((resolve, reject) => {
        this.wss.close((err: any) => (err ? reject(err) : resolve()));
      });
      if (this.wsHttpServer) {
        await new Promise<void>((resolve, reject) => {
          this.wsHttpServer.close((err: any) =>
            err ? reject(err) : resolve()
          );
        });
      }
    } catch (err) {
      throw new Error(`Failed to close WebSocket server: ${err}`);
    }
  }

  #createPayload(...args: any[]): HMRPayload {
    if (typeof args[0] === 'string') {
      return {
        type: 'custom',
        event: args[0],
        data: args[1]
      };
    } else {
      return args[0];
    }
  }

  #getSocketClient(socket: WebSocketRaw) {
    if (!this.clientsMap.has(socket)) {
      this.clientsMap.set(socket, {
        send: (...args) => {
          const payload: HMRPayload = this.#createPayload(...args);
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
