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
  green: (value: string) => value
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
      compiler.compiling = true;
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
  compiler.compiling = true;
  await engine.hmrUpdate({ path: source, type: 'added' }, true);
  await engine.hmrUpdate({ path: source, type: 'removed' }, true);
  compiler.compiling = false;
  await engine.recompileAndSendResult();
  expect(compiler.update).toHaveBeenCalledWith([
    { path: source, type: 'removed' }
  ]);
});

it('rechecks queue positions after an asynchronous stat while a batch drains', async () => {
  const { source, compiler, engine } = await fixture(true);
  const other = `${source}.other`;
  await fs.writeFile(other, 'export default 1');
  compiler.compiling = true;
  await engine.hmrUpdate(
    [
      { path: other, type: 'added' },
      { path: source, type: 'added' }
    ],
    true
  );
  let release!: (value: Awaited<ReturnType<typeof fs.stat>>) => void;
  const sourceStat = await fs.stat(source);
  vi.mocked(stat).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        release = resolve;
      }) as ReturnType<typeof stat>
  );
  const pending = engine.hmrUpdate({ path: source, type: 'removed' }, true);
  await engine.recompileAndSendResult();
  release(sourceStat);
  await pending;
  compiler.compiling = false;
  await engine.recompileAndSendResult();
  expect(compiler.update.mock.calls[1]?.[0]).toEqual([
    { path: source, type: 'removed' }
  ]);
});
