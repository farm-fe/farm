import { describe, expect, it } from 'vitest';
import { normalizeCliArgv } from './argv.js';

describe('farm cli argv normalizer', () => {
  it('normalizes ssr subcommands from multi-token argv', () => {
    expect(
      normalizeCliArgv(['node', 'farm', 'ssr', 'build', 'examples/ssr-toolkit'])
    ).toEqual(['node', 'farm', 'ssr build', 'examples/ssr-toolkit']);
    expect(
      normalizeCliArgv([
        'node',
        'farm',
        'ssr',
        'preview',
        'examples/ssr-toolkit'
      ])
    ).toEqual(['node', 'farm', 'ssr preview', 'examples/ssr-toolkit']);
    expect(normalizeCliArgv(['node', 'farm', 'ssr', 'dev'])).toEqual([
      'node',
      'farm',
      'ssr dev'
    ]);
  });

  it('keeps argv unchanged for non-ssr commands', () => {
    expect(normalizeCliArgv(['node', 'farm', 'build'])).toEqual([
      'node',
      'farm',
      'build'
    ]);
    expect(normalizeCliArgv(['node', 'farm', 'ssr', 'unknown'])).toEqual([
      'node',
      'farm',
      'ssr',
      'unknown'
    ]);
    expect(normalizeCliArgv(['node', 'farm', 'ssr build'])).toEqual([
      'node',
      'farm',
      'ssr build'
    ]);
  });
});
