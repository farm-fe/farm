import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import dts from '../dist/index.js';

test('finish awaits declarations in an output directory outside the project root', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-dts-output-test-'));
  const root = path.join(directory, 'source');
  const output = path.join(directory, 'dist');
  await fs.mkdir(root);
  await fs.writeFile(path.join(root, 'tsconfig.json'), JSON.stringify({ compilerOptions: { target: 'ES2022', types: [] } }));
  const source = 'export interface Product { price: number }\n';
  const entry = path.join(root, 'index.ts');
  await fs.writeFile(entry, source);
  try {
    const plugin = dts({ outputDir: output, entryRoot: root });
    plugin.configResolved({ compilation: { root, mode: 'production', output: { path: output } } });
    await plugin.transform.executor({ resolvedPath: entry, content: source, moduleType: 'dts' });
    await plugin.finish.executor();
    assert.match(await fs.readFile(path.join(output, 'index.d.ts'), 'utf8'), /price: number/);
    await fs.writeFile(entry, 'export interface Product { price: number; cacheUpdateProbe?: string }\n');
    const warm = dts({ outputDir: output, entryRoot: root });
    warm.configResolved({ compilation: { root, mode: 'production', output: { path: output } } });
    await warm.finish.executor();
    assert.match(await fs.readFile(path.join(output, 'index.d.ts'), 'utf8'), /cacheUpdateProbe\?: string/);
    await fs.writeFile(entry, source);
    await warm.updateFinished.executor();
    assert.doesNotMatch(await fs.readFile(path.join(output, 'index.d.ts'), 'utf8'), /cacheUpdateProbe/);
  } finally {
    const resolved = await fs.realpath(directory);
    assert.equal(path.dirname(resolved), await fs.realpath(os.tmpdir()));
    assert.ok(path.basename(resolved).startsWith('farm-dts-output-test-'));
    await fs.rm(resolved, { recursive: true });
  }
});
