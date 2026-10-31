import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

test('Examples build IDs appear only after actual command success, never for absent manifests', async () => {
  const original = process.cwd(), directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-example-counts-'));
  try {
    fs.mkdirSync(path.join(directory, 'examples', 'present'), { recursive: true });
    fs.mkdirSync(path.join(directory, 'examples', 'missing'), { recursive: true });
    fs.writeFileSync(path.join(directory, 'examples', 'present', 'package.json'), JSON.stringify({ name: 'present' }));
    process.chdir(directory);
    const { buildExamples } = await import(new URL('./build.mjs?actual-counts', import.meta.url));
    const calls = [], execute = async (...args) => { calls.push(args); };
    assert.deepEqual(await buildExamples({ example: 'present', execute }), ['examples-build:examples/present']);
    assert.equal(calls.length, 1); assert.deepEqual(calls[0].slice(0, 2), ['npm', ['run', 'build']]);
    assert.deepEqual(await buildExamples({ example: 'missing', execute }), []);
    assert.equal(calls.length, 1);
    await assert.rejects(buildExamples({ example: 'absent-directory', execute }));
    await assert.rejects(buildExamples({ example: 'present', execute: async () => { throw Error('Expected command failure'); } }));
  } finally { process.chdir(original); fs.rmSync(directory, { recursive: true, force: true }); }
});
