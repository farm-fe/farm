import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import ts from 'typescript';

test('Rust business plugin configuration contains native plugins only', async () => {
  // Keep the same relative import depth without requiring Node's TS loader.
  const temporary = new URL(`./.native-purity-${process.pid}.mjs`, import.meta.url);
  const source = await fs.readFile(new URL('../configs/rust.ts', import.meta.url), 'utf8');
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  });
  await fs.writeFile(temporary, outputText, { flag: 'wx' });
  try {
    const { rustPlugins } = await import(temporary.href);
    const config = rustPlugins();
    assert.equal(config.vitePlugins?.length ?? 0, 0);
    assert.ok(config.plugins.length >= 15);
    for (const plugin of config.plugins) {
      if (!plugin) continue; // Farm permits a disabled optional plugin.
      assert.ok(Array.isArray(plugin), `JS business hook still present: ${plugin.name}`);
      assert.equal(typeof plugin[0], 'string');
      assert.match(plugin[0], /\.farm$/);
    }
  } finally {
    await fs.unlink(temporary);
  }
});
