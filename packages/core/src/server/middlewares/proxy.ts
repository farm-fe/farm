import type * as http from 'node:http';
import type * as net from 'node:net';
import type Connect from 'connect';
import type Server from 'http-proxy';
import httpProxy from 'http-proxy';
import { ResolvedUserConfig } from '../../config/types.js';
import { CommonServerOptions } from '../http.js';
import type { Server as DevServer, HttpServer } from '../index.js';
import { PreviewServer } from '../preview.js';

export interface ProxyOptions extends httpProxy.ServerOptions {
  rewrite?: (path: string) => string;
  configure?: (proxy: httpProxy, options: ProxyOptions) => void;
  bypass?: (
    req: http.IncomingMessage,
    res: http.ServerResponse,
    options: ProxyOptions
  ) => void | null | undefined | false | string;
  rewriteWsOrigin?: boolean | undefined;
}

export type ManagedProxyMiddleware = Connect.NextHandleFunction & {
  upgrade(req: http.IncomingMessage, socket: net.Socket, head: Buffer): void;
  close(): void;
  retire(onDrained: () => void): void;
};

export function proxyMiddleware(
  app: Pick<DevServer | PreviewServer, 'config'>,
  middlewareServer: HttpServer,
  config: NonNullable<CommonServerOptions['proxy']>,
  attachUpgrade = true
): ManagedProxyMiddleware {
  const { config: resolvedUserConfig } = app;
  let closed = false;
  let disposed = false;
  let proxiesDisposed = false;
  let retirementFailed = false;
  let onDrained: (() => void) | undefined;
  const requests = new Set<http.ClientRequest>();
  const wsRequests = new Set<http.ClientRequest>();
  const responses = new Set<http.ServerResponse>();
  const notifyDrained = () => {
    if (requests.size || responses.size || retirementFailed || !onDrained)
      return;
    const notify = onDrained;
    onDrained = undefined;
    notify();
  };
  const sockets = new Set<net.Socket>();
  const ownRequest = (request: http.ClientRequest) => {
    requests.add(request);
    request.once('close', () => {
      requests.delete(request);
      notifyDrained();
    });
    if (disposed) request.destroy();
  };
  const ownSocket = (socket: net.Socket) => {
    sockets.add(socket);
    socket.once('close', () => sockets.delete(socket));
    if (closed) socket.destroy();
  };
  const proxies: Record<string, [Server, ProxyOptions]> = {};
  const created: Server[] = [];
  const dispose = (forceHttp = true) => {
    const errors: unknown[] = [];
    const attempt = (operation: () => unknown) => {
      try {
        operation();
      } catch (error) {
        errors.push(error);
      }
    };
    for (const request of requests)
      if (forceHttp || wsRequests.has(request))
        attempt(() => request.destroy());
    if (forceHttp)
      for (const response of responses) attempt(() => response.destroy());
    for (const socket of sockets) attempt(() => socket.destroy());
    if (!proxiesDisposed) {
      proxiesDisposed = true;
      for (const proxy of created) attempt(() => proxy.close());
    }
    return errors;
  };
  try {
    Object.keys(config).forEach((context) => {
      if (context.startsWith('^')) new RegExp(context);
      let opts = config[context];
      if (!opts) {
        return;
      }
      if (typeof opts === 'string') {
        opts = { target: opts, changeOrigin: true } as ProxyOptions;
      }
      const proxy = httpProxy.createProxyServer(opts) as Server;
      created.push(proxy);
      proxy.on('proxyReq', ownRequest);
      proxy.on('open', ownSocket);
      proxy.on('proxyReqWs', (request, _incoming, socket) => {
        ownRequest(request);
        wsRequests.add(request);
        if (closed) request.destroy();
        request.once('close', () => wsRequests.delete(request));
        ownSocket(socket);
      });

      proxy.on('error', (err, _req, originalRes) => {
        // When it is ws proxy, res is net.Socket
        // originalRes can be falsy if the proxy itself errored
        const res = originalRes as http.ServerResponse | net.Socket | undefined;
        if (closed) {
          res?.destroy();
          return;
        }
        if (!res) {
          resolvedUserConfig.logger?.error(
            `http proxy error: ${err.message} \n ${err.stack}`
          );
        } else if ('req' in res) {
          resolvedUserConfig.logger?.error(
            // @ts-ignore
            `http proxy error: ${originalRes.req.url}\n${err.stack}`
          );

          if (!res.headersSent && !res.writableEnded) {
            res
              .writeHead(500, {
                'Content-Type': 'text/plain'
              })
              .end();
          }
        } else {
          resolvedUserConfig.logger?.error(`ws proxy error:\n ${err.stack}`);
          res.end();
        }
      });

      if (opts.configure) opts.configure(proxy, opts);

      proxy.on('proxyReqWs', (proxyReq, _req, socket, options, _head) => {
        rewriteOriginHeader(proxyReq, options, resolvedUserConfig);

        socket.on('error', (err) => {
          resolvedUserConfig.logger?.error(
            `ws proxy socket error: \n ${err.stack}`
          );
        });
      });

      // https://github.com/http-party/node-http-proxy/issues/1520#issue-877626125
      // https://github.com/chimurai/http-proxy-middleware/blob/cd58f962aec22c925b7df5140502978da8f87d5f/src/plugins/default/debug-proxy-errors-plugin.ts#L25-L37
      proxy.on('proxyRes', (proxyRes, _req, res) => {
        res.on('close', () => {
          if (!res.writableEnded) {
            proxyRes.destroy();
          }
        });
      });

      // clone before saving because http-proxy mutates the options
      proxies[context] = [proxy, { ...opts }];
    });
  } catch (error) {
    closed = true;
    const errors = dispose();
    if (errors.length)
      throw new AggregateError(
        [error, ...errors],
        'proxy preparation and cleanup failed'
      );
    throw error;
  }

  const upgrade = (
    req: http.IncomingMessage,
    socket: net.Socket,
    head: Buffer
  ) => {
    if (closed) return;
    const url = req.url ?? '';
    for (const context in proxies) {
      if (doesProxyContextMatchUrl(context, url)) {
        const [proxy, opts] = proxies[context];
        if (
          opts.ws ||
          opts.target?.toString().startsWith('ws:') ||
          opts.target?.toString().startsWith('wss:')
        ) {
          if (opts.bypass) {
            const bypassResult = opts.bypass(req, undefined as any, opts);
            if (typeof bypassResult === 'string') {
              req.url = bypassResult;
              return;
            } else if (bypassResult === false) {
              socket.end('HTTP/1.1 404 Not Found\r\n\r\n');
              return;
            }
          }
          if (opts.rewrite) {
            req.url = opts.rewrite(url);
          }
          proxy.ws(req, socket, head);
          return;
        }
      }
    }
  };
  if (attachUpgrade && middlewareServer)
    middlewareServer.on('upgrade', upgrade);
  const middleware: Connect.NextHandleFunction = function handleProxyMiddleware(
    req,
    res,
    next
  ) {
    if (closed) return next();
    const url = req.url ?? '';
    for (const context in proxies) {
      if (doesProxyContextMatchUrl(context, url)) {
        const [proxy, opts] = proxies[context];

        if (opts.bypass) {
          const bypassResult = opts.bypass(req, res, opts);
          if (typeof bypassResult === 'string') {
            req.url = bypassResult;
            return next();
          } else if (bypassResult === false) {
            res.statusCode = 404;
            return res.end();
          }
        }

        if (opts.rewrite) {
          req.url = opts.rewrite(req.url ?? '');
        }

        responses.add(res);
        const completed = () => {
          responses.delete(res);
          notifyDrained();
        };
        res.once('finish', completed);
        res.once('close', completed);
        proxy.web(req, res, opts);
        return;
      }
    }
    next();
  };
  return Object.assign(middleware, {
    upgrade,
    close() {
      if (disposed) return;
      disposed = true;
      closed = true;
      if (attachUpgrade && middlewareServer)
        middlewareServer.off('upgrade', upgrade);
      const errors = dispose();
      if (errors.length)
        throw new AggregateError(errors, 'proxy cleanup failed');
    },
    retire(drained: () => void) {
      if (closed) return;
      closed = true;
      onDrained = drained;
      if (attachUpgrade && middlewareServer)
        middlewareServer.off('upgrade', upgrade);
      const errors = dispose(false);
      retirementFailed = errors.length > 0;
      notifyDrained();
      if (errors.length)
        throw new AggregateError(errors, 'proxy retirement failed');
    }
  });
}

function rewriteOriginHeader(
  proxyReq: http.ClientRequest,
  options: ProxyOptions,
  config: ResolvedUserConfig
) {
  // Browsers may send Origin headers even with same-origin
  // requests. It is common for WebSocket servers to check the Origin
  // header, so if rewriteWsOrigin is true we change the Origin to match
  // the target URL.
  if (!options.rewriteWsOrigin) return;

  const { target } = options;

  if (proxyReq.headersSent) {
    config.logger?.warn(
      'Unable to rewrite Origin header as headers are already sent.'
    );
    return;
  }

  if (proxyReq.getHeader('origin') && target) {
    const changedOrigin =
      typeof target === 'object'
        ? `${target.protocol}//${target.host}`
        : target;

    proxyReq.setHeader('origin', changedOrigin);
  }
}

function doesProxyContextMatchUrl(context: string, url: string): boolean {
  return (
    (context[0] === '^' && new RegExp(context).test(url)) ||
    url.startsWith(context)
  );
}
