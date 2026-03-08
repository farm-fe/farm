import { describe, expect, test } from 'vitest';
import { ModuleRunnerDiagnosticsBus } from '../src/module-runner/diagnostics.js';
import { EvaluatedModules } from '../src/module-runner/evaluatedModules.js';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type {
  ModuleEvaluator,
  ModuleRunnerTransport
} from '../src/module-runner/types.js';

describe('farm module runner convergence (main path)', () => {
  test('dedupes concurrent fetchModule requests for same entry', async () => {
    let fetchCount = 0;

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id !== '/entry.mjs') {
          throw new Error(`unexpected id: ${String(id)}`);
        }

        fetchCount++;
        await new Promise((resolve) => setTimeout(resolve, 10));
        return {
          code: "__farm_ssr_export_name__('value', () => 42);",
          file: '/entry.mjs',
          id: '/entry.mjs',
          url: '/entry.mjs',
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false
    });

    try {
      const [a, b] = await Promise.all([
        runner.import<{ value: number }>('/entry.mjs'),
        runner.import<{ value: number }>('/entry.mjs')
      ]);
      expect(a.value).toBe(42);
      expect(b.value).toBe(42);
      expect(fetchCount).toBe(1);
    } finally {
      await runner.close();
    }
  });

  test('invalidates cache by ttl and refetches module', async () => {
    let value = 1;
    const cachedFlags: boolean[] = [];

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, , options] = data as [
          string,
          string | undefined,
          { cached?: boolean } | undefined
        ];
        expect(id).toBe('/entry.mjs');
        cachedFlags.push(Boolean(options?.cached));

        if (options?.cached) {
          return { cache: true };
        }

        return {
          code: `const value = ${value++}; __farm_ssr_export_name__('value', () => value);`,
          file: '/entry.mjs',
          id: '/entry.mjs',
          url: '/entry.mjs',
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      cachePolicy: {
        ttlMs: 5
      }
    });

    try {
      const first = await runner.import<{ value: number }>('/entry.mjs');
      expect(first.value).toBe(1);
      await new Promise((resolve) => setTimeout(resolve, 20));
      const second = await runner.import<{ value: number }>('/entry.mjs');
      expect(second.value).toBe(2);
      expect(cachedFlags).toEqual([false, false]);
    } finally {
      await runner.close();
    }
  });

  test('evicts least recently used cache entry when maxEntries is exceeded', async () => {
    const cachedFlags: Record<string, boolean[]> = {
      '/a.mjs': [],
      '/b.mjs': []
    };

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, , options] = data as [
          string,
          string | undefined,
          { cached?: boolean } | undefined
        ];

        if (id !== '/a.mjs' && id !== '/b.mjs') {
          throw new Error(`unexpected id: ${String(id)}`);
        }

        cachedFlags[id].push(Boolean(options?.cached));

        if (options?.cached) {
          return { cache: true };
        }

        const value = id === '/a.mjs' ? 1 : 2;
        return {
          code: `const value = ${value}; __farm_ssr_export_name__('value', () => value);`,
          file: id,
          id,
          url: id,
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      cachePolicy: {
        maxEntries: 1
      }
    });

    try {
      const a1 = await runner.import<{ value: number }>('/a.mjs');
      expect(a1.value).toBe(1);

      const b = await runner.import<{ value: number }>('/b.mjs');
      expect(b.value).toBe(2);

      const a2 = await runner.import<{ value: number }>('/a.mjs');
      expect(a2.value).toBe(1);

      expect(cachedFlags['/a.mjs']).toEqual([false, false]);
      expect(cachedFlags['/b.mjs']).toEqual([false]);
    } finally {
      await runner.close();
    }
  });

  test('incremental gc prunes invalidated modules from graph maps', async () => {
    const evaluatedModules = new EvaluatedModules();
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id, , options] = data as [
          string,
          string | undefined,
          { cached?: boolean } | undefined
        ];

        if (id !== '/a.mjs' && id !== '/b.mjs') {
          throw new Error(`unexpected id: ${String(id)}`);
        }

        if (options?.cached) {
          return { cache: true };
        }

        return {
          code: "__farm_ssr_export_name__('value', () => 1);",
          file: id,
          id,
          url: id,
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluatedModules,
      cachePolicy: {
        maxEntries: 1,
        gcSweepPerCycle: 1
      }
    });

    try {
      await runner.import('/a.mjs');
      await runner.import('/b.mjs');

      expect(evaluatedModules.getModuleById('/a.mjs')).toBeUndefined();
      expect(evaluatedModules.getModuleById('/b.mjs')).toBeDefined();
    } finally {
      await runner.close();
    }
  });

  test('applies custom resolver for entry import', async () => {
    const fetchIds: string[] = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;
        fetchIds.push(id as string);

        if (id === '/src/entry.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 1);",
            file: '/src/entry.mjs',
            id: '/src/entry.mjs',
            url: '/src/entry.mjs',
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
        if (request === '/entry-alias') {
          return { id: '/src/entry.mjs' };
        }
        return { id: request };
      }
    });

    try {
      const mod = await runner.import<{ value: number }>('/entry-alias');
      expect(mod.value).toBe(1);
      expect(fetchIds).toEqual(['/src/entry.mjs']);
    } finally {
      await runner.close();
    }
  });

  test('applies custom resolver for nested dependencies', async () => {
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
      resolver: async (request, context) => {
        if (request === '@/dep.mjs') {
          expect(context?.importer).toBe('/src/entry.mjs');
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

  test('supports cjs interop on main path and throws structured named-missing error', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('/dep.cjs');",
              "__farm_ssr_export_name__('defaultValue', () => dep.default.value);",
              "__farm_ssr_export_name__('namedValue', () => dep.value);",
              "__farm_ssr_export_name__('missing', () => dep.missing);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dep.cjs') {
          return {
            externalize: '/dep.cjs',
            type: 'commonjs'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const customEvaluator: ModuleEvaluator = {
      startOffset: 2,
      async runExternalModule() {
        return { value: 7 };
      },
      async runInlinedModule(context, code) {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval
        const AsyncFunction = async function () {}
          .constructor as FunctionConstructor;
        const init = new AsyncFunction(
          '__farm_ssr_exports__',
          '__farm_ssr_import_meta__',
          '__farm_ssr_import__',
          '__farm_ssr_dynamic_import__',
          '__farm_ssr_export_all__',
          '__farm_ssr_export_name__',
          code
        );

        await init(
          context.__farm_ssr_exports__,
          context.__farm_ssr_import_meta__,
          context.__farm_ssr_import__,
          context.__farm_ssr_dynamic_import__,
          context.__farm_ssr_export_all__,
          context.__farm_ssr_export_name__
        );
      }
    };

    const diagnostics = new ModuleRunnerDiagnosticsBus();
    const events: string[] = [];
    const off = diagnostics.subscribe((event) => events.push(event.type));

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: customEvaluator,
      diagnostics
    });

    try {
      const mod = await runner.import<{
        defaultValue: number;
        namedValue: number;
        missing: unknown;
      }>('/entry.mjs');
      expect(mod.defaultValue).toBe(7);
      expect(mod.namedValue).toBe(7);

      const error = (() => {
        try {
          // trigger missing named getter
          // eslint-disable-next-line @typescript-eslint/no-unused-expressions
          mod.missing;
          return null;
        } catch (e) {
          return e as Error & { code?: string };
        }
      })();

      expect(error?.code).toBe('ERR_MISSING_NAMED_EXPORT');
      expect(events).toContain('interop:wrap');
      expect(events).toContain('interop:error');
    } finally {
      off();
      await runner.close();
    }
  });

  test('supports nonJsPolicy on main path', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/src/entry.mjs') {
          return {
            code: [
              "const style = await __farm_ssr_import__('./style.css');",
              "const logo = await __farm_ssr_import__('./logo.png');",
              "__farm_ssr_export_name__('styleType', () => typeof style.default);",
              "__farm_ssr_export_name__('logo', () => logo.default);"
            ].join('\n'),
            file: '/src/entry.mjs',
            id: '/src/entry.mjs',
            url: '/src/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/src/style.css') {
          return {
            externalize: 'file:///tmp/style.css?t=1',
            type: 'module',
            bailoutReason: 'not-script'
          };
        }

        if (id === '/src/logo.png') {
          return {
            externalize: 'file:///tmp/logo.png?t=1',
            type: 'module'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false
    });

    try {
      const mod = await runner.import<{ styleType: string; logo: string }>(
        '/src/entry.mjs'
      );
      expect(mod.styleType).toBe('object');
      expect(mod.logo).toContain('file:///tmp/logo.png');
    } finally {
      await runner.close();
    }
  });

  test('supports externalPolicy.globals before evaluator external loading', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const react = await __farm_ssr_import__('react');",
              "__farm_ssr_export_name__('v', () => react.version);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'react') {
          return {
            externalize: 'react',
            type: 'module'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    let evaluatorCalled = 0;
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: {
        async runInlinedModule(context, code) {
          // eslint-disable-next-line @typescript-eslint/no-implied-eval
          const AsyncFunction = async function () {}
            .constructor as FunctionConstructor;
          const init = new AsyncFunction(
            '__farm_ssr_exports__',
            '__farm_ssr_import_meta__',
            '__farm_ssr_import__',
            '__farm_ssr_dynamic_import__',
            '__farm_ssr_export_all__',
            '__farm_ssr_export_name__',
            code
          );
          await init(
            context.__farm_ssr_exports__,
            context.__farm_ssr_import_meta__,
            context.__farm_ssr_import__,
            context.__farm_ssr_dynamic_import__,
            context.__farm_ssr_export_all__,
            context.__farm_ssr_export_name__
          );
        },
        async runExternalModule() {
          evaluatorCalled++;
          return { version: 'from-evaluator' };
        }
      },
      externalPolicy: {
        globals: {
          react: { version: 'from-policy' }
        }
      }
    });

    try {
      const mod = await runner.import<{ v: string }>('/entry.mjs');
      expect(mod.v).toBe('from-policy');
      expect(evaluatorCalled).toBe(0);
    } finally {
      await runner.close();
    }
  });

  test('supports externalPolicy.network stub before evaluator external loading', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const cdn = await __farm_ssr_import__('https://cdn.example.com/pkg.mjs');",
              "__farm_ssr_export_name__('v', () => cdn.default.url);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'https://cdn.example.com/pkg.mjs') {
          return {
            externalize: 'https://cdn.example.com/pkg.mjs?t=1',
            type: 'network'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    let evaluatorCalled = 0;
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: {
        async runInlinedModule(context, code) {
          // eslint-disable-next-line @typescript-eslint/no-implied-eval
          const AsyncFunction = async function () {}
            .constructor as FunctionConstructor;
          const init = new AsyncFunction(
            '__farm_ssr_exports__',
            '__farm_ssr_import_meta__',
            '__farm_ssr_import__',
            '__farm_ssr_dynamic_import__',
            '__farm_ssr_export_all__',
            '__farm_ssr_export_name__',
            code
          );
          await init(
            context.__farm_ssr_exports__,
            context.__farm_ssr_import_meta__,
            context.__farm_ssr_import__,
            context.__farm_ssr_dynamic_import__,
            context.__farm_ssr_export_all__,
            context.__farm_ssr_export_name__
          );
        },
        async runExternalModule() {
          evaluatorCalled++;
          return { url: 'from-evaluator' };
        }
      },
      externalPolicy: {
        network: {
          mode: 'stub',
          stub: (context) => ({ url: context.externalize })
        }
      }
    });

    try {
      const mod = await runner.import<{ v: string }>('/entry.mjs');
      expect(mod.v).toBe('https://cdn.example.com/pkg.mjs?t=1');
      expect(evaluatorCalled).toBe(0);
    } finally {
      await runner.close();
    }
  });

  test('supports externalPolicy.builtin error mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const fs = await __farm_ssr_import__('node:fs');",
              "__farm_ssr_export_name__('v', () => Boolean(fs));"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'node:fs') {
          return {
            externalize: 'node:fs',
            type: 'builtin'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      externalPolicy: {
        builtin: {
          mode: 'error',
          message: (context) =>
            `blocked external ${context.type}:${context.requestId}`
        }
      }
    });

    try {
      await expect(runner.import('/entry.mjs')).rejects.toThrow(
        'blocked external builtin:node:fs'
      );
    } finally {
      await runner.close();
    }
  });

  test('supports externalPolicy.custom for module externals', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('virtual:custom');",
              "__farm_ssr_export_name__('v', () => dep.default.answer);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'virtual:custom') {
          return {
            externalize: 'virtual:custom',
            type: 'module'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    let evaluatorCalled = 0;
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: {
        async runInlinedModule(context, code) {
          // eslint-disable-next-line @typescript-eslint/no-implied-eval
          const AsyncFunction = async function () {}
            .constructor as FunctionConstructor;
          const init = new AsyncFunction(
            '__farm_ssr_exports__',
            '__farm_ssr_import_meta__',
            '__farm_ssr_import__',
            '__farm_ssr_dynamic_import__',
            '__farm_ssr_export_all__',
            '__farm_ssr_export_name__',
            code
          );
          await init(
            context.__farm_ssr_exports__,
            context.__farm_ssr_import_meta__,
            context.__farm_ssr_import__,
            context.__farm_ssr_dynamic_import__,
            context.__farm_ssr_export_all__,
            context.__farm_ssr_export_name__
          );
        },
        async runExternalModule() {
          evaluatorCalled++;
          return { answer: -1 };
        }
      },
      externalPolicy: {
        custom: (context) => {
          if (context.requestId === 'virtual:custom') {
            return { answer: 42 };
          }
          return undefined;
        }
      }
    });

    try {
      const mod = await runner.import<{ v: number }>('/entry.mjs');
      expect(mod.v).toBe(42);
      expect(evaluatorCalled).toBe(0);
    } finally {
      await runner.close();
    }
  });
});
