import { describe, expect, it, vi } from 'vitest';
import {
  runSsrCommandWithResolvers,
  type SsrRunCommandOptions
} from '../src/command.js';

describe('farm ssr command runner', () => {
  it('runs build command without starting server', async () => {
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrServer: vi.fn(async () => ({ type: 'create' }) as never),
      startSsrServer: vi.fn(async () => ({ type: 'start' }) as never)
    };

    const result = await runSsrCommandWithResolvers(
      {
        command: 'build',
        client: {},
        server: {}
      } as SsrRunCommandOptions,
      resolvers
    );

    expect(result).toBeUndefined();
    expect(resolvers.buildSsrApp).toHaveBeenCalledTimes(1);
    expect(resolvers.createSsrServer).toHaveBeenCalledTimes(0);
    expect(resolvers.startSsrServer).toHaveBeenCalledTimes(0);
  });

  it('starts server by default for dev/preview commands', async () => {
    const startedServer = { type: 'start' };
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrServer: vi.fn(async () => ({ type: 'create' }) as never),
      startSsrServer: vi.fn(async () => startedServer as never)
    };

    const devResult = await runSsrCommandWithResolvers(
      {
        client: {}
      } as SsrRunCommandOptions,
      resolvers
    );
    const previewResult = await runSsrCommandWithResolvers(
      {
        command: 'preview',
        client: {},
        server: {}
      } as SsrRunCommandOptions,
      resolvers
    );

    expect(devResult).toBe(startedServer);
    expect(previewResult).toBe(startedServer);
    expect(resolvers.startSsrServer).toHaveBeenCalledTimes(2);
    expect(resolvers.createSsrServer).toHaveBeenCalledTimes(0);
    expect(resolvers.startSsrServer).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({
        client: {}
      })
    );
    expect(resolvers.startSsrServer).toHaveBeenNthCalledWith(
      2,
      expect.objectContaining({
        command: 'preview',
        client: {},
        server: {}
      })
    );
  });

  it('uses create server flow when start=false', async () => {
    const createdServer = { type: 'create' };
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrServer: vi.fn(async () => createdServer as never),
      startSsrServer: vi.fn(async () => ({ type: 'start' }) as never)
    };

    const result = await runSsrCommandWithResolvers(
      {
        command: 'preview',
        start: false,
        client: {},
        server: {}
      } as SsrRunCommandOptions,
      resolvers
    );

    expect(result).toBe(createdServer);
    expect(resolvers.createSsrServer).toHaveBeenCalledTimes(1);
    expect(resolvers.startSsrServer).toHaveBeenCalledTimes(0);
    expect(resolvers.createSsrServer).toHaveBeenCalledWith({
      command: 'preview',
      client: {},
      server: {}
    });
  });
});
