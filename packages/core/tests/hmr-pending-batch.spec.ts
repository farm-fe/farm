import { expect, it, vi } from 'vitest';
import { HmrEngine } from '../src/server/hmr-engine.js';

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
function fixture() {
  const barrier = deferred();
  const batches: unknown[] = [];
  const compiler = {
    compiling: true,
    waitForCompileFinish: vi.fn(async () => {
      if (compiler.compiling) await barrier.promise;
    }),
    hasModule: vi.fn(() => {
      expect(compiler.compiling).toBe(false);
      return true;
    }),
    getParentFiles: vi.fn(() => ['/parent.ts'])
  };
  const engine = new HmrEngine({ compiler } as any);
  vi.spyOn(engine, 'recompileAndSendResult').mockImplementation(async () => {
    batches.push([...(engine as any)._updateQueue]);
    (engine as any)._updateQueue = [];
    return undefined;
  });
  return {
    engine,
    compiler,
    batches,
    release() {
      compiler.compiling = false;
      barrier.resolve();
    }
  };
}

it('admits all changes before the barrier and prepares one batch', async () => {
  const { engine, compiler, batches, release } = fixture();
  const requests = ['/one.ts', '/two.ts', '/three.ts'].map((path) =>
    engine.queueWatchUpdate({ path, type: 'updated' })
  );
  expect(compiler.hasModule).not.toHaveBeenCalled();
  release();
  await Promise.all(requests);
  expect(batches).toEqual([
    [
      ...['/one.ts', '/two.ts', '/three.ts'].map((path) => ({
        path,
        type: 'updated'
      }))
    ]
  ]);
});

it('preserves same-path membership order rather than last-event wins', async () => {
  const { engine, batches, release } = fixture();
  const requests = (['added', 'removed', 'added'] as const).map((type) =>
    engine.queueWatchUpdate({ path: '/new.ts', type })
  );
  release();
  await Promise.all(requests);
  expect(batches).toEqual(
    ['added', 'removed', 'added'].map((type) => [{ path: '/new.ts', type }])
  );
});

it('captures unlink importers at the safe boundary before removal', async () => {
  const { engine, compiler, batches, release } = fixture();
  const request = engine.queueWatchRemove('/gone.ts');
  expect(compiler.getParentFiles).not.toHaveBeenCalled();
  release();
  await request;
  expect(batches).toEqual([
    [
      { path: '/gone.ts', type: 'removed' },
      { path: '/parent.ts', type: 'updated' }
    ]
  ]);
});

it('does not lose arrivals while the active watcher batch awaits a result', async () => {
  const { engine, release, batches } = fixture();
  const active = deferred();
  const started = deferred();
  vi.mocked(engine.recompileAndSendResult).mockImplementation(async () => {
    batches.push([...(engine as any)._updateQueue]);
    (engine as any)._updateQueue = [];
    if (batches.length === 1) {
      started.resolve();
      await active.promise;
    }
    return undefined;
  });
  const first = engine.queueWatchUpdate({ path: '/first.ts', type: 'updated' });
  release();
  await started.promise;
  const second = engine.queueWatchUpdate({
    path: '/second.ts',
    type: 'updated'
  });
  const third = engine.queueWatchUpdate({ path: '/third.ts', type: 'updated' });
  active.resolve();
  await Promise.all([first, second, third]);
  expect(batches).toEqual([
    [{ path: '/first.ts', type: 'updated' }],
    [
      { path: '/second.ts', type: 'updated' },
      { path: '/third.ts', type: 'updated' }
    ]
  ]);
});

it('rejects a preparation failure and still processes the following removal and addition', async () => {
  const { engine, release } = fixture();
  const failure = new Error('stat failed');
  const update = vi
    .spyOn(engine, 'hmrUpdate')
    .mockRejectedValueOnce(failure)
    .mockResolvedValue(undefined);
  const first = engine
    .queueWatchUpdate({ path: '/new.ts', type: 'added' })
    .catch((error) => error);
  const second = engine.queueWatchUpdate({ path: '/new.ts', type: 'removed' });
  const third = engine.queueWatchUpdate({ path: '/new.ts', type: 'added' });
  release();
  expect(await first).toBe(failure);
  await Promise.all([second, third]);
  expect(update.mock.calls.map(([items]) => items)).toEqual(
    ['added', 'removed', 'added'].map((type) => [{ path: '/new.ts', type }])
  );
});

