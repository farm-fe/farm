import { createServer } from 'node:http';
import type { AddressInfo, Socket } from 'node:net';
import { describe, expect, it } from 'vitest';
import { WebSocket } from 'ws';
import { WsServer } from '../src/server/ws.js';

describe('HMR origin handshake', () => {
  for (const [host, urlHost, publicPath] of [
    ['127.0.0.1', '127.0.0.1', '/'],
    ['127.0.0.1', '127.0.0.1', '/nested/'],
    ['::1', '[::1]', '/']
  ]) {
    it(`accepts the bound ${host} origin with publicPath ${publicPath}`, async () => {
      const http = createServer();
      const sockets = new Set<Socket>();
      http.on('connection', (socket) => {
        sockets.add(socket);
        socket.on('close', () => sockets.delete(socket));
      });
      await new Promise<void>((resolve) => http.listen(0, host, resolve));
      const port = (http.address() as AddressInfo).port;
      const server = {
        httpServer: http,
        publicPath,
        config: {
          compilation: { output: { publicPath } },
          server: {
            host,
            hostname: { name: host },
            port,
            protocol: 'http',
            allowedHosts: [],
            hmr: { host, port, path: '/__hmr' }
          }
        }
      };
      const ws = new WsServer(server as any);
      let client: WebSocket | undefined;
      let foreign: WebSocket | undefined;
      try {
        await ws.createWebSocketServer();
        client = new WebSocket(
          `ws://${urlHost}:${port}${publicPath}__hmr`,
          'farm_hmr',
          {
            origin: `http://${urlHost}:${port}`,
            handshakeTimeout: 700
          }
        );
        const connectedClient = client;
        const connected = await new Promise<string>((resolve, reject) => {
          connectedClient.once('message', (data) => resolve(data.toString()));
          connectedClient.once('error', reject);
        });
        expect(JSON.parse(connected)).toEqual({ type: 'connected' });
        foreign = new WebSocket(
          `ws://${urlHost}:${port}${publicPath}__hmr`,
          'farm_hmr',
          {
            origin: 'http://untrusted.invalid',
            handshakeTimeout: 200
          }
        );
        const foreignClient = foreign;
        await expect(
          new Promise((resolve, reject) => {
            foreignClient.once('open', resolve);
            foreignClient.once('error', reject);
          })
        ).rejects.toThrow('Opening handshake has timed out');
      } finally {
        client?.terminate();
        foreign?.terminate();
        for (const socket of sockets) socket.destroy();
        await new Promise<void>((resolve) => ws.wss.close(() => resolve()));
        await new Promise<void>((resolve) => http.close(() => resolve()));
      }
    });
  }
});
