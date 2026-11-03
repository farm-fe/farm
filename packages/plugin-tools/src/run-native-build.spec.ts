import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { expect, test } from 'vitest';
import { runNativeBuild } from './run-native-build.js';

test('native CLI paths and arguments preserve spaces without shell evaluation', async () => {
  const root = mkdtempSync(path.join(tmpdir(), 'farm native cli '));
  try {
    const cli = path.join(root, 'fake cli.mjs');
    writeFileSync(
      cli,
      "import fs from 'node:fs'; fs.writeFileSync('args.json', JSON.stringify(process.argv.slice(2)));"
    );
    await runNativeBuild(
      cli,
      ['--profile', 'ci', 'literal $(echo unsafe)'],
      root
    );
    expect(
      JSON.parse(readFileSync(path.join(root, 'args.json'), 'utf8'))
    ).toEqual(['build', '--profile', 'ci', 'literal $(echo unsafe)']);
    writeFileSync(cli, 'process.exit(7);');
    await expect(runNativeBuild(cli, [], root)).rejects.toThrow('code=7');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
