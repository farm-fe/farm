import assert from 'node:assert/strict';
import test from 'node:test';
import { virtualWindow } from '../packages/ui/virtual-window.mjs';

test('order viewport includes visible rows and bounded overscan', () => {
  assert.deepEqual(virtualWindow(10000, 4800, 480, 48, 4), {
    start: 96, end: 114, before: 4608, after: 474528,
  });
});
test('empty, filtered and final viewports cannot point beyond the dataset', () => {
  assert.deepEqual(virtualWindow(0, 0, 480, 48, 4), { start: 0, end: 0, before: 0, after: 0 });
  assert.deepEqual(virtualWindow(3, 99999, 480, 48, 4), { start: 0, end: 3, before: 0, after: 0 });
  const bottom = virtualWindow(10000, 480000, 480, 48, 4);
  assert.equal(bottom.end, 10000);
  assert.ok(bottom.end - bottom.start <= 18);
  assert.equal(bottom.after, 0);
});
test('invalid geometry is rejected rather than emitting broken spacers', () => {
  for (const args of [[2, 0, 480, 0, 4], [-1, 0, 480, 48, 4], [2, NaN, 480, 48, 4]]) {
    assert.throws(() => virtualWindow(...args), RangeError);
  }
});
