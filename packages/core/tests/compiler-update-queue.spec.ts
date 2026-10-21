import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Compiler } from '../src/compiler/index.js';
import type { ResolvedUserConfig } from '../src/index.js';

const binding = vi.hoisted(() => ({
  callbacks: [] as Array<() => Promise<void>>,
  calls: [] as Array<{ sync: boolean; generate: boolean }>,
  fail: false,
  failCall: 0,
  inline: false,
  gate: undefined as Promise<void> | undefined
}));

vi.mock('../binding/index.js', () => ({
  Compiler: class {
    async update(
      _paths: unknown,
      callback: () => Promise<void>,
      sync: boolean,
      generate: boolean
    ) {
      binding.calls.push({ sync, generate });
      if (binding.calls.length === 1 && binding.gate) await binding.gate;
      if (binding.fail || binding.calls.length === binding.failCall) {
        throw new Error('Native update failed');
      }
      if (binding.inline) await callback();
      else binding.callbacks.push(callback);
      return {};
    }
  }
}));

beforeEach(() => {
  binding.callbacks = [];
  binding.calls = [];
  binding.fail = false;
  binding.failCall = 0;
  binding.inline = false;
  binding.gate = undefined;
});

async function finish() {
  const callback = binding.callbacks.shift();
  if (!callback) throw new Error('Missing completion callback');
  await callback();
}

describe('queued compiler updates', () => {
  it('allows recovery when completion precedes the active binding result', async () => {
    let release!: () => void;
    binding.gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    binding.inline = true;
    binding.failCall = 2;
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    const active = compiler.update([]);
    const failed = compiler.update([]).catch(async (error: Error) => {
      expect(error.message).toBe('Native update failed');
      await compiler.update([], false, false, false);
    });
    release();
    await Promise.all([active, failed]);
    await compiler.waitForCompileFinish();
    expect(binding.calls).toHaveLength(3);
    expect(binding.calls[2]).toEqual({ sync: false, generate: false });
    expect(compiler.compiling).toBe(false);
  });

  it('preserves each request resource-generation and scheduling options', async () => {
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    await compiler.update([], true, false, false);
    const queued = compiler.update([], false, false, true);
    await finish();
    await queued;
    expect(binding.calls).toEqual([
      { sync: true, generate: false },
      { sync: false, generate: true }
    ]);
    await finish();
    await compiler.waitForCompileFinish();
  });

  it('rejects queued callers when their native update fails', async () => {
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    await compiler.update([]);
    let outcome = 'pending';
    const queued = compiler.update([]).then(
      () => {
        outcome = 'resolved';
      },
      (error: Error) => {
        outcome = error.message;
      }
    );
    let remaining = 'pending';
    const tail = compiler.update([]).catch((error: Error) => {
      remaining = error.message;
    });
    binding.fail = true;
    // A native completion callback must not produce an unhandled rejection.
    const completion = await finish().then(
      () => undefined,
      (error: Error) => error.message
    );
    expect(outcome).toBe('Native update failed');
    expect(completion).toBeUndefined();
    expect(remaining).toBe('Native update failed');
    await queued;
    await tail;
    await compiler.waitForCompileFinish();
    expect(compiler.compiling).toBe(false);
    binding.fail = false;
    await compiler.update([]);
    await finish();
    expect(compiler.compiling).toBe(false);
  });

  it('rejects waiting callers when the active update fails before completion', async () => {
    let fail!: (error: Error) => void;
    binding.gate = new Promise<void>((_resolve, reject) => {
      fail = reject;
    });
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    const active = compiler.update([]).catch((error: Error) => error.message);
    let outcome = 'pending';
    const queued = compiler.update([]).catch((error: Error) => {
      outcome = error.message;
    });
    fail(new Error('Active update failed'));
    expect(await active).toBe('Active update failed');
    expect(outcome).toBe('Active update failed');
    await queued;
    await compiler.waitForCompileFinish();
    expect(compiler.compiling).toBe(false);
  });
});
