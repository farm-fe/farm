import path from 'path';
import { fileURLToPath } from 'url';
import { describe, expect, test } from 'vitest';

import { parseUserConfig } from '../src/config/schema.js';
import {
  DEFAULT_DEV_SERVER_OPTIONS,
  normalizeDevServerConfig,
  resolveConfig
} from '../src/index.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    {
      configFile: path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
      server: { hmr: false }
    },
    'dev',
    'development'
  );

  expect(config.compilation.define).toEqual({
    // FARM_HMR_HOST: true,
    // FARM_HMR_PATH: '/__hmr',
    // FARM_HMR_PORT: '9000',
    FARM_PROCESS_ENV: {
      NODE_ENV: 'development',
      mode: 'development',
      BASE_URL: '/',
      PROD: false,
      DEV: true
    },
    // FARM_HMR_PROTOCOL: 'ws',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
      '"development"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.mode': '"development"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.BASE_URL': '"/"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.PROD': 'false',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.DEV': 'true'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    clean: true,
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
  });
  expect(config.compilation.lazyCompilation).toEqual(true);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(false);
  expect(config.compilation.presetEnv).toEqual(false);
  expect(config.server).toEqual(normalizeDevServerConfig(config));
});

test('resolveUserConfig-prod', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    { configFile: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    'build',
    'production'
  );

  expect(config.compilation.define).toEqual({
    FARM_PROCESS_ENV: {
      NODE_ENV: 'production',
      mode: 'production',
      BASE_URL: '/',
      PROD: true,
      DEV: false
    },
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"production"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.mode': '"production"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.BASE_URL': '"/"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.PROD': 'true',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.DEV': 'false'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    assetsFilename: '[resourceName].[contentHash].[ext]',
    clean: true,
    filename: '[resourceName].[contentHash].[ext]',
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
  });
  expect(config.compilation.lazyCompilation).toEqual(false);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(true);
  expect(config.compilation.presetEnv).toEqual(false);
  expect(config.server).toEqual(normalizeDevServerConfig(config));
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
    { configFile: configFilePath },
    'build',
    'production'
  );

  expect(config.compilation.input).toEqual({
    index: './index.html'
  });

  expect(config.compilation.define).toEqual({
    FARM_PROCESS_ENV: {
      NODE_ENV: 'production',
      mode: 'production',
      BASE_URL: '/',
      PROD: true,
      DEV: false
    },
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"production"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.mode': '"production"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.BASE_URL': '"/"',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.PROD': 'true',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.DEV': 'false'
  });

  expect(config.compilation.output).toEqual({
    assetsFilename: '[resourceName].[contentHash].[ext]',
    clean: true,
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
        'chrome >= 5',
        'edge >= 12',
        'firefox >= 4',
        'ie >= 9',
        'ios >= 6',
        'opera >= 12.1',
        'safari >= 5'
      ]
    }
  });

  expect(config.server).toEqual(normalizeDevServerConfig(config));
});

describe('normalize-dev-server-options', () => {
  test('default', () => {
    const options = normalizeDevServerConfig({});
    expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
    expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
    expect(options.hmr).not.toBe(false);
  });

  test('custom port', () => {
    const options = normalizeDevServerConfig({ server: { port: 8080 } });
    expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
    expect(options.port).toBe(8080);
  });

  test('disable HMR in prod', () => {
    const options = normalizeDevServerConfig({
      mode: 'production'
    });
    expect(options.hmr).toBe(false);
  });
});

describe('parseUserConfig', () => {
  test('non-objects', () => {
    // @ts-expect-error should throw error here
    expect(() => parseUserConfig('should throw')).toThrowError(
      'Expected object, received string'
    );
  });

  test('extraneous config', () => {
    expect(() =>
      parseUserConfig({
        // @ts-expect-error should throw error here
        extra: 'should throw'
      })
    ).toThrowError('Unrecognized key');
  });

  test('type error config', () => {
    expect(() =>
      parseUserConfig({
        server: {
          // @ts-expect-error should throw error here
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
