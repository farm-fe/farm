import { describe, expect, it, vi } from 'vitest';
import { Compiler } from '../src/compiler/index.js';
import type { ResolvedUserConfig } from '../src/index.js';

const binding = vi.hoisted(() => ({
  callbacks: [] as Array<() => Promise<void>>
}));
vi.mock('../binding/index.js', () => ({
  Compiler: class {
    async compile() {}
    async update(_paths: unknown, callback: () => Promise<void>) {
      binding.callbacks.push(callback);
      return {};
    }
  }
}));

async function finishResources() {
  const callback = binding.callbacks.shift();
  if (!callback) throw new Error('Missing resource regeneration callback');
  await callback();
}

describe('resource readiness during incremental compilation', () => {
  it('waits again when an update-finish listener starts another compilation', async () => {
    binding.callbacks = [];
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    compiler.onUpdateFinish(() => {
      void compiler.update([]);
    });
    await compiler.update([]);
    let ready = false;
    const waiting = compiler.waitForCompileFinish().then(() => {
      ready = true;
    });
    await finishResources();
    await Promise.resolve();
    expect(ready).toBe(false);
    await finishResources();
    await waiting;
    expect(ready).toBe(true);
  });
  it('waits for resource regeneration after the update result becomes available', async () => {
    binding.callbacks = [];
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    await compiler.compile();
    await compiler.update([]);
    let ready = false;
    const waiting = compiler.waitForCompileFinish().then(() => {
      ready = true;
    });
    await Promise.resolve();
    await Promise.resolve();
    expect(ready).toBe(false);
    await finishResources();
    await waiting;
    expect(ready).toBe(true);
    expect(compiler.compiling).toBe(false);
  });

  it('keeps resource readers waiting through a queued update', async () => {
    binding.callbacks = [];
    const compiler = new Compiler({ compilation: {} } as ResolvedUserConfig);
    await compiler.update([]);
    const queued = compiler.update([]);
    let ready = false;
    const waiting = compiler.waitForCompileFinish().then(() => {
      ready = true;
    });
    await finishResources();
    await queued;
    expect(ready).toBe(false);
    await finishResources();
    await waiting;
    expect(ready).toBe(true);
  });
});
