import { JsPlugin, UserConfig } from '@farmfe/core';
import sirv from 'sirv';

import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import http from 'node:http';
import { RecordViewerOptions } from './types';

const path = require('path');

const PLUGIN_DIR = path.dirname(fileURLToPath(import.meta.url));

export const PLUGIN_DIR_CLIENT = resolve(PLUGIN_DIR, '../client');

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

export default function farmRecorderPlugin(
  options: RecordViewerOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let recordViewerOptions: RecordViewerOptions;

  return {
    name: 'farm-plugin-record-viewer',
    config: (config) => {
      farmConfig = config || {};
      farmConfig.record = true;
      recordViewerOptions = options;
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
            console.error(`Record Viewer Server Error: ${err.message}`);
            res.writeHead(500, { 'Content-Type': 'text/plain' });
            res.end('Proxy Error');
          });

          req.pipe(proxy, { end: true });
        } else {
          StaticFilesHandler(req, res);
        }
      });
      server.listen(
        recordViewerOptions.port || 9527,
        recordViewerOptions.host || '127.0.0.1',
        () => {}
      );
      console.log(
        `Farm Record Viewer run at http://${
          recordViewerOptions.host || '127.0.0.1'
        }:${recordViewerOptions.port || 9527}`
      );
    }
  };
}
