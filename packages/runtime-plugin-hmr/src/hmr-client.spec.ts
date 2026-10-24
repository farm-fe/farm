import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type { ModuleInitialization } from '../../runtime/src/module-system.js';

vi.mock('./overlay.js', () => ({ ErrorOverlay: class {}, overlayId: 'overlay' }));

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('window', { location: { port: '4397', hostname: 'localhost' } });
  vi.stubGlobal('location', { protocol: 'http:', reload: vi.fn() });
  vi.stubGlobal('FARM_HMR_HOST', false);
  vi.stubGlobal('FARM_HMR_PROTOCOL', '');
  vi.stubGlobal('FARM_HMR_PATH', '/__hmr');
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV__', 'browser');
  vi.stubGlobal('__FARM_RUNTIME_TARGET_ENV_INJECTED_VALUE__', 'browser');
  vi.stubGlobal('__FARM_ENABLE_EXTERNAL_MODULES__', false);
  vi.stubGlobal('__FARM_ENABLE_RUNTIME_PLUGIN__', false);
  vi.stubGlobal('__FARM_ENABLE_TOP_LEVEL_AWAIT__', false);
});
afterEach(() => { vi.unstubAllGlobals(); });

it.each(['component-first', 'dependency-first'])('invalidates the whole HMR batch before evaluating a boundary (%s)', async order => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { HotModuleState } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const registerDomain = (extra: number) => modules.g('domain', ((_, exports) => {
    exports.receive = (stock: number) => stock + 25 + extra;
  }) as ModuleInitialization);
  const registerComponent = () => modules.g('component', ((_, exports, require) => {
    const domain = require('domain');
    exports.receive = (stock: number) => domain.receive(stock);
  }) as ModuleInitialization);
  registerDomain(0); registerComponent();
  expect(modules.r('component').receive(17)).toBe(42);
  registerDomain(100); registerComponent();
  const client = new HmrClient(modules);
  const accepted: number[] = [];
  const hot = new HotModuleState('component', client);
  hot.accept((component: { receive(stock: number): number }) => accepted.push(component.receive(17)));
  client.registeredHotModulesMap.set('component', hot);
  const component = ['component'];
  const dependency = ['domain', 'component'];
  const boundaries = order === 'component-first'
    ? { component: [component], domain: [dependency] }
    : { domain: [dependency], component: [component] };
  await client.applyHotUpdates({ added: [], changed: ['component', 'domain'], removed: [], boundaries, dynamicResources: null, dynamicModuleResourcesMap: null }, modules);
  expect(accepted).toEqual([142]);
  expect(location.reload).not.toHaveBeenCalled();
});

it('disposes old resources and calls old accept callbacks before dependent factories replace hot contexts', async () => {
  const { __farm_internal_module_system__: modules } = await import('../../runtime/src/module-system.js');
  const { initModuleSystem } = await import('../../runtime/src/modules/module-system-helper.js');
  const { HmrClient } = await import('./hmr-client.js');
  const { createHotContext } = await import('./hot-module-state.js');
  initModuleSystem(modules);
  const client = new HmrClient(modules);
  const resources = new Set<string>();
  const accepted: string[] = [];
  const events: string[] = [];
  const register = (id: string, version: string) => modules.g(id, ((_, exports, require) => {
    const label = `${id}-${version}`;
    const hot = createHotContext(id, client);
    resources.add(label); events.push(`create:${label}`);
    hot.dispose(() => { resources.delete(label); events.push(`dispose:${label}`); });
    hot.accept(() => accepted.push(label));
    exports.value = id === 'component' ? require('domain').value : version;
  }) as ModuleInitialization);
  register('domain', 'old'); register('component', 'old');
  expect(modules.r('component').value).toBe('old');
  events.length = 0;
  register('domain', 'new'); register('component', 'new');
  await client.applyHotUpdates({ added: [], changed: ['component', 'domain'], removed: [], boundaries: { component: [['component']], domain: [['domain']] }, dynamicResources: null, dynamicModuleResourcesMap: null }, modules);
  expect([...resources].sort()).toEqual(['component-new', 'domain-new']);
  expect(accepted).toEqual(['component-old', 'domain-old']);
  expect(events.slice(0, 2)).toEqual(['dispose:component-old', 'dispose:domain-old']);
  expect(modules.r('component').value).toBe('new');
});
