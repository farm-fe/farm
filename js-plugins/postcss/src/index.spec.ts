import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import postcss from 'postcss';
import { afterEach, describe, expect, it, vi } from 'vitest';

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
afterEach(async () => {
  vi.unstubAllEnvs();
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((dir) => fs.rm(dir, { recursive: true, force: true }))
  );
});

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

it('UnoCSS rescans a changed Vue class in development and after a plugin restart', async () => {
  vi.stubEnv('NODE_ENV', 'development');
  const requireExample = createRequire(
    new URL('../../../examples/vite-adapter-vue/package.json', import.meta.url)
  );
  const unoPostcss = requireExample('@unocss/postcss');
  const { presetUno } = await import(
    pathToFileURL(requireExample.resolve('unocss')).href
  );
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
    return plugin.transform as { executor: (...args: any[]) => Promise<any> };
  };
  const addWatchFile = vi.fn();
  const transform = async (plugin: Awaited<ReturnType<typeof createPlugin>>) =>
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
  await fs.writeFile(vueFile, '<template><div class="w-100%" /></template>');
  const future = new Date(Date.now() + 2000);
  await fs.utimes(vueFile, future, future);
  expect((await transform(plugin)).content).toMatch(/width:\s*100%/);
  expect((await transform(await createPlugin())).content).toMatch(
    /width:\s*100%/
  );
});
