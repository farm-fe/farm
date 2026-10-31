import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

test('runner prepares clean extension sources before CLI and preserves modified sources', async () => {
  const fixture = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-run-prerequisite-'));
  const lab = path.join(fixture, 'examples', 'commerce-lab');
  const source = fileURLToPath(new URL('../', import.meta.url));
  await fs.mkdir(path.join(lab, 'scripts'), { recursive: true });
  await fs.mkdir(path.join(lab, 'benchmark'), { recursive: true });
  await fs.copyFile(path.join(source, 'scripts/run.mjs'), path.join(lab, 'scripts/run.mjs'));
  for (const name of await fs.readdir(path.join(source, 'benchmark'))) {
    if (name.endsWith('.mjs')) await fs.copyFile(path.join(source, 'benchmark', name), path.join(lab, 'benchmark', name));
  }
  const cli = path.join(fixture, 'packages', 'cli', 'bin');
  await fs.mkdir(cli, { recursive: true });
  await fs.writeFile(path.join(cli, 'farm.mjs'), `
import fs from 'node:fs';
import path from 'node:path';
const index = path.resolve('packages/extensions/baseline/index.ts');
if (!fs.existsSync(index)) throw new Error('Extension missing before CLI');
fs.appendFileSync('cli-calls.txt', JSON.stringify(process.argv.slice(2)) + '\\n');
`);
  const run = group => spawnSync(process.execPath, [path.join(lab, 'scripts/run.mjs'), 'build'], {
    cwd: lab,
    env: { ...process.env, COMMERCE_GROUP: group, COMMERCE_SCALE: 'baseline' },
    encoding: 'utf8',
    timeout: 20_000
  });
  for (const group of ['js', 'rust', 'mixed']) {
    const result = run(group);
    assert.equal(result.status, 0, result.stderr);
  }
  const calls = await fs.readFile(path.join(lab, 'cli-calls.txt'), 'utf8');
  assert.equal(calls.trim().split('\n').length, 3);
  for (const config of ['farm.config.ts', 'farm.rust.config.ts', 'farm.mixed.config.ts']) assert.ok(calls.includes(config));
  const form = path.join(lab, 'packages/extensions/baseline/category-001/Form.tsx');
  const modified = (await fs.readFile(form, 'utf8')) + '\n// preserved custom edit\n';
  await fs.writeFile(form, modified);
  const { generateScale } = await import(pathToFileURL(path.join(lab, 'benchmark/extension-generator.mjs')).href);
  await assert.rejects(generateScale('baseline'), /Refusing to overwrite modified extension/);
  const preserved = run('js');
  assert.equal(preserved.status, 0, preserved.stderr);
  assert.equal(await fs.readFile(form, 'utf8'), modified);
  assert.equal((await fs.readFile(path.join(lab, 'cli-calls.txt'), 'utf8')).trim().split('\n').length, 4);
});
