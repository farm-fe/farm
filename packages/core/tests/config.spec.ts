import { builtinModules } from 'module';
import path from 'path';
import { fileURLToPath } from 'url';
import { describe, expect, test } from 'vitest';

import {
  DEFAULT_DEV_SERVER_OPTIONS,
  normalizeDevServerOptions,
  resolveConfig
} from '../src/index.js';
import { parseUserConfig } from '../src/config/schema.js';
import { DefaultLogger } from '../src/utils/logger.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    { configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    new DefaultLogger(),
    'development'
  );

  expect(config.compilation.define).toEqual({
    FARM_HMR_HOST: true,
    FARM_HMR_PATH: '/__hmr',
    FARM_HMR_PORT: '9000',
    FARM_PROCESS_ENV: {
      NODE_ENV: 'test'
    },
    'process.env.NODE_ENV': 'test'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    path: './dist',
    publicPath: '/'
  });
  expect(config.compilation.lazyCompilation).toEqual(true);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(false);
  expect(config.compilation.presetEnv).toEqual(false);
  expect(config.compilation.persistentCache).toEqual({
    buildDependencies: [
      // path.join(filePath, '..', 'src', 'config.ts'),
      path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
      path.join(filePath, 'fixtures', 'config', 'util.ts'),
      'module',
      'package-lock.json',
      'pnpm-lock.yaml',
      'yarn.lock'
    ],
    envs: {
      NODE_ENV: 'test'
    },
    moduleCacheKeyStrategy: {}
  });
  expect(config.server).toEqual(
    normalizeDevServerOptions(config.server, 'development')
  );
});

describe('normalize-dev-server-options', () => {
  test('default', () => {
    const options = normalizeDevServerOptions({}, 'development');
    expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
    expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
    expect(options.hmr).not.toBe(false);
  });

  test('custom port', () => {
    const options = normalizeDevServerOptions({ port: 8080 }, 'development');
    expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
    expect(options.port).toBe(8080);
  });

  test('disable HMR in prod', () => {
    const options = normalizeDevServerOptions({}, 'production');
    expect(options.hmr).toBe(false);
  });
});

describe('parseUserConfig', () => {
  test('non-objects', () => {
    expect(() => parseUserConfig('should throw')).toThrowError(
      'Expected object, received string'
    );
  });

  test('extraneous config', () => {
    expect(() =>
      parseUserConfig({
        extra: 'should throw'
      })
    ).toThrowError('Unrecognized key');
  });

  test('type error config', () => {
    expect(() =>
      parseUserConfig({
        server: {
          port: 'should throw'
        }
      })
    ).toThrowError('Validation error');
  });

  test('valid template config', () => {
    expect(() =>
      parseUserConfig({
        compilation: {
          input: {
            index: './index.html'
          },
          resolve: {
            symlinks: true,
            mainFields: ['module', 'main', 'customMain']
          },
          define: {
            BTN: 'Click me'
          },
          output: {
            path: './build'
          }
        },
        server: {
          hmr: true
        },
        plugins: ['@farmfe/plugin-react']
      })
    ).not.toThrow();
  });
});
