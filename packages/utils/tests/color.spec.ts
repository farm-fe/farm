import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createFormatter } from '../src/color.js';

const loadColors = () => import('../src/color.js');

describe('createFormatter', () => {
  it('should wrap string with open and close', () => {
    const fmt = createFormatter('[', ']');
    expect(fmt('abc')).toBe('[abc]');
  });

  it('should return input if open or close is empty', () => {
    const fmt1 = createFormatter('', ']');
    const fmt2 = createFormatter('[', '');
    expect(fmt1('abc')).toBe('abc');
    expect(fmt2('abc')).toBe('abc');
  });

  it('should handle string containing close', () => {
    const fmt = createFormatter('[', ']');
    expect(fmt('a]b]c')).toBe('[a[b[c]');
  });

  it('should use replace param for nested close', () => {
    const fmt = createFormatter('[', ']', '|');
    expect(fmt('a]b]c')).toBe('[a|b|c]');
  });
});

describe('color functions', () => {
  let colors: Awaited<ReturnType<typeof loadColors>>;
  beforeEach(async () => {
    // Vitest 2 stringifies undefined; register restoration before deleting the key.
    vi.stubEnv('NO_COLOR', '');
    Reflect.deleteProperty(process.env, 'NO_COLOR');
    vi.stubEnv('FORCE_COLOR', '1');
    vi.resetModules();
    colors = await loadColors();
  });
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.resetModules();
  });
  it('bold should wrap with ansi code', () => {
    expect(colors.bold('abc')).toMatch(/\x1b\[1mabc\x1b\[22m/);
  });

  it('underline should wrap with ansi code', () => {
    expect(colors.underline('abc')).toMatch(/\x1b\[4mabc\x1b\[24m/);
  });

  it('red should wrap with ansi code', () => {
    expect(colors.red('abc')).toMatch(/\x1b\[38;2;219;90;107mabc\x1b\[39m/);
  });

  it('bgBlue should wrap with ansi code', () => {
    expect(colors.bgBlue('abc')).toMatch(/\x1b\[44mabc\x1b\[49m/);
  });

  it('reset should wrap with ansi code', () => {
    expect(colors.reset('abc')).toMatch(/\x1b\[0mabc\x1b\[0m/);
  });

  it('NO_COLOR takes precedence over forced colors', async () => {
    vi.stubEnv('NO_COLOR', '1');
    vi.resetModules();
    const disabled = await loadColors();
    expect(disabled.red('abc')).toBe('abc');
    expect(disabled.bgBlue('abc')).toBe('abc');
    expect(disabled.bold('abc')).toBe('abc');
  });
});
