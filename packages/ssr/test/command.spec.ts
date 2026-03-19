import { describe, expect, it, vi } from 'vitest';
import {
  runSsrCommandWithResolvers,
  type SsrRunCommandOptions
} from '../src/command.js';

describe('farm ssr command runner', () => {
  it('runs build command without starting server', async () => {
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrRuntime: vi.fn(async () => ({ start: vi.fn() }) as never)
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
    expect(resolvers.createSsrRuntime).toHaveBeenCalledTimes(0);
  });

  it('starts server by default for dev/preview commands', async () => {
    const runtime = {
      start: vi.fn(async () => undefined)
    };
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrRuntime: vi.fn(async () => runtime as never)
    };

    const devResult = await runSsrCommandWithResolvers(
      {
        client: {},
        $client: { mode: 'development' }
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

    expect(devResult).toBe(runtime);
    expect(previewResult).toBe(runtime);
    expect(resolvers.createSsrRuntime).toHaveBeenCalledTimes(2);
    expect(runtime.start).toHaveBeenCalledTimes(2);
    expect(resolvers.createSsrRuntime).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({
        client: {}
      })
    );
    expect(resolvers.createSsrRuntime).toHaveBeenNthCalledWith(
      2,
      expect.objectContaining({
        command: 'preview',
        client: {},
        server: {}
      })
    );
  });

  it('uses create server flow when start=false', async () => {
    const runtime = {
      start: vi.fn(async () => undefined)
    };
    const resolvers = {
      buildSsrApp: vi.fn(async () => undefined),
      createSsrRuntime: vi.fn(async () => runtime as never)
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

    expect(result).toBe(runtime);
    expect(resolvers.createSsrRuntime).toHaveBeenCalledTimes(1);
    expect(runtime.start).toHaveBeenCalledTimes(0);
    expect(resolvers.createSsrRuntime).toHaveBeenCalledWith({
      command: 'preview',
      client: {},
      server: {}
    });
  });
});
