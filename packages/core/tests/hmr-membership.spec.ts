import fs, { stat } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('node:fs/promises', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs/promises')>();
  return { ...actual, stat: vi.fn(actual.stat) };
});

vi.mock('../binding/index.js', () => ({
  Compiler: class {},
  Resolver: class {},
  JsFileWatcher: class {},
  bindingPath: ''
}));
vi.mock('../src/utils/index.js', () => ({
  bold: (value: string) => value,
  cyan: (value: string) => value,
  green: (value: string) => value,
  normalizePath: (value: string) => value.replaceAll('\\', '/')
}));

import { HmrEngine } from '../src/server/hmr-engine.js';

const roots: string[] = [];
afterEach(async () => {
  vi.restoreAllMocks();
  await Promise.all(
    roots.splice(0).map((root) => fs.rm(root, { recursive: true, force: true }))
  );
});

async function fixture(hasSource = false) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-hmr-membership-'));
  roots.push(root);
  const source = path.join(root, 'added.js');
  await fs.writeFile(source, "export default 'new';");
  const result = {
    added: ['added.js'],
    changed: ['index.js'],
    removed: [],
    immutableModules: '{}',
    mutableModules: '{"index.js": function() {}}',
    boundaries: {},
    extraWatchResult: { add: [], remove: [] }
  };
  const callbacks: (() => Promise<void>)[] = [];
  const compiler = {
    config: { root },
    compiling: false,
    waitForCompileFinish: vi.fn(async () => {}),
    hasModule: vi.fn(() => hasSource),
    transformModulePath: (_root: string, value: string) =>
      path.join(root, value),
    onUpdateFinish: vi.fn((callback: () => Promise<void>) =>
      callbacks.push(callback)
    ),
    update: vi.fn(async (_paths: unknown) => result),
    writeResourcesToDisk: vi.fn()
  };
  const send = vi.fn();
  const server = {
    compiler,
    config: { server: { hmr: { overlay: true } } },
    logger: { info: vi.fn(), error: vi.fn(), formatTime: () => '1ms' },
    ws: { wss: { clients: new Set([{ send }]) } }
  };
  return {
    source,
    result,
    compiler,
    send,
    callbacks,
    engine: new HmrEngine(server as never)
  };
}

it('sends compiler updates for newly added files absent from the old graph', async () => {
  const { source, compiler, send, engine } = await fixture();
  await engine.hmrUpdate({ path: source, type: 'added' });
  expect(compiler.update).toHaveBeenCalledWith([
    { path: source, type: 'added' }
  ]);
  expect(send).toHaveBeenCalledTimes(1);
  expect(send.mock.calls[0][0]).toContain("type: 'farm-update'");
  expect(send.mock.calls[0][0]).toContain("changed: ['index.js']");
});

it('does not rebuild an event again from the compiler completion callback', async () => {
  const { source, result, compiler, send, callbacks, engine } =
    await fixture(true);
  compiler.update.mockImplementation(async () => {
    if (compiler.update.mock.calls.length > 1)
      throw new Error('same event was compiled twice');
    const callback = callbacks.shift();
    await callback?.();
    return result;
  });
  await engine.hmrUpdate({ path: source, type: 'updated' });
  expect(compiler.update).toHaveBeenCalledTimes(1);
  expect(send).toHaveBeenCalledTimes(1);
  expect(send.mock.calls[0][0]).toContain("type: 'farm-update'");
});
it('sends consecutive versions in compile order when completion fires early', async () => {
  const { source, result, compiler, send, callbacks, engine } =
    await fixture(true);
  compiler.update.mockImplementation(async () => {
    if (compiler.update.mock.calls.length === 1) {
      // Native completion has fired, but the update result has not returned.
      // This is the supported reentrant admission window, not a held graph lock.
      compiler.compiling = false;
      await engine.hmrUpdate({ path: source, type: 'updated' }, true);
      compiler.compiling = false;
      await callbacks.shift()?.();
      return { ...result, mutableModules: 'version-A' };
    }
    return { ...result, mutableModules: 'version-B' };
  });
  await engine.hmrUpdate({ path: source, type: 'updated' }, true);
  expect(
    send.mock.calls.map(([message]) => message.match(/version-[AB]/)?.[0])
  ).toEqual(['version-A', 'version-B']);
});
it('retains a failed batch for retry on the next update', async () => {
  const { source, result, compiler, send, engine } = await fixture(true);
  compiler.update
    .mockRejectedValueOnce(new Error('invalid source'))
    .mockResolvedValue(result);
  await engine.hmrUpdate({ path: source, type: 'updated' }, true);
  await engine.hmrUpdate({ path: source, type: 'updated' }, true);
  expect(compiler.update).toHaveBeenCalledTimes(2);
  expect(send.mock.calls[0][0]).toContain("type: 'error'");
  expect(send.mock.calls[1][0]).toContain("type: 'farm-update'");
});
it('keeps a removal queued after an addition of the same path', async () => {
  const { source, compiler, engine } = await fixture(true);
  (engine as any)._recompileInProgress = true;
  await engine.hmrUpdate({ path: source, type: 'added' }, true);
  await engine.hmrUpdate({ path: source, type: 'removed' }, true);
  (engine as any)._recompileInProgress = false;
  await engine.recompileAndSendResult();
  expect(compiler.update).toHaveBeenCalledWith([
    { path: source, type: 'removed' }
  ]);
});

