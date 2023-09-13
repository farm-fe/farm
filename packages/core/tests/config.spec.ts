import { builtinModules } from 'module';
import path from 'path';
import { fileURLToPath } from 'url';
import { test, expect, describe } from 'vitest';

import {
  DEFAULT_DEV_SERVER_OPTIONS,
  normalizeDevServerOptions,
  resolveUserConfig
} from '../src/index.js';
import { parseUserConfig } from '../src/config/schema.js';
import { DefaultLogger } from '../src/utils/logger.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveUserConfig(
    { configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    new DefaultLogger()
  );

  expect(config).toEqual({
    compilation: {
      input: {
        main: './main.tsx'
      },
      external: builtinModules
    },
    root: path.join(filePath, 'fixtures', 'config'),
    server: normalizeDevServerOptions(config.server, 'development')
  });
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