it('preserves the public update path for plugin and controlled callers', async () => {
  const { engine, release } = fixture();
  const admission = vi.spyOn(engine, 'queueWatchUpdate');
  release();
  await engine.hmrUpdate(
    [
      { path: '/public.ts?one', type: 'updated' },
      { path: '/public.ts?two', type: 'updated' }
    ],
    true
  );
  expect(admission).not.toHaveBeenCalled();
});

it('does not strand a virtual public update when an independent compile wins the new continuation', async () => {
  const { engine, compiler, batches, release } = fixture();
  release();
  const external = deferred();
  const waiting = deferred();
  compiler.waitForCompileFinish.mockImplementation(async () => {
    if (compiler.compiling) {
      waiting.resolve();
      await external.promise;
    }
  });
  const update = engine.hmrUpdate({
    path: '/virtual-no-stat.ts',
    type: 'updated'
  });
  queueMicrotask(() => {
    compiler.compiling = true;
  });
  const checkpoint = await Promise.race([
    update.then(() => 'returned'),
    waiting.promise.then(() => 'waiting')
  ]);
  // Baseline can submit before the external request. v4 returns without submitting.
  const beforeRelease = batches.length;
  compiler.compiling = false;
  external.resolve();
  await update;
  expect(checkpoint === 'waiting' || beforeRelease === 1).toBe(true);
  expect(batches).toEqual([[{ path: '/virtual-no-stat.ts', type: 'updated' }]]);
});

it('submits prepared watcher items after an external compile without another file event', async () => {
  const { engine, compiler, batches, release } = fixture();
  release();
  const external = deferred();
  const waiting = deferred();
  const original = (engine as any).prepareHmrUpdate.bind(engine);
  vi.spyOn(engine as any, 'prepareHmrUpdate').mockImplementation(
    async (...args: unknown[]) => {
      await original(...args);
      compiler.compiling = true;
    }
  );
  compiler.waitForCompileFinish.mockImplementation(async () => {
    if (compiler.compiling) {
      waiting.resolve();
      await external.promise;
    }
  });
  const update = engine.queueWatchUpdate({
    path: '/watcher-no-stat.ts',
    type: 'updated'
  });
  const checkpoint = await Promise.race([
    update.then(() => 'returned'),
    waiting.promise.then(() => 'waiting')
  ]);
  compiler.compiling = false;
  external.resolve();
  await update;
  expect(checkpoint).toBe('waiting');
  expect(batches).toEqual([[{ path: '/watcher-no-stat.ts', type: 'updated' }]]);
});

it('settles the selected batch without waiting for later stream requests', async () => {
  const { engine, batches, release } = fixture();
  const firstStarted = deferred();
  const firstDone = deferred();
  const secondStarted = deferred();
  const secondDone = deferred();
  vi.mocked(engine.recompileAndSendResult).mockImplementation(async () => {
    batches.push([...(engine as any)._updateQueue]);
    (engine as any)._updateQueue = [];
    if (batches.length === 1) {
      firstStarted.resolve();
      await firstDone.promise;
    } else {
      secondStarted.resolve();
      await secondDone.promise;
    }
    return undefined;
  });
  const first = engine.queueWatchUpdate({ path: '/first.ts', type: 'updated' });
  release();
  await firstStarted.promise;
  let secondSettled = false;
  const second = engine
    .queueWatchUpdate({ path: '/later.ts', type: 'updated' })
    .then(() => {
      secondSettled = true;
    });
  firstDone.resolve();
  await first;
  expect(secondSettled).toBe(false);
  await secondStarted.promise;
  secondDone.resolve();
  await second;
});
