import { test, expect } from 'vitest';
import { Module } from '../src/module.js';
import { FarmRuntimePluginContainer } from '../src/plugin.js';

test('plugin creation', () => {
  const pluginContainer = new FarmRuntimePluginContainer([]);
  expect(pluginContainer).toBeTruthy();
});

test('plugin hooks serial execution', async () => {
  const calledPlugins: string[] = [];
  const pluginContainer = new FarmRuntimePluginContainer([
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

  await pluginContainer.hookSerial(
    'moduleCreated',
    new Module('test-module', () => {
      /** */
    })
  );
  expect(calledPlugins).toEqual(['test-1', 'test-2']);
});

test('plugin hook bail execution', async () => {
  const calledPlugins: string[] = [];
  const pluginContainer = new FarmRuntimePluginContainer([
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

  const res = await pluginContainer.hookBail(
    'readModuleCache',
    new Module('test-module', () => {
      /** */
    })
  );
  expect(res).toBe(true);
  expect(calledPlugins).toEqual(['test-0', 'test-1']);
});
