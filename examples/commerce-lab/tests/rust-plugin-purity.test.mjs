import assert from 'node:assert/strict';
import test from 'node:test';

test('Rust business plugin configuration contains native plugins only', async () => {
  const { rustPlugins } = await import('../configs/rust.ts');
  const config = rustPlugins();
  assert.equal(config.vitePlugins?.length ?? 0, 0);
  assert.ok(config.plugins.length >= 15);
  for (const plugin of config.plugins) {
    if (!plugin) continue; // Farm permits a disabled optional plugin.
    assert.ok(Array.isArray(plugin), `JS business hook still present: ${plugin.name}`);
    assert.equal(typeof plugin[0], 'string');
    assert.match(plugin[0], /\.farm$/);
  }
});
