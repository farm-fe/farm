import { describe, expect, test } from 'vitest';
import { ModuleRunnerDiagnosticsBus } from '../src/module-runner/diagnostics.js';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type {
  FetchFunctionOptions,
  ModuleRunnerDiagnosticsEvent,
  ModuleRunnerTransport,
  RunnerHotPayload
} from '../src/module-runner/types.js';

describe('farm module runner diagnostics', () => {
  test('emits fetch/eval/hmr lifecycle events', async () => {
    let hmrMessageHandler: ((payload: RunnerHotPayload) => void) | undefined;
    const initialized = new Set<string>();
    const diagnostics = new ModuleRunnerDiagnosticsBus();
    const events: ModuleRunnerDiagnosticsEvent[] = [];
    const off = diagnostics.subscribe((event) => events.push(event));

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

        if (options?.cached && initialized.has(id)) {
          return { cache: true };
        }
        initialized.add(id);

        if (id === '/entry.mjs') {
          return {
            code: [
              "const dep = await __farm_ssr_import__('/dep.mjs');",
              "__farm_ssr_export_name__('value', () => dep.value + 1);"
            ].join('\n'),
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/dep.mjs') {
          return {
            code: "__farm_ssr_export_name__('value', () => 1);",
            file: '/dep.mjs',
            id: '/dep.mjs',
            url: '/dep.mjs',
            invalidate: false,
            map: null
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: true,
      diagnostics
    });

    try {
      await runner.import('/entry.mjs');
      await runner.import('/entry.mjs');
      hmrMessageHandler?.({
        type: 'update',
        updates: [
          {
            type: 'js-update',
            path: '/dep.mjs',
            acceptedPath: '/dep.mjs',
            timestamp: Date.now()
          }
        ]
      });

      const eventTypes = events.map((event) => event.type);
      expect(eventTypes).toContain('fetch:start');
      expect(eventTypes).toContain('fetch:end');
      expect(eventTypes).toContain('eval:start');
      expect(eventTypes).toContain('eval:end');
      expect(eventTypes).toContain('hmr:update');
    } finally {
      off();
      await runner.close();
    }
  });

  test('emits external policy diagnostics events for builtin stub mode', async () => {
    const diagnostics = new ModuleRunnerDiagnosticsBus();
    const events: ModuleRunnerDiagnosticsEvent[] = [];
    const off = diagnostics.subscribe((event) => events.push(event));

    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: [
              "const fs = await __farm_ssr_import__('node:fs');",
              "__farm_ssr_export_name__('ok', () => typeof fs === 'object');"
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
      diagnostics,
      externalPolicy: {
        builtin: 'stub'
      }
    });

    try {
      const mod = await runner.import<{ ok: boolean }>('/entry.mjs');
      expect(mod.ok).toBe(true);

      const policyEvent = events.find(
        (event) => event.type === 'external:policy'
      ) as
        | Extract<ModuleRunnerDiagnosticsEvent, { type: 'external:policy' }>
        | undefined;
      expect(policyEvent).toBeDefined();
      expect(policyEvent?.policy).toBe('builtin');
      expect(policyEvent?.action).toBe('stub');
      expect(policyEvent?.requestId).toBe('node:fs');
    } finally {
      off();
      await runner.close();
    }
  });
});
