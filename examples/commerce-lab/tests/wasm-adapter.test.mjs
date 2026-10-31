import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { wasm, generateWasmGlue } from '../adapters/wasm.mjs';

const addBytes = Buffer.from('AGFzbQEAAAABBwFgAn9/AX8DAgEABwcBA2FkZAAACgkBBwAgACABags=', 'base64');

test('generated JS glue instantiates real WASM and exports a callable function', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-wasm-'));
  try {
    const url = `data:application/wasm;base64,${addBytes.toString('base64')}`;
    const content = generateWasmGlue(new WebAssembly.Module(addBytes), url);
    const file = path.join(directory, 'add.mjs');
    await fs.writeFile(file, content);
    const { add } = await import(pathToFileURL(file).href);
    assert.equal(add(17, 25), 42);
    assert.equal(add(-4, 9), 5);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});

test('load hook preserves the WASM bytes and uses a deterministic emitted URL', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-wasm-'));
  try {
    const file = path.join(directory, 'add.wasm');
    await fs.writeFile(file, addBytes);
    const emitted = [];
    const context = { emitFile: value => emitted.push(value) };
    const plugin = wasm();
    const first = await plugin.load.executor({ resolvedPath: file, query: [], moduleId: file }, context);
    const second = await plugin.load.executor({ resolvedPath: file, query: [], moduleId: file }, context);
    assert.deepEqual(Buffer.from(emitted[0].content), addBytes);
    assert.equal(emitted[0].name, emitted[1].name);
    assert.equal(first.content, second.content);
    assert.equal(first.moduleType, 'js');
    assert.ok(first.content.includes(JSON.stringify(`/${emitted[0].name}`)));
    await fs.writeFile(file, Buffer.from('invalid wasm'));
    await assert.rejects(plugin.load.executor({ resolvedPath: file, query: [], moduleId: file }, context), WebAssembly.CompileError);
    assert.equal(emitted.length, 2, 'invalid input must not be emitted');
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
