import { describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type { ModuleRunnerTransport } from '../src/module-runner/types.js';

describe('module runner resolver (main path)', () => {
  test('applies resolver for nested fetchModule requests', async () => {
    const fetchIds: string[] = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;
        fetchIds.push(id as string);

        if (id === '/src/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('@/dep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value + 1);"
            ].join('\n'),
            file: '/src/entry.mjs',
            id: '/src/entry.mjs',
            url: '/src/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/src/dep.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 1);",
            file: '/src/dep.mjs',
            id: '/src/dep.mjs',
            url: '/src/dep.mjs',
            invalidate: false,
            map: null
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      resolver: async (request) => {
        if (request === '@/dep.mjs') {
          return { id: '/src/dep.mjs' };
        }
        return { id: request };
      }
    });

    try {
      const mod = await runner.import<{ value: number }>('/src/entry.mjs');
      expect(mod.value).toBe(2);
      expect(fetchIds).toEqual(['/src/entry.mjs', '/src/dep.mjs']);
    } finally {
      await runner.close();
    }
  });
});
