import { describe, expect, test } from 'vitest';
import { mergeInlineCliOptions } from '../../src/config/index.js';
import path from 'path';

describe('mergeInlineCliOptions', () => {
  test('inlineOption.root not empty', () => {
    const result = mergeInlineCliOptions({}, { root: '/path/to/' });

    expect(result).toEqual({ root: '/path/to/' });
  });

  test('userConfig.root not empty', () => {
    const result = mergeInlineCliOptions({ root: '/path/to/' }, {});

    expect(result).toEqual({ root: '/path/to/' });
  });

  test('userConfig.root both inlineOption not empty', () => {
    const result = mergeInlineCliOptions(
      { root: '/path/to/userConfig' },
      { root: '/path/to/inlineOption' }
    );

    expect(result).toEqual({ root: '/path/to/userConfig' });
  });

  test('userConfig.root relative, should have configPath', () => {
    expect(() => {
      mergeInlineCliOptions(
        { root: './path/userConfig' },
        { root: './path/to/' }
      );
    }).toThrow();

    const result = mergeInlineCliOptions(
      { root: './path/userConfig' },
      { root: './path/to/', configPath: process.cwd() }
    );
    expect(result).toEqual({
      root: path.resolve('./path/userConfig')
    });
  });
});
