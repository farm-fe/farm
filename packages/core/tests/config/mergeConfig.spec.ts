import { describe, expect, test } from 'vitest';
import { mergeConfig } from '../../src/config/merge-config.js';
import { UserConfig } from '../../src/index.js';

describe('mergeConfig', () => {
  test('merge object', () => {
    const fileConfig: UserConfig = {
      compilation: {
        input: {
          index: 'src/index.ts'
        }
      }
    };

    const inputConfig: UserConfig = {
      compilation: {
        input: {
          index2: 'src/index.ts'
        }
      }
    };
    const result: UserConfig = mergeConfig(fileConfig, inputConfig);

    expect(result).toEqual({
      compilation: {
        input: {
          index: 'src/index.ts',
          index2: 'src/index.ts'
        }
      }
    });
  });

  test('merge arr', () => {
    const fileConfig: UserConfig = {
      plugins: ['a']
    };

    const inputConfig: UserConfig = {
      plugins: ['b'],
      vitePlugins: [{ name: 'test' }]
    };
    const result: UserConfig = mergeConfig(fileConfig, inputConfig);

    expect(result).toEqual({
      plugins: ['a', 'b'],
      vitePlugins: [{ name: 'test' }]
    });
  });

  test('merge right to left', () => {
    const fileConfig: UserConfig = {};

    const inputConfig: UserConfig = {
      plugins: ['b']
    };
    const result: UserConfig = mergeConfig(fileConfig, inputConfig);

    expect(result).toEqual({
      plugins: ['b']
    });
  });
});
