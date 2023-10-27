import { JsPlugin, UserConfig } from '@farmfe/core';
import sirv from 'sirv';

import { resolve } from 'node:path';
import http from 'node:http';

const path = require('path');

const PLUGIN_DIR =
  typeof __dirname !== 'undefined'
    ? __dirname
    : path.dirname(require.main?.filename);

export const PLUGIN_DIR_CLIENT = resolve(PLUGIN_DIR, '../build/client');

function StaticFilesHandler(
  req: http.IncomingMessage,
  res: http.ServerResponse
) {
  const staticFilesServer = sirv(PLUGIN_DIR_CLIENT, {
    etag: true,
    single: true
  });
  return new Promise<void>((resolve) => {
    staticFilesServer(req, res, () => {
      resolve();
    });
  });
}

export default function farmRecorderPlugin(): JsPlugin {
  let farmConfig: UserConfig['compilation'];

  return {
    name: 'farm-plugin-record-viewer',
    config: (config) => {
      farmConfig = config || {};
      farmConfig.record = true;
      return config;
    },
    configDevServer: (devServer) => {
      const server = http.createServer((req, res) => {
        if (req.url?.startsWith('/__record')) {
          const options = {
            hostname: devServer.config.hostname,
            port: devServer.config.port,
            path: req.url,
            method: req.method,
            headers: req.headers
          };
          const proxy = http.request(options, (proxyRes) => {
            res.writeHead(proxyRes.statusCode as number, proxyRes.headers);
            proxyRes.pipe(res);
          });
          proxy.on('error', (err) => {
            console.error(`Proxy Error: ${err.message}`);
            res.writeHead(500, { 'Content-Type': 'text/plain' });
            res.end('Proxy Error');
          });

          req.pipe(proxy, { end: true });
        } else {
          StaticFilesHandler(req, res);
        }
      });
      server.listen(9527, '127.0.0.1', () => {});
    },
    buildEnd: {
      executor: (param, ctx) => {
        // const server = http.createServer((req, res) => {
        //   StaticFilesHandler(req, res);
        // });
        // server.listen(9527, '127.0.0.1', () => {})
      }
    }
  };
}
