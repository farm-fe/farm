import { test, expect } from 'vitest';
import { Module, ModuleSystem } from '../src/index.js';
import { initModuleSystem } from '../dist/modules/plugin.js';

test('plugin hooks serial execution', async () => {
  const moduleSystem = {} as ModuleSystem;
  initModuleSystem(moduleSystem);
  const calledPlugins: string[] = [];
  moduleSystem.p.p([
    {
      name: 'test-1',
      moduleCreated: () => {
        calledPlugins.push('test-1');
      }
    },
    {
      name: 'test-2',
      moduleCreated: () => {
        calledPlugins.push('test-2');
      }
    }
  ]);

  moduleSystem.p.s(
    'moduleCreated',
    {
      id: 'test-module',
      meta: {
        env: {}
      },
      exports: {},
    } as Module
  );
  expect(calledPlugins).toEqual(['test-1', 'test-2']);
});

test('plugin hook bail execution', async () => {
  const moduleSystem = {} as ModuleSystem;
  initModuleSystem(moduleSystem);
  const calledPlugins: string[] = [];
  moduleSystem.p.p([
    {
      name: 'test-0',
      readModuleCache: () => {
        calledPlugins.push('test-0');
        return false;
      }
    },
    {
      name: 'test-1',
      readModuleCache: () => {
        calledPlugins.push('test-1');
        return true;
      }
    },
    {
      name: 'test-2',
      readModuleCache: () => {
        calledPlugins.push('test-2');
        return true;
      }
    }
  ]);

  const res = await moduleSystem.p.b(
    'readModuleCache',
    {
      id: 'test-module',
      meta: {
        env: {}
      },
      exports: {},
    } as Module
  );
  expect(res).toBe(true);
  expect(calledPlugins).toEqual(['test-0', 'test-1']);
});
