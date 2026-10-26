import assert from 'node:assert/strict';
import { EventEmitter, once } from 'node:events';
import os from 'node:os';
import test from 'node:test';
import { initBrowserContext, visitPage } from './farm-runner.mjs';
import { waitForStyle, waitForText } from '../examples/vue/e2e.spec.mjs';

class Page extends EventEmitter {
  closed = false;
  isClosed() { return this.closed; }
  async goto() {}
  async close() { if (!this.closed) { this.closed = true; this.emit('close'); } }
}
const context = (page) => ({ browser: () => ({ isConnected: () => true }), newPage: async () => page });
const consoleError = () => ({ type: () => 'error', text: () => 'VIRTUAL_STYLE_RESOLVE_FAILED', location: () => ({}) });
const tick = () => new Promise(resolve => setImmediate(resolve));

for (const event of ['console', 'pageerror']) {
  test(event + ' failure waits for callback fixture restoration before returning', async () => {
    const page = new Page();
    initBrowserContext(context(page));
    let release, entered, settled = false, restored = false;
    const cleanup = new Promise(resolve => { release = resolve; });
    const started = new Promise(resolve => { entered = resolve; });
    const result = visitPage('http://fixture', os.tmpdir(), async () => {
      const closed = once(page, 'close');
      entered();
      page.emit(event, event === 'console' ? consoleError() : new Error('VIRTUAL_STYLE_RESOLVE_FAILED'));
      try { await closed; throw new Error('page closed'); }
      finally { await cleanup; restored = true; }
    }, 'start').then(() => { settled = true; return null; }, error => { settled = true; return error; });
    await started;
    await tick();
    try {
      assert.equal(page.isClosed(), true);
      assert.equal(settled, false, 'retry must not begin while callback finally is pending');
      assert.equal(restored, false);
    } finally { release(); }
    const error = await result;
    assert.match(error.message, /VIRTUAL_STYLE_RESOLVE_FAILED/);
    assert.equal(restored, true);
    assert.equal(page.listenerCount('console'), 0);
    initBrowserContext(null);
  });
}

test('navigation console failure cancels a callback that has not started', async () => {
  const page = new Page();
  page.goto = async () => { page.emit('console', consoleError()); };
  initBrowserContext(context(page));
  let called = false;
  await assert.rejects(visitPage('http://fixture', os.tmpdir(), async () => { called = true; }, 'start'), /VIRTUAL_STYLE_RESOLVE_FAILED/);
  assert.equal(called, false);
  initBrowserContext(null);
});

test('callback rejection closes its page and preserves the assertion error', async () => {
  const page = new Page();
  initBrowserContext(context(page));
  await assert.rejects(visitPage('http://fixture', os.tmpdir(), async () => { throw new Error('ASSERTION_FAILED'); }, 'start'), /ASSERTION_FAILED/);
  assert.equal(page.isClosed(), true);
  initBrowserContext(null);
});

test('successful callback remains successful during page teardown', async () => {
  const page = new Page();
  const close = page.close.bind(page);
  page.close = async () => { page.emit('console', consoleError()); await close(); };
  initBrowserContext(context(page));
  await visitPage('http://fixture', os.tmpdir(), async () => {}, 'start');
  assert.equal(page.isClosed(), true);
  initBrowserContext(null);
});

test('Vue style polling propagates closed-page errors promptly', async () => {
  const page = { isClosed: () => true, $eval: async () => { throw new Error('PAGE_CLOSED'); } };
  await assert.rejects(waitForStyle(page, '.intro', 'color', 'green'), /PAGE_CLOSED/);
});

test('Vue text polling propagates closed-page errors promptly', async () => {
  const page = { isClosed: () => true, textContent: async () => { throw new Error('PAGE_CLOSED'); } };
  await assert.rejects(waitForText(page, '.intro', 'Farm'), /PAGE_CLOSED/);
});
