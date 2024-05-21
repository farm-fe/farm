import path from 'path';
import { describe, expect, test } from 'vitest';
import { normalizeUserCompilationConfig } from '../../src/config/index.js';
import { mergeFarmCliConfig } from '../../src/config/mergeConfig.js';
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
