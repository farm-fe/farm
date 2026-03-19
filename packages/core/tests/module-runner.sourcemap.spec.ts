import { describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type { ModuleRunnerTransport } from '../src/module-runner/types.js';

describe('farm module runner source map hooks', () => {
  test('retrieveSourceMap hook remaps stack without inlined map payload', async () => {
    const generatedId = '/src/generated-hook.ts';
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("hook-remap")',
          file: generatedId,
          id: generatedId,
          url: generatedId,
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      sourceMapInterceptor: {
        native: false,
        hooks: {
          retrieveSourceMap(source) {
            if (source !== generatedId) {
              return undefined;
            }

            return JSON.stringify({
              version: 3,
              file: generatedId,
              sources: ['/src/original-hook.ts'],
              sourcesContent: ['throw new Error("hook-remap")'],
              names: [],
              mappings: 'AAAA'
            });
          }
        }
      }
    });

    try {
      const error = await runner
        .import(generatedId)
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(String(error?.stack ?? '')).toContain('/src/original-hook.ts');
    } finally {
      await runner.close();
    }
  });

  test('retrieveFile hook redirects source lookup before retrieveSourceMap', async () => {
    const generatedId = 'virtual:generated-file-hook.ts';
    const redirectedId = '/src/redirected-file-hook.ts';
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("hook-file-remap")',
          file: generatedId,
          id: generatedId,
          url: generatedId,
          invalidate: false,
          map: null
        };
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      sourceMapInterceptor: {
        native: false,
        hooks: {
          retrieveFile(source) {
            if (source === generatedId) {
              return redirectedId;
            }
            return source;
          },
          retrieveSourceMap(source) {
            if (source !== redirectedId) {
              return undefined;
            }

            return JSON.stringify({
              version: 3,
              file: redirectedId,
              sources: ['/src/original-file-hook.ts'],
              sourcesContent: ['throw new Error("hook-file-remap")'],
              names: [],
              mappings: 'AAAA'
            });
          }
        }
      }
    });

    try {
      const error = await runner
        .import(generatedId)
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(String(error?.stack ?? '')).toContain(
        '/src/original-file-hook.ts'
      );
    } finally {
      await runner.close();
    }
  });

  test('formatStack hook can override remapped stack output', async () => {
    const generatedId = '/src/generated-format-hook.ts';
    const transport: ModuleRunnerTransport = {
      async invoke(name) {
        expect(name).toBe('fetchModule');
        return {
          code: 'throw new Error("hook-format")',
          file: generatedId,
          id: generatedId,
          url: generatedId,
          invalidate: false,
          map: JSON.stringify({
            version: 3,
            file: generatedId,
            sources: ['/src/original-format-hook.ts'],
            sourcesContent: ['throw new Error("hook-format")'],
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
        native: false,
        hooks: {
          formatStack({ remappedStack }) {
            return `HOOK_FORMAT\n${remappedStack}`;
          }
        }
      }
    });

    try {
      const error = await runner
        .import(generatedId)
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(String(error?.stack ?? '')).toContain('HOOK_FORMAT');
      expect(String(error?.stack ?? '')).toContain(
        '/src/original-format-hook.ts'
      );
    } finally {
      await runner.close();
    }
  });
});
