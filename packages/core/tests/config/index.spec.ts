import path from 'path';
import { describe, expect, test } from 'vitest';
import {
  ResolvedCompilation,
  normalizeUserCompilationConfig
} from '../../src/config/index.js';
import { mergeFarmCliConfig } from '../../src/config/merge-config.js';
import { normalizeOutput } from '../../src/config/normalize-config/normalize-output.js';
import { NoopLogger } from '../../src/index.js';

describe('mergeFarmCliConfig', () => {
  test('inlineOption.root not empty', () => {
    const result = mergeFarmCliConfig({}, { root: '/path/to/' }, 'development');

    expect(result).toEqual({
      clearScreen: false,
      compilation: {
        input: {},
        output: {}
      },
      watch: false,
      root: '/path/to/'
    });
  });

  test('userConfig.root not empty', () => {
    const result = mergeFarmCliConfig({ root: '/path/to/' }, {}, 'development');

    expect(result).toEqual({
      root: '/path/to/',
      clearScreen: false,
      compilation: {
        input: {},
        output: {}
      },
      watch: false
    });
  });

  test('userConfig.root both inlineOption not empty', () => {
    const result = mergeFarmCliConfig(
      { root: '/path/to/inlineOption' },
      { root: '/path/to/userConfig' },
      'development'
    );

    expect(result).toEqual({
      root: '/path/to/userConfig',
      clearScreen: false,
      compilation: {
        input: {},
        output: {}
      },
      watch: false
    });
  });

  test('userConfig.root relative, should have configPath', () => {
    expect(() => {
      mergeFarmCliConfig(
        { root: './path/to/' },
        { root: './path/userConfig' },
        'development'
      );
    }).toThrow();

    const result = mergeFarmCliConfig(
      { root: './path/to/', configFile: process.cwd() },
      { root: './path/userConfig' },
      'development'
    );

    expect(result).toEqual({
      clearScreen: false,
      compilation: {
        input: {},
        output: {}
      },
      watch: false,
      root: path.resolve(process.cwd(), './path/userConfig')
    });
  });

  describe('normalizeUserCompilationConfig', () => {
    test('normalizeUserCompilationConfig without default and userConfig.compilation.input', async () => {
      expect(async () => {
        await normalizeUserCompilationConfig(
          {
            logger: new NoopLogger()
          },
          'development'
        );
      }).rejects.toThrowError();

      let config = await normalizeUserCompilationConfig(
        {
          logger: new NoopLogger(),
          compilation: {
            input: {
              index: 'index.ts'
            }
          }
        },
        'development'
      );

      expect(config.input).toEqual({ index: './index.ts' });
    });

    test('concatenateModule should conflict by mode', async () => {
      let config = await normalizeUserCompilationConfig(
        {
          logger: new NoopLogger(),
          compilation: {
            input: {
              index: 'index.ts'
            },
            concatenateModules: true
          }
        },
        'development'
      );

      expect(config.concatenateModules).toBeFalsy();

      config = await normalizeUserCompilationConfig(
        {
          logger: new NoopLogger(),
          compilation: {
            input: {
              index: 'index.ts'
            },
            concatenateModules: true
          }
        },
        'production'
      );

      expect(config.concatenateModules).toBeTruthy();
    });
  });
});

describe('normalizeOutput', () => {
  test('normalizeOutput with default', () => {
    const resolvedConfig: ResolvedCompilation = {};
    normalizeOutput(resolvedConfig, true, new NoopLogger());

    expect(resolvedConfig.output?.targetEnv).toEqual('browser');
    expect(resolvedConfig.output?.filename).toEqual(
      '[resourceName].[contentHash].[ext]'
    );
    expect(resolvedConfig.output?.assetsFilename).toEqual(
      '[resourceName].[contentHash].[ext]'
    );
    expect(resolvedConfig.output?.publicPath).toEqual('/');
  });

  test('normalizeOutput with targetEnv', () => {
    const resolvedConfig: ResolvedCompilation = {
      output: {
        targetEnv: 'node'
      }
    };
    normalizeOutput(resolvedConfig, true, new NoopLogger());

    expect(resolvedConfig.output?.targetEnv).toEqual('node');
    expect(resolvedConfig.output?.publicPath).toEqual('./');
  });

  test('normalizeOutput with node targetEnv and absolute publicPath shoud use user input publicPath', () => {
    const resolvedConfig: ResolvedCompilation = {
      output: {
        targetEnv: 'node',
        publicPath: '/public/'
      }
    };

    normalizeOutput(resolvedConfig, true, new NoopLogger());
    expect(resolvedConfig.output.targetEnv).toEqual('node');
    expect(resolvedConfig.output.publicPath).toEqual('/public/');
  });

  test('normalizeOutput with node targetEnv shoud use default publicPath by targetEnv', () => {
    (
      [
        { targetEnv: 'node', expectPublic: './' },
        { targetEnv: 'browser', expectPublic: '/' }
      ] as const
    ).forEach((item) => {
      const resolvedConfig: ResolvedCompilation = {
        output: {
          targetEnv: item.targetEnv
        }
      };

      normalizeOutput(resolvedConfig, true, new NoopLogger());
      expect(resolvedConfig.output.targetEnv).toEqual(item.targetEnv);
      expect(resolvedConfig.output.publicPath).toEqual(item.expectPublic);
    });
  });
});
