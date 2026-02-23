import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { afterEach, describe, expect, test, vi } from 'vitest';
import { createDefaultImportMeta } from '../src/module-runner/createImportMeta.js';
import { detectHostEvaluatorType } from '../src/module-runner/evaluator.js';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import {
  createModuleRunnerInvokeHandlers,
  createServerModuleRunnerInvokeHandlers
} from '../src/module-runner/serverInvoke.js';
import { createModuleRunnerTransportFromInvokeHandlers } from '../src/module-runner/serverTransport.js';
import { createRunnerSourceMapInterceptor } from '../src/module-runner/sourceMapInterceptor.js';
import type {
  FetchFunctionOptions,
  ModuleEvaluator,
  ModuleRunnerTransport,
  RunnerHotPayload
} from '../src/module-runner/types.js';

const tempDirs: string[] = [];

async function createTempDir(prefix: string): Promise<string> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  await Promise.allSettled(
    tempDirs
      .splice(0, tempDirs.length)
      .map((dir) => fs.rm(dir, { recursive: true, force: true }))
  );
});

describe('farm module runner', () => {
  test('detects evaluator runtime type from host globals', () => {
    expect(detectHostEvaluatorType({})).toBe('node');
    expect(detectHostEvaluatorType({ Bun: {} })).toBe('bun');
    expect(detectHostEvaluatorType({ Deno: {} })).toBe('deno');
    expect(
      detectHostEvaluatorType({
        process: {
          getBuiltinModule(name: string) {
            if (name === 'node:worker_threads') {
              return { isMainThread: false };
            }
            return undefined;
          }
        }
      })
    ).toBe('worker');
    expect(
      detectHostEvaluatorType({
        process: {
          getBuiltinModule() {
            return { isMainThread: true };
          }
        }
      })
    ).toBe('node');

    class FakeWorkerGlobalScope {}
    const fakeWorker = new FakeWorkerGlobalScope();
    expect(
      detectHostEvaluatorType({
        WorkerGlobalScope: FakeWorkerGlobalScope,
        self: fakeWorker
      })
    ).toBe('worker');
  });

  test('runner logs and stores transform bailout reason on externalized fallback', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin',
          bailoutReason: 'unsupported-ts'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false
    });

    try {
      await runner.import('/entry.ts');
      await runner.import('/entry.ts');

      expect(warn).toHaveBeenCalledTimes(1);
      expect(warn).toHaveBeenCalledWith(
        expect.stringContaining('bailoutReason="unsupported-ts"')
      );

      const meta = runner.evaluatedModules.getModuleById('node:fs')?.meta;
      expect(
        meta && 'externalize' in meta ? meta.bailoutReason : undefined
      ).toBe('unsupported-ts');
    } finally {
      warn.mockRestore();
      await runner.close();
    }
  });

  test('transport adapter forwards hot payloads and handles disconnect', async () => {
    const messages: RunnerHotPayload[] = [];
    const unsubscribe = vi.fn();
    let subscriber: ((payload: RunnerHotPayload) => void) | null = null;
    let disconnected = 0;

    const transport = createModuleRunnerTransportFromInvokeHandlers({
      invokeHandlers: {
        fetchModule: async () => ({
          externalize: 'node:fs',
          type: 'builtin'
        }),
        getBuiltins: async () => ['node:fs']
      },
      hotBus: {
        subscribe(cb) {
          subscriber = cb;
          return () => {
            subscriber = null;
            unsubscribe();
          };
        }
      }
    });

    transport.connect?.({
      onMessage(payload) {
        messages.push(payload);
      },
      onDisconnection() {
        disconnected++;
      }
    });

    expect(messages[0]).toEqual({ type: 'connected' });

    subscriber?.({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/entry.ts',
          acceptedPath: '/entry.ts',
          timestamp: 1
        }
      ]
    });

    expect(messages[1]).toEqual({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/entry.ts',
          acceptedPath: '/entry.ts',
          timestamp: 1
        }
      ]
    });

    transport.disconnect?.();

    expect(unsubscribe).toHaveBeenCalledTimes(1);
    expect(disconnected).toBe(1);
    expect(subscriber).toBeNull();
  });

  test('transport adapter delegates invoke to invoke handlers', async () => {
    const fetchModule = vi.fn(async () => ({
      externalize: 'node:fs',
      type: 'builtin' as const
    }));

    const getBuiltins = vi.fn(async () => ['node:fs']);

    const transport = createModuleRunnerTransportFromInvokeHandlers({
      invokeHandlers: {
        fetchModule,
        getBuiltins
      },
      hotBus: {
        subscribe() {
          return () => undefined;
        }
      }
    });

    await expect(
      transport.invoke('fetchModule', [
        '/entry.ts',
        undefined,
        { cached: true }
      ])
    ).resolves.toEqual({
      externalize: 'node:fs',
      type: 'builtin'
    });

    await expect(transport.invoke('getBuiltins', [])).resolves.toEqual([
      'node:fs'
    ]);

    expect(fetchModule).toHaveBeenCalledWith('/entry.ts', undefined, {
      cached: true
    });
    expect(getBuiltins).toHaveBeenCalledWith();
  });

  test('runner throws on transform bailout externalize when strictTransform is enabled', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin',
          bailoutReason: 'unsupported-ts'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      strictTransform: true
    });

    try {
      const error = await runner
        .import('/entry.ts')
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(error?.message).toContain('strictTransform=true');
      expect(error?.message).toContain('bailoutReason="unsupported-ts"');
      expect(warn).not.toHaveBeenCalled();
    } finally {
      warn.mockRestore();
      await runner.close();
    }
  });

  test('strictTransform does not block normal externalize without bailoutReason', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      strictTransform: true
    });

    try {
      const result = await runner.import<Record<string, unknown>>('/entry.ts');
      expect(result).toBeTypeOf('object');
    } finally {
      await runner.close();
    }
  });

  test('runner resolves relative imports from inlined bare specifier using fetched module url', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (id === '/entry.ts') {
          return {
            code: [
              "const pkg = await __farm_ssr_import__('virtual-pkg');",
              "__farm_ssr_export_name__('value', () => pkg.value);"
            ].join('\n'),
            file: '/entry.ts',
            id: '/entry.ts',
            url: '/entry.ts',
            invalidate: false,
            map: null
          };
        }

        if (id === 'virtual-pkg') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('./dep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n'),
            file: '/node_modules/virtual-pkg/index.mjs',
            id: 'virtual-pkg',
            url: '/node_modules/virtual-pkg/index.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/node_modules/virtual-pkg/dep.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 42);",
            file: '/node_modules/virtual-pkg/dep.mjs',
            id: '/node_modules/virtual-pkg/dep.mjs',
            url: '/node_modules/virtual-pkg/dep.mjs',
            invalidate: false,
            map: null
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false
    });

    try {
      const result = await runner.import<{ value: number }>('/entry.ts');
      expect(result.value).toBe(42);
    } finally {
      await runner.close();
    }
  });

  test('worker evaluator rejects commonjs external module', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/dep.cjs',
          type: 'commonjs'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker'
    });

    const error = await runner
      .import('/entry.cjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in worker evaluator');
    await runner.close();
  });

  test('worker evaluator can resolve commonjs via resolveExternalModule hook', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/worker-dep.cjs',
          type: 'commonjs'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 12 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-cjs-hook.mjs'
    );
    expect(result.value).toBe(12);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: '/tmp/worker-dep.cjs',
      type: 'commonjs',
      evaluator: 'worker',
      reason: 'unsupported-external'
    });
    expect(calls[0].error).toBeInstanceOf(Error);
    expect((calls[0].error as Error).message).toContain(
      'commonjs external module is not supported'
    );
    await runner.close();
  });

  test('worker evaluator rejects builtin external module', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker'
    });

    const error = await runner
      .import('/entry-builtin.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in worker evaluator');
    await runner.close();
  });

  test('worker evaluator can resolve builtin via resolveExternalModule hook', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 11 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-hook.mjs'
    );
    expect(result.value).toBe(11);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'node:fs',
      type: 'builtin',
      evaluator: 'worker',
      reason: 'unsupported-external'
    });
    expect(calls[0].error).toBeInstanceOf(Error);
    expect((calls[0].error as Error).message).toContain(
      'builtin external module is not supported'
    );
    await runner.close();
  });

  test('worker evaluator forwards import options to resolveExternalModule hook on unsupported builtin', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (id === '/entry-worker-builtin-options-hook.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('node:fs', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n'),
            file: '/entry-worker-builtin-options-hook.mjs',
            id: '/entry-worker-builtin-options-hook.mjs',
            url: '/entry-worker-builtin-options-hook.mjs',
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

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      importOptions?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 18 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-builtin-options-hook.mjs'
    );
    expect(result.value).toBe(18);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'node:fs',
      type: 'builtin',
      evaluator: 'worker',
      reason: 'unsupported-external',
      importOptions: { with: { type: 'json' } }
    });
    expect(calls[0].importOptions).toEqual({ with: { type: 'json' } });
    await runner.close();
  });

  test('worker evaluator reports resolveExternalModule hook failure with context', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async () => {
        throw new Error('hook-boom');
      }
    });

    const error = await runner
      .import('/entry-worker-hook-error.mjs')
      .then(() => null)
      .catch((e) => e as Error & { cause?: unknown });

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('resolveExternalModule failed');
    expect(error?.message).toContain('node:fs');
    expect(error?.message).toContain('type=builtin');
    expect(error?.message).toContain('evaluator=worker');
    expect(error?.message).toContain('reason=unsupported-external');
    expect(error?.cause).toBeInstanceOf(Error);
    expect((error?.cause as Error)?.message).toContain('hook-boom');
    await runner.close();
  });

  test('worker evaluator reports invalid resolveExternalModule return type with context', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async () =>
        'invalid-result' as unknown as {
          resolved: boolean;
          module?: unknown;
        }
    });

    const error = await runner
      .import('/entry-worker-hook-invalid-return.mjs')
      .then(() => null)
      .catch((e) => e as Error & { cause?: unknown });

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('resolveExternalModule failed');
    expect(error?.message).toContain('node:fs');
    expect(error?.cause).toBeInstanceOf(Error);
    expect((error?.cause as Error)?.message).toContain(
      'must return an object with boolean "resolved"'
    );
    await runner.close();
  });

  test('worker evaluator reports invalid resolveExternalModule shape with context', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/worker-shape.cjs',
          type: 'commonjs'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async () =>
        ({ module: { value: 1 } }) as unknown as {
          resolved: boolean;
          module?: unknown;
        }
    });

    const error = await runner
      .import('/entry-worker-hook-invalid-shape.mjs')
      .then(() => null)
      .catch((e) => e as Error & { cause?: unknown });

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('resolveExternalModule failed');
    expect(error?.message).toContain('/tmp/worker-shape.cjs');
    expect(error?.cause).toBeInstanceOf(Error);
    expect((error?.cause as Error)?.message).toContain(
      'must return an object with boolean "resolved"'
    );
    await runner.close();
  });

  test('worker evaluator treats undefined resolveExternalModule result as unresolved', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async () => undefined
    });

    const error = await runner
      .import('/entry-worker-hook-undefined.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain(
      'builtin external module is not supported'
    );
    await runner.close();
  });

  test('worker evaluator keeps import failure when resolveExternalModule returns null', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'unsupported+scheme://null-hook-entry',
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async () =>
        null as unknown as {
          resolved: boolean;
          module?: unknown;
        }
    });

    const error = await runner
      .import('/entry-worker-hook-null.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('unsupported+scheme://null-hook-entry');
    await runner.close();
  });

  test('worker evaluator can resolve module externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'unsupported+scheme://virtual-entry',
          type: 'module'
        };
      }
    };

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 13 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-module-hook.mjs'
    );
    expect(result.value).toBe(13);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://virtual-entry',
      type: 'module',
      evaluator: 'worker',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('worker evaluator forwards import options to resolveExternalModule hook on import-fail', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (id === '/entry-worker-import-options-hook.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('unsupported+scheme://worker-options-entry', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n'),
            file: '/entry-worker-import-options-hook.mjs',
            id: '/entry-worker-import-options-hook.mjs',
            url: '/entry-worker-import-options-hook.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'unsupported+scheme://worker-options-entry') {
          return {
            externalize: id,
            type: 'module'
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      importOptions?: unknown;
    }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 27 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-import-options-hook.mjs'
    );
    expect(result.value).toBe(27);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://worker-options-entry',
      type: 'module',
      evaluator: 'worker',
      reason: 'import-failed',
      importOptions: { with: { type: 'json' } }
    });
    expect(calls[0].importOptions).toEqual({ with: { type: 'json' } });
    await runner.close();
  });

  test('worker evaluator can resolve network externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'https://example.invalid/farm-worker-network.mjs',
          type: 'network'
        };
      }
    };

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 14 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-worker-network-hook.mjs'
    );
    expect(result.value).toBe(14);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'https://example.invalid/farm-worker-network.mjs',
      type: 'network',
      evaluator: 'worker',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('worker evaluator keeps native import runtime error without hook fallback', async () => {
    const dir = await createTempDir('farm-runner-worker-runtime-error-');
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new Error('external-module-runtime-boom');\n"
    );

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=boom`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 99 }
        };
      }
    });

    const error = await runner
      .import('/entry-worker-runtime-error.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('external-module-runtime-boom');
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('worker evaluator does not fallback hook for runtime error with loader-like message', async () => {
    const dir = await createTempDir(
      'farm-runner-worker-runtime-loader-like-error-'
    );
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new Error('Cannot find module fake-runtime-value');\n"
    );

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=loader-like`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 1001 }
        };
      }
    });

    const error = await runner
      .import('/entry-worker-runtime-loader-like-error.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('Cannot find module fake-runtime-value');
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('worker evaluator does not fallback hook for runtime TypeError with loader-like message', async () => {
    const dir = await createTempDir(
      'farm-runner-worker-runtime-loader-like-typeerror-'
    );
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new TypeError('Cannot find module fake-runtime-typeerror');\n"
    );

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=loader-like-typeerror`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 1002 }
        };
      }
    });

    const error = await runner
      .import('/entry-worker-runtime-loader-like-typeerror.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(TypeError);
    expect(error?.message).toContain(
      'Cannot find module fake-runtime-typeerror'
    );
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('bun evaluator rejects commonjs external module in baseline mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/dep.cjs',
          type: 'commonjs'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun'
    });

    const error = await runner
      .import('/entry-bun.cjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in bun evaluator');
    await runner.close();
  });

  test('bun evaluator can resolve commonjs via resolveExternalModule hook', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/dep.cjs',
          type: 'commonjs'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 22 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-bun-hook.mjs'
    );
    expect(result.value).toBe(22);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: '/tmp/dep.cjs',
      type: 'commonjs',
      evaluator: 'bun',
      reason: 'unsupported-external'
    });
    expect(calls[0].error).toBeInstanceOf(Error);
    expect((calls[0].error as Error).message).toContain(
      'commonjs external module is not supported'
    );
    await runner.close();
  });

  test('bun evaluator can resolve module externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'unsupported+scheme://bun-virtual-entry',
          type: 'module'
        };
      }
    };

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 23 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-bun-module-hook.mjs'
    );
    expect(result.value).toBe(23);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://bun-virtual-entry',
      type: 'module',
      evaluator: 'bun',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('bun evaluator forwards import options to resolveExternalModule hook on import-fail', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (id === '/entry-bun-import-options-hook.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('unsupported+scheme://bun-options-entry', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n'),
            file: '/entry-bun-import-options-hook.mjs',
            id: '/entry-bun-import-options-hook.mjs',
            url: '/entry-bun-import-options-hook.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'unsupported+scheme://bun-options-entry') {
          return {
            externalize: id,
            type: 'module'
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      importOptions?: unknown;
    }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 28 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-bun-import-options-hook.mjs'
    );
    expect(result.value).toBe(28);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://bun-options-entry',
      type: 'module',
      evaluator: 'bun',
      reason: 'import-failed',
      importOptions: { with: { type: 'json' } }
    });
    expect(calls[0].importOptions).toEqual({ with: { type: 'json' } });
    await runner.close();
  });

  test('bun evaluator can resolve network externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'https://example.invalid/farm-bun-network.mjs',
          type: 'network'
        };
      }
    };

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 25 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-bun-network-hook.mjs'
    );
    expect(result.value).toBe(25);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'https://example.invalid/farm-bun-network.mjs',
      type: 'network',
      evaluator: 'bun',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('bun evaluator keeps native import runtime error without hook fallback', async () => {
    const dir = await createTempDir('farm-runner-bun-runtime-error-');
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new Error('bun-external-runtime-boom');\n"
    );

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=boom`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 199 }
        };
      }
    });

    const error = await runner
      .import('/entry-bun-runtime-error.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('bun-external-runtime-boom');
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('bun evaluator rejects builtin external module in baseline mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun'
    });

    const error = await runner
      .import('/entry-bun-builtin.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in bun evaluator');
    await runner.close();
  });

  test('bun evaluator can resolve builtin via resolveExternalModule hook', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:fs',
          type: 'builtin'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'bun',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 24 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-bun-builtin-hook.mjs'
    );
    expect(result.value).toBe(24);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'node:fs',
      type: 'builtin',
      evaluator: 'bun',
      reason: 'unsupported-external'
    });
    expect(calls[0].error).toBeInstanceOf(Error);
    expect((calls[0].error as Error).message).toContain(
      'builtin external module is not supported'
    );
    await runner.close();
  });

  test('deno evaluator rejects builtin external module in baseline mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'node:path',
          type: 'builtin'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno'
    });

    const error = await runner
      .import('/entry-deno-builtin.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in deno evaluator');
    await runner.close();
  });

  test('deno evaluator rejects commonjs external module in baseline mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/deno-dep.cjs',
          type: 'commonjs'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno'
    });

    const error = await runner
      .import('/entry-deno-cjs.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('not supported in deno evaluator');
    await runner.close();
  });

  test('deno evaluator can resolve commonjs via resolveExternalModule hook', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/deno-dep-hook.cjs',
          type: 'commonjs'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 34 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-deno-cjs-hook.mjs'
    );
    expect(result.value).toBe(34);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: '/tmp/deno-dep-hook.cjs',
      type: 'commonjs',
      evaluator: 'deno',
      reason: 'unsupported-external'
    });
    expect(calls[0].error).toBeInstanceOf(Error);
    expect((calls[0].error as Error).message).toContain(
      'commonjs external module is not supported'
    );
    await runner.close();
  });

  test('deno evaluator can resolve network externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'https://example.invalid/farm-deno-network.mjs',
          type: 'network'
        };
      }
    };

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 33 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-deno-network-hook.mjs'
    );
    expect(result.value).toBe(33);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'https://example.invalid/farm-deno-network.mjs',
      type: 'network',
      evaluator: 'deno',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('deno evaluator forwards import options to resolveExternalModule hook on import-fail', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (id === '/entry-deno-import-options-hook.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('unsupported+scheme://deno-options-entry', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.value);"
            ].join('\n'),
            file: '/entry-deno-import-options-hook.mjs',
            id: '/entry-deno-import-options-hook.mjs',
            url: '/entry-deno-import-options-hook.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === 'unsupported+scheme://deno-options-entry') {
          return {
            externalize: id,
            type: 'module'
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      importOptions?: unknown;
    }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 38 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-deno-import-options-hook.mjs'
    );
    expect(result.value).toBe(38);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://deno-options-entry',
      type: 'module',
      evaluator: 'deno',
      reason: 'import-failed',
      importOptions: { with: { type: 'json' } }
    });
    expect(calls[0].importOptions).toEqual({ with: { type: 'json' } });
    await runner.close();
  });

  test('deno evaluator keeps native import runtime error without hook fallback', async () => {
    const dir = await createTempDir('farm-runner-deno-runtime-error-');
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new Error('deno-external-runtime-boom');\n"
    );

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=boom`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'deno',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 299 }
        };
      }
    });

    const error = await runner
      .import('/entry-deno-runtime-error.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('deno-external-runtime-boom');
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('worker evaluator runs inlined module in baseline mode', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'const value = 7; __farm_ssr_export_name__("value", () => value);',
          file: '/src/worker-inline.ts',
          id: '/src/worker-inline.ts',
          url: '/src/worker-inline.ts',
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'worker'
    });

    const result = await runner.import<{ value: number }>(
      '/src/worker-inline.ts'
    );
    expect(result.value).toBe(7);
    await runner.close();
  });

  test('inlined module still runs when Buffer global is unavailable', async () => {
    const bufferDescriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      'Buffer'
    );

    if (bufferDescriptor && !bufferDescriptor.configurable) {
      return;
    }

    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: "const value = 9; __farm_ssr_export_name__('value', () => value);",
          file: '/src/no-buffer.ts',
          id: '/src/no-buffer.ts',
          url: '/src/no-buffer.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/no-buffer.ts',
            sources: ['/src/no-buffer.ts'],
            sourcesContent: ['const value = 9; export { value };'],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    };

    try {
      Object.defineProperty(globalThis, 'Buffer', {
        configurable: true,
        enumerable: false,
        writable: true,
        value: undefined
      });

      const runner = new FarmModuleRunner({ transport, hmr: false });
      const result = await runner.import<{ value: number }>(
        '/src/no-buffer.ts'
      );
      expect(result.value).toBe(9);
      await runner.close();
    } finally {
      if (bufferDescriptor) {
        Object.defineProperty(globalThis, 'Buffer', bufferDescriptor);
      } else {
        Reflect.deleteProperty(globalThis, 'Buffer');
      }
    }
  });

  test('node evaluator errors clearly when commonjs loader capability is unavailable', async () => {
    const processDescriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      'process'
    );
    const processLike = globalThis.process as NodeJS.Process | undefined;

    if (!processLike) {
      return;
    }

    const builtinDescriptor = Object.getOwnPropertyDescriptor(
      processLike,
      'getBuiltinModule'
    );

    if (builtinDescriptor && !builtinDescriptor.configurable) {
      return;
    }

    try {
      Object.defineProperty(processLike, 'getBuiltinModule', {
        configurable: true,
        enumerable: true,
        writable: true,
        value: undefined
      });

      const transport: ModuleRunnerTransport = {
        async invoke(name) {
          expect(name).toBe('fetchModule');
          return {
            externalize: '/tmp/no-cjs-loader.cjs',
            type: 'commonjs'
          };
        }
      };

      const runner = new FarmModuleRunner({
        transport,
        hmr: false,
        evaluator: 'node'
      });

      const error = await runner
        .import('/entry-no-cjs-loader.cjs')
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(error?.message).toContain('requires Node module loader support');
      await runner.close();
    } finally {
      if (builtinDescriptor) {
        Object.defineProperty(
          processLike,
          'getBuiltinModule',
          builtinDescriptor
        );
      } else {
        Reflect.deleteProperty(processLike, 'getBuiltinModule');
      }

      if (processDescriptor) {
        Object.defineProperty(globalThis, 'process', processDescriptor);
      }
    }
  });

  test('node evaluator can resolve commonjs via hook when loader capability is unavailable', async () => {
    const processLike = globalThis.process as NodeJS.Process | undefined;

    if (!processLike) {
      return;
    }

    const builtinDescriptor = Object.getOwnPropertyDescriptor(
      processLike,
      'getBuiltinModule'
    );

    if (builtinDescriptor && !builtinDescriptor.configurable) {
      return;
    }

    try {
      Object.defineProperty(processLike, 'getBuiltinModule', {
        configurable: true,
        enumerable: true,
        writable: true,
        value: undefined
      });

      const transport: ModuleRunnerTransport = {
        async invoke(name) {
          expect(name).toBe('fetchModule');
          return {
            externalize: '/tmp/no-cjs-loader-hook.cjs',
            type: 'commonjs'
          };
        }
      };

      const calls: Array<{ id: string; type: string; evaluator: string }> = [];
      const runner = new FarmModuleRunner({
        transport,
        hmr: false,
        evaluator: 'node',
        resolveExternalModule: async (context) => {
          calls.push(context);
          return {
            resolved: true,
            module: { value: 17 }
          };
        }
      });

      const result = await runner.import<{ value: number }>(
        '/entry-no-cjs-loader-hook.cjs'
      );
      expect(result.value).toBe(17);
      expect(calls).toHaveLength(1);
      expect(calls[0]).toMatchObject({
        id: '/tmp/no-cjs-loader-hook.cjs',
        type: 'commonjs',
        evaluator: 'node',
        reason: 'missing-node-loader'
      });
      expect(calls[0].error).toBeInstanceOf(Error);
      expect((calls[0].error as Error).message).toContain(
        'requires Node module loader support'
      );
      await runner.close();
    } finally {
      if (builtinDescriptor) {
        Object.defineProperty(
          processLike,
          'getBuiltinModule',
          builtinDescriptor
        );
      } else {
        Reflect.deleteProperty(processLike, 'getBuiltinModule');
      }
    }
  });

  test('node evaluator forwards import options to resolveExternalModule hook when loader capability is unavailable', async () => {
    const processLike = globalThis.process as NodeJS.Process | undefined;

    if (!processLike) {
      return;
    }

    const builtinDescriptor = Object.getOwnPropertyDescriptor(
      processLike,
      'getBuiltinModule'
    );

    if (builtinDescriptor && !builtinDescriptor.configurable) {
      return;
    }

    try {
      Object.defineProperty(processLike, 'getBuiltinModule', {
        configurable: true,
        enumerable: true,
        writable: true,
        value: undefined
      });

      const transport: ModuleRunnerTransport = {
        async invoke(name, data) {
          expect(name).toBe('fetchModule');
          const [id] = data as [
            string,
            string | undefined,
            FetchFunctionOptions | undefined
          ];

          if (id === '/entry-no-cjs-loader-options-hook.cjs') {
            return {
              code: [
                "const dep = await __farm_ssr_import__('/tmp/no-cjs-loader-options.cjs', { with: { type: 'commonjs' } });",
                "__farm_ssr_export_name__('value', () => dep.value);"
              ].join('\n'),
              file: '/entry-no-cjs-loader-options-hook.cjs',
              id: '/entry-no-cjs-loader-options-hook.cjs',
              url: '/entry-no-cjs-loader-options-hook.cjs',
              invalidate: false,
              map: null
            };
          }

          if (id === '/tmp/no-cjs-loader-options.cjs') {
            return {
              externalize: '/tmp/no-cjs-loader-options.cjs',
              type: 'commonjs'
            };
          }

          throw new Error(`unexpected fetch id: ${id}`);
        }
      };

      const calls: Array<{
        id: string;
        type: string;
        evaluator: string;
        reason?: string;
        importOptions?: unknown;
      }> = [];
      const runner = new FarmModuleRunner({
        transport,
        hmr: false,
        evaluator: 'node',
        resolveExternalModule: async (context) => {
          calls.push(context);
          return {
            resolved: true,
            module: { value: 19 }
          };
        }
      });

      const result = await runner.import<{ value: number }>(
        '/entry-no-cjs-loader-options-hook.cjs'
      );
      expect(result.value).toBe(19);
      expect(calls).toHaveLength(1);
      expect(calls[0]).toMatchObject({
        id: '/tmp/no-cjs-loader-options.cjs',
        type: 'commonjs',
        evaluator: 'node',
        reason: 'missing-node-loader',
        importOptions: { with: { type: 'commonjs' } }
      });
      expect(calls[0].importOptions).toEqual({ with: { type: 'commonjs' } });
      await runner.close();
    } finally {
      if (builtinDescriptor) {
        Object.defineProperty(
          processLike,
          'getBuiltinModule',
          builtinDescriptor
        );
      } else {
        Reflect.deleteProperty(processLike, 'getBuiltinModule');
      }
    }
  });

  test('node evaluator can resolve module externalize when native import fails', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: 'unsupported+scheme://node-virtual-entry',
          type: 'module'
        };
      }
    };

    const calls: Array<{
      id: string;
      type: string;
      evaluator: string;
      reason?: string;
      error?: unknown;
    }> = [];
    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'node',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 18 }
        };
      }
    });

    const result = await runner.import<{ value: number }>(
      '/entry-node-module-hook.mjs'
    );
    expect(result.value).toBe(18);
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      id: 'unsupported+scheme://node-virtual-entry',
      type: 'module',
      evaluator: 'node',
      reason: 'import-failed'
    });
    expect(calls[0].error).toBeDefined();
    await runner.close();
  });

  test('node evaluator keeps native import runtime error without hook fallback', async () => {
    const dir = await createTempDir('farm-runner-node-runtime-error-');
    const entry = path.join(dir, 'entry.mjs');
    await fs.writeFile(
      entry,
      "throw new Error('node-external-runtime-boom');\n"
    );

    const calls: Array<{ id: string; type: string; evaluator: string }> = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: `${pathToFileURL(entry).toString()}?runtime=boom`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: 'node',
      resolveExternalModule: async (context) => {
        calls.push(context);
        return {
          resolved: true,
          module: { value: 1018 }
        };
      }
    });

    const error = await runner
      .import('/entry-node-runtime-error.mjs')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('node-external-runtime-boom');
    expect(calls).toEqual([]);
    await runner.close();
  });

  test('runner accepts custom module evaluator from options', async () => {
    let inlinedCalls = 0;
    let externalCalls = 0;
    const customEvaluator: ModuleEvaluator = {
      async runInlinedModule() {
        inlinedCalls++;
      },
      async runExternalModule() {
        externalCalls++;
        return { value: 42 };
      }
    };

    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          externalize: '/tmp/custom.mjs',
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      evaluator: customEvaluator
    });

    const result = await runner.import<{ value: number }>('/entry.mjs');
    expect(result.value).toBe(42);
    expect(externalCalls).toBe(1);
    expect(inlinedCalls).toBe(0);
    await runner.close();
  });

  test('default import.meta generation does not require node:path or node:url', () => {
    const posixMeta = createDefaultImportMeta('/src/entry.ts');
    expect(posixMeta.url).toBe('file:///src/entry.ts');
    expect(posixMeta.dirname).toBe('/src');
    expect(posixMeta.filename).toBe('/src/entry.ts');

    const windowsMeta = createDefaultImportMeta('C:\\repo\\entry.ts');
    expect(windowsMeta.url).toBe('file:///C:/repo/entry.ts');
    expect(windowsMeta.dirname).toBe('C:/repo');
    expect(windowsMeta.filename).toBe('C:\\repo\\entry.ts');
  });

  test('default import.meta generation resolves relative path via runtime cwd', () => {
    const processLike = globalThis.process as NodeJS.Process | undefined;

    if (!processLike) {
      return;
    }

    const cwdDescriptor = Object.getOwnPropertyDescriptor(processLike, 'cwd');

    if (cwdDescriptor && !cwdDescriptor.configurable) {
      return;
    }

    try {
      Object.defineProperty(processLike, 'cwd', {
        configurable: true,
        enumerable: false,
        writable: true,
        value: () => '/runtime/root'
      });

      const relativeMeta = createDefaultImportMeta('src/entry.ts');
      expect(relativeMeta.url).toBe('file:///runtime/root/src/entry.ts');
    } finally {
      if (cwdDescriptor) {
        Object.defineProperty(processLike, 'cwd', cwdDescriptor);
      } else {
        Reflect.deleteProperty(processLike, 'cwd');
      }
    }
  });

  test('inlined module resolves relative imports without node:path dependency', async () => {
    const fetchIds: string[] = [];

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');

        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];
        fetchIds.push(id);

        if (id === '/dir/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('./dep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value + 1);"
            ].join('\n'),
            file: '/dir/entry.mjs',
            id: '/dir/entry.mjs',
            url: '/dir/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dir/dep.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 2);",
            file: '/dir/dep.mjs',
            id: '/dir/dep.mjs',
            url: '/dir/dep.mjs',
            invalidate: false,
            map: null
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const mod = await runner.import<{ value: number }>('/dir/entry.mjs');
    expect(mod.value).toBe(3);
    expect(fetchIds).toEqual(['/dir/entry.mjs', '/dir/dep.mjs']);
    await runner.close();
  });

  test('inlined module accepts dynamic import options argument', async () => {
    const fetchIds: string[] = [];

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');

        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];
        fetchIds.push(id);

        if (id === '/dir/entry-with-options.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('./dep.mjs', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.value + 1);"
            ].join('\n'),
            file: '/dir/entry-with-options.mjs',
            id: '/dir/entry-with-options.mjs',
            url: '/dir/entry-with-options.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dir/dep.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 2);",
            file: '/dir/dep.mjs',
            id: '/dir/dep.mjs',
            url: '/dir/dep.mjs',
            invalidate: false,
            map: null
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const mod = await runner.import<{ value: number }>(
      '/dir/entry-with-options.mjs'
    );
    expect(mod.value).toBe(3);
    expect(fetchIds).toEqual(['/dir/entry-with-options.mjs', '/dir/dep.mjs']);
    await runner.close();
  });

  test('forwards dynamic import options to external module loading', async () => {
    const dir = await createTempDir('farm-runner-dynamic-options-external-');
    const depJson = path.join(dir, 'dep.json');
    await fs.writeFile(depJson, JSON.stringify({ value: 2 }));

    const fetchIds: string[] = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];
        fetchIds.push(id);

        if (id === '/dir/entry-external-options.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_dynamic_import__('./dep.json', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.default.value + 1);"
            ].join('\n'),
            file: '/dir/entry-external-options.mjs',
            id: '/dir/entry-external-options.mjs',
            url: '/dir/entry-external-options.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dir/dep.json') {
          return {
            externalize: pathToFileURL(depJson).toString(),
            type: 'module'
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const mod = await runner.import<{ value: number }>(
      '/dir/entry-external-options.mjs'
    );
    expect(mod.value).toBe(3);
    expect(fetchIds).toEqual([
      '/dir/entry-external-options.mjs',
      '/dir/dep.json'
    ]);
    await runner.close();
  });

  test('forwards static import attributes options to external module loading', async () => {
    const dir = await createTempDir('farm-runner-static-attributes-external-');
    const depJson = path.join(dir, 'dep.json');
    await fs.writeFile(depJson, JSON.stringify({ value: 2 }));

    const fetchIds: string[] = [];
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];
        fetchIds.push(id);

        if (id === '/dir/entry-static-attributes.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('./dep.json', { with: { type: 'json' } });",
              "__farm_ssr_export_name__('value', () => dep.default.value + 1);"
            ].join('\n'),
            file: '/dir/entry-static-attributes.mjs',
            id: '/dir/entry-static-attributes.mjs',
            url: '/dir/entry-static-attributes.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dir/dep.json') {
          return {
            externalize: pathToFileURL(depJson).toString(),
            type: 'module'
          };
        }

        throw new Error(`unexpected fetch id: ${id}`);
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const mod = await runner.import<{ value: number }>(
      '/dir/entry-static-attributes.mjs'
    );
    expect(mod.value).toBe(3);
    expect(fetchIds).toEqual([
      '/dir/entry-static-attributes.mjs',
      '/dir/dep.json'
    ]);
    await runner.close();
  });

  test('reuses local cache when fetchModule returns cache=true', async () => {
    const dir = await createTempDir('farm-runner-cache-');
    const entry = path.join(dir, 'entry.mjs');

    await fs.writeFile(entry, 'export const value = 1;\n');

    let fetchCount = 0;
    let cacheChecks = 0;

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        fetchCount++;
        const [id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        expect(id).toBe('/entry.mjs');

        if (options?.cached) {
          cacheChecks++;
          return { cache: true };
        }

        return {
          externalize: pathToFileURL(entry).toString() + '?t=1',
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });

    const first = await runner.import<{ value: number }>('/entry.mjs');
    const second = await runner.import<{ value: number }>('/entry.mjs');

    expect(first.value).toBe(1);
    expect(second.value).toBe(1);
    expect(fetchCount).toBe(2);
    expect(cacheChecks).toBe(1);

    await runner.close();
  });

  test('invalidates cached module on hmr update payload', async () => {
    const dir = await createTempDir('farm-runner-hmr-');
    const entry = path.join(dir, 'entry.mjs');

    await fs.writeFile(entry, 'export const value = 1;\n');

    let currentVersion = 1;
    let invalidated = false;
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const cachedRequestFlags: boolean[] = [];

    const transport: ModuleRunnerTransport = {
      connect({ onMessage }) {
        hmrMessageHandler = onMessage;
        onMessage({ type: 'connected' });
      },
      async invoke(name, data) {
        expect(name).toBe('fetchModule');

        const [_id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];
        cachedRequestFlags.push(Boolean(options?.cached));

        if (options?.cached && !invalidated) {
          return { cache: true };
        }

        invalidated = false;

        return {
          externalize: pathToFileURL(entry).toString() + `?t=${currentVersion}`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    const first = await runner.import<{ value: number }>('/entry.mjs');
    expect(first.value).toBe(1);

    await fs.writeFile(entry, 'export const value = 2;\n');
    currentVersion = 2;
    invalidated = true;
    expect(hmrMessageHandler).toBeTypeOf('function');

    hmrMessageHandler?.({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/entry.mjs',
          acceptedPath: '/entry.mjs',
          timestamp: Date.now()
        }
      ]
    });

    await runner.import<{ value: number }>('/entry.mjs');
    expect(cachedRequestFlags).toEqual([false, true]);

    await runner.close();
  });

  test('hmr update keeps unaffected module cache warm', async () => {
    const dir = await createTempDir('farm-runner-hmr-targeted-');
    const entryA = path.join(dir, 'entry-a.mjs');
    const entryB = path.join(dir, 'entry-b.mjs');

    await Promise.all([
      fs.writeFile(entryA, 'export const value = 1;\n'),
      fs.writeFile(entryB, 'export const value = 10;\n')
    ]);

    let versionA = 1;
    let pendingAInvalidation = false;
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const cachedFlagsById: Record<string, boolean[]> = {
      '/entry-a.mjs': [],
      '/entry-b.mjs': []
    };
    const externalizeLoads: Record<string, number> = {
      '/entry-a.mjs': 0,
      '/entry-b.mjs': 0
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

        if (id === '/entry-a.mjs') {
          if (options?.cached && !pendingAInvalidation) {
            return { cache: true };
          }

          pendingAInvalidation = false;
          externalizeLoads[id]++;
          return {
            externalize: `${pathToFileURL(entryA).toString()}?t=${versionA}`,
            type: 'module'
          };
        }

        if (options?.cached) {
          return { cache: true };
        }

        externalizeLoads[id]++;
        return {
          externalize: `${pathToFileURL(entryB).toString()}?t=1`,
          type: 'module'
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    await runner.import('/entry-a.mjs');
    await runner.import('/entry-b.mjs');

    versionA = 2;
    pendingAInvalidation = true;
    hmrMessageHandler?.({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/entry-a.mjs',
          acceptedPath: '/entry-a.mjs',
          timestamp: Date.now()
        }
      ]
    });

    await runner.import('/entry-a.mjs');
    await runner.import('/entry-b.mjs');

    expect(cachedFlagsById['/entry-a.mjs']).toEqual([false, true]);
    expect(cachedFlagsById['/entry-b.mjs']).toEqual([false, true]);
    expect(externalizeLoads['/entry-a.mjs']).toBe(2);
    expect(externalizeLoads['/entry-b.mjs']).toBe(1);

    await runner.close();
  });

  test('hmr update invalidates circular importers without infinite recursion', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const cachedFlagsById: Record<string, boolean[]> = {
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

        if (options?.cached) {
          return { cache: true };
        }

        if (id === '/a.mjs') {
          return {
            code: [
              "const b = await __farm_ssr_import__('/b.mjs');",
              "__farm_ssr_export_name__('value', () => b.value + 1);"
            ].join('\n'),
            file: '/a.mjs',
            id: '/a.mjs',
            url: '/a.mjs',
            invalidate: false,
            map: null
          };
        }

        return {
          code: [
            "const a = await __farm_ssr_import__('/a.mjs');",
            "__farm_ssr_export_name__('touchA', () => Boolean(a));",
            "__farm_ssr_export_name__('value', () => 1);"
          ].join('\n'),
          file: '/b.mjs',
          id: '/b.mjs',
          url: '/b.mjs',
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    const first = await runner.import<{ value: number }>('/a.mjs');
    expect(first.value).toBe(2);

    const beforeUpdateA = runner.evaluatedModules.getModuleByUrl('/a.mjs');
    const beforeUpdateB = runner.evaluatedModules.getModuleByUrl('/b.mjs');
    expect(beforeUpdateA?.importers.has('/b.mjs')).toBe(true);
    expect(beforeUpdateB?.importers.has('/a.mjs')).toBe(true);

    hmrMessageHandler?.({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/a.mjs',
          acceptedPath: '/a.mjs',
          timestamp: Date.now()
        }
      ]
    });

    const modA = runner.evaluatedModules.getModuleByUrl('/a.mjs');
    const modB = runner.evaluatedModules.getModuleByUrl('/b.mjs');

    expect(modA?.meta).toBeUndefined();
    expect(modB?.meta).toBeUndefined();
    expect(cachedFlagsById['/a.mjs'].length).toBeGreaterThan(0);
    expect(cachedFlagsById['/b.mjs'].length).toBeGreaterThan(0);

    await runner.close();
  });

  test('hmr update invalidates circular group and keeps unrelated module warm', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const cachedFlagsById: Record<string, boolean[]> = {
      '/a.mjs': [],
      '/b.mjs': [],
      '/c.mjs': []
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

        if (id === '/a.mjs') {
          return {
            code: [
              "const b = await __farm_ssr_import__('/b.mjs');",
              "__farm_ssr_export_name__('value', () => b.value + 1);"
            ].join('\n'),
            file: '/a.mjs',
            id: '/a.mjs',
            url: '/a.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/b.mjs') {
          return {
            code: [
              "const a = await __farm_ssr_import__('/a.mjs');",
              "__farm_ssr_export_name__('touchA', () => Boolean(a));",
              "__farm_ssr_export_name__('value', () => 1);"
            ].join('\n'),
            file: '/b.mjs',
            id: '/b.mjs',
            url: '/b.mjs',
            invalidate: false,
            map: null
          };
        }

        return {
          code: "__farm_ssr_export_name__('value', () => 3);",
          file: '/c.mjs',
          id: '/c.mjs',
          url: '/c.mjs',
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: true });

    const firstA = await runner.import<{ value: number }>('/a.mjs');
    const firstC = await runner.import<{ value: number }>('/c.mjs');
    expect(firstA.value).toBe(2);
    expect(firstC.value).toBe(3);

    hmrMessageHandler?.({
      type: 'update',
      updates: [
        {
          type: 'js-update',
          path: '/a.mjs',
          acceptedPath: '/a.mjs',
          timestamp: Date.now()
        }
      ]
    });

    const modA = runner.evaluatedModules.getModuleByUrl('/a.mjs');
    const modB = runner.evaluatedModules.getModuleByUrl('/b.mjs');
    const modC = runner.evaluatedModules.getModuleByUrl('/c.mjs');

    expect(modA?.meta).toBeUndefined();
    expect(modB?.meta).toBeUndefined();
    expect(modC?.meta).toBeDefined();

    const secondC = await runner.import<{ value: number }>('/c.mjs');
    expect(secondC.value).toBe(3);
    expect(cachedFlagsById['/c.mjs']).toEqual([false, true]);

    await runner.close();
  });

  test('loads externalized commonjs modules via require semantics', async () => {
    const dir = await createTempDir('farm-runner-cjs-');
    const entry = path.join(dir, 'entry.cjs');
    await fs.writeFile(entry, 'module.exports = { value: 42 };\n');

    let fetchCount = 0;
    let cacheChecks = 0;

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        fetchCount++;

        const [_id, _importer, options] = data as [
          string,
          string | undefined,
          FetchFunctionOptions | undefined
        ];

        if (options?.cached) {
          cacheChecks++;
          return { cache: true };
        }

        return {
          externalize: `${pathToFileURL(entry).toString()}?t=1`,
          type: 'commonjs'
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const first = await runner.import<{ value: number }>('/entry.cjs');
    const second = await runner.import<{ value: number }>('/entry.cjs');

    expect(first.value).toBe(42);
    expect(second.value).toBe(42);
    expect(fetchCount).toBe(2);
    expect(cacheChecks).toBe(1);

    await runner.close();
  });

  test('inlined module executes with source map payload', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: '__farm_ssr_export_name__("value", () => 7);',
          file: '/src/entry-server.ts',
          id: '/src/entry-server.ts',
          url: '/src/entry-server.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/entry-server.ts',
            sources: ['/src/entry-server.ts'],
            names: [],
            mappings: ''
          })
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const mod = await runner.import<{ value: number }>('/src/entry-server.ts');
    expect(mod.value).toBe(7);

    await runner.close();
  });

  test('inlined module error stack points to source file when map exists', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("runner-boom")',
          file: '/src/entry-server.ts',
          id: '/src/entry-server.ts',
          url: '/src/entry-server.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/entry-server.ts',
            sources: ['/src/entry-server.ts'],
            sourcesContent: ['throw new Error("runner-boom")'],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const error = await runner
      .import('/src/entry-server.ts')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('runner-boom');
    expect(String(error?.stack ?? '')).toContain('/src/entry-server.ts');

    await runner.close();
  });

  test('source map interceptor remaps stack to original source', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("runner-remap-boom")',
          file: '/src/generated-entry.ts',
          id: '/src/generated-entry.ts',
          url: '/src/generated-entry.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/generated-entry.ts',
            sources: ['/src/original-entry.ts'],
            sourcesContent: ['throw new Error("runner-remap-boom")'],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    };

    const runner = new FarmModuleRunner({ transport, hmr: false });
    const error = await runner
      .import('/src/generated-entry.ts')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('runner-remap-boom');
    expect(String(error?.stack ?? '')).toContain('/src/original-entry.ts');

    await runner.close();
  });

  test('source map interceptor can be disabled', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("runner-remap-disabled")',
          file: '/src/generated-disabled.ts',
          id: '/src/generated-disabled.ts',
          url: '/src/generated-disabled.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/generated-disabled.ts',
            sources: ['/src/original-disabled.ts'],
            sourcesContent: ['throw new Error("runner-remap-disabled")'],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      sourceMapInterceptor: false
    });

    const error = await runner
      .import('/src/generated-disabled.ts')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('runner-remap-disabled');
    expect(String(error?.stack ?? '')).toContain('/src/generated-disabled.ts');
    expect(String(error?.stack ?? '')).not.toContain(
      '/src/original-disabled.ts'
    );

    await runner.close();
  });

  test('source map interceptor degrades when process global is unavailable', () => {
    const processDescriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      'process'
    );

    if (processDescriptor && !processDescriptor.configurable) {
      return;
    }

    const sourceMap = JSON.stringify({
      version: 3,
      file: '/src/no-process.ts',
      sources: ['/src/original-no-process.ts'],
      sourcesContent: ['throw new Error("runner-no-process")'],
      names: [],
      mappings: 'AAAA'
    });

    let interceptor:
      | ReturnType<typeof createRunnerSourceMapInterceptor>
      | undefined;

    try {
      Object.defineProperty(globalThis, 'process', {
        configurable: true,
        enumerable: false,
        writable: true,
        value: undefined
      });

      interceptor = createRunnerSourceMapInterceptor(true, true);
      interceptor.register('/src/no-process.ts', sourceMap);
      interceptor.unregister('/src/no-process.ts');
      interceptor.clear();
    } finally {
      interceptor?.close();

      if (processDescriptor) {
        Object.defineProperty(globalThis, 'process', processDescriptor);
      } else {
        Reflect.deleteProperty(globalThis, 'process');
      }
    }
  });

  test('source map interceptor retries SourceMap lookup after process restore', async () => {
    const processDescriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      'process'
    );

    if (processDescriptor && !processDescriptor.configurable) {
      return;
    }

    const realProcess = globalThis.process as NodeJS.Process | undefined;

    if (!realProcess) {
      return;
    }

    const getBuiltinModuleDescriptor = Object.getOwnPropertyDescriptor(
      realProcess,
      'getBuiltinModule'
    );

    if (
      getBuiltinModuleDescriptor &&
      !getBuiltinModuleDescriptor.configurable
    ) {
      return;
    }

    const originalGetBuiltinModule =
      realProcess.getBuiltinModule?.bind(realProcess);

    if (!originalGetBuiltinModule) {
      return;
    }

    const sourceMap = JSON.stringify({
      version: 3,
      file: '/src/retry-process.ts',
      sources: ['/src/original-retry-process.ts'],
      sourcesContent: ['throw new Error("runner-retry-process")'],
      names: [],
      mappings: 'AAAA'
    });

    const freshModuleUrl = new URL(
      `../src/module-runner/sourceMapInterceptor.js?retry=${Date.now()}`,
      import.meta.url
    );
    const { createRunnerSourceMapInterceptor: createFreshInterceptor } =
      await import(freshModuleUrl.href);

    let lookupCalls = 0;

    try {
      Object.defineProperty(realProcess, 'getBuiltinModule', {
        configurable: true,
        enumerable: true,
        writable: true,
        value: (id: string) => {
          lookupCalls++;
          return originalGetBuiltinModule(id);
        }
      });

      Object.defineProperty(globalThis, 'process', {
        configurable: true,
        enumerable: false,
        writable: true,
        value: undefined
      });

      const first = createFreshInterceptor(true, true);
      first.register('/src/retry-process.ts', sourceMap);
      first.close();

      Object.defineProperty(globalThis, 'process', {
        configurable: true,
        enumerable: false,
        writable: true,
        value: realProcess
      });

      const second = createFreshInterceptor(true, true);
      second.register('/src/retry-process.ts', sourceMap);
      second.close();

      expect(lookupCalls).toBeGreaterThan(0);
    } finally {
      if (getBuiltinModuleDescriptor) {
        Object.defineProperty(
          realProcess,
          'getBuiltinModule',
          getBuiltinModuleDescriptor
        );
      } else {
        Reflect.deleteProperty(realProcess, 'getBuiltinModule');
      }

      if (processDescriptor) {
        Object.defineProperty(globalThis, 'process', processDescriptor);
      } else {
        Reflect.deleteProperty(globalThis, 'process');
      }
    }
  });

  test('source map interceptor degrades when Error.prepareStackTrace is locked', () => {
    const prepareDescriptor = Object.getOwnPropertyDescriptor(
      Error,
      'prepareStackTrace'
    );

    if (prepareDescriptor && !prepareDescriptor.configurable) {
      return;
    }

    const sourceMap = JSON.stringify({
      version: 3,
      file: '/src/locked-prepare.ts',
      sources: ['/src/original-locked-prepare.ts'],
      sourcesContent: ['throw new Error("runner-locked-prepare")'],
      names: [],
      mappings: 'AAAA'
    });

    let interceptor:
      | ReturnType<typeof createRunnerSourceMapInterceptor>
      | undefined;

    try {
      Object.defineProperty(Error, 'prepareStackTrace', {
        configurable: true,
        enumerable: false,
        writable: false,
        value: prepareDescriptor?.value
      });

      interceptor = createRunnerSourceMapInterceptor(true, true);
      interceptor.register('/src/locked-prepare.ts', sourceMap);
      interceptor.unregister('/src/locked-prepare.ts');
      interceptor.clear();
    } finally {
      interceptor?.close();

      if (prepareDescriptor) {
        Object.defineProperty(Error, 'prepareStackTrace', prepareDescriptor);
      } else {
        Reflect.deleteProperty(Error, 'prepareStackTrace');
      }
    }
  });

  test('source map interceptor close tolerates locked Error.prepareStackTrace', () => {
    const prepareDescriptor = Object.getOwnPropertyDescriptor(
      Error,
      'prepareStackTrace'
    );

    if (prepareDescriptor && !prepareDescriptor.configurable) {
      return;
    }

    const interceptor = createRunnerSourceMapInterceptor(true, true);

    try {
      Object.defineProperty(Error, 'prepareStackTrace', {
        configurable: true,
        enumerable: false,
        writable: false,
        value: Error.prepareStackTrace
      });

      expect(() => interceptor.close()).not.toThrow();
    } finally {
      if (prepareDescriptor) {
        Object.defineProperty(Error, 'prepareStackTrace', prepareDescriptor);
      } else {
        Reflect.deleteProperty(Error, 'prepareStackTrace');
      }
    }
  });

  test('source map interceptor remaps stack when native toggle is off', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("runner-remap-native-off")',
          file: '/src/generated-native-off.ts',
          id: '/src/generated-native-off.ts',
          url: '/src/generated-native-off.ts',
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: '/src/generated-native-off.ts',
            sources: ['/src/original-native-off.ts'],
            sourcesContent: ['throw new Error("runner-remap-native-off")'],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      sourceMapInterceptor: {
        native: false
      }
    });

    const error = await runner
      .import('/src/generated-native-off.ts')
      .then(() => null)
      .catch((e) => e as Error);

    expect(error).toBeInstanceOf(Error);
    expect(error?.message).toContain('runner-remap-native-off');
    expect(String(error?.stack ?? '')).toContain('/src/original-native-off.ts');

    await runner.close();
  });

  test('source map interceptor supports multi-runner override and close fallback', async () => {
    const generatedId = '/src/shared-generated.ts';

    const createThrowingTransport = (
      originalSource: string,
      message: string
    ): ModuleRunnerTransport => ({
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: `throw new Error(${JSON.stringify(message)})`,
          file: generatedId,
          id: generatedId,
          url: generatedId,
          invalidate: true,
          map: JSON.stringify({
            version: 3,
            file: generatedId,
            sources: [originalSource],
            sourcesContent: [`throw new Error(${JSON.stringify(message)})`],
            names: [],
            mappings: 'AAAA'
          })
        };
      }
    });

    const runnerA = new FarmModuleRunner({
      transport: createThrowingTransport('/src/original-a.ts', 'runner-a-boom'),
      hmr: false
    });

    const errorA = await runnerA
      .import(generatedId)
      .then(() => null)
      .catch((e) => e as Error);

    expect(errorA).toBeInstanceOf(Error);
    expect(String(errorA?.stack ?? '')).toContain('/src/original-a.ts');

    const runnerB = new FarmModuleRunner({
      transport: createThrowingTransport('/src/original-b.ts', 'runner-b-boom'),
      hmr: false
    });

    const errorB = await runnerB
      .import(generatedId)
      .then(() => null)
      .catch((e) => e as Error);

    expect(errorB).toBeInstanceOf(Error);
    expect(String(errorB?.stack ?? '')).toContain('/src/original-b.ts');
    expect(String(errorB?.stack ?? '')).not.toContain('/src/original-a.ts');

    await runnerB.close();

    const errorAAfterClose = await runnerA
      .import(generatedId)
      .then(() => null)
      .catch((e) => e as Error);

    expect(errorAAfterClose).toBeInstanceOf(Error);
    expect(String(errorAAfterClose?.stack ?? '')).toContain(
      '/src/original-a.ts'
    );
    expect(String(errorAAfterClose?.stack ?? '')).not.toContain(
      '/src/original-b.ts'
    );

    await runnerA.close();
  });

  test('server invoke uses compiler.fetchModule result before fallback path', async () => {
    let fetchModuleCalls = 0;
    let fallbackTouched = false;

    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => {
          fetchModuleCalls++;
          return {
            externalize: 'node:fs',
            type: 'builtin'
          };
        },
        resource: () => {
          fallbackTouched = true;
          return null;
        },
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('/entry.ts');

    expect(result).toEqual({
      externalize: 'node:fs',
      type: 'builtin'
    });
    expect(fetchModuleCalls).toBe(1);
    expect(fallbackTouched).toBe(false);
  });

  test('server invoke retries compiler.fetchModule with root-absolute id for root-relative requests', async () => {
    const root = await createTempDir('farm-runner-invoke-root-retry-');
    const absoluteEntry = path.join(root, 'src', 'entry-server.ts');
    const calls: string[] = [];

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: (id: string) => {
          calls.push(id);
          if (id !== absoluteEntry) {
            return null;
          }

          return {
            code: 'const value = 1; __farm_ssr_export_name__("value", () => value);',
            file: absoluteEntry,
            id: 'src/entry-server.ts',
            url: '/src/entry-server.ts',
            invalidate: false,
            map: null
          };
        },
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(handlers.fetchModule('/src/entry-server.ts')).resolves.toEqual(
      {
        code: 'const value = 1; __farm_ssr_export_name__("value", () => value);',
        file: absoluteEntry,
        id: 'src/entry-server.ts',
        url: '/src/entry-server.ts',
        invalidate: false,
        map: null
      }
    );

    expect(calls).toEqual(['/src/entry-server.ts', absoluteEntry]);
  });

  test('server wrapper and context invoke handlers return consistent result', async () => {
    const root = await createTempDir('farm-runner-wrapper-context-');
    const entry = path.join(root, 'entry.js');
    await fs.writeFile(
      path.join(root, 'package.json'),
      JSON.stringify({ type: 'commonjs' })
    );
    await fs.writeFile(entry, 'module.exports = { value: 1 };\n');

    const compiler = {
      compiling: false,
      waitForCompileFinish: async () => undefined,
      fetchModule: () => ({
        externalize: pathToFileURL(entry).toString(),
        type: 'module' as const
      }),
      resource: () => null,
      resources: () => ({})
    };

    const serverLike = {
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler
    };

    const serverHandlers = createServerModuleRunnerInvokeHandlers(
      serverLike as never
    );
    const contextHandlers = createModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler
    });

    const byServer = await serverHandlers.fetchModule('/entry.js');
    const byContext = await contextHandlers.fetchModule('/entry.js');

    expect(byContext).toEqual(byServer);
  });

  test('server invoke passes through compiler.fetchModule bailout reason', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: 'node:fs',
          type: 'builtin',
          bailoutReason: 'unsupported-ts'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(handlers.fetchModule('/entry.ts')).resolves.toEqual({
      externalize: 'node:fs',
      type: 'builtin',
      bailoutReason: 'unsupported-ts'
    });
  });

  test('server invoke normalizes inlined fetch source map field', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          code: 'export const value = 1;',
          file: null,
          id: '/entry.ts',
          url: '/entry.ts',
          invalidate: false
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('/entry.ts');

    expect(result).toEqual({
      code: 'export const value = 1;',
      file: null,
      id: '/entry.ts',
      url: '/entry.ts',
      invalidate: false,
      map: null
    });
  });

  test('server invoke rejects invalid compiler.fetchModule payload', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: 'node:fs',
          type: 'invalid'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(handlers.fetchModule('/entry.ts')).rejects.toThrow(
      'Invalid fetchModule result'
    );
  });

  test('server invoke rejects invalid compiler.fetchModule bailout reason', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: 'node:fs',
          type: 'builtin',
          bailoutReason: 'unknown-reason'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(handlers.fetchModule('/entry.ts')).rejects.toThrow(
      'Invalid fetchModule result'
    );
  });

  test('server invoke fallback classifies builtin and network requests', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(
      handlers.fetchModule('data:text/javascript,export%20default%201')
    ).resolves.toEqual({
      externalize: 'data:text/javascript,export%20default%201',
      type: 'builtin'
    });

    await expect(handlers.fetchModule('node:fs')).resolves.toEqual({
      externalize: 'node:fs',
      type: 'builtin'
    });

    await expect(
      handlers.fetchModule('https://example.com/mod.js')
    ).resolves.toEqual({
      externalize: 'https://example.com/mod.js',
      type: 'network'
    });
  });

  test('server invoke provides inlined fetch result for vite vue export helper virtual module', async () => {
    const handlers = createServerModuleRunnerInvokeHandlers({
      root: process.cwd(),
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('\0plugin-vue:export-helper');
    expect(result).toMatchObject({
      id: '\0plugin-vue:export-helper',
      url: '\0plugin-vue:export-helper',
      invalidate: false
    });
    expect('code' in result ? result.code : '').toContain(
      '__farm_ssr_export_name__("default", () => exportHelper);'
    );
  });

  test('server invoke returns vue style query noop module for not-script bailout', async () => {
    const root = await createTempDir('farm-runner-vue-style-noop-');
    const aboutPagePath = path.join(root, 'src', 'pages', 'AboutPage.vue');
    await fs.mkdir(path.dirname(aboutPagePath), { recursive: true });
    await fs.writeFile(
      aboutPagePath,
      '<template><div>about</div></template><style scoped>.x{color:red}</style>\n'
    );

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: pathToFileURL(aboutPagePath).toString(),
          type: 'module',
          bailoutReason: 'not-script'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const styleId = '/src/pages/AboutPage.vue?vue&type=style&index=0&lang.css';
    const result = await handlers.fetchModule(styleId);

    expect(result).toMatchObject({
      id: styleId,
      url: styleId,
      file: aboutPagePath,
      invalidate: false,
      map: null
    });
    expect('code' in result ? result.code : '').toContain(
      '__farm_ssr_export_name__("default", () => __farm_ssr_style_noop__);'
    );
  });

  test('server invoke fallback resolves direct files with versioned externalize and cache hit', async () => {
    const root = await createTempDir('farm-runner-invoke-');
    const entry = path.join(root, 'entry.cjs');
    await fs.writeFile(entry, 'module.exports = { value: 1 };\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 9,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const first = await handlers.fetchModule('/entry.cjs');
    expect(first).toMatchObject({
      type: 'commonjs'
    });
    expect('externalize' in first ? first.externalize : '').toContain('?t=9-');

    await expect(
      handlers.fetchModule('/entry.cjs', undefined, { cached: true })
    ).resolves.toEqual({ cache: true });
  });

  test('server invoke fallback invalidates direct file cache when mtime changes', async () => {
    const root = await createTempDir('farm-runner-mtime-invalidate-');
    const entry = path.join(root, 'entry.mjs');
    await fs.writeFile(entry, 'export const value = 1;\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 11,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const first = await handlers.fetchModule('/entry.mjs');
    expect('externalize' in first ? first.externalize : '').toContain('?t=11-');

    const currentStat = await fs.stat(entry);
    const updatedTime = new Date(currentStat.mtimeMs + 5000);
    await fs.utimes(entry, updatedTime, updatedTime);

    const second = await handlers.fetchModule('/entry.mjs', undefined, {
      cached: true
    });

    expect(second).toMatchObject({
      type: 'module'
    });
    expect('cache' in second).toBe(false);

    if ('externalize' in first && 'externalize' in second) {
      expect(second.externalize).not.toBe(first.externalize);
    }
  });

  test('context invoke handlers keep state isolated across contexts', async () => {
    const root = await createTempDir('farm-runner-context-isolate-');
    const entry = path.join(root, 'entry.mjs');
    await fs.writeFile(entry, 'export const value = 1;\n');

    const compiler = {
      compiling: false,
      waitForCompileFinish: async () => undefined,
      fetchModule: () => null,
      resource: () => null,
      resources: () => ({})
    };

    const handlersA = createModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 13,
      compiler
    });

    const handlersB = createModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 13,
      compiler
    });

    await handlersA.fetchModule('/entry.mjs');

    await expect(
      handlersA.fetchModule('/entry.mjs', undefined, { cached: true })
    ).resolves.toEqual({ cache: true });

    const resultFromB = await handlersB.fetchModule('/entry.mjs', undefined, {
      cached: true
    });

    expect('cache' in resultFromB).toBe(false);
    expect(resultFromB).toMatchObject({
      type: 'module'
    });
  });

  test('server invoke fallback classifies js file as commonjs when nearest package type is commonjs', async () => {
    const root = await createTempDir('farm-runner-js-cjs-');
    const entry = path.join(root, 'entry.js');
    await fs.writeFile(
      path.join(root, 'package.json'),
      JSON.stringify({ type: 'commonjs' })
    );
    await fs.writeFile(entry, 'module.exports = { value: 1 };\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('/entry.js');
    expect(result).toMatchObject({
      type: 'commonjs'
    });
  });

  test('server invoke fallback classifies js file as module when nearest package type is module', async () => {
    const root = await createTempDir('farm-runner-js-esm-');
    const entry = path.join(root, 'entry.js');
    await fs.writeFile(
      path.join(root, 'package.json'),
      JSON.stringify({ type: 'module' })
    );
    await fs.writeFile(entry, 'export const value = 1;\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('/entry.js');
    expect(result).toMatchObject({
      type: 'module'
    });
  });

  test('server invoke normalizes compiler externalize js type by nearest package type', async () => {
    const root = await createTempDir('farm-runner-compiler-type-normalize-');
    const entry = path.join(root, 'entry.js');
    await fs.writeFile(
      path.join(root, 'package.json'),
      JSON.stringify({ type: 'commonjs' })
    );
    await fs.writeFile(entry, 'module.exports = { value: 1 };\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: pathToFileURL(entry).toString(),
          type: 'module'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    await expect(handlers.fetchModule('/entry.js')).resolves.toEqual({
      externalize: pathToFileURL(entry).toString(),
      type: 'commonjs'
    });
  });

  test('server invoke fallback resolves root-relative entry for importer-less relative ids', async () => {
    const root = await createTempDir('farm-runner-relative-entry-');
    const entry = path.join(root, 'entry.mjs');
    await fs.writeFile(entry, 'export const value = 1;\n');

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 3,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('./entry.mjs');
    expect(result).toMatchObject({
      type: 'module'
    });
    expect('externalize' in result ? result.externalize : '').toContain(
      '/entry.mjs'
    );
  });

  test('server invoke resolves bare specifier from root-resolved relative importer path', async () => {
    const root = await createTempDir('farm-runner-relative-importer-base-');
    const entry = path.join(root, 'src', 'entry.mjs');
    const depDir = path.join(root, 'node_modules', 'dep');
    await fs.mkdir(path.dirname(entry), { recursive: true });
    await fs.mkdir(depDir, { recursive: true });
    await fs.writeFile(entry, 'export const value = 1;\n');
    await fs.writeFile(
      path.join(depDir, 'package.json'),
      JSON.stringify({ name: 'dep', type: 'module', main: './index.mjs' })
    );
    await fs.writeFile(
      path.join(depDir, 'index.mjs'),
      'export const dep = "ok";\n'
    );

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 5,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => null,
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const result = await handlers.fetchModule('dep', 'src/entry.mjs');
    expect(result).toMatchObject({
      type: 'module'
    });
    expect('externalize' in result ? result.externalize : '').toContain(
      '/node_modules/dep/index.mjs'
    );
  });
});
