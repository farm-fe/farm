import assert from 'node:assert/strict';
import test from 'node:test';
import { instrumentPluginHooks } from '../benchmark/hook-diagnostics.mjs';
test('diagnostics preserve hook receiver and result while tracking unfinished promises', async () => {
  let release;
  const pending = new Map();
  const plugins = [{ name: 'probe', load: { marker: 'bound', executor(param) {
    assert.equal(this.marker, 'bound');
    return new Promise(resolve => { release = () => resolve({ content: param.resolvedPath }); });
  } }, transform: { executor() { return 'sync-result'; } } }];
  instrumentPluginHooks(plugins, pending);
  const result = plugins[0].load.executor({ resolvedPath: '/catalog.scss' });
  assert.deepEqual([...pending.values()], [{ plugin: 'probe', hook: 'load', path: '/catalog.scss' }]);
  assert.equal(plugins[0].transform.executor(), 'sync-result');
  assert.equal(pending.size, 1);
  release();
  assert.deepEqual(await result, { content: '/catalog.scss' });
  assert.equal(pending.size, 0);
});
test('throwing and rejected hooks clear diagnostics and retain the original error', async () => {
  const error = new Error('hook failed');
  const pending = new Map();
  const plugins = [{ name: 'failure', load: { executor() { throw error; } }, transform: { async executor() { throw error; } } }];
  instrumentPluginHooks(plugins, pending);
  assert.throws(() => plugins[0].load.executor(), failure => failure === error);
  await assert.rejects(plugins[0].transform.executor(), failure => failure === error);
  assert.equal(pending.size, 0);
});
