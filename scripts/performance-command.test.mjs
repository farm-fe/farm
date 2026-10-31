import test from 'node:test';
import assert from 'node:assert/strict';
import { EventEmitter, once } from 'node:events';
import { spawn } from 'node:child_process';
import { waitForCommand, rethrowUnsafeCleanup } from './performance-report.mjs';

test('unconfirmed cleanup aborts sampling instead of starting another child', () => {
  const error = Object.assign(new Error('owned process still live'), { cleanupConfirmed: false });
  assert.throws(() => rethrowUnsafeCleanup(error), value => value === error);
  assert.doesNotThrow(() => rethrowUnsafeCleanup(new Error('ordinary sample failure')));
});

test('normal and nonzero child closes retain exact exit codes and release listeners', async () => {
  for (const code of [0, 7]) {
    const child = new EventEmitter(); child.pid = 123;
    const pending = waitForCommand(child, { timeoutMs: 100 });
    child.emit('close', code, null);
    assert.equal((await pending).code, code);
    assert.equal(child.listenerCount('close'), 0);
    assert.equal(child.listenerCount('error'), 0);
  }
});
test('cleanup failure, hanging cleanup and handles that never close return bounded failures', async () => {
  for (const terminateProcess of [async () => { throw new Error('kill denied'); }, () => new Promise(() => {}), async () => {}]) {
    const child = new EventEmitter(); child.pid = 456;
    const started = Date.now();
    await assert.rejects(waitForCommand(child, { timeoutMs: 5, cleanupTimeoutMs: 20, terminateProcess }), error => {
      assert.equal(error.timedOut, true); assert.equal(error.pid, 456);
      assert.equal(error.cleanupConfirmed, false); assert.ok(error.cleanupError);
      return true;
    });
    assert.ok(Date.now() - started < 1000);
    assert.equal(child.listenerCount('close'), 0);
  }
});
test('close during cleanup cannot turn a timeout into success', async () => {
  const child = new EventEmitter(); child.pid = 789;
  await assert.rejects(waitForCommand(child, { timeoutMs: 5, cleanupTimeoutMs: 20,
    terminateProcess: async target => { target.emit('close', 0, null); } }), error => error.timedOut && error.cleanupConfirmed);
});
test('real cooperative parent and descendant clean up without leaking fixture processes', async () => {
  const code = `const {spawn}=require('node:child_process');
const descendant=spawn(process.execPath,['-e','setTimeout(()=>{},2000)'],{stdio:'ignore',windowsHide:true});
process.send({pid:descendant.pid});
process.on('message',value=>{if(value==='stop')descendant.kill();});
descendant.once('close',()=>process.exit(0));
setTimeout(()=>descendant.kill(),1500);`;
  const child = spawn(process.execPath, ['-e', code], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'], windowsHide: true });
  const closed = once(child, 'close');
  const [identity] = await once(child, 'message');
  assert.ok(Number.isInteger(identity.pid));
  try {
    await assert.rejects(waitForCommand(child, { timeoutMs: 50, cleanupTimeoutMs: 500,
      // Exercises a real descendant tree with a cooperative fixture provider;
      // this does not claim that the Windows taskkill provider succeeded.
      terminateProcess: async target => { target.send('stop'); }
    }), error => error.timedOut && error.cleanupConfirmed);
  } finally {
    if (child.connected) child.send('stop');
    let deadline;
    try { await Promise.race([closed, new Promise((_, reject) => { deadline = setTimeout(() => reject(new Error('Owned fixture cleanup deadline')), 3000); })]); }
    finally { clearTimeout(deadline); }
  }
  assert.ok(child.exitCode !== null || child.signalCode !== null);
  assert.throws(() => process.kill(identity.pid, 0), error => error.code === 'ESRCH');
});
