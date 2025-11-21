import type { IncomingMessage, Server } from 'node:http';
import { createServer as createHttpServer, STATUS_CODES } from 'node:http';
import { createServer as createHttpsServer } from 'node:https';
import type { Socket } from 'node:net';
import path from 'node:path';
import type { Duplex } from 'node:stream';
import type { WebSocket as WebSocketRaw } from 'ws';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';
import { DEFAULT_HMR_OPTIONS } from '../config/constants.js';
import { ResolvedUserConfig } from '../config/types.js';
import type { WebSocket as WebSocketTypes } from '../types/ws.js';
import { resolveHostname, resolveServerUrls } from '../utils/http.js';
import { ILogger, Logger } from '../utils/logger.js';
import { isObject } from '../utils/share.js';
import type { Server as FarmDevServer, ServerConfig } from './index.js';
import {
  CustomPayload,
  ErrorPayload,
  HMRPayload,
  InferCustomEventPayload
} from './type.js';

const WS_CONNECTED_MESSAGE = JSON.stringify({ type: 'connected' });
const WS_CUSTOM_EVENT_TYPE = 'custom';

export interface WebSocketServer {
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

const HMR_HEADER = 'farm_hmr';

export type WebSocketCustomListener<T> = (
  data: T,
  client: WebSocketClient
) => void;

// const WebSocketServerRaw = process.versions.bun
//   ? // @ts-expect-error: Bun defines `import.meta.require`
//     import.meta.require('ws').WebSocketServer
//   : WebSocketServerRaw_;
const WebSocketServerRaw = WebSocketServerRaw_;

export class WsServer {
  public wss: WebSocketServerRaw_;
  public customListeners = new Map<string, Set<WebSocketCustomListener<any>>>();
  public clientsMap = new WeakMap<WebSocketRaw, WebSocketClient>();
  public bufferedError: ErrorPayload | null = null;
  public logger: ILogger;
  public httpServer: any;
  wsHttpServer: Server | undefined;
  private serverConfig: ServerConfig;
  private port: number;
  private host: string | undefined;
  private hmrServerWsListener: (
    req: InstanceType<typeof IncomingMessage>,
    socket: Duplex,
    head: Buffer
  ) => void;
  private hmrOrigins: string[];

  /**
   * Creates a new WebSocket server instance.
   */
  constructor(private readonly devServer: FarmDevServer) {
    this.logger = devServer.logger ?? new Logger();
    this.serverConfig = devServer.config.server;
  }

  /**
   * Gets the server name.
   * @returns {string} Returns "ws".
   */
  get name(): string {
    return 'ws';
  }

  private async generateHMROrigins(
    config: ResolvedUserConfig
  ): Promise<string[]> {
    const { protocol, hostname, port } = config.server ?? {};
    const origins = [];

    // Add localhost with configured port
    const urls = await resolveServerUrls(this.httpServer, config);
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

    if (config.server?.host !== config.server?.hmr?.host) {
      const hmrHostname = await resolveHostname(config.server?.hmr?.host);
      origins.push(
        `${config.server?.hmr?.protocol || protocol}://${hmrHostname.name}:${config.server.hmr?.port || config.server.port}`
      );
    }

    return origins;
  }

  /**
   * Creates the WebSocket server.
   */
  async createWebSocketServer() {
    const hmr = isObject(this.serverConfig.hmr)
      ? this.serverConfig.hmr
      : undefined;
    const hmrServer = hmr?.server;
    const hmrPort = hmr?.port;
    const portsAreCompatible = !hmrPort || hmrPort === this.serverConfig.port;
    this.httpServer =
      hmrServer || (portsAreCompatible && this.devServer.httpServer);

    this.port = (hmrPort as number) || DEFAULT_HMR_OPTIONS.port;
    this.host = ((hmr && hmr.host) as string) || undefined;

    if (this.httpServer) {
      let hmrBase = this.devServer.publicPath;

      const hmrPath = hmr?.path;

      if (hmrPath) {
        hmrBase = path.posix.join(hmrBase, hmrPath as string);
      }

      this.wss = new WebSocketServerRaw({ noServer: true });
      this.hmrServerWsListener = async (req, socket, head) => {
        if (!this.hmrOrigins) {
          this.hmrOrigins = await this.generateHMROrigins(
            this.devServer.config ?? {}
          );
        }

        const origin = req.headers['origin'];

        if (
          req.headers['sec-websocket-protocol'] === HMR_HEADER &&
          req.url === hmrBase &&
          (this.hmrOrigins.includes(origin) ||
            this.serverConfig.allowedHosts?.includes(origin))
        ) {
          this.wss.handleUpgrade(req, socket as Socket, head, (ws) => {
            this.wss.emit('connection', ws, req);
          });
        }
      };

      this.httpServer.on('upgrade', this.hmrServerWsListener);
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

      if (this.devServer.httpsOptions) {
        this.wsHttpServer = createHttpsServer(
          this.devServer.httpsOptions,
          route
        );
      } else {
        this.wsHttpServer = createHttpServer(route);
      }

      this.wsHttpServer.listen(this.port, this.host);

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
          this.devServer.hmrEngine.hmrUpdate(parsed.path, true);
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
        throw new Error(
          'WebSocket server error: Port is already in use. Please set a different port using the `server.hmr.port` option.'
        );
      } else {
        throw new Error(`WebSocket server error ${e.stack || e.message}`);
      }
    });
  }

  /**
   * Starts listening for WebSocket connections.
   */
  listen() {
    this.wsHttpServer?.listen(this.port, this.host);
  }

  /**
   * Adds a listener for the specified event.
   * @param {string} event - The name of the event.
   * @param {Function} fn - The listener function.
   */
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

  /**
   * Removes a listener for the specified event.
   * @param {string} event - The name of the event.
   * @param {Function} fn - The listener function to remove.
   */
  off(event: string, fn: () => void) {
    if (wsServerEvents.includes(event)) {
      this.wss.off(event, fn);
    } else {
      this.customListeners.get(event)?.delete(fn);
    }
  }

  /**
   * Gets all connected clients.
   * @returns {Set<WebSocketClient>} A set of connected clients.
   */
  get clients() {
    return new Set(
      Array.from(this.wss.clients).map((socket: any) =>
        this.#getSocketClient(socket)
      )
    );
  }

  /**
   * Sends a message to all connected clients.
   * @param {...any} args - The message arguments to send.
   */
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

  /**
   * Closes the WebSocket server.
   * @returns {Promise<void>} A promise that resolves when the server is closed.
   */
  async close() {
    // should remove listener if hmr.server is set
    // otherwise the old listener swallows all WebSocket connections
    if (this.hmrServerWsListener && this.httpServer) {
      this.httpServer.off('upgrade', this.hmrServerWsListener);
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

  /**
   * Creates an HMR payload.
   * @private
   * @param {...any} args - The payload arguments.
   * @returns {HMRPayload} The HMR payload object.
   */
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

  /**
   * Gets the client object associated with a WebSocket.
   * @private
   * @param {WebSocketRaw} socket - The raw WebSocket object.
   * @returns {WebSocketClient} The client object.
   */
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