it('rechecks queue positions after an asynchronous stat while a batch drains', async () => {
  const { source, compiler, engine } = await fixture(true);
  const other = `${source}.other`;
  await fs.writeFile(other, 'export default 1');
  (engine as any)._recompileInProgress = true;
  await engine.hmrUpdate(
    [
      { path: other, type: 'added' },
      { path: source, type: 'added' }
    ],
    true
  );
  let release!: (value: Awaited<ReturnType<typeof fs.stat>>) => void;
  let entered!: () => void;
  const statEntered = new Promise<void>((resolve) => {
    entered = resolve;
  });
  const sourceStat = await fs.stat(source);
  vi.mocked(stat).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        release = resolve;
        entered();
      }) as ReturnType<typeof stat>
  );
  const pending = engine.hmrUpdate({ path: source, type: 'removed' }, true);
  await statEntered;
  (engine as any)._recompileInProgress = false;
  await engine.recompileAndSendResult();
  release(sourceStat);
  await pending;
  compiler.compiling = false;
  await engine.recompileAndSendResult();
  expect(compiler.update.mock.calls[1]?.[0]).toEqual([
    { path: source, type: 'removed' }
  ]);
});

it('rechecks native readiness after stat yields to an independent compilation', async () => {
  const { source, compiler, engine } = await fixture(true);
  const second = `${source}.second`;
  await fs.writeFile(second, 'export default 2');
  const sourceStat = await fs.stat(source);
  let releaseStat!: (value: typeof sourceStat) => void;
  let statEntered!: () => void;
  const entered = new Promise<void>((resolve) => {
    statEntered = resolve;
  });
  vi.mocked(stat).mockImplementationOnce(() => {
    statEntered();
    return new Promise((resolve) => {
      releaseStat = resolve;
    });
  });
  let releaseCompilation!: () => void;
  let readinessEntered!: () => void;
  const readiness = new Promise<void>((resolve) => {
    readinessEntered = resolve;
  });
  const background = new Promise<void>((resolve) => {
    releaseCompilation = resolve;
  });
  compiler.waitForCompileFinish.mockImplementation(async () => {
    if (compiler.compiling) {
      readinessEntered();
      await background;
    }
  });
  compiler.hasModule.mockImplementation(() => {
    if (compiler.compiling) throw new Error('native graph queried while held');
    return true;
  });
  const update = engine.hmrUpdate(
    [
      { path: source, type: 'updated' },
      { path: second, type: 'updated' }
    ],
    true
  );
  // Attach rejection handling before intentionally forcing the red interleaving.
  const outcome = update.then(
    () => undefined,
    (error) => error
  );
  await entered;
  compiler.compiling = true;
  releaseStat(sourceStat);
  // On old code outcome wins with the graph-lock error; candidate waits at readiness.
  const checkpoint = await Promise.race([
    readiness.then(() => 'waiting'),
    outcome.then((error) => error ?? 'finished')
  ]);
  compiler.compiling = false;
  releaseCompilation();
  expect(checkpoint).toBe('waiting');
  expect(await outcome).toBeUndefined();
  expect(compiler.hasModule).toHaveBeenCalledWith(second);
});

it('retries the retained prepared batch at unchanged mtime only after a new event', async () => {
  const { source, compiler, engine, result, send } = await fixture(true);
  let partialHookEffects = 0;
  compiler.update.mockImplementation(async () => {
    partialHookEffects++;
    if (partialHookEffects === 1)
      throw new Error('hook failed after side effect');
    return result;
  });
  await engine.hmrUpdate({ path: source, type: 'updated' });
  expect(partialHookEffects).toBe(1);
  expect(send.mock.calls[0][0]).toContain("type: 'error'");
  // No write, no mtime change, and no forced update. Existing prepared retry survives.
  await engine.hmrUpdate({ path: source, type: 'updated' });
  expect(partialHookEffects).toBe(2);
  expect(send.mock.calls[1][0]).toContain("type: 'farm-update'");
});

