import { afterEach, expect, test, vi } from 'vitest';
import type { ModuleSystem } from '../src/module-system.js';

afterEach(() => {
  vi.unstubAllGlobals();
});

test('a detached dynamic resource snapshot restores later lazy imports', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const loaded: string[] = [];
  const system = {
    m: () => ({ page: () => {} }),
    r: () => ({ page: 'exports' }),
    p: { b: (_hook: string, resource: { path: string }) => {
      loaded.push(resource.path);
      return Promise.resolve({ success: true });
    } }
  } as unknown as ModuleSystem;
  initModuleSystem(system);
  system.sd([{ path: 'committed.js', type: 0 }], { page: [0] });
  const committed = system.gd();
  const unrelated = system.gd();
  unrelated.resources[0].path = 'tampered.js';
  unrelated.moduleResources.page.push(100);
  system.sd([{ path: 'candidate.js', type: 0 }], { page: [0] });
  await system.d('page');
  system.sd(committed.resources, committed.moduleResources);
  committed.resources[0].path = 'mutated-after-restore.js';
  committed.moduleResources.page.push(200);
  await system.d('page');
  expect(loaded).toEqual(['candidate.js', 'committed.js']);
  expect(system.gd()).toEqual({ resources: [{ path: 'committed.js', type: 0 }], moduleResources: { page: [0] } });
});

test('resource rollback drains candidate loading and permits reloading its discarded path', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  let finish!: (result: { success: boolean }) => void;
  const loaded: string[] = [];
  const system = {
    m: () => ({ page: () => {} }), r: () => ({}),
    p: { b: (_hook: string, resource: { path: string }) => {
      loaded.push(resource.path);
      if (loaded.length === 1) return new Promise(resolve => { finish = resolve; });
      return Promise.resolve({ success: true });
    } }
  } as unknown as ModuleSystem;
  initModuleSystem(system);
  system.sd([{ path: 'old.js', type: 0 }], { page: [0] });
  const transaction = await system.beginResourceTransaction();
  await expect(system.beginResourceTransaction()).rejects.toThrow(/active/);
  system.sd([{ path: 'discarded.js', type: 0 }], { page: [0] });
  const candidate = system.d('page');
  let restored = false;
  const rollback = transaction.rollback().then(errors => { restored = true; return errors; });
  await Promise.resolve();
  expect(restored).toBe(false);
  finish({ success: true });
  await candidate;
  expect(await rollback).toEqual([]);
  expect(system.gd().resources[0].path).toBe('old.js');
  system.sd([{ path: 'discarded.js', type: 0 }], { page: [0] });
  await system.d('page');
  expect(loaded).toEqual(['discarded.js', 'discarded.js']);
  await expect(transaction.commit()).rejects.toThrow(/closed/);
});

test('resource commit keeps successful loaded flags while rollback keeps previously loaded flags', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const loaded: string[] = [];
  const system = { m: () => ({ page: () => {} }), r: () => ({}), p: { b: (_: string, resource: { path: string }) => {
    loaded.push(resource.path); return Promise.resolve({ success: true });
  } } } as unknown as ModuleSystem;
  initModuleSystem(system);
  system.sd([{ path: 'already.js', type: 0 }], { page: [0] });
  await system.d('page');
  const candidate = await system.beginResourceTransaction();
  system.sd([{ path: 'committed.js', type: 0 }], { page: [0] });
  await system.d('page'); await candidate.commit(); await system.d('page');
  const failed = await system.beginResourceTransaction();
  system.sd([{ path: 'already.js', type: 0 }], { page: [0] });
  await failed.rollback(); await system.d('page');
  expect(loaded).toEqual(['already.js', 'committed.js']);
});

test('rollback removes only owned script/style elements after their requests settle', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', false);
  const elements: Array<{ tag: string; removed: boolean; onload(): void; remove(): void }> = [];
  vi.stubGlobal('document', {
    createElement(tag: string) {
      const element = { tag, removed: false, onload() {}, remove() { this.removed = true; } };
      elements.push(element); return element;
    }, body: { appendChild() {} }, head: { appendChild() {} }
  });
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const system = { m: () => ({ page: () => {} }), r: () => ({}) } as unknown as ModuleSystem;
  initModuleSystem(system); system.sp(['/']);
  system.sd([{ path: 'page.js', type: 0 }, { path: 'page.css', type: 1 }], { page: [0, 1] });
  const transaction = await system.beginResourceTransaction();
  const loading = system.l('page');
  const rollback = transaction.rollback();
  expect(elements.map(element => element.removed)).toEqual([false, false]);
  elements.forEach(element => element.onload());
  await loading; await rollback;
  expect(elements.map(element => element.removed)).toEqual([true, true]);
  const retry = system.l('page');
  expect(elements).toHaveLength(4);
  elements.slice(2).forEach(element => element.onload());
  await retry;
  expect(elements.slice(2).every(element => !element.removed)).toBe(true);
});

