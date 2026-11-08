import { beforeEach, expect, it, vi } from 'vitest';
import { Compiler } from '../src/compiler/index.js';

const native = vi.hoisted(() => ({
  calls: [] as Array<{
    done: (error?: Error | null) => void;
    resolve: (value: unknown) => void;
    reject: (error: Error) => void;
  }>,
  inline: false
}));
vi.mock('../binding/index.js', () => ({
  Compiler: class {
    update(_paths: unknown, done: (error?: Error | null) => void) {
      return new Promise((resolve, reject) => {
        native.calls.push({ done, resolve, reject });
        if (native.inline) {
          done(null);
          resolve({});
        }
      });
    }
  }
}));
beforeEach(() => {
  native.calls = [];
  native.inline = false;
});
const turn = async () => {
  for (let i = 0; i < 8; i++) await Promise.resolve();
};
it.each(['native-first', 'completion-first'])(
  'waits for both success signals (%s)',
  async (order) => {
    const compiler = new Compiler({ compilation: {} } as any);
    let settled = false;
    const result = compiler.update([]).then(() => {
      settled = true;
    });
    const queued = compiler.update([]);
    const call = native.calls[0];
    if (order === 'native-first') call.resolve({});
    else call.done(null);
    await turn();
    expect(settled).toBe(false);
    expect(native.calls).toHaveLength(1);
    if (order === 'native-first') call.done(null);
    else call.resolve({});
    await result;
    expect(native.calls).toHaveLength(2);
    native.calls[1].done(null);
    native.calls[1].resolve({});
    await queued;
    await compiler.waitForCompileFinish();
    expect(compiler.compiling).toBe(false);
  }
);
it.each(['native-first', 'error-first', 'native-reject-no-callback'])(
  'rejects the active and queued callers, clears listeners and recovers (%s)',
  async (order) => {
    const compiler = new Compiler({ compilation: {} } as any);
    const listener = vi.fn();
    compiler.onUpdateFinish(listener);
    const active = compiler.update([]).catch((error: Error) => error.message);
    const queued = compiler.update([]).catch((error: Error) => error.message);
    const call = native.calls[0];
    const error = new Error('late discovery failure');
    if (order === 'native-first') {
      call.resolve({});
      await turn();
      call.done(error);
    } else if (order === 'error-first') {
      call.done(error);
      await turn();
      call.resolve({});
    } else call.reject(error);
    expect(await active).toBe(error.message);
    expect(await queued).toBe(error.message);
    await compiler.waitForCompileFinish();
    expect(listener).not.toHaveBeenCalled();
    call.done(null);
    call.done(error);
    await turn();
    native.inline = true;
    await compiler.update([]);
    expect(listener).not.toHaveBeenCalled();
    expect(compiler.compiling).toBe(false);
  }
);
it('supports an inline completion without a callback/result deadlock', async () => {
  native.inline = true;
  const compiler = new Compiler({ compilation: {} } as any);
  await compiler.update([]);
  await compiler.waitForCompileFinish();
  expect(compiler.compiling).toBe(false);
});

it('a rejecting success listener cannot clear a newer native attempt', async () => {
  const logger = { error: vi.fn() };
  const compiler = new Compiler({ compilation: {}, logger } as any);
  let newer: Promise<unknown> | undefined;
  compiler.onUpdateFinish(() => {
    newer = compiler.update([]);
    return Promise.reject(new Error('listener failed'));
  });
  const active = compiler.update([]).catch((error: Error) => error.message);
  native.calls[0].resolve({});
  native.calls[0].done(null);
  expect(await active).toEqual({});
  await turn();
  expect(logger.error).toHaveBeenCalledWith(new Error('listener failed'), {
    exit: false
  });
  expect(compiler.compiling).toBe(true);
  const listener = vi.fn();
  compiler.onUpdateFinish(listener);
  const queued = compiler.update([]);
  let ready = false;
  const waiting = compiler.waitForCompileFinish().then(() => {
    ready = true;
  });
  await turn();
  expect(ready).toBe(false);
  expect(native.calls).toHaveLength(2);
  native.calls[1].resolve({});
  native.calls[1].done(null);
  await newer;
  expect(native.calls).toHaveLength(3);
  expect(listener).not.toHaveBeenCalled();
  native.calls[2].resolve({});
  native.calls[2].done(null);
  await queued;
  await waiting;
  expect(listener).toHaveBeenCalledTimes(1);
  expect(ready).toBe(true);
});

it('a success listener may await the successful update without a cycle', async () => {
  const compiler = new Compiler({ compilation: {} } as any);
  const active = compiler.update([]);
  let observed!: () => void;
  const finished = new Promise<void>((resolve) => {
    observed = resolve;
  });
  compiler.onUpdateFinish(async () => {
    await active;
    observed();
  });
  native.calls[0].resolve({});
  native.calls[0].done(null);
  await active;
  await finished;
  expect(compiler.compiling).toBe(false);
});
