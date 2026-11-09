import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import postcss from 'postcss';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

vi.mock('@farmfe/core', () => ({ checkPublicFile: vi.fn() }));
vi.mock('postcss-load-config', () => ({ default: vi.fn() }));
vi.mock('./utils.js', () => ({
  pluginName: '@farmfe/js-plugin-postcss',
  getPostcssImplementation: () => postcss,
  tryRead: (file: string) => fs.readFile(file, 'utf8')
}));

import loadConfig from 'postcss-load-config';
import farmPostcssPlugin from './index.js';

const temporaryDirectories: string[] = [];
const pendingFixtures: Promise<unknown>[] = [];
function runFixture<T>(run: () => Promise<T>): Promise<T> {
  const pending = run();
  pendingFixtures.push(pending);
  // A runner timeout does not cancel the asynchronous test body. Observe a
  // late rejection, while returning the original promise to the test runner.
  void pending.catch(() => {});
  return pending;
}
async function cleanupFixtures() {
  // Finish all fixture writers before removing their directories or restoring
  // environment state, including when Vitest has already timed out the test.
  await Promise.allSettled(pendingFixtures.splice(0));
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((dir) => fs.rm(dir, { recursive: true, force: true }))
  );
}
afterEach(async () => {
  await cleanupFixtures();
  vi.unstubAllEnvs();
});

it.each([false, true])(
  'waits for a late fixture writer before cleanup (failure: %s)',
  async (fail) => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-postcss-cleanup-')
    );
    temporaryDirectories.push(root);
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    const operation = runFixture(async () => {
      await gate;
      await fs.writeFile(path.join(root, 'late.css'), '.late {}');
      if (fail) throw new Error('fixture failure');
    });
    const outcome = operation.then(
      () => 'success',
      (error: Error) => error.message
    );
    const remove = vi.spyOn(fs, 'rm');
    let cleaned = false;
    const cleanup = cleanupFixtures().then(() => {
      cleaned = true;
    });
    try {
      // Model a timed-out runner entering teardown while its body is paused.
      await new Promise<void>((resolve) => setImmediate(resolve));
      expect(remove).not.toHaveBeenCalled();
      expect(cleaned).toBe(false);
      expect((await fs.stat(root)).isDirectory()).toBe(true);
    } finally {
      release();
      await cleanup;
      remove.mockRestore();
    }
    expect(await outcome).toBe(fail ? 'fixture failure' : 'success');
    await expect(fs.stat(root)).rejects.toMatchObject({ code: 'ENOENT' });
  }
);

describe('PostCSS cache dependencies', () => {
  it.each(['development', 'production'])(
    'tracks source dependencies in %s',
    async (mode) => {
      vi.stubEnv('NODE_ENV', mode);
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'farm-postcss-watch-')
      );
      temporaryDirectories.push(root);
      await fs.writeFile(
        path.join(root, 'App.vue'),
        '<template><div class="p-4" /></template>'
      );
      const dependency = path.join(root, 'uno.config.ts');
      await fs.writeFile(dependency, 'export default {}');
      const dependencyPlugin = {
        postcssPlugin: 'source-dependency-fixture',
        Once(_root: unknown, { result }: { result: postcss.Result }) {
          result.messages.push({
            type: 'dependency',
            plugin: 'fixture',
            file: dependency
          });
          result.messages.push({
            type: 'dir-dependency',
            plugin: 'fixture',
            dir: root,
            glob: '*.vue'
          });
        }
      };
      vi.mocked(loadConfig).mockResolvedValue({
        plugins: [dependencyPlugin],
        options: {},
        file: path.join(root, 'postcss.config.js')
      });
      const plugin = farmPostcssPlugin();
      await plugin.configResolved?.({ root } as never);
      const addWatchFile = vi.fn();
      const transform = plugin.transform as {
        executor: (...args: any[]) => Promise<any>;
      };
      const result = await transform.executor(
        {
          moduleId: 'index.css',
          resolvedPath: path.join(root, 'index.css'),
          content: '.app { color: red; }',
          moduleType: 'css'
        },
        {
          sourceMapEnabled: () => false,
          addWatchFile,
          error: (message: string) => {
            throw new Error(message);
          }
        }
      );
      expect(result.content).toContain('color: red');
      expect(addWatchFile).toHaveBeenCalledWith('index.css', dependency);
      expect(addWatchFile).toHaveBeenCalledWith(
        'index.css',
        path.join(root, 'App.vue')
      );
    }
  );
});

describe('UnoCSS development lifecycle', () => {
  let unoPostcss: any;
  let presetUno: any;
  beforeAll(async () => {
    // Load only the preset exported by this pinned UnoCSS installation. Module
    // initialization belongs to setup, not the scanning/restart assertion budget.
    const requireExample = createRequire(
      new URL(
        '../../../examples/vite-adapter-vue/package.json',
        import.meta.url
      )
    );
    unoPostcss = requireExample('@unocss/postcss');
    await import(
      pathToFileURL(requireExample.resolve('@unocss/postcss/esm')).href
    );
    const requireUno = createRequire(requireExample.resolve('unocss'));
    ({ default: presetUno } = await import(
      pathToFileURL(requireUno.resolve('@unocss/preset-uno')).href
    ));
  });
  it('UnoCSS rescans a changed Vue class in development and after a plugin restart', () =>
    runFixture(async () => {
      vi.stubEnv('NODE_ENV', 'development');
      const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-unocss-vue-'));
      temporaryDirectories.push(root);
      const vueFile = path.join(root, 'App.vue');
      const cssFile = path.join(root, 'index.css');
      await fs.writeFile(vueFile, '<template><div class="w-50%" /></template>');
      const createPlugin = async () => {
        vi.mocked(loadConfig).mockResolvedValue({
          plugins: [
            unoPostcss({
              cwd: root,
              configOrPath: {
                content: { filesystem: ['*.vue'] },
                presets: [presetUno()]
              }
            })
          ],
          options: {},
          file: path.join(root, 'postcss.config.js')
        });
        const plugin = farmPostcssPlugin();
        await plugin.configResolved?.({ root } as never);
        return plugin.transform as {
          executor: (...args: any[]) => Promise<any>;
        };
      };
      const addWatchFile = vi.fn();
      const transform = async (
        plugin: Awaited<ReturnType<typeof createPlugin>>
      ) =>
        plugin.executor(
          {
            moduleId: 'index.css',
            resolvedPath: cssFile,
            content: '@unocss;',
            moduleType: 'css'
          },
          {
            sourceMapEnabled: () => false,
            addWatchFile,
            error: (message: string) => {
              throw new Error(message);
            }
          }
        );
      const plugin = await createPlugin();
      expect((await transform(plugin)).content).toMatch(/width:\s*50%/);
      expect(addWatchFile).toHaveBeenCalledWith('index.css', vueFile);
      await fs.writeFile(
        vueFile,
        '<template><div class="w-100%" /></template>'
      );
      const future = new Date(Date.now() + 2000);
      await fs.utimes(vueFile, future, future);
      expect((await transform(plugin)).content).toMatch(/width:\s*100%/);
      expect((await transform(await createPlugin())).content).toMatch(
        /width:\s*100%/
      );
    }));
});
