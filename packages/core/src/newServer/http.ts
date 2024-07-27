/**
 * The following is modified based on source found in
 * https://github.com/vitejs/vite/blob/main/packages/vite/src/node/env.ts
 *
 * MIT License
 * Copyright (c) 2019-present, Yuxi (Evan)
 * https://github.com/vitejs/vite/blob/main/LICENSE
 *
 * Farm draws on the code of part of the vite server in order to better achieve the compatibility
 * progress of the vite ecosystem and the integrity of vite's ecological development,
 * which can reduce many unknown or known problems.
 */

import type { OutgoingHttpHeaders as HttpServerHeaders } from 'node:http';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import path from 'node:path';
import connect from 'connect';
import fse from 'fs-extra';
import { Logger } from '../utils/logger.js';
import { HttpServer } from './index.js';
import { ProxyOptions } from './middlewares/proxy.js';

export interface CommonServerOptions {
  port?: number;
  strictPort?: boolean;
  host?: string | boolean;
  https?: HttpsServerOptions;
  open?: boolean | string;
  proxy?: Record<string, string | ProxyOptions>;
  cors?: CorsOptions | boolean;
  headers?: HttpServerHeaders;
}

export type CorsOrigin = boolean | string | RegExp | (string | RegExp)[];

export interface CorsOptions {
  origin?:
    | CorsOrigin
    | ((origin: string, cb: (err: Error, origins: CorsOrigin) => void) => void);
  methods?: string | string[];
  allowedHeaders?: string | string[];
  exposedHeaders?: string | string[];
  credentials?: boolean;
  maxAge?: number;
  preflightContinue?: boolean;
  optionsSuccessStatus?: number;
}

// For the unencrypted tls protocol, we use http service.
// In other cases, https / http2 is used.
export async function resolveHttpServer(
  { proxy }: CommonServerOptions,
  app: connect.Server,
  httpsOptions?: HttpsServerOptions
): Promise<HttpServer> {
  if (!httpsOptions) {
    const { createServer } = await import('node:http');
    return createServer(app);
  }

  // EXISTING PROBLEM:
  // https://github.com/http-party/node-http-proxy/issues/1237

  // MAYBE SOLUTION:
  // https://github.com/nxtedition/node-http2-proxy
  // https://github.com/fastify/fastify-http-proxy
  if (proxy) {
    console.log('走的是哪里');

    const { createServer } = await import('node:https');
    return createServer(httpsOptions, app);
  } else {
    console.log('我现在用的就是 http2');

    const { createSecureServer } = await import('node:http2');
    return createSecureServer(
      {
        maxSessionMemory: 1000,
        ...httpsOptions,
        allowHTTP1: true
      },
      // @ts-ignore
      app
    );
  }
}

export async function resolveHttpsConfig(
  https: HttpsServerOptions | undefined
): Promise<HttpsServerOptions | undefined> {
  if (!https) return undefined;

  const [ca, cert, key, pfx] = await Promise.all([
    readFileIfExists(https.ca),
    readFileIfExists(https.cert),
    readFileIfExists(https.key),
    readFileIfExists(https.pfx)
  ]);
  return { ...https, ca, cert, key, pfx };
}

async function readFileIfExists(value?: string | Buffer | any[]) {
  if (typeof value === 'string') {
    return fse.readFile(path.resolve(value)).catch(() => value);
  }
  return value;
}

export async function httpServerStart(
  httpServer: HttpServer,
  serverOptions: {
    port: number;
    strictPort: boolean | undefined;
    host: string | undefined;
    logger: Logger;
  }
): Promise<number> {
  let { port, strictPort, host, logger } = serverOptions;

  return new Promise((resolve, reject) => {
    const onError = (e: Error & { code?: string }) => {
      if (e.code === 'EADDRINUSE') {
        if (strictPort) {
          httpServer.removeListener('error', onError);
          reject(new Error(`Port ${port} is already in use`));
        } else {
          logger.info(`Port ${port} is in use, trying another one...`);
          httpServer.listen(++port, host);
        }
      } else {
        httpServer.removeListener('error', onError);
        reject(e);
      }
    };

    httpServer.on('error', onError);

    httpServer.listen(port, host, () => {
      httpServer.removeListener('error', onError);
      resolve(port);
    });
  });
}
