import assert from 'node:assert/strict';
import test from 'node:test';
import { publicationStrip } from '../adapters/publication-strip.mjs';

test('JS production strip removes only configured static diagnostics and preserves computed business calls', () => {
  const plugin = publicationStrip('js', 'production');
  const result = plugin.transform.executor({
    moduleType: 'js', resolvedPath: '/commerce/publication-diagnostics.mjs',
    content: "commerceDiagnostics['business'].trace(commitOrder()); commerceDiagnostics.trace('sample');",
  });
  assert.ok(result.content.includes("commerceDiagnostics['business'].trace(commitOrder())"));
  assert.ok(!result.content.includes("commerceDiagnostics.trace('sample')"));
  assert.ok(JSON.parse(result.sourceMap).mappings.length > 0);
});

test('development disables stripping in all three plugin paths', () => {
  for (const group of ['js', 'rust', 'mixed']) assert.equal(publicationStrip(group, 'development'), false);
  assert.throws(() => publicationStrip('js', 'unknown'), /mode/);
});
