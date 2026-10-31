import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';
import { waitForDevServer } from '../benchmark/dev-server-startup.mjs';

function child() { const process = new EventEmitter(); process.pid = 123; process.stdout = new EventEmitter(); process.stderr = new EventEmitter(); return process; }
function options(extra = {}) { const diagnostics = []; const logs = []; return { diagnostics, logs, port: 4340, timeoutMs: 100,
  append: async chunk => logs.push(chunk.toString()), checkpoint: async data => diagnostics.push(structuredClone(data)), snapshot: async () => ({ tree: { processes: [{ pid: 123 }] } }), ...extra }; }
test('split colored readiness output is retained and listeners are removed', async () => {
  const process = child(), opts = options(); const pending = waitForDevServer(process, opts);
  process.stdout.emit('data', Buffer.from('\u001b[32mLo')); process.stdout.emit('data', Buffer.from('cal:\u001b[0m http://localhost:4340/'));
  await pending;
  assert.match(opts.logs.join(''), /Local:/); assert.equal(opts.diagnostics.at(-1).state, 'ready');
  assert.equal(process.stdout.listenerCount('data'), 0); assert.equal(process.listenerCount('close'), 0);
});
test('wrong port rejects readiness and records failure', async () => {
  const process = child(), opts = options(); const pending = waitForDevServer(process, opts);
  process.stdout.emit('data', Buffer.from('Local: http://localhost:4341/'));
  await assert.rejects(pending, /port unavailable/); assert.equal(opts.diagnostics.at(-1).state, 'failed');
});
test('silent timeout preserves process observation and stderr', async () => {
  const process = child(), opts = options({ timeoutMs: 20 }); const pending = waitForDevServer(process, opts);
  process.stderr.emit('data', Buffer.from('loading module'));
  await assert.rejects(pending, /startup timed out/);
  const last = opts.diagnostics.at(-1); assert.equal(last.pid, 123); assert.equal(last.stderrTail, 'loading module'); assert.equal(last.timeoutProcess.tree.processes[0].pid, 123);
});
test('early exit clears timeout and captures exit code', async () => {
  const process = child(), opts = options(); const pending = waitForDevServer(process, opts);
  process.emit('close', 7); await assert.rejects(pending, /exited: 7/); assert.equal(opts.diagnostics.at(-1).exitCode, 7);
});
