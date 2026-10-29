import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { archiveFiles } from './release-archive.mjs';

test('normal create-farm pack preserves actual JS, bin, declarations and original legal files', async () => {
  const source = fileURLToPath(new URL('../packages/create-farm/', import.meta.url));
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-generator-pack-'));
  try {
    const manifest = JSON.parse(await fs.readFile(path.join(source, 'package.json')));
    for (const entry of ['index.js', 'index.d.ts', 'create-farm.js']) assert.ok(manifest.files.includes(entry));
    await fs.writeFile(path.join(root, 'package.json'), JSON.stringify(manifest));
    for (const entry of manifest.files) await fs.cp(path.join(source, entry), path.join(root, entry), { recursive: true });
    const npm = path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
    execFileSync(process.execPath, [npm, 'pack', '--ignore-scripts', '--json', '--pack-destination', root], { cwd: root, timeout: 30000, stdio: 'pipe' });
    const files = await fs.readdir(root), archive = files.filter(file => file.endsWith('.tgz'));
    assert.equal(archive.length, 1);
    const contents = archiveFiles(await fs.readFile(path.join(root, archive[0])));
    for (const entry of [manifest.main, manifest.types, manifest.bin['create-farm'], 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'license-components.json']) assert.ok(contents.has(`package/${entry}`), `Missing normally packed entry: ${entry}`);
    assert.match(contents.get('package/index.d.ts').toString(), /export declare function run/);
  } finally { await fs.rm(root, { recursive: true }); }
});
