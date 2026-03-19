import { describe, expect, it, vi } from 'vitest';
import {
  createSsrServerWithResolvers,
  type SsrServerOptions,
  startSsrServerWithResolvers
} from '../src/server.js';

describe('farm ssr unified server api', () => {
  it('dispatches to dev by default', async () => {
    const devServer = { type: 'dev' };
    const previewServer = { type: 'preview' };
    const resolvers = {
      createDevServer: vi.fn(async () => devServer as never),
      createPreviewServer: vi.fn(async () => previewServer as never),
      startDevServer: vi.fn(async () => devServer as never),
      startPreviewServer: vi.fn(async () => previewServer as never)
    };

    const result = await createSsrServerWithResolvers(
      { client: {} } as SsrServerOptions,
      resolvers
    );

    expect(result).toBe(devServer);
    expect(resolvers.createDevServer).toHaveBeenCalledTimes(1);
    expect(resolvers.createPreviewServer).toHaveBeenCalledTimes(0);
  });

  it('dispatches to preview when command=preview', async () => {
    const devServer = { type: 'dev' };
    const previewServer = { type: 'preview' };
    const resolvers = {
      createDevServer: vi.fn(async () => devServer as never),
      createPreviewServer: vi.fn(async () => previewServer as never),
      startDevServer: vi.fn(async () => devServer as never),
      startPreviewServer: vi.fn(async () => previewServer as never)
    };

    const result = await createSsrServerWithResolvers(
      {
        command: 'preview',
        client: {},
        server: {}
      } as SsrServerOptions,
      resolvers
    );

    expect(result).toBe(previewServer);
    expect(resolvers.createDevServer).toHaveBeenCalledTimes(0);
    expect(resolvers.createPreviewServer).toHaveBeenCalledTimes(1);
  });

  it('start api follows the same command dispatching', async () => {
    const devServer = { type: 'dev' };
    const previewServer = { type: 'preview' };
    const resolvers = {
      createDevServer: vi.fn(async () => devServer as never),
      createPreviewServer: vi.fn(async () => previewServer as never),
      startDevServer: vi.fn(async () => devServer as never),
      startPreviewServer: vi.fn(async () => previewServer as never)
    };

    const previewResult = await startSsrServerWithResolvers(
      {
        command: 'preview',
        client: {},
        server: {}
      } as SsrServerOptions,
      resolvers
    );
    const devResult = await startSsrServerWithResolvers(
      { client: {} } as SsrServerOptions,
      resolvers
    );

    expect(previewResult).toBe(previewServer);
    expect(devResult).toBe(devServer);
    expect(resolvers.startPreviewServer).toHaveBeenCalledTimes(1);
    expect(resolvers.startDevServer).toHaveBeenCalledTimes(1);
  });

  it('mode does not change dispatch semantics', async () => {
    const devServer = { type: 'dev' };
    const previewServer = { type: 'preview' };
    const resolvers = {
      createDevServer: vi.fn(async () => devServer as never),
      createPreviewServer: vi.fn(async () => previewServer as never),
      startDevServer: vi.fn(async () => devServer as never),
      startPreviewServer: vi.fn(async () => previewServer as never)
    };

    const result = await createSsrServerWithResolvers(
      {
        mode: 'production',
        client: {}
      } as SsrServerOptions,
      resolvers
    );

    expect(result).toBe(devServer);
    expect(resolvers.createDevServer).toHaveBeenCalledTimes(1);
    expect(resolvers.createPreviewServer).toHaveBeenCalledTimes(0);
  });
});
