import { describe, expect, test, vi } from 'vitest';
import { parseUserConfig } from '../src/config/schema.js';

// Config parsing does not require loading the compiler or native bindings.
vi.mock('../src/utils/logger.js', () => ({ Logger: class Logger {} }));

describe('library output configuration', () => {
  test.each(['single-bundle', 'multiple-bundle', 'bundle-less'] as const)(
    'preserves the native library strategy %s',
    (libraryBundleType) => {
      const output = { targetEnv: 'library' as const, libraryBundleType };
      expect(
        parseUserConfig({ compilation: { output } }).compilation?.output
      ).toEqual(output);
    }
  );

  test('leaves the native single-bundle default in charge when omitted', () => {
    const output = { targetEnv: 'library' as const };
    expect(
      parseUserConfig({ compilation: { output } }).compilation?.output
    ).toEqual(output);
  });

  test.each(['multiple', '', null, 1, true])(
    'rejects invalid strategy %s',
    (value) => {
      expect(() =>
        parseUserConfig({
          compilation: { output: { libraryBundleType: value } }
        } as never)
      ).toThrow();
    }
  );
});
