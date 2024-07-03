import path from 'path';
import { describe, expect, test } from 'vitest';
import {
  ResolvedCompilation,
  normalizeUserCompilationConfig
} from '../../src/config/index.js';
import { mergeFarmCliConfig } from '../../src/config/mergeConfig.js';
import { normalizeOutput } from '../../src/config/normalize-config/normalize-output.js';
import { NoopLogger } from '../../src/index.js';

describe('mergeFarmCliConfig', () => {
  test('inlineOption.root not empty', () => {
    const result = mergeFarmCliConfig({}, { root: '/path/to/' });

    expect(result).toEqual({ root: '/path/to/' });
  });

  test('userConfig.root not empty', () => {
    const result = mergeFarmCliConfig({ root: '/path/to/' }, {});

    expect(result).toEqual({ root: '/path/to/' });
  });

  test('userConfig.root both inlineOption not empty', () => {
    const result = mergeFarmCliConfig(
      { root: '/path/to/inlineOption' },
      { root: '/path/to/userConfig' }
    );

    expect(result).toEqual({ root: '/path/to/userConfig' });
  });

  test('userConfig.root relative, should have configPath', () => {
    expect(() => {
      mergeFarmCliConfig({ root: './path/to/' }, { root: './path/userConfig' });
    }).toThrow();

    const result = mergeFarmCliConfig(
      { root: './path/to/', configPath: process.cwd() },
      { root: './path/userConfig' }
    );

    expect(result).toEqual({
      root: path.resolve(process.cwd(), './path/userConfig')
    });
  });

  describe('normalizeUserCompilationConfig', () => {
    test('with default', async () => {
      let config = await normalizeUserCompilationConfig(
        {},
        {},
        new NoopLogger(),
        'development',
        true
      );

      expect(config.input).toEqual({});
    });

    test('normalizeUserCompilationConfig without default and userConfig.compilation.input', async () => {
      expect(async () => {
        await normalizeUserCompilationConfig(
          {},
          {},
          new NoopLogger(),
          'development',
          false
        );
      }).rejects.toThrowError();

      let config = await normalizeUserCompilationConfig(
        {},
        {
          compilation: {
            input: {
              index: 'index.ts'
            }
          }
        },
        new NoopLogger(),
        'development',
        false
      );

      expect(config.input).toEqual({ index: 'index.ts' });
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

  test('normalizeOutput with node targetEnv and absolute publicPath', () => {
    const resolvedConfig: ResolvedCompilation = {
      output: {
        targetEnv: 'node',
        publicPath: '/public/'
      }
    };

    normalizeOutput(resolvedConfig, true, new NoopLogger());
    expect(resolvedConfig.output.targetEnv).toEqual('node');
    expect(resolvedConfig.output.publicPath).toEqual('./public/');
  });
});
