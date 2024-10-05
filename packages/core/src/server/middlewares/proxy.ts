import httpProxy from 'http-proxy';
import { ResolvedUserConfig } from '../../config/types.js';
import { colors } from '../../utils/color.js';

import type * as http from 'node:http';
import type * as net from 'node:net';
import type Server from 'http-proxy';

import type Connect from 'connect';
import type { Server as DevServer, HttpServer } from '../index.js';
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

export function proxyMiddleware(
  app: DevServer,
  middlewareServer: HttpServer | unknown
): Connect.NextHandleFunction {
  const { serverOptions, resolvedUserConfig } = app;

  const proxies: Record<string, [Server, ProxyOptions]> = {};
  Object.keys(serverOptions.proxy).forEach((context) => {
    let opts = serverOptions.proxy[context];
    if (!opts) {
      return;
    }
    if (typeof opts === 'string') {
      opts = { target: opts, changeOrigin: true } as ProxyOptions;
    }
    const proxy = httpProxy.createProxyServer(opts) as Server;

    if (opts.configure) {
      opts.configure(proxy, opts);
    }

    proxy.on('error', (err, _req, originalRes) => {
      // When it is ws proxy, res is net.Socket
      // originalRes can be falsy if the proxy itself errored
      const res = originalRes as http.ServerResponse | net.Socket | undefined;
      if (!res) {
        console.log(
          `${colors.red(`http proxy error: ${err.message}`)}\n${err.stack}`
        );
      } else if ('req' in res) {
        // console.log(
        //   `${colors.red(`http proxy error: ${originalRes.req.url}`)}\n${
        //     err.stack
        //   }`,
        // );

        if (!res.headersSent && !res.writableEnded) {
          res
            .writeHead(500, {
              'Content-Type': 'text/plain'
            })
            .end();
        }
      } else {
        console.log(`${colors.red(`ws proxy error:`)}\n${err.stack}`);
        res.end();
      }
    });

    proxy.on('proxyReqWs', (proxyReq, _req, socket, options, _head) => {
      rewriteOriginHeader(proxyReq, options, resolvedUserConfig);

      socket.on('error', (err) => {
        console.log(`${colors.red(`ws proxy socket error:`)}\n${err.stack}`);
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

  if (app.httpServer) {
    app.httpServer.on('upgrade', (req: any, socket: any, head: any) => {
      const url = req.url;
      for (const context in proxies) {
        if (doesProxyContextMatchUrl(context, url)) {
          const [proxy, opts] = proxies[context];
          if (
            opts.ws ||
            opts.target?.toString().startsWith('ws:') ||
            opts.target?.toString().startsWith('wss:')
          ) {
            if (opts.rewrite) {
              req.url = opts.rewrite(url);
            }
            proxy.ws(req, socket, head);
            return;
          }
        }
      }
    });
  }
  return function handleProxyMiddleware(req, res, next) {
    const url = req.url;
    for (const context in proxies) {
      if (doesProxyContextMatchUrl(context, url)) {
        const [proxy, opts] = proxies[context];
        const options: httpProxy.ServerOptions = {};

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
          req.url = opts.rewrite(req.url!);
        }
        proxy.web(req, res, options);
        return;
      }
    }
    next();
  };
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
  if (options.rewriteWsOrigin) {
    const { target } = options;

    if (proxyReq.headersSent) {
      console.warn(
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
}

function doesProxyContextMatchUrl(context: string, url: string): boolean {
  return (
    (context[0] === '^' && new RegExp(context).test(url)) ||
    url.startsWith(context)
  );
}
