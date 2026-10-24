import assert from 'node:assert/strict';
import test from 'node:test';
import { validateHmrRecords } from '../benchmark/hmr-records.mjs';

const scenarios = ['hmr-leaf', 'hmr-shared', 'hmr-sass', 'hmr-tailwind', 'hmr-svg', 'hmr-mdx'];
const records = () => scenarios.map(scenario => ({ group: 'rust', scenario, success: true, durationMs: 10, sameDocument: true, statePreserved: scenario === 'hmr-mdx' ? null : true, restored: true }));
test('complete HMR round requires state, document and restoration evidence', () => {
  assert.doesNotThrow(() => validateHmrRecords(records(), 'rust'));
  for (const field of ['sameDocument', 'statePreserved', 'restored', 'success']) {
    const sample = records(); sample[0][field] = false;
    assert.throws(() => validateHmrRecords(sample, 'rust'));
  }
  assert.throws(() => validateHmrRecords(records().slice(1), 'rust'));
  assert.throws(() => validateHmrRecords(records(), 'js'));
  const sample = records(); sample[0].durationMs = NaN;
  assert.throws(() => validateHmrRecords(sample, 'rust'));
});
