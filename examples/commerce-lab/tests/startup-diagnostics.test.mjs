import test from 'node:test';
import assert from 'node:assert/strict';
import { createStartupDiagnosticPlugin } from '../benchmark/startup-diagnostics.mjs';

test('startup diagnostics install hook tracking before compiler creation and retain the compiler identity', async () => {
  const pending = new Map(), observed = [];
  let release;
  const probe = { name: 'fixture-data', transform: { marker: 'receiver', executor(param) {
    assert.equal(this.marker, 'receiver');
    return new Promise(resolve => { release = () => resolve(param.content); });
  } } };
  const diagnostic = createStartupDiagnosticPlugin(pending, (stage, value) => observed.push({ stage, value }));
  const config = { jsPlugins: [diagnostic, probe] };
  diagnostic.configResolved(config);
  const result = probe.transform.executor({ resolvedPath: '/fixture.csv', content: 'ready' });
  assert.equal(pending.size, 1);
  const compiler = { compiling: true };
  diagnostic.configureCompiler(compiler);
  assert.deepEqual(observed.map(item => item.stage), ['config-resolved', 'compiler-created']);
  assert.equal(observed[0].value, config);
  assert.equal(observed[1].value, compiler);
  assert.deepEqual([...pending.values()], [{ plugin: 'fixture-data', hook: 'transform', path: '/fixture.csv' }]);
  release();
  assert.equal(await result, 'ready');
  assert.equal(pending.size, 0);
});
