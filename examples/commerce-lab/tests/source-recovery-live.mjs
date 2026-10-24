import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { restoreSourceProbes } from '../benchmark/source-recovery.mjs';
import { terminateOwnedWindowsTree } from '../benchmark/process-tree.mjs';

const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-aborted-edit-'));
const file = path.join(root, 'fixture.ts');
await fs.writeFile(file, 'original source');
const child = spawn(process.execPath, ['-e', "require('node:fs').writeFileSync(process.argv[1],'probe source');console.log('edited');setInterval(()=>{},1000)", file], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
const closed = once(child, 'close');
const report = { kind: 'aborted-driver-source-recovery', success: false };
try {
  await once(child.stdout, 'data');
  assert.equal(await fs.readFile(file, 'utf8'), 'probe source');
  report.termination = await terminateOwnedWindowsTree(child);
  await closed;
  report.recovery = await restoreSourceProbes(root, new Map([['fixture.ts', 'original source']]), [['fixture.ts', 'original', 'probe']]);
  assert.deepEqual(report.recovery.restored, ['fixture.ts']);
  assert.deepEqual(report.recovery.errors, []);
  assert.deepEqual(report.recovery.conflicts, []);
  assert.equal(await fs.readFile(file, 'utf8'), 'original source');
  report.success = true;
} finally {
  if (child.exitCode === null && child.signalCode === null) await terminateOwnedWindowsTree(child);
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/source-recovery-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
}
