import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { afterEach, describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import { createServerModuleRunnerInvokeHandlers } from '../src/module-runner/serverInvoke.js';
import type { ModuleRunnerTransport } from '../src/module-runner/types.js';

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

describe('module runner non-js policy (main path)', () => {
  test('keeps vue style-query noop fallback behavior before B3', async () => {
    const root = await createTempDir('farm-runner-v2-vue-style-');
    const vueFile = path.join(root, 'src/App.vue');
    await fs.mkdir(path.dirname(vueFile), { recursive: true });
    await fs.writeFile(
      vueFile,
      '<template><div>app</div></template><style scoped>.x{color:red}</style>\n'
    );

    const handlers = createServerModuleRunnerInvokeHandlers({
      root,
      publicPath: '/',
      moduleRunnerStamp: 1,
      compiler: {
        compiling: false,
        waitForCompileFinish: async () => undefined,
        fetchModule: () => ({
          externalize: pathToFileURL(vueFile).toString(),
          type: 'module',
          bailoutReason: 'not-script'
        }),
        resource: () => null,
        resources: () => ({})
      }
    } as never);

    const id = '/src/App.vue?vue&type=style&index=0&lang.css';
    const result = await handlers.fetchModule(id);

    expect(result).toMatchObject({
      id,
      url: id,
      map: null,
      invalidate: false
    });
    expect('code' in result ? result.code : '').toContain(
      '__farm_ssr_style_noop__'
    );
  });

  test('stubs style and asset modules through nonJsPolicy defaults', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/src/entry.mjs') {
          return {
            code: [
              "const style = await __farm_ssr_import__('./style.css');",
              "const logo = await __farm_ssr_import__('./logo.png');",
              "__farm_ssr_export_name__('styleDefaultType', () => typeof style.default);",
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
            externalize: 'file:///tmp/logo.png?t=2',
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
      const mod = await runner.import<{
        styleDefaultType: string;
        logo: string;
      }>('/src/entry.mjs');

      expect(mod.styleDefaultType).toBe('object');
      expect(mod.logo).toContain('file:///tmp/logo.png');
    } finally {
      await runner.close();
    }
  });

  test('supports nonJsPolicy=error for style modules', async () => {
    const transport: ModuleRunnerTransport = {
      async invoke(name, data) {
        expect(name).toBe('fetchModule');
        const [id] = data;

        if (id === '/entry.mjs') {
          return {
            code: "const style = await __farm_ssr_import__('./entry.css'); __farm_ssr_export_name__('style', () => style);",
            file: '/entry.mjs',
            id: '/entry.mjs',
            url: '/entry.mjs',
            invalidate: false,
            map: null
          };
        }

        if (id === '/entry.css') {
          return {
            externalize: 'file:///tmp/entry.css?t=3',
            type: 'module',
            bailoutReason: 'not-script'
          };
        }

        throw new Error(`unexpected id: ${String(id)}`);
      }
    };

    const runner = new FarmModuleRunner({
      transport,
      hmr: false,
      nonJsPolicy: {
        style: 'error'
      }
    });

    try {
      const error = await runner
        .import('/entry.mjs')
        .then(() => null)
        .catch((e) => e as Error);

      expect(error).toBeInstanceOf(Error);
      expect(error?.message).toContain('nonJsPolicy=error');
      expect(error?.message).toContain('/entry.css');
    } finally {
      await runner.close();
    }
  });
});
