import { describe, expect, it } from 'vitest';
import {
  bgBlue,
  bold,
  createFormatter,
  red,
  reset,
  underline
} from '../src/color.js';

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
  it('bold should wrap with ansi code', () => {
    expect(bold('abc')).toMatch(/\x1b\[1mabc\x1b\[22m/);
  });

  it('underline should wrap with ansi code', () => {
    expect(underline('abc')).toMatch(/\x1b\[4mabc\x1b\[24m/);
  });

  it('red should wrap with ansi code', () => {
    expect(red('abc')).toMatch(/\x1b\[38;2;219;90;107mabc\x1b\[39m/);
  });

  it('bgBlue should wrap with ansi code', () => {
    expect(bgBlue('abc')).toMatch(/\x1b\[44mabc\x1b\[49m/);
  });

  it('reset should wrap with ansi code', () => {
    expect(reset('abc')).toMatch(/\x1b\[0mabc\x1b\[0m/);
  });
});
