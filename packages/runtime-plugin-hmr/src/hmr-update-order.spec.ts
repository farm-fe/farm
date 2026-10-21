import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ModuleInitialization } from '../../runtime/src/module-system.js';

vi.mock('./overlay.js', () => ({ ErrorOverlay: class {}, overlayId: 'overlay' }));

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('window', { location: { port: '4397', hostname: 'localhost' } });
  vi.stubGlobal('location', { protocol: 'http:', reload: vi.fn() });
  vi.stubGlobal('document', { querySelectorAll: () => [] });
  for (const [name, value] of Object.entries({ FARM_HMR_HOST: false, FARM_HMR_PROTOCOL: '', FARM_HMR_PATH: '/__hmr',
    __FARM_RUNTIME_TARGET_ENV__: 'browser', __FARM_RUNTIME_TARGET_ENV_INJECTED_VALUE__: 'browser',
    __FARM_ENABLE_EXTERNAL_MODULES__: false, __FARM_ENABLE_RUNTIME_PLUGIN__: false, __FARM_ENABLE_TOP_LEVEL_AWAIT__: false })) vi.stubGlobal(name, value);
});
afterEach(() => vi.unstubAllGlobals());

function gate() {
  let release!: () => void;
  const promise = new Promise<void>(resolve => { release = resolve; });
  return { promise, release };
}

it('moves a full reload to the new server origin while retaining the current path query and fragment', async () => {
  const { HmrClient } = await import('./hmr-client.js');
  const assign = vi.fn(), reload = vi.fn();
  vi.stubGlobal('location', { href: 'http://localhost:9000//guide/docs/?query=one#section', assign, reload });
  const client = new HmrClient({} as never);
  await client.handleMessage({ type: 'full-reload', origin: 'https://localhost:9443/' });
  expect(assign).toHaveBeenCalledWith('https://localhost:9443//guide/docs/?query=one#section');
  expect(reload).not.toHaveBeenCalled();
});

it('restores factories caches and hot registrations after a controlled batch failure then accepts repair', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const oldFactory = ((_, exports) => { exports.version = 'old'; }) as ModuleInitialization;
  modules.g('component', oldFactory);
  modules.g('deleted', ((_, exports) => { exports.retained = true; }) as ModuleInitialization);
  const oldExports = modules.r('component'), deleted = modules.r('deleted');
  const client = new HmrClient(modules), hot = new HotModuleState('component', client);
  client.registeredHotModulesMap.set('component', hot);
  const events: string[] = [];
  client.registerUpdateParticipant('docs', async runtime => {
    expect(runtime).toBe(modules);
    return {
    validate() { events.push('validate'); },
    commit() { events.push('commit'); },
    rollback() { events.push('rollback'); },
    recover() { expect(modules.r('component')).toBe(oldExports); events.push('recover'); }
    };
  });
  const oldDispose = vi.fn(); client.disposeMap.set('component', oldDispose);
  const oldListener = () => {}; client.customListenersMap.set('retained', [oldListener]);
  hot.data = { retained: 'old' };
  const accept = vi.fn(async (next: { version: string }) => {
    if (next.version === 'bad') {
      hot.acceptCallbacks = [];
      (hot.data as { retained: string }).retained = 'bad';
      client.disposeMap.set('component', () => {});
      client.customListenersMap.set('retained', []);
      throw Error('candidate-render-failed');
    }
    events.push(next.version);
  });
  hot.accept(accept);
  const result = { added: ['new'], changed: ['component'], removed: ['deleted'],
    boundaries: { component: [['component']] }, immutableModules: '',
    mutableModules: "this.moduleSystem.u('component', (_, exports) => { exports.version = 'bad'; }); this.moduleSystem.g('new', () => {});",
    dynamicResources: null, dynamicModuleResourcesMap: null };
  await client.handleMessage({ type: 'farm-update', result });
  expect(events).toEqual(['rollback', 'recover']);
  expect(modules.m().component).toBe(oldFactory);
  expect(modules.r('deleted')).toBe(deleted);
  expect(modules.m().new).toBeUndefined();
  expect(hot.data).toEqual({ retained: 'old' });
  expect(client.disposeMap.get('component')).toBe(oldDispose);
  expect(client.customListenersMap.get('retained')).toEqual([oldListener]);
  await client.handleMessage({ type: 'farm-update', result: { ...result, added: [], removed: [],
    mutableModules: "this.moduleSystem.u('component', (_, exports) => { exports.version = 'fixed'; });" } });
  expect(accept).toHaveBeenCalledTimes(2);
  expect(events).toEqual(['rollback', 'recover', 'fixed', 'validate', 'commit']);
  expect(modules.r('component').version).toBe('fixed');
  expect(location.reload).not.toHaveBeenCalled();
});

