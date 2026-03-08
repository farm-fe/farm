import { describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type {
  FetchFunctionOptions,
  ModuleRunnerTransport,
  RunnerHotPayload
} from '../src/module-runner/types.js';

function inlinedModule(id: string, code: string) {
  return {
    code,
    file: id,
    id,
    url: id,
    invalidate: false,
    map: null
  };
}

describe('farm module runner hmr consistency', () => {
  test('update invalidates path + acceptedPath while keeping unrelated module warm', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry.mjs': [],
      '/dep.mjs': [],
      '/boundary.mjs': [],
      '/keep.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        cachedFlagsById[id].push(Boolean(options?.cached));
        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return inlinedModule(
            id,
            [
              "const dep = await __farm_ssr_import__('/dep.mjs');",
              "const keep = await __farm_ssr_import__('/keep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value + keep.value);"
            ].join('\n')
          );
        }

        if (id === '/dep.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 1);"
          );
        }

        if (id === '/keep.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 9);"
          );
        }

        if (id === '/boundary.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('ok', () => true);"
          );
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    try {
      await runner.import('/entry.mjs');
      await runner.import('/boundary.mjs');

      hmrMessageHandler?.({
        type: 'update',
        updates: [
          {
            type: 'js-update',
            path: '/dep.mjs',
            acceptedPath: '/boundary.mjs',
            timestamp: Date.now()
          }
        ]
      });

      expect(
        runner.evaluatedModules.getModuleByUrl('/dep.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/entry.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/boundary.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/keep.mjs')?.meta
      ).toBeDefined();

      await runner.import('/keep.mjs');
      expect(cachedFlagsById['/keep.mjs']).toEqual([false, true]);
    } finally {
      await runner.close();
    }
  });

  test('prune invalidates targeted module and importer chain only', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry.mjs': [],
      '/a.mjs': [],
      '/b.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        cachedFlagsById[id].push(Boolean(options?.cached));
        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return inlinedModule(
            id,
            [
              "const a = await __farm_ssr_import__('/a.mjs');",
              "const b = await __farm_ssr_import__('/b.mjs');",
              "__farm_ssr_export_name__('value', () => a.value + b.value);"
            ].join('\n')
          );
        }

        if (id === '/a.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 1);"
          );
        }

        if (id === '/b.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 2);"
          );
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    try {
      await runner.import('/entry.mjs');

      hmrMessageHandler?.({
        type: 'prune',
        paths: ['/a.mjs']
      });

      expect(
        runner.evaluatedModules.getModuleByUrl('/a.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/entry.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/b.mjs')?.meta
      ).toBeDefined();

      await runner.import('/b.mjs');
      expect(cachedFlagsById['/b.mjs']).toEqual([false, true]);
    } finally {
      await runner.close();
    }
  });

  test('full-reload clears graph maps and next load starts cold', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry.mjs': [],
      '/dep.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        cachedFlagsById[id].push(Boolean(options?.cached));
        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return inlinedModule(
            id,
            [
              "const dep = await __farm_ssr_import__('/dep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n')
          );
        }

        if (id === '/dep.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 3);"
          );
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    try {
      await runner.import('/entry.mjs');
      expect(runner.evaluatedModules.idToModuleMap.size).toBeGreaterThan(0);

      hmrMessageHandler?.({ type: 'full-reload' });

      expect(runner.evaluatedModules.idToModuleMap.size).toBe(0);
      expect(runner.evaluatedModules.urlToModuleMap.size).toBe(0);

      await runner.import('/dep.mjs');
      expect(cachedFlagsById['/dep.mjs']).toEqual([false, false]);
    } finally {
      await runner.close();
    }
  });

  test('update invalidates circular + dynamic branch while keeping sibling dynamic branch warm', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry.mjs': [],
      '/a.mjs': [],
      '/b.mjs': [],
      '/c.mjs': [],
      '/keep.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        cachedFlagsById[id].push(Boolean(options?.cached));
        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return inlinedModule(
            id,
            [
              "const a = await __farm_ssr_dynamic_import__('/a.mjs');",
              "const keep = await __farm_ssr_dynamic_import__('/keep.mjs');",
              "__farm_ssr_export_name__('value', () => a.value + keep.value);"
            ].join('\n')
          );
        }

        if (id === '/a.mjs') {
          return inlinedModule(
            id,
            [
              "const b = await __farm_ssr_import__('/b.mjs');",
              "__farm_ssr_export_name__('value', () => b.value + 1);"
            ].join('\n')
          );
        }

        if (id === '/b.mjs') {
          return inlinedModule(
            id,
            [
              "const c = await __farm_ssr_dynamic_import__('/c.mjs');",
              "__farm_ssr_export_name__('value', () => c.value + 1);"
            ].join('\n')
          );
        }

        if (id === '/c.mjs') {
          return inlinedModule(
            id,
            [
              "const a = await __farm_ssr_import__('/a.mjs');",
              "__farm_ssr_export_name__('touchA', () => Boolean(a));",
              "__farm_ssr_export_name__('value', () => 1);"
            ].join('\n')
          );
        }

        if (id === '/keep.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 9);"
          );
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    try {
      const first = await runner.import<{ value: number }>('/entry.mjs');
      expect(first.value).toBe(12);

      hmrMessageHandler?.({
        type: 'update',
        updates: [
          {
            type: 'js-update',
            path: '/c.mjs',
            acceptedPath: '/c.mjs',
            timestamp: Date.now()
          }
        ]
      });

      expect(
        runner.evaluatedModules.getModuleByUrl('/c.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/b.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/a.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/entry.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/keep.mjs')?.meta
      ).toBeDefined();

      await runner.import('/keep.mjs');
      expect(cachedFlagsById['/keep.mjs']).toEqual([false, true]);
    } finally {
      await runner.close();
    }
  });

  test('prune invalidates circular dynamic branch and preserves sibling branch cache', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry.mjs': [],
      '/a.mjs': [],
      '/b.mjs': [],
      '/sibling.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        cachedFlagsById[id].push(Boolean(options?.cached));
        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return inlinedModule(
            id,
            [
              "const a = await __farm_ssr_dynamic_import__('/a.mjs');",
              "const sibling = await __farm_ssr_dynamic_import__('/sibling.mjs');",
              "__farm_ssr_export_name__('value', () => a.value + sibling.value);"
            ].join('\n')
          );
        }

        if (id === '/a.mjs') {
          return inlinedModule(
            id,
            [
              "const b = await __farm_ssr_import__('/b.mjs');",
              "__farm_ssr_export_name__('value', () => b.value + 1);"
            ].join('\n')
          );
        }

        if (id === '/b.mjs') {
          return inlinedModule(
            id,
            [
              "const a = await __farm_ssr_import__('/a.mjs');",
              "__farm_ssr_export_name__('touchA', () => Boolean(a));",
              "__farm_ssr_export_name__('value', () => 1);"
            ].join('\n')
          );
        }

        if (id === '/sibling.mjs') {
          return inlinedModule(
            id,
            "__farm_ssr_export_name__('value', () => 10);"
          );
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    try {
      const first = await runner.import<{ value: number }>('/entry.mjs');
      expect(first.value).toBe(12);

      hmrMessageHandler?.({
        type: 'prune',
        paths: ['/a.mjs']
      });

      expect(
        runner.evaluatedModules.getModuleByUrl('/a.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/b.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/entry.mjs')?.meta
      ).toBeUndefined();
      expect(
        runner.evaluatedModules.getModuleByUrl('/sibling.mjs')?.meta
      ).toBeDefined();

      await runner.import('/sibling.mjs');
      expect(cachedFlagsById['/sibling.mjs']).toEqual([false, true]);
    } finally {
      await runner.close();
    }
  });
});
