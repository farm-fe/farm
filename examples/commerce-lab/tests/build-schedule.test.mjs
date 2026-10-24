import assert from 'node:assert/strict';
import test from 'node:test';
import { buildSchedule } from '../benchmark/build-schedule.mjs';

test('seeded rounds cover every group with isolated cold and matching warm caches', () => {
  const plan = buildSchedule({ runId: 'probe-1', rounds: 10, seed: 42 });
  assert.deepEqual(plan, buildSchedule({ runId: 'probe-1', rounds: 10, seed: 42 }));
  assert.equal(plan.length, 60);
  const caches = new Set();
  for (let i = 0; i < plan.length; i += 6) {
    const round = plan.slice(i, i + 6);
    assert.deepEqual(new Set(round.map(item => item.group)), new Set(['js', 'rust', 'mixed']));
    for (let j = 0; j < 6; j += 2) {
      const [cold, warm] = round.slice(j, j + 2);
      assert.equal(cold.cacheMode, 'cold');
      assert.equal(warm.cacheMode, 'warm');
      assert.equal(cold.group, warm.group);
      assert.equal(cold.cacheId, warm.cacheId);
      assert.equal(cold.outputId, warm.outputId);
      assert.ok(!caches.has(cold.cacheId));
      caches.add(cold.cacheId);
    }
  }
  assert.ok(new Set(plan.filter(item => item.cacheMode === 'cold').filter((_, i) => i % 3 === 0).map(item => item.group)).size > 1);
});

test('invalid schedule identities and counts are rejected', () => {
  for (const runId of ['../outside', '', 'a/b']) assert.throws(() => buildSchedule({ runId, rounds: 10, seed: 42 }));
  for (const rounds of [0, -1, 1.5, Infinity]) assert.throws(() => buildSchedule({ runId: 'test', rounds, seed: 42 }));
});