it('prepares before factories change and restores modules before recovery on validation failure', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const original = ((_, exports) => { exports.version = 'old'; }) as ModuleInitialization;
  modules.g('component', original); modules.r('component');
  const client = new HmrClient(modules), hot = new HotModuleState('component', client);
  client.registeredHotModulesMap.set('component', hot); hot.accept(() => {});
  const paused = gate(), events: string[] = [];
  hot.registerUpdateParticipant(async () => {
    events.push('prepare'); await paused.promise;
    return {
      validate() { expect(modules.r('component').version).toBe('new'); events.push('validate'); throw Error('candidate-layout-failed'); },
      commit() { events.push('commit'); },
      rollback() { expect(modules.r('component').version).toBe('new'); events.push('rollback'); },
      recover() { expect(modules.r('component').version).toBe('old'); events.push('recover'); }
    };
  });
  client.customListenersMap.set('farm:afterUpdate', [() => { events.push('after'); }]);
  client.customListenersMap.set('farm:updateRolledBack', [() => { events.push('rolled-back'); }]);
  const update = client.handleMessage({ type: 'farm-update', result: { added: [], changed: ['component'], removed: [],
    boundaries: { component: [['component']] }, immutableModules: '',
    mutableModules: "this.moduleSystem.u('component', (_, exports) => { exports.version = 'new'; });",
    dynamicResources: null, dynamicModuleResourcesMap: null } });
  try {
    for (let i = 0; i < 8; i++) await Promise.resolve();
    expect(events).toEqual(['prepare']);
    expect(modules.m().component).toBe(original);
  } finally { paused.release(); await update; }
  expect(events).toEqual(['prepare', 'validate', 'rollback', 'recover', 'rolled-back']);
  expect(location.reload).not.toHaveBeenCalled();
});

it('attempts every cleanup and registry restoration then blocks later batches after cleanup failure', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const original = ((_, exports) => { exports.version = 'old'; }) as ModuleInitialization;
  modules.g('component', original); const old = modules.r('component');
  const client = new HmrClient(modules), hot = new HotModuleState('component', client);
  client.registeredHotModulesMap.set('component', hot);
  hot.accept(() => { throw Error('candidate failed'); });
  const events: string[] = [];
  client.registerUpdateParticipant('first', () => ({ rollback() { events.push('first'); }, recover() { events.push('recover'); } }));
  client.registerUpdateParticipant('second', () => ({ rollback() { events.push('second'); throw Error('cleanup failed'); } }));
  const payload = { type: 'farm-update' as const, result: { added: [], changed: ['component'], removed: [],
    boundaries: { component: [['component']] }, immutableModules: '',
    mutableModules: "this.moduleSystem.u('component', (_, exports) => { exports.version = 'bad'; });",
    dynamicResources: null, dynamicModuleResourcesMap: null } };
  await expect(client.handleMessage(payload)).rejects.toThrow();
  expect(events).toEqual(['second', 'first']);
  expect(modules.m().component).toBe(original);
  expect(modules.r('component')).toBe(old);
  await client.handleMessage(payload);
  expect(modules.m().component).toBe(original);
  expect(events).toEqual(['second', 'first']);
});

it('rejects an asynchronous finalizer and blocks later updates', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  modules.g('component', ((_, exports) => { exports.version = 'old'; }) as ModuleInitialization);
  const client = new HmrClient(modules), hot = new HotModuleState('component', client);
  client.registeredHotModulesMap.set('component', hot); hot.accept(() => {});
  const prepare = vi.fn(() => ({ async commit() { throw Error('async-finalizer-failed'); } }));
  client.registerUpdateParticipant('invalid', prepare);
  const payload = { type: 'farm-update' as const, result: { added: [], changed: ['component'], removed: [],
    boundaries: { component: [['component']] }, immutableModules: '', mutableModules: '',
    dynamicResources: null, dynamicModuleResourcesMap: null } };
  await expect(client.handleMessage(payload)).rejects.toThrow(/synchronous/);
  await client.handleMessage(payload);
  expect(prepare).toHaveBeenCalledTimes(1);
});

it('revalidates a participant when later validation admits new work', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  initModuleSystem(modules);
  const client = new HmrClient(modules), events: string[] = [];
  let revision = 0, validated = -1, changed = false;
  client.registerUpdateParticipant('first', () => ({
    validate() { validated = revision; events.push('validate:first'); },
    ready() { return validated === revision; },
    commit() { expect(validated).toBe(revision); events.push('commit:first'); }
  }));
  client.registerUpdateParticipant('second', () => ({
    validate() { if (!changed) { revision++; changed = true; } events.push('validate:second'); },
    commit() { events.push('commit:second'); }
  }));
  await client.handleMessage({ type: 'farm-update', result: { added: [], changed: [], removed: [], boundaries: {},
    immutableModules: '', mutableModules: '', dynamicResources: null, dynamicModuleResourcesMap: null } });
  expect(events).toEqual(['validate:first', 'validate:second', 'validate:first', 'validate:second', 'commit:first', 'commit:second']);
});

