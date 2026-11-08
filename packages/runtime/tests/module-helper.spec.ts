import { expect, test } from 'vitest';
import type { ModuleSystem } from '../src/module-system.js';
import {
  defineExportFrom,
  defineExportStar,
  initModuleSystem
} from '../src/modules/module-helper.js';

test('shared runtime exposes helpers required by opaque prebuilt bundles', () => {
  const cache: Record<string, any> = { existing: { exports: { value: 42 } } };
  const modules = { existing: () => 42 };
  const requireModule = () => cache.existing.exports;
  const loader = async () => {};
  const ms = { r: requireModule, c: () => cache, m: () => modules, l: loader } as unknown as ModuleSystem;
  initModuleSystem(ms);
  const helpers = ms.r as any;
  for (const key of ['o', 'd', '_m', '_e', 'i', '_g', 'w', '_', 'f']) {
    expect(helpers[key]).toBeTypeOf('function');
  }
  const exports = {};
  helpers._m(exports);
  helpers._(exports, 'button', { default: 'Button' }, 'default');
  helpers._e(exports, { other: 42, default: 'ignored' });
  expect(exports).toEqual({ button: 'Button', other: 42 });
  expect(helpers.i('value')).toEqual({ default: 'value' });
  expect(helpers.w({ named: 1 }).named).toBe(1);
  expect(ms.r).toBe(requireModule);
  expect(ms.m()).toBe(modules);
  expect(ms.c()).toBe(cache);
  expect(ms.c().existing.exports.value).toBe(42);
  expect(ms.l).toBe(loader);
});

test('reinitialization preserves the existing runtime helper cache entry', () => {
  const existing = { id: '@farm-runtime/module-helper', exports: { identity: true } };
  const cache = { [existing.id]: existing };
  const ms = { r: () => {}, c: () => cache } as unknown as ModuleSystem;
  initModuleSystem(ms);
  expect(ms.c()[existing.id]).toBe(existing);
});

test('named reexports retain live string-name bindings including an empty source name', () => {
  const source = { 's-k': 1, '': 2, alias: 99, plain: 3 };
  const target: Record<string, unknown> = {};
  defineExportFrom(target, 'public-name', source, 's-k');
  defineExportFrom(target, 'alias', source, '');
  defineExportFrom(target, '', source, 's-k');
  defineExportFrom(target, 'plain', source);
  expect(target).toEqual({ 'public-name': 1, alias: 2, '': 1, plain: 3 });
  source['s-k'] = 4;
  source[''] = 5;
  source.plain = 6;
  expect(target).toEqual({ 'public-name': 4, alias: 5, '': 4, plain: 6 });
});

test('star reexports retain live string-name bindings and preserve explicit exports', () => {
  const source = { 's-k': 1, '': 2, default: 3, explicit: 4 };
  const target: Record<string, unknown> = { explicit: 99 };
  defineExportStar(target, source);
  expect(target).toEqual({ 's-k': 1, '': 2, explicit: 99 });
  source['s-k'] = 5;
  source[''] = 6;
  expect(target).toEqual({ 's-k': 5, '': 6, explicit: 99 });
});
