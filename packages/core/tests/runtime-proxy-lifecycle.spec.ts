import { EventEmitter } from 'node:events';
import { beforeEach, expect, it, vi } from 'vitest';

const state = vi.hoisted(() => ({ create: vi.fn() }));
vi.mock('http-proxy', () => ({ default: { createProxyServer: state.create } }));

import { proxyMiddleware } from '../src/server/middlewares/proxy.js';

beforeEach(() => state.create.mockReset());
const proxy = () =>
  Object.assign(new EventEmitter(), {
    close: vi.fn(),
    web: vi.fn(),
    ws: vi.fn()
  });

it('retains a retired route when its response finishes before the upstream request closes', () => {
  const instance = proxy(),
    server = new EventEmitter();
  const request = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  const response = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  state.create.mockReturnValue(instance);
  const middleware = proxyMiddleware({ config: {} }, server as never, {
    '/upload': { target: 'http://localhost:3001' }
  });
  middleware({ url: '/upload' } as never, response as never, vi.fn());
  instance.emit('proxyReq', request);
  const drained = vi.fn();
  middleware.retire(drained);
  response.emit('finish');
  expect(drained).not.toHaveBeenCalled();
  expect(request.destroy).not.toHaveBeenCalled();
  request.emit('close');
  expect(drained).toHaveBeenCalledOnce();
});

it('cleans every partially constructed proxy and owned request while preserving configuration and cleanup errors', () => {
  const first = proxy(),
    second = proxy();
  const cleanupError = new Error('first cleanup failed');
  const configureError = new Error('candidate configure failed');
  first.close.mockImplementation(() => {
    throw cleanupError;
  });
  state.create.mockReturnValueOnce(first).mockReturnValueOnce(second);
  const request = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  const server = new EventEmitter();
  let failure: unknown;
  try {
    proxyMiddleware({ config: {} }, server as never, {
      '/one': { target: 'http://localhost:3001' },
      '/two': {
        target: 'http://localhost:3002',
        configure(instance) {
          instance.emit('proxyReq', request);
          throw configureError;
        }
      }
    });
  } catch (error) {
    failure = error;
  }
  expect(first.close).toHaveBeenCalledOnce();
  expect(second.close).toHaveBeenCalledOnce();
  expect(request.destroy).toHaveBeenCalledOnce();
  expect(failure).toBeInstanceOf(AggregateError);
  expect((failure as AggregateError).errors).toEqual([
    configureError,
    cleanupError
  ]);
  expect(server.listenerCount('upgrade')).toBe(0);
});

it('validates a candidate regexp before attaching upgrade handlers and releases prior allocations', () => {
  const first = proxy(),
    server = new EventEmitter();
  state.create.mockReturnValue(first);
  expect(() =>
    proxyMiddleware({ config: {} }, server as never, {
      '/one': { target: 'http://localhost:3001' },
      '^[': { target: 'http://localhost:3002' }
    })
  ).toThrow(SyntaxError);
  expect(first.close).toHaveBeenCalledOnce();
  expect(server.listenerCount('upgrade')).toBe(0);
});

it('disposes a route once and removes its attached upgrade handler and active sockets', () => {
  const instance = proxy(),
    server = new EventEmitter();
  const client = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  const upstream = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  const request = Object.assign(new EventEmitter(), { destroy: vi.fn() });
  state.create.mockReturnValue(instance);
  const middleware = proxyMiddleware({ config: {} }, server as never, {
    '/one': { target: 'http://localhost:3001' }
  });
  instance.emit('proxyReqWs', request, {}, client, {}, Buffer.alloc(0));
  instance.emit('open', upstream);
  expect(server.listenerCount('upgrade')).toBe(1);
  middleware.close();
  middleware.close();
  expect(server.listenerCount('upgrade')).toBe(0);
  expect(request.destroy).toHaveBeenCalledOnce();
  expect(client.destroy).toHaveBeenCalledOnce();
  expect(upstream.destroy).toHaveBeenCalledOnce();
  expect(instance.close).toHaveBeenCalledOnce();
});
