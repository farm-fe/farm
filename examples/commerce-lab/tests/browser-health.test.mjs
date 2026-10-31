import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { test } from 'node:test';
import { monitorBrowserHealth } from './browser-health.mjs';

function fixture() {
  const page = new EventEmitter();
  const context = new EventEmitter();
  page.context = () => context;
  return { page, context, health: monitorBrowserHealth(page) };
}

test('browser health retains successful asset evidence and ignores ordinary logging', () => {
  const { page, context, health } = fixture();
  page.emit('console', { type: () => 'log', text: () => 'ready' });
  context.emit('response', { url: () => 'http://localhost/worker.js', status: () => 200, request: () => ({ resourceType: () => 'script' }) });
  health.assertHealthy();
  assert.deepEqual(health.evidence.responses, [{ url: 'http://localhost/worker.js', status: 200, resourceType: 'script' }]);
});

for (const [event, target, value, field] of [
  ['pageerror', 'page', new Error('uncaught'), 'pageErrors'],
  ['console', 'page', { type: () => 'error', text: () => 'render failed', location: () => ({ url: 'http://localhost/app.js', lineNumber: 3 }) }, 'consoleErrors'],
  ['requestfailed', 'context', { url: () => 'http://localhost/worker.js', method: () => 'GET', resourceType: () => 'script', failure: () => ({ errorText: 'connection closed' }) }, 'failedRequests'],
  ['response', 'context', { url: () => 'http://localhost/missing.svg', status: () => 404, request: () => ({ resourceType: () => 'image' }) }, 'httpErrors'],
]) {
  test(`browser health rejects ${field} with diagnostic evidence`, () => {
    const current = fixture();
    current[target].emit(event, value);
    assert.equal(current.health.evidence[field].length, 1);
    assert.throws(() => current.health.assertHealthy(), new RegExp(field));
  });
}

test('browser health detaches only its listeners before owned context shutdown', () => {
  const { page, context, health } = fixture();
  let unrelated = 0;
  context.on('requestfailed', () => unrelated++);
  health.stop();
  context.emit('requestfailed', {});
  page.emit('console', { type: () => 'error', text: () => 'closed' });
  assert.equal(unrelated, 1);
  health.assertHealthy();
});
