import { describe, expect, it } from 'vitest';
import { resolveSsrRunOptions } from './ssr.js';

describe('farm cli ssr option resolver', () => {
  it('resolves build command with default config files', () => {
    const result = resolveSsrRunOptions({
      command: 'build',
      root: '/repo/app',
      options: {}
    });

    expect(result.command).toBe('build');
    expect(result.client.configFile).toBe('farm.config.client.ts');
    expect(result.server.configFile).toBe('farm.config.server.ts');
    expect(result.client.root).toBe('/repo/app');
    expect(result.server.root).toBe('/repo/app');
    expect(result.client.compilation?.input).toEqual({
      index: './index.html'
    });
    expect(result.server.compilation?.input).toEqual({
      index: '/src/entry-server.mjs'
    });
    expect(result.client.compilation?.output).toMatchObject({
      path: 'dist/client'
    });
    expect(result.server.compilation?.output).toMatchObject({
      path: 'dist/server',
      targetEnv: 'node',
      format: 'esm'
    });
  });

  it('uses convention fallback entry when dev/preview entry is omitted', () => {
    const devResult = resolveSsrRunOptions({
      command: 'dev',
      root: '/repo/app',
      options: {}
    });
    const previewResult = resolveSsrRunOptions({
      command: 'preview',
      root: '/repo/app',
      options: {}
    });

    expect(devResult.ssr?.entry).toBe('/src/entry-server.mjs');
    expect(previewResult.ssr?.entry).toBe('index.js');
  });

  it('maps dev options to run command payload', () => {
    const result = resolveSsrRunOptions({
      command: 'dev',
      root: '/repo/app',
      options: {
        entry: '/src/entry-server.ts',
        host: '127.0.0.1',
        port: '3011' as unknown as number,
        mode: 'development',
        placeholder: '<!--app-html-->'
      }
    });

    expect(result.command).toBe('dev');
    expect(result.mode).toBe('development');
    expect(result.host).toEqual({
      hostname: '127.0.0.1',
      port: 3011
    });
    expect(result.ssr).toEqual({
      entry: '/src/entry-server.ts',
      template: {
        resource: 'index.html',
        placeholder: '<!--app-html-->'
      }
    });
  });

  it('rejects template-resource for preview command', () => {
    expect(() =>
      resolveSsrRunOptions({
        command: 'preview',
        root: '/repo/app',
        options: {
          entry: 'index.js',
          templateResource: 'index.html'
        }
      })
    ).toThrow('--template-resource is only supported');
  });

  it('uses --config as shared fallback for client/server config', () => {
    const result = resolveSsrRunOptions({
      command: 'build',
      root: '/repo/app',
      options: {
        config: './farm.config.ts'
      }
    });

    expect(result.client.configFile).toBe('./farm.config.ts');
    expect(result.server.configFile).toBe('./farm.config.ts');
  });

  it('uses dedicated config over shared config', () => {
    const result = resolveSsrRunOptions({
      command: 'build',
      root: '/repo/app',
      options: {
        config: './farm.config.ts',
        clientConfig: './farm.config.client.ts',
        serverConfig: './farm.config.server.ts'
      }
    });

    expect(result.client.configFile).toBe('./farm.config.client.ts');
    expect(result.server.configFile).toBe('./farm.config.server.ts');
  });

  it('respects entry precedence: cli > env > convention', () => {
    const previousEnvEntry = process.env.FARM_SSR_ENTRY;
    const previousDevEnvEntry = process.env.FARM_SSR_DEV_ENTRY;
    process.env.FARM_SSR_ENTRY = '/from/env/shared.ts';
    process.env.FARM_SSR_DEV_ENTRY = '/from/env/dev.ts';

    try {
      const envResult = resolveSsrRunOptions({
        command: 'dev',
        root: '/repo/app',
        options: {}
      });
      const cliResult = resolveSsrRunOptions({
        command: 'dev',
        root: '/repo/app',
        options: {
          entry: '/from/cli.ts'
        }
      });

      expect(envResult.ssr?.entry).toBe('/from/env/dev.ts');
      expect(cliResult.ssr?.entry).toBe('/from/cli.ts');
    } finally {
      if (previousEnvEntry == null) {
        delete process.env.FARM_SSR_ENTRY;
      } else {
        process.env.FARM_SSR_ENTRY = previousEnvEntry;
      }

      if (previousDevEnvEntry == null) {
        delete process.env.FARM_SSR_DEV_ENTRY;
      } else {
        process.env.FARM_SSR_DEV_ENTRY = previousDevEnvEntry;
      }
    }
  });
});