it.each([0, 1])(
  'does not lose Updated suffixes when stat fails at item %s',
  async (failedIndex) => {
    const { source, compiler, engine, result } = await fixture(true);
    const files = [source, `${source}.b`, `${source}.c`];
    for (const file of files.slice(1))
      await fs.writeFile(file, 'export default 1');
    const realStat = await fs.stat(source);
    const failure = Object.assign(
      new Error('removed between exists and stat'),
      { code: 'ENOENT' }
    );
    let fail = true;
    vi.mocked(stat).mockImplementation(async (file) => {
      if (String(file) === files[failedIndex] && fail) throw failure;
      return realStat;
    });
    let release!: () => void;
    const barrier = new Promise<void>((resolve) => {
      release = resolve;
    });
    compiler.compiling = true;
    compiler.waitForCompileFinish.mockImplementation(async () => {
      if (compiler.compiling) await barrier;
    });
    compiler.update.mockResolvedValue(result);
    const requests = files.map((path) =>
      engine.queueWatchUpdate({ path, type: 'updated' }).then(
        () => ({ ok: true }),
        (error) => ({ ok: false, error })
      )
    );
    const membership = engine.queueWatchUpdate({
      path: `${source}.new`,
      type: 'added'
    });
    compiler.compiling = false;
    release();
    const outcomes = await Promise.all(requests);
    await membership;
    expect(outcomes[failedIndex]).toEqual({ ok: false, error: failure });
    expect(outcomes.filter((_, index) => index !== failedIndex)).toEqual([
      { ok: true },
      { ok: true }
    ]);
    expect(compiler.update.mock.calls.map(([paths]) => paths)).toEqual([
      files
        .filter((_, index) => index !== failedIndex)
        .map((path) => ({ path, type: 'updated' })),
      [{ path: `${source}.new`, type: 'added' }]
    ]);
    fail = false;
    await engine.queueWatchUpdate({
      path: files[failedIndex],
      type: 'updated'
    });
    expect(compiler.update.mock.calls[2][0]).toEqual([
      { path: files[failedIndex], type: 'updated' }
    ]);
  }
);

it('keeps public arrays fail-fast instead of applying watcher per-item recovery', async () => {
  const { source, compiler, engine } = await fixture(true);
  const failure = Object.assign(new Error('missing while preparing'), {
    code: 'ENOENT'
  });
  vi.mocked(stat).mockRejectedValueOnce(failure);
  await expect(
    engine.hmrUpdate([
      { path: source, type: 'updated' },
      { path: '/unvisited-public-suffix.ts', type: 'updated' }
    ])
  ).rejects.toBe(failure);
  expect(compiler.hasModule).not.toHaveBeenCalledWith(
    '/unvisited-public-suffix.ts'
  );
  expect(compiler.update).not.toHaveBeenCalled();
});

it.each(['public', 'watcher'] as const)(
  'preserves early admission of a %s caller while a public recompile owns the result',
  async (caller) => {
    const { source, compiler, engine, result, send } = await fixture(true);
    const second = `${source}.second`;
    await fs.writeFile(second, 'export default 2');
    let started!: () => void;
    const entered = new Promise<void>((resolve) => {
      started = resolve;
    });
    let release!: () => void;
    const resultPending = new Promise<void>((resolve) => {
      release = resolve;
    });
    compiler.update.mockImplementation(async () => {
      if (compiler.update.mock.calls.length === 1) {
        // Model native completion before its result Promise reaches HmrEngine.
        // The graph is available, but _recompileInProgress still owns version A.
        compiler.compiling = false;
        started();
        await resultPending;
        return { ...result, mutableModules: 'version-A' };
      }
      return { ...result, mutableModules: 'version-B' };
    });
    let firstSettled = false;
    const first = engine
      .hmrUpdate({ path: source, type: 'updated' }, true)
      .then(() => {
        firstSettled = true;
      });
    await entered;
    const item = { path: second, type: 'updated' as const };
    const admitted =
      caller === 'public'
        ? engine.hmrUpdate(item, true)
        : engine.queueWatchUpdate(item);
    await admitted;
    expect(firstSettled).toBe(false);
    expect(compiler.update).toHaveBeenCalledTimes(1);
    release();
    await first;
    expect(compiler.update.mock.calls.map(([items]) => items)).toEqual([
      [{ path: source, type: 'updated' }],
      [{ path: second, type: 'updated' }]
    ]);
    expect(
      send.mock.calls.map(([message]) => message.match(/version-[AB]/)?.[0])
    ).toEqual(['version-A', 'version-B']);
  }
);
