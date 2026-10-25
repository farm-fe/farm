import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';

const require = createRequire(new URL('../package.json', import.meta.url));
const ts = require('typescript');
const code = await fs.readFile(new URL('../src/build-cache.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(code, { compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 }, reportDiagnostics: true });
assert.equal(compiled.diagnostics?.length ?? 0, 0);
const { docsCacheIdentity, isolateDocsTargetCache } = await import(`data:text/javascript;base64,${Buffer.from(compiled.outputText).toString('base64')}`);

test('only explicitly bound transaction paths are relocatable', () => {
  const identity = (stage, extra = {}) => docsCacheIdentity({ plan: `${stage}/plan.json`, output: `${stage}/browser`, ...extra }, new Map([[`${stage}/plan.json`, 'transaction:plan'], [`${stage}/browser`, 'transaction:browser']]));
  assert.equal(identity('/first'), identity('/second'));
  assert.notEqual(identity('/first', { alias: '/first/other' }), identity('/second', { alias: '/second/other' }));
  assert.notEqual(identity('/first', { define: '/first/plan.json-extra' }), identity('/second', { define: '/second/plan.json-extra' }));
  assert.equal(docsCacheIdentity({ a: 1, b: 2 }, new Map()), docsCacheIdentity({ b: 2, a: 1 }, new Map()));
  assert.throws(() => docsCacheIdentity({ callback() {} }, new Map()), /serializable/);
});

test('target, full configuration, snapshot, plugin options and actual binaries isolate cache', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-docs-cache-'));
  try {
    const plugin = path.join(directory, 'plugin.farm');
    await fs.writeFile(plugin, Buffer.from([0, 255, 17]));
    async function identity({ target = 'browser', define = 'one', snapshotId = 'one', core = 'one', option = 'one', env = 'one' } = {}) {
      const resolved = { compilation: { output: { targetEnv: target }, define: { VALUE: define }, persistentCache: { namespace: 'existing', envs: { USER: env }, buildDependencies: ['retained'] } }, rustPlugins: [[plugin, JSON.stringify({ option })]] };
      await isolateDocsTargetCache(resolved, { target, entryName: 'app', snapshotId, site: { title: 'Site' }, nativeIdentity: { path: '/actual/core.node', sha256: core }, roles: new Map(), nativeHashes: new Map() });
      assert.deepEqual(resolved.compilation.persistentCache.buildDependencies, ['retained']);
      assert.equal(resolved.compilation.persistentCache.envs.USER, env);
      return { namespace: resolved.compilation.persistentCache.namespace, hash: resolved.compilation.persistentCache.envs.FARM_DOCS_TARGET_IDENTITY };
    }
    const original = await identity();
    assert.deepEqual(original, await identity());
    for (const changed of [{ target: 'node' }, { define: 'two' }, { snapshotId: 'two' }, { core: 'two' }, { option: 'two' }, { env: 'two' }]) assert.notEqual(original.hash, (await identity(changed)).hash);
    assert.notEqual(original.namespace, (await identity({ target: 'node' })).namespace);
    await fs.writeFile(plugin, Buffer.from([0, 255, 18]));
    assert.notEqual(original.hash, (await identity()).hash);
    await fs.rm(plugin);
    await assert.rejects(identity(), /ENOENT/);
  } finally { await fs.rm(directory, { recursive: true, force: true }); }
});
