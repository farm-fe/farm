import { describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type { ModuleRunnerTransport } from '../src/module-runner/types.js';

describe('module runner import.meta (main path)', () => {
  test('supports import.meta.resolve', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/src/entry.mjs') {
          return {
            code: [
              "const resolved = await __farm_ssr_import_meta__.resolve('./dep.mjs');",
              "__farm_ssr_export_name__('resolved', () => resolved);"
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

        throw new Error(`unexpected module id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false
    });

    try {
      const entry = await runner.import<{ resolved: string }>('/src/entry.mjs');
      expect(entry.resolved).toBe('/src/dep.mjs');
    } finally {
      await runner.close();
    }
  });
});
