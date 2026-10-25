import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { afterEach, expect, test, vi } from 'vitest';
import { Compiler, resolveConfig } from '../src/index.js';

const roots: string[] = [];
afterEach(async () => {
  vi.unstubAllEnvs();
  await Promise.all(
    roots.splice(0).map((root) => fs.rm(root, { recursive: true, force: true }))
  );
});

test('production env and Vite HTML pre/post hooks reach native build output', async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), 'farm-native-html-env-')
  );
  roots.push(root);
  vi.stubEnv('FARM_NATIVE_ENV_REGRESSION', '');
  delete process.env.FARM_NATIVE_ENV_REGRESSION;
  await Promise.all([
    fs.writeFile(
      path.join(root, 'index.html'),
      '<html><head>{title}</head><body><script type="module" src="./index.js"></script></body></html>'
    ),
    fs.writeFile(
      path.join(root, 'index.js'),
      'globalThis.envRegression = import.meta.env.FARM_NATIVE_ENV_REGRESSION;'
    ),
    fs.writeFile(
      path.join(root, '.env.production'),
      'FARM_NATIVE_ENV_REGRESSION=production-marker'
    ),
    fs.writeFile(
      path.join(root, '.env.development'),
      'FARM_NATIVE_ENV_REGRESSION=development-marker'
    )
  ]);
  const config = await resolveConfig(
    {
      root,
      configFile: false,
      vitePlugins: [
        {
          name: 'native-pre-html',
          transformIndexHtml: {
            order: 'pre',
            handler: (html: string) =>
              html.replace('{title}', '<title>native-pre-marker</title>')
          }
        },
        {
          name: 'native-post-html',
          transformIndexHtml: {
            order: 'post',
            handler: (html: string) => {
              expect(html).toContain('native-pre-marker');
              return [
                {
                  tag: 'meta',
                  attrs: { name: 'native-post-marker' },
                  injectTo: 'head'
                }
              ];
            }
          }
        }
      ],
      compilation: {
        input: { index: './index.html' },
        output: { path: 'dist', targetEnv: 'browser', showFileSize: false },
        progress: false,
        persistentCache: false
      }
    },
    'build',
    'production'
  );
  const compiler = new Compiler(config);
  await compiler.compile();
  const resources = compiler.resources();
  const html = resources['index.html'].toString();
  expect(html).toContain('native-pre-marker');
  expect(html).toContain('native-post-marker');
  expect(html).not.toContain('{title}');
  const output = Object.values(resources)
    .map((resource) => resource.toString())
    .join('\n');
  expect(output).toContain('production-marker');
  expect(output).not.toContain('development-marker');
}, 30_000);
