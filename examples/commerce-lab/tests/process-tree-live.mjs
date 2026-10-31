import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { snapshotWindowsTree, terminateOwnedWindowsTree } from '../benchmark/process-tree.mjs';

const fixture = `const {spawn}=require('node:child_process'); const child=spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{windowsHide:true}); console.log(child.pid); setInterval(()=>{},1000);`;
const child = spawn(process.execPath, ['-e', fixture], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
const report = { kind: 'windows-process-tree-verification', success: false };
const closed = once(child, 'close');
try {
  const [data] = await once(child.stdout, 'data');
  const descendantPid = Number(data.toString().trim());
  assert.ok(Number.isSafeInteger(descendantPid));
  report.observation = await snapshotWindowsTree(child.pid);
  assert.ok(report.observation.tree.processes.some(row => row.pid === child.pid));
  assert.ok(report.observation.tree.processes.some(row => row.pid === descendantPid));
  assert.ok(report.observation.tree.workingSetBytes > 0);
  report.termination = await terminateOwnedWindowsTree(child);
  await closed;
  report.after = await snapshotWindowsTree(child.pid);
  assert.equal(report.after.tree, null);
  report.descendantAfter = await snapshotWindowsTree(descendantPid);
  assert.equal(report.descendantAfter.tree, null);
  report.success = true;
} finally {
  if (child.exitCode === null && child.signalCode === null) await terminateOwnedWindowsTree(child);
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/process-tree-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
}
