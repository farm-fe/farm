import { expect, test } from 'vitest';
import type { Module, ModuleSystem } from '../../runtime/src/module-system.js';
import { initModuleSystem } from '../../runtime/src/modules/module-helper.js';
import { captureUpdateSnapshot } from './update-transaction.js';

function fixture() {
  const owner: Module = { id: 'origin' };
  const cache: Record<string, Module> = { origin: owner };
  const factories = { origin: () => {} };
  const ms = { r: () => {}, c: () => cache, m: () => factories } as unknown as ModuleSystem;
  initModuleSystem(ms);
  const bridge = (ms.r as any).entryBindings;
  return { owner, cache, ms, bridge };
}

test('synchronous cell writes notify aliases and preserve postfix and compound values', () => {
  const { owner, bridge } = fixture();
  let value = 1;
  const cell = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  let named: unknown;
  let alias: unknown;
  bridge.subscribe('origin:value', (next: unknown) => named = next, value);
  const stop = bridge.subscribe('origin:value', (next: unknown) => alias = next, value);
  expect(cell.value++).toBe(1);
  expect([value, named, alias]).toEqual([2, 2, 2]);
  stop();
  cell.value += 3;
  expect([value, named, alias]).toEqual([5, 5, 2]);
});

test('real HMR snapshot restores lexical values, facade subscriptions and old generation', () => {
  const { owner, cache, ms, bridge } = fixture();
  let value = 1;
  const old = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  let named: unknown;
  bridge.subscribe('origin:value', (next: unknown) => named = next, value);
  bridge.complete(owner, ['origin:value']);
  const restore = captureUpdateSnapshot(ms, new Map(), [], new Map());
  old.value = 3;
  const candidate: Module = { id: 'origin' };
  cache.origin = candidate;
  let nextValue = 10;
  const next = bridge.bind('origin:value', candidate, () => nextValue, (next: number) => nextValue = next);
  next.value = 11;
  expect(named).toBe(11);
  old.value = 4;
  expect(named).toBe(11);
  let candidateNotifications = 0;
  bridge.subscribe('origin:value', () => candidateNotifications++, nextValue);
  restore();
  expect(cache.origin).toBe(owner);
  expect([value, named]).toEqual([1, 1]);
  next.value = 12;
  expect(named).toBe(1);
  old.value = 2;
  expect(named).toBe(2);
  expect(candidateNotifications).toBe(1);
});

test('cache deletion stops publication and helper reinitialization preserves subscriptions', () => {
  const { owner, cache, ms, bridge } = fixture();
  let value = 1;
  const cell = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  let named: unknown;
  bridge.subscribe('origin:value', (next: unknown) => named = next, value);
  initModuleSystem(ms);
  expect((ms.r as any).entryBindings).toBe(bridge);
  delete cache.origin;
  cell.value = 2;
  expect(value).toBe(2);
  expect(named).toBe(1);
});

test('late old factory registration cannot steal the current generation', async () => {
  const { owner, cache, bridge } = fixture();
  let oldValue = 1;
  let resume!: () => void;
  const gate = new Promise<void>(resolve => { resume = resolve; });
  const late = (async () => { await gate; return bridge.bind('origin:value', owner, () => oldValue, (next: number) => oldValue = next); })();
  const nextOwner = { id: 'origin' };
  cache.origin = nextOwner;
  let nextValue = 10;
  const next = bridge.bind('origin:value', nextOwner, () => nextValue, (value: number) => nextValue = value);
  let slot: unknown;
  bridge.subscribe('origin:value', (value: unknown) => slot = value, nextValue);
  next.value = 11;
  resume();
  const stale = await late;
  stale.value = 2;
  next.value = 12;
  expect([oldValue, slot]).toEqual([2, 12]);
});

test('unsubscribe after rollback targets its restored token and leaves other listeners intact', () => {
  const { owner, bridge } = fixture();
  let value = 1;
  const cell = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  let first: unknown;
  let second: unknown;
  const stop = bridge.subscribe('origin:value', (next: unknown) => first = next, value);
  const restore = bridge.snapshot();
  restore();
  bridge.subscribe('origin:value', (next: unknown) => second = next, value);
  stop();
  cell.value = 2;
  expect([first, second]).toEqual([1, 2]);
});

test('rollback restores native slot independently of a cleared source owner', () => {
  const { owner, cache, ms, bridge } = fixture();
  let value = 1;
  const old = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  let slot: unknown;
  bridge.subscribe('origin:value', (next: unknown) => slot = next, value);
  old.value = 1;
  delete cache.origin;
  old.value = 2;
  const restore = captureUpdateSnapshot(ms, new Map(), [], new Map());
  const candidate = { id: 'origin' };
  cache.origin = candidate;
  let candidateValue = 10;
  bridge.bind('origin:value', candidate, () => candidateValue, (next: number) => candidateValue = next).value = 11;
  expect(slot).toBe(11);
  restore();
  expect(cache.origin).toBeUndefined();
  expect([value, slot]).toEqual([2, 1]);
});

test('uninitialized and failed-write cells never make snapshot read or restore unsafe state', () => {
  const { owner, bridge } = fixture();
  const cell = bridge.bind('origin:value', owner, () => value, (next: number) => value = next);
  const restore = bridge.snapshot();
  expect(() => restore()).not.toThrow();
  let value = 1;
  const rejected = bridge.bind('origin:constant', owner, () => { throw new Error('uninitialized'); }, () => { throw new TypeError('constant assignment'); });
  expect(() => { rejected.value = 2; }).toThrow(TypeError);
  expect(() => bridge.snapshot()()).not.toThrow();
  cell.value = 2;
  expect(value).toBe(2);
});


test('completion visits only supplied keys, including with no consumers', () => {
  const { cache, bridge } = fixture();
  let reads = 0;
  const owners = Array.from({ length: 100 }, (_, index) => ({ id: `module-${index}` }));
  for (const owner of owners) {
    cache[owner.id] = owner;
    bridge.bind(owner.id, owner, () => { reads++; return 1; }, () => {});
  }
  const original = Map.prototype[Symbol.iterator];
  let iterations = 0;
  Map.prototype[Symbol.iterator] = function () { iterations++; return original.call(this); };
  try {
    for (const owner of owners) bridge.complete(owner, [owner.id]);
    bridge.complete(owners[0], [owners[1].id, 'missing']);
  } finally {
    Map.prototype[Symbol.iterator] = original;
  }
  expect(reads).toBe(100);
  expect(iterations).toBe(0);
});
