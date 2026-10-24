import test from 'node:test';
import assert from 'node:assert/strict';
import { gunzipSync } from 'node:zlib';
import { compression } from '../adapters/compression.mjs';

test('gzip preserves originals and compresses eligible resources at the threshold', async () => {
  const make = (name, content, emitted = false) => ({ name, bytes: [...Buffer.from(content)], emitted, resourceType: 'asset', origin: { type: 'Module', value: 'fixture' } });
  const resourcesMap = {
    'main.js': make('main.js', 'x'.repeat(1024)),
    'small.css': make('small.css', 'x'.repeat(1023)),
    'image.png': make('image.png', 'x'.repeat(2048)),
    'runtime.js': make('runtime.js', 'x'.repeat(2048), true),
  };
  const before = structuredClone(resourcesMap);
  const result = await compression().finalizeResources.executor({ resourcesMap });
  assert.deepEqual(resourcesMap, before);
  assert.deepEqual(Object.keys(result).sort(), [...Object.keys(before), 'main.js.gz'].sort());
  assert.deepEqual(gunzipSync(Buffer.from(result['main.js.gz'].bytes)), Buffer.from(before['main.js'].bytes));
  assert.equal(result['main.js.gz'].emitted, false);
});