test('resource commit failure can be rolled back with its diagnostic retained', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const failure = Error('candidate-request-failed');
  const system = { m: () => ({ page: () => {} }), p: { b: () => Promise.reject(failure) } } as unknown as ModuleSystem;
  initModuleSystem(system); system.sd([{ path: 'bad.js', type: 0 }], { page: [0] });
  const transaction = await system.beginResourceTransaction();
  await expect(system.l('page')).rejects.toBe(failure);
  await expect(transaction.commit()).rejects.toThrow(/transaction failed/);
  expect(await transaction.rollback()).toEqual([failure]);
  const next = await system.beginResourceTransaction();
  await next.commit();
});

test('an early resource rejection cannot let rollback overtake another pending resource', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  let finish!: (result: { success: boolean }) => void;
  const failure = Error('script failed first');
  const loaded: string[] = [];
  let attempt = 0;
  const system = { m: () => ({ page: () => {} }), p: { b: (_: string, resource: { path: string }) => {
    loaded.push(resource.path);
    if (++attempt === 1) return Promise.reject(failure);
    if (attempt === 2) return new Promise(resolve => { finish = resolve; });
    return Promise.resolve({ success: true });
  } } } as unknown as ModuleSystem;
  initModuleSystem(system);
  system.sd([{ path: 'failed.js', type: 0 }, { path: 'late.css', type: 1 }], { page: [0, 1] });
  const transaction = await system.beginResourceTransaction();
  await expect(system.l('page')).rejects.toBe(failure);
  let settled = false;
  const rollback = transaction.rollback().then(errors => { settled = true; return errors; });
  try {
    for (let i = 0; i < 8; i++) await Promise.resolve();
    expect(settled).toBe(false);
  } finally {
    finish({ success: true });
    await rollback;
  }
  await system.l('page');
  expect(loaded).toEqual(['failed.js', 'late.css', 'failed.js', 'late.css']);
});

test('resource finalization is synchronous and requires validation of the latest work', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const system = { m: () => ({ page: () => {} }), p: { b: () => Promise.resolve({ success: true }) } } as unknown as ModuleSystem;
  initModuleSystem(system); system.sd([{ path: 'late.js', type: 0 }], { page: [0] });
  const transaction = await system.beginResourceTransaction();
  expect(() => transaction.finalize()).toThrow(/validated/);
  await transaction.validate();
  await system.l('page');
  expect(() => transaction.finalize()).toThrow(/validated/);
  await transaction.rollback();
  const next = await system.beginResourceTransaction();
  await next.validate();
  expect(next.finalize()).toBeUndefined();
  await expect(next.rollback()).rejects.toThrow(/closed/);
});

test('opening a resource transaction rechecks loads started in its empty-drain microtask gap', async () => {
  vi.resetModules();
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', false);
  const elements: Array<{ removed: boolean; onload(): void; remove(): void }> = [];
  vi.stubGlobal('document', { createElement() {
    const element = { removed: false, onload() {}, remove() { this.removed = true; } };
    elements.push(element); return element;
  }, body: { appendChild() {} } });
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const system = { m: () => ({ page: () => {} }) } as unknown as ModuleSystem;
  initModuleSystem(system); system.sp(['/']);
  system.sd([{ path: 'prior.js', type: 0 }], { page: [0] });
  let opened = false;
  const opening = system.beginResourceTransaction().then(transaction => { opened = true; return transaction; });
  const loading = system.l('page');
  try {
    for (let i = 0; i < 8; i++) await Promise.resolve();
    expect(opened).toBe(false);
  } finally {
    elements[0].onload(); await loading;
    const transaction = await opening;
    await transaction.rollback();
  }
  expect(elements[0].removed).toBe(false);
  await system.l('page');
  expect(elements).toHaveLength(1);
});

test('settling permits synchronous restoration and later work invalidates readiness', async () => {
  vi.resetModules(); vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser'); vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', true);
  const { initModuleSystem } = await import('../src/modules/dynamic-import.js');
  const system = { m: () => ({ page: () => {} }), p: { b: () => Promise.resolve({ success: true }) } } as unknown as ModuleSystem;
  initModuleSystem(system); system.sd([{ path: 'candidate.js', type: 0 }], { page: [0] });
  const transaction = await system.beginResourceTransaction();
  await transaction.settle(); expect(transaction.isSettled()).toBe(true);
  await system.l('page'); expect(transaction.isSettled()).toBe(false);
  expect(() => transaction.restore()).toThrow(/settled/);
  await transaction.settle(); expect(transaction.restore()).toEqual([]);
  await expect(transaction.rollback()).rejects.toThrow(/closed/);
});
