// cspell:words raf hashchange
import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';
import { waitForExtensionCategories, verifyBusinessBrowser, browserFailureEvidence } from '../examples/commerce-lab/benchmark/business-correctness.mjs';
import { correctnessFailureReceipt } from './performance-correctness-worker.mjs';

function routeFixture(counts) {
  let count = 0, frames = 0;
  const category = { waitFor: async options => assert.equal(options.timeout, 15000),
    locator: name => { assert.equal(name, 'option'); return { count: async () => count }; } };
  const page = { getByLabel: (label, options) => { assert.equal(label, 'Extension category'); assert.equal(options.exact, true); return category; },
    waitForFunction: async (predicate, expected, options) => {
      assert.equal(expected, 2048); assert.equal(options.timeout, 15000); assert.equal(options.polling, 'raf');
      const previous = globalThis.document;
      try {
        for (const next of counts) {
          count = next; frames++;
          globalThis.document = { querySelectorAll: selector => {
            assert.equal(selector, 'select[aria-label="Extension category"]');
            return [{ options: { length: count } }];
          } };
          await Promise.resolve(); // Simulated later browser render, not a timing measurement.
          if (predicate(expected)) return;
        }
        throw new Error('Bounded route readiness expired');
      } finally { if (previous === undefined) delete globalThis.document; else globalThis.document = previous; }
    } };
  return { page, category, frames: () => frames };
}

test('delayed lazy Components route waits for all 2048 options instead of counting immediately', async () => {
  const fixture = routeFixture([0, 0, 2048]); let healthChecks = 0;
  assert.equal(await waitForExtensionCategories(fixture.page, { assertHealthy: () => { healthChecks++; } }), fixture.category);
  assert.equal(fixture.frames(), 3); assert.equal(healthChecks, 2);
});

test('a ready select with missing business categories is not accepted', async () => {
  const fixture = routeFixture([0, 2047, 2047]);
  await assert.rejects(waitForExtensionCategories(fixture.page, { assertHealthy() {} }), /readiness expired/);
});

test('real runtime and network errors remain attached when route readiness fails, with bounded diagnostics', async () => {
  const page = new EventEmitter(), context = new EventEmitter();
  const timeout = new Error('Components route did not become ready');
  page.context = () => context; page.url = () => 'http://127.0.0.1:1234/#components';
  page.evaluate = async () => ({ title: 'Commerce', headings: ['Workspace'], body: '<main>Loading</main>' });
  page.goto = async () => {};
  page.getByTestId = () => ({ waitFor: async () => {} });
  page.getByRole = () => ({ click: async () => {} });
  page.getByLabel = () => ({ waitFor: async () => {} });
  page.waitForFunction = async () => {
    page.emit('pageerror', new Error('Lazy route runtime exploded'));
    for (let index = 0; index < 60; index++) page.emit('console', { type: () => 'error', text: () => 'Broken module ' + 'x'.repeat(3000), location: () => ({ url: page.url(), lineNumber: index }) });
    context.emit('requestfailed', { url: () => page.url() + '/route.js', method: () => 'GET', resourceType: () => 'script', failure: () => ({ errorText: 'connection closed' }) });
    context.emit('response', { url: () => page.url() + '/style.css', status: () => 404, request: () => ({ resourceType: () => 'stylesheet' }) });
    throw timeout;
  };
  await assert.rejects(verifyBusinessBrowser(page, 'http://127.0.0.1:1234/'), error => {
    assert.equal(error, timeout); assert.match(error.browserHealthFailure, /pageErrors/);
    const receipt = correctnessFailureReceipt(error);
    assert.equal(receipt.status, 'invalid'); assert.equal(receipt.measured, false);
    assert.equal(receipt.diagnostics.url, page.url());
    assert.match(receipt.diagnostics.document.body, /Loading/);
    assert.deepEqual(receipt.diagnostics.document.headings, ['Workspace']);
    assert.deepEqual(receipt.diagnostics.browserHealth.pageErrors, ['Lazy route runtime exploded']);
    assert.equal(receipt.diagnostics.counts.consoleErrors, 60);
    assert.equal(receipt.diagnostics.browserHealth.consoleErrors.length, 50);
    assert.ok(receipt.diagnostics.browserHealth.consoleErrors.every(row => row.text.length <= 2000));
    assert.equal(receipt.diagnostics.browserHealth.failedRequests.length, 1);
    assert.equal(receipt.diagnostics.browserHealth.httpErrors.length, 1);
    return true;
  });
  assert.equal(page.listenerCount('pageerror'), 0); assert.equal(page.listenerCount('console'), 0);
  assert.equal(context.listenerCount('requestfailed'), 0); assert.equal(context.listenerCount('response'), 0);
});

test('closed-page diagnostic collection preserves the original readiness failure', async () => {
  const page = new EventEmitter(), context = new EventEmitter(), original = new Error('Navigation failed');
  page.context = () => context; page.url = () => 'about:blank';
  page.goto = async () => { throw original; };
  page.evaluate = async () => { throw new Error('Page is closed'); };
  await assert.rejects(verifyBusinessBrowser(page, 'http://127.0.0.1/'), error => {
    assert.equal(error, original); assert.match(error.browserEvidence.documentUnavailable, /Page is closed/); return true;
  });
});

test('failure document capture bounds title, headings and DOM before returning it from the page', async () => {
  const previous = globalThis.document;
  globalThis.document = { title: 't'.repeat(800), body: { innerHTML: 'x'.repeat(20000) },
    querySelectorAll: () => Array.from({ length: 30 }, () => ({ textContent: 'h'.repeat(300) })) };
  try {
    const evidence = await browserFailureEvidence({ url: () => 'u'.repeat(3000), evaluate: async callback => callback() }, { evidence: { pageErrors: [] } });
    assert.equal(evidence.url.length, 2000); assert.equal(evidence.document.title.length, 500);
    assert.equal(evidence.document.body.length, 8192); assert.equal(evidence.document.headings.length, 20);
    assert.ok(evidence.document.headings.every(text => text.length === 200));
  } finally { if (previous === undefined) delete globalThis.document; else globalThis.document = previous; }
});

test('unresponsive DOM diagnostics are bounded without losing already captured health or URL', async () => {
  let rejectLate;
  const page = { url: () => 'http://127.0.0.1/#components', evaluate: () => new Promise((_, reject) => { rejectLate = reject; }) };
  const health = { evidence: { pageErrors: ['Original runtime failure'] } };
  const result = await browserFailureEvidence(page, health, { timeoutMs: 10 });
  assert.equal(result.url, page.url()); assert.deepEqual(result.browserHealth.pageErrors, ['Original runtime failure']);
  assert.match(result.documentUnavailable, /deadline exceeded/);
  rejectLate(new Error('Late page closure rejection'));
  await Promise.resolve();
});
