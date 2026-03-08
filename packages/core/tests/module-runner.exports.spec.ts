import { describe, expect, test } from 'vitest';
import { ModuleRunnerDiagnosticsBus } from '../src/module-runner/diagnostics.js';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type {
  ModuleEvaluator,
  ModuleRunnerTransport
} from '../src/module-runner/types.js';

describe('module runner exports (main path)', () => {
  test('loads inlined module exports', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
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
      const mod = await runner.import<{ value: number }>('/entry.mjs');
      expect(mod.value).toBe(42);
    } finally {
      await runner.close();
    }
  });

  test('throws structured error when default import targets module without default export', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('./named-only.mjs');",
              "__farm_ssr_export_name__('value', () => dep.default);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/named-only.mjs') {
          return {
            code: "__farm_ssr_export_name__('named', () => 1);",
            file: '/named-only.mjs',
            id: '/named-only.mjs',
            url: '/named-only.mjs',
            invalidate: false,
            map: null
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
      const mod = await runner.import<{ value: unknown }>('/entry.mjs');
      const error = (() => {
        try {
          // Accessing getter triggers default export read.
          // eslint-disable-next-line @typescript-eslint/no-unused-expressions
          mod.value;
          return null;
        } catch (e) {
          return e as Error & { code?: string };
        }
      })();

      expect(error).toBeInstanceOf(Error);
      expect(error?.message).toContain('Missing default export');
      expect(error?.code).toBe('ERR_MISSING_DEFAULT_EXPORT');
    } finally {
      await runner.close();
    }
  });

  test('bridges cjs default import to module.exports value', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('/dep.cjs');",
              "__farm_ssr_export_name__('defaultValue', () => dep.default.value);",
              "__farm_ssr_export_name__('namedValue', () => dep.value);"
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
      async runExternalModule(file, type) {
        expect(file).toBe('/dep.cjs');
        expect(type).toBe('commonjs');
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

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: customEvaluator
    });

    try {
      const mod = await runner.import<{
        defaultValue: number;
        namedValue: number;
      }>('/entry.mjs');
      expect(mod.defaultValue).toBe(7);
      expect(mod.namedValue).toBe(7);
    } finally {
      await runner.close();
    }
  });

  test('throws structured error when cjs named export is missing', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('/dep.cjs');",
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
    const off = diagnostics.subscribe((event) => {
      events.push(event.type);
    });

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: customEvaluator,
      diagnostics
    });

    try {
      const mod = await runner.import<{ missing: unknown }>('/entry.mjs');
      const error = (() => {
        try {
          // trigger getter
          // eslint-disable-next-line @typescript-eslint/no-unused-expressions
          mod.missing;
          return null;
        } catch (e) {
          return e as Error & { code?: string };
        }
      })();

      expect(error).toBeInstanceOf(Error);
      expect(error?.code).toBe('ERR_MISSING_NAMED_EXPORT');
      expect(error?.message).toContain('Missing named export "missing"');
      expect(events).toContain('interop:wrap');
      expect(events).toContain('interop:error');
    } finally {
      off();
      await runner.close();
    }
  });
});
