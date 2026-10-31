import assert from 'node:assert/strict';
import test from 'node:test';
import { processTree } from '../benchmark/process-tree.mjs';

test('tree includes every descendant once, excluding unrelated processes', () => {
  const rows = [
    { pid: 11, parentPid: 9, workingSetBytes: 100 },
    { pid: 12, parentPid: 11, workingSetBytes: 200 },
    { pid: 13, parentPid: 12, workingSetBytes: 300 },
    { pid: 14, parentPid: 11, workingSetBytes: 400 },
    { pid: 99, parentPid: 9, workingSetBytes: 10000 },
  ];
  const result = processTree(rows, 11);
  assert.deepEqual(result.processes.map(row => row.pid), [11, 12, 13, 14]);
  assert.equal(result.workingSetBytes, 1000);
  assert.equal(processTree(rows, 100), null, 'missing root is not zero memory');
});

test('cycles terminate and invalid memory observations fail explicitly', () => {
  assert.equal(processTree([{ pid: 1, parentPid: 2, workingSetBytes: 1 }, { pid: 2, parentPid: 1, workingSetBytes: 2 }], 1).workingSetBytes, 3);
  assert.throws(() => processTree([{ pid: 1, parentPid: 0, workingSetBytes: -1 }], 1));
});
