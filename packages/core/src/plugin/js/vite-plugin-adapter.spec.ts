import { describe, expect, it, vi } from 'vitest';
import { VitePluginAdapter } from './vite-plugin-adapter.js';

function createAdapterStub(params: {
  modulesByFile: Record<string, Array<{ id: string; file?: string }>>;
  handleHotUpdate?: (ctx: unknown) => Promise<Array<{ id: string }> | void>;
}) {
  const adapter = Object.create(
    VitePluginAdapter.prototype
  ) as VitePluginAdapter & {
    _rawPlugin: { handleHotUpdate?: (ctx: unknown) => Promise<unknown> };
    _viteDevServer: {
      moduleGraph: {
        getModulesByFile: (
          file: string
        ) => Array<{ id: string; file?: string }>;
      };
    };
    wrapExecutor: <T>(executor: T) => T;
    wrapRawPluginHook: (_name: string, hook: unknown) => unknown;
  };

  adapter.name = 'vite-plugin-adapter-test';
  adapter._rawPlugin = {
    handleHotUpdate: params.handleHotUpdate
  };
  adapter._viteDevServer = {
    moduleGraph: {
      getModulesByFile: vi.fn(
        (file: string) => params.modulesByFile[file] ?? []
      )
    }
  };
  adapter.wrapExecutor = <T>(executor: T) => executor;
  adapter.wrapRawPluginHook = (_name: string, hook: unknown) => hook;

  return adapter;
}

describe('VitePluginAdapter updateModules', () => {
  it('returns update tuples for vite handleHotUpdate results', async () => {
    const styleModuleId =
      '/src/pages/AboutPage.vue?vue&type=style&index=0&lang.less';
    const adapter = createAdapterStub({
      modulesByFile: {
        '/src/pages/AboutPage.vue': [{ id: styleModuleId }]
      },
      handleHotUpdate: async (ctx: { modules: Array<{ id: string }> }) =>
        ctx.modules
    });

    const hook = (adapter as any).viteHandleHotUpdateToFarmUpdateModules();
    const result = await hook.executor(
      {
        paths: [['/src/pages/AboutPage.vue', 'updated']]
      },
      {}
    );

    expect(result).toEqual([[styleModuleId, 'updated']]);
  });

  it('falls back to moduleGraph modules and keeps highest-priority update type', async () => {
    const styleModuleId =
      '/src/pages/AboutPage.vue?vue&type=style&index=0&lang.less';
    const adapter = createAdapterStub({
      modulesByFile: {
        '/src/pages/AboutPage.vue': [{ id: styleModuleId }],
        '/src/pages/AboutPage.vue?removed': [{ id: styleModuleId }]
      },
      handleHotUpdate: async () => undefined
    });

    const hook = (adapter as any).viteHandleHotUpdateToFarmUpdateModules();
    const result = await hook.executor(
      {
        paths: [
          ['/src/pages/AboutPage.vue', 'added'],
          ['/src/pages/AboutPage.vue?removed', 'removed']
        ]
      },
      {}
    );

    expect(result).toEqual([[styleModuleId, 'removed']]);
  });
});