it('accepts a dependency once when overlapping update chains reach the same boundary', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  modules.g('application', ((_, exports) => { exports.version = 'new'; }) as ModuleInitialization);
  const client = new HmrClient(modules), hot = new HotModuleState('entry', client);
  client.registeredHotModulesMap.set('entry', hot);
  const accept = vi.fn(async () => {});
  hot.accept('application', accept);
  const other = new HotModuleState('other-entry', client), otherAccept = vi.fn(async () => {});
  other.accept('application', otherAccept);
  client.registeredHotModulesMap.set('other-entry', other);
  await client.handleMessage({ type: 'farm-update', result: {
    added: [], changed: ['wrapper', 'application'], removed: [],
    boundaries: { wrapper: [['wrapper', 'application', 'entry']], application: [['application', 'entry'], ['application', 'other-entry']] },
    immutableModules: '', mutableModules: '', dynamicResources: null, dynamicModuleResourcesMap: null
  } });
  expect(accept).toHaveBeenCalledTimes(1);
  expect(otherAccept).toHaveBeenCalledTimes(1);
  expect(location.reload).not.toHaveBeenCalled();
});

it.each(['before', 'dispose', 'accept'] as const)('farm afterUpdate follows the complete asynchronous %s phase', async phase => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const events: string[] = [], paused = gate();
  modules.g('component', ((_, exports) => { events.push('factory'); exports.version = 'new'; }) as ModuleInitialization);
  const client = new HmrClient(modules), hot = new HotModuleState('component', client);
  client.registeredHotModulesMap.set('component', hot);
  client.customListenersMap.set('farm:beforeUpdate', [async () => {
    events.push('before:start'); if (phase === 'before') await paused.promise; events.push('before:end');
  }]);
  client.disposeMap.set('component', async () => {
    events.push('dispose:start'); if (phase === 'dispose') await paused.promise; events.push('dispose:end');
  });
  hot.accept(async () => { events.push('accept:start'); if (phase === 'accept') await paused.promise; events.push('accept:end'); });
  client.customListenersMap.set('farm:afterUpdate', [() => { events.push('after'); }]);
  let settled = false;
  const work = client.handleMessage({ type: 'farm-update', result: { added: [], changed: ['component'], removed: [],
    boundaries: { component: [['component']] }, immutableModules: '', mutableModules: '', dynamicResources: null, dynamicModuleResourcesMap: null } }).then(() => { settled = true; });
  try {
    for (let attempt = 0; attempt < 20 && !events.includes(`${phase}:start`); attempt++) await Promise.resolve();
    expect(events).toContain(`${phase}:start`);
    expect(events).not.toContain('after');
    expect(settled).toBe(false);
  } finally {
    paused.release(); await work;
    // The old broken implementation returns before its detached apply task.
    await new Promise(resolve => setTimeout(resolve, 0));
  }
  expect(events).toEqual(['before:start', 'before:end', 'dispose:start', 'dispose:end', 'factory', 'accept:start', 'accept:end', 'after']);
  expect(location.reload).not.toHaveBeenCalled();
});

it('processes actual WebSocket messages in order while an asynchronous listener is paused', async () => {
  const { HmrClient } = await import('./hmr-client.js');
  class Socket {
    static OPEN = 1;
    listeners = new Map<string, (event: { data: string }) => void>();
    addEventListener(name: string, listener: (event: { data: string }) => void) { this.listeners.set(name, listener); }
    send() {}
    emit(id: number) { this.listeners.get('message')!({ data: JSON.stringify({ type: 'custom', event: 'sequence-probe', data: { id } }) }); }
  }
  vi.stubGlobal('WebSocket', Socket);
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const client = new HmrClient(modules), events: string[] = [], paused = gate();
  client.customListenersMap.set('sequence-probe', [async ({ id }) => {
    events.push(`start:${id}`); if (id === 1) await paused.promise; events.push(`end:${id}`);
  }]);
  const socket = client.connect() as unknown as Socket;
  socket.emit(1); socket.emit(2);
  try {
    for (let attempt = 0; attempt < 20; attempt++) await Promise.resolve();
    expect(events).toEqual(['start:1']);
  } finally {
    paused.release(); await new Promise(resolve => setTimeout(resolve, 0));
  }
  expect(events).toEqual(['start:1', 'end:1', 'start:2', 'end:2']);
});

it('settles a failed WebSocket message and continues with the following frame', async () => {
  const { HmrClient } = await import('./hmr-client.js');
  class Socket {
    listeners = new Map<string, (event: { data: string }) => void>();
    addEventListener(name: string, listener: (event: { data: string }) => void) { this.listeners.set(name, listener); }
    emit(id: number) { this.listeners.get('message')!({ data: JSON.stringify({ type: 'custom', event: 'failure-probe', data: { id } }) }); }
  }
  vi.stubGlobal('WebSocket', Socket);
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const client = new HmrClient(modules), events: string[] = [];
  vi.mocked(location.reload).mockImplementation(() => { events.push('reload'); });
  client.customListenersMap.set('failure-probe', [({ id }) => {
    events.push(`start:${id}`);
    if (id === 1) throw new Error('P08 deliberate listener failure');
    events.push(`end:${id}`);
  }]);
  const socket = client.connect() as unknown as Socket;
  socket.emit(1); socket.emit(2);
  for (let attempt = 0; attempt < 30; attempt++) await Promise.resolve();
  expect(events).toEqual(['start:1', 'reload', 'start:2', 'end:2']);
  expect(location.reload).toHaveBeenCalledTimes(1);
});
