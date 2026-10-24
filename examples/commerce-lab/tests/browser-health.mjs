import assert from 'node:assert/strict';

export function monitorBrowserHealth(page) {
  const network = page.context();
  const evidence = { pageErrors: [], consoleErrors: [], failedRequests: [], httpErrors: [], responses: [] };
  const pageError = error => evidence.pageErrors.push(error.message);
  const consoleMessage = message => {
    if (message.type() === 'error') evidence.consoleErrors.push({ text: message.text(), location: message.location() });
  };
  const requestFailed = request => evidence.failedRequests.push({
    url: request.url(), method: request.method(), resourceType: request.resourceType(), error: request.failure()?.errorText ?? 'unknown request failure',
  });
  const response = value => {
    const record = { url: value.url(), status: value.status(), resourceType: value.request().resourceType() };
    evidence.responses.push(record);
    if (record.status >= 400) evidence.httpErrors.push(record);
  };
  page.on('pageerror', pageError);
  page.on('console', consoleMessage);
  network.on('requestfailed', requestFailed);
  network.on('response', response);
  return {
    evidence,
    assertHealthy() {
      for (const field of ['pageErrors', 'consoleErrors', 'failedRequests', 'httpErrors']) {
        assert.deepEqual(evidence[field], [], `Commerce browser ${field}`);
      }
    },
    stop() {
      page.off('pageerror', pageError);
      page.off('console', consoleMessage);
      network.off('requestfailed', requestFailed);
      network.off('response', response);
    },
  };
}
