import path from 'path';
import { fileURLToPath } from 'url';
import { describe, expect, test } from 'vitest';

import { parseUserConfig } from '../src/config/schema.js';
import {
  DEFAULT_DEV_SERVER_OPTIONS,
  normalizeDevServerOptions,
  resolveConfig
} from '../src/index.js';
import { Logger } from '../src/utils/logger.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    { configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    new Logger(),
    'development'
  );

  expect(config.compilation.define).toEqual({
    // FARM_HMR_HOST: true,
    // FARM_HMR_PATH: '/__hmr',
    // FARM_HMR_PORT: '9000',
    FARM_PROCESS_ENV: {
      NODE_ENV: 'development'
    },
    // FARM_HMR_PROTOCOL: 'ws',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"development"'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
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
      FARM_PROCESS_ENV: '{"NODE_ENV":"development"}',
      NODE_ENV: 'development',
      'package.json[name]': 'farm-fe',
      'package.json[type]': 'unknown',
      '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
        '"development"',
      'package.json[browser]': 'unknown',
      'package.json[exports]': 'unknown',
      'package.json[main]': 'unknown',
      'package.json[module]': 'unknown'
      // FARM_HMR_HOST: 'true',
      // FARM_HMR_PATH: '/__hmr',
      // FARM_HMR_PORT: '9000',
      // FARM_HMR_PROTOCOL: 'ws'
    },
    moduleCacheKeyStrategy: {}
  });
  expect(config.server).toEqual(
    normalizeDevServerOptions(config.server, 'development')
  );
});

test('resolveUserConfig-prod', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    { configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    new Logger(),
    'production'
  );

  expect(config.compilation.define).toEqual({
    FARM_PROCESS_ENV: {
      NODE_ENV: 'production'
    },
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"production"'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    assetsFilename: '[resourceName].[contentHash].[ext]',
    filename: '[resourceName].[contentHash].[ext]',
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
  });
  expect(config.compilation.lazyCompilation).toEqual(false);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(true);
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
      FARM_PROCESS_ENV: '{"NODE_ENV":"production"}',
      NODE_ENV: 'production',
      'package.json[name]': 'farm-fe',
      'package.json[type]': 'unknown',
      'package.json[browser]': 'unknown',
      'package.json[exports]': 'unknown',
      'package.json[main]': 'unknown',
      'package.json[module]': 'unknown',
      '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
        '"production"'
    },
    moduleCacheKeyStrategy: {}
  });
  expect(config.server).toEqual(
    normalizeDevServerOptions(config.server, 'production')
  );
});

test('resolveUserConfig-input-html-prod', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));
  const configFilePath = path.join(
    filePath,
    'fixtures',
    'config',
    'input-html',
    'farm.config.ts'
  );
  const config = await resolveConfig(
    { configPath: configFilePath },
    new Logger(),
    'production'
  );

  expect(config.compilation.input).toEqual({
    index: './index.html'
  });

  expect(config.compilation.define).toEqual({
    FARM_PROCESS_ENV: {
      NODE_ENV: 'production'
    },
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"production"'
  });

  expect(config.compilation.output).toEqual({
    assetsFilename: '[resourceName].[contentHash].[ext]',
    filename: '[resourceName].[contentHash].[ext]',
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
  });

  expect(config.compilation.lazyCompilation).toEqual(false);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(true);

  expect(config.compilation.presetEnv).toEqual({
    options: {
      targets: [
        'edge >= 15',
        'firefox >= 52',
        'chrome >= 55',
        'safari >= 11',
        'opera >= 42',
        'ios_saf >= 11.2',
        'and_chr >= 119',
        'and_ff >= 119',
        'and_uc >= 15.5',
        'samsung >= 6.4',
        'and_qq >= 13.1',
        'baidu >= 13.18',
        'kaios >= 3.1',
        'unreleased edge versions',
        'unreleased firefox versions',
        'unreleased chrome versions',
        'unreleased safari versions',
        'unreleased opera versions',
        'unreleased ios_saf versions',
        'unreleased and_chr versions',
        'unreleased and_ff versions',
        'unreleased and_uc versions',
        'unreleased samsung versions',
        'unreleased and_qq versions',
        'unreleased baidu versions',
        'unreleased kaios versions'
      ]
    }
  });

  expect(config.compilation.persistentCache).toEqual({
    buildDependencies: [
      configFilePath,
      'module',
      'package-lock.json',
      'pnpm-lock.yaml',
      'yarn.lock'
    ],
    envs: {
      FARM_PROCESS_ENV: '{"NODE_ENV":"production"}',
      NODE_ENV: 'production',
      'package.json[name]': 'farm-fe',
      'package.json[type]': 'unknown',
      'package.json[browser]': 'unknown',
      'package.json[exports]': 'unknown',
      'package.json[main]': 'unknown',
      'package.json[module]': 'unknown',
      '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
        '"production"'
    },
    moduleCacheKeyStrategy: {}
  });

  expect(config.server).toEqual(
    normalizeDevServerOptions(config.server, 'production')
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
