import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import postcss from 'postcss';
import { afterEach, expect, it, vi } from 'vitest';
import {
  Compiler,
  resolveConfig,
  Server
} from '../../../packages/core/src/index.js';
import tailwindcss from '../../tailwindcss/src/index.js';

// Source execution does not use the bundled utils module's package-relative path.
vi.mock('../src/utils.js', () => ({
  pluginName: '@farmfe/js-plugin-postcss',
  getPostcssImplementation: () => postcss,
  tryRead: (file: string) => fs.readFile(file, 'utf8')
}));

import farmPostcssPlugin from '../src/index.js';

const requireExample = createRequire(
  new URL('../../../examples/vite-adapter-vue/package.json', import.meta.url)
);
const temporaryRoots: string[] = [];
const originalWorkingDirectory = process.cwd();
const servers: Server[] = [];
const manifestTimestamps = new Map<string, number>();
afterEach(async () => {
  vi.unstubAllEnvs();
  process.chdir(originalWorkingDirectory);
  await Promise.all(servers.splice(0).map((server) => server.close()));
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => fs.rm(root, { recursive: true, force: true }))
  );
});

async function findManifests(directory: string): Promise<string[]> {
  let entries;
  try {
    entries = await fs.readdir(directory, { withFileTypes: true });
  } catch {
    return [];
  }
  return (
    await Promise.all(
      entries.map(async (entry) => {
        const file = path.join(directory, entry.name);
        if (entry.isDirectory()) return findManifests(file);
        return entry.name === 'farm-cache.json' ? [file] : [];
      })
    )
  ).flat();
}

async function awaitCacheWrite(root: string) {
  const previous = manifestTimestamps.get(root) ?? 0;
  await expect
    .poll(
      async () => {
        const manifests = await findManifests(path.join(root, '.farm'));
        const timestamp = Math.max(
          0,
          ...(await Promise.all(
            manifests.map(async (file) => {
              try {
                const manifest = JSON.parse(await fs.readFile(file, 'utf8'));
                if (
                  !Object.keys(manifest).some((key) =>
                    key.includes('mutable-module')
                  )
                )
                  return 0;
                return (await fs.stat(file)).mtimeMs;
              } catch {
                return 0;
              }
            })
          ))
        );
        if (timestamp > previous) manifestTimestamps.set(root, timestamp);
        return timestamp > previous;
      },
      { timeout: 20_000 }
    )
    .toBe(true);
}

async function createRoot(prefix: string) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  temporaryRoots.push(root);
  await fs.writeFile(path.join(root, 'package.json'), '{"type":"module"}');
  return root;
}

async function compile(
  root: string,
  plugins: Parameters<typeof resolveConfig>[0]['plugins'],
  mode: 'development' | 'production',
  vitePlugins?: Parameters<typeof resolveConfig>[0]['vitePlugins']
) {
  const userConfig: Parameters<typeof resolveConfig>[0] = {
    root,
    configFile: false,
    plugins,
    vitePlugins,
    server: { host: '127.0.0.1', port: 0, hmr: { port: 0 }, open: false },
    compilation: {
      input: { index: './index.js' },
      output: {
        path: mode === 'development' ? '.farm' : 'dist',
        targetEnv: 'browser',
        showFileSize: false
      },
      external: ['vue'],
      progress: false,
      lazyCompilation: false,
      minify: false,
      sourcemap: false,
      persistentCache: {
        cacheDir: path.join(root, '.farm', 'cache'),
        namespace: 'stylesheet-regression'
      }
    }
  };
  let compiler: Compiler;
  if (mode === 'development') {
    await Promise.all(servers.splice(0).map((server) => server.close()));
    const server = await Server.createServer(userConfig);
    servers.push(server);
    await server.listen();
    if (!server.compiler) throw new Error('server compiler missing');
    compiler = server.compiler;
  } else {
    const config = await resolveConfig(userConfig, 'build', mode);
    compiler = new Compiler(config);
    await compiler.compile();
  }
  await awaitCacheWrite(root);
  return compiler;
}

function generatedContent(compiler: Compiler) {
  return Object.values(compiler.resources())
    .map((resource) => resource.toString())
    .join('\n');
}

it('UnoCSS Vue class changes regenerate CSS during development and across warm starts', async () => {
  vi.stubEnv('NODE_ENV', 'development');
  const root = await createRoot('farm-uno-vue-cache-');
  // UnoCSS reloads its config relative to cwd, matching an app launched in its project.
  process.chdir(root);
  const vueFile = path.join(root, 'App.vue');
  const writeVue = (width: number) =>
    fs.writeFile(
      vueFile,
      `<template><div class="w-${width}%">Example</div></template>`
    );
  await writeVue(50);
  await fs.writeFile(
    path.join(root, 'index.js'),
    "import App from './App.vue'; import './index.css'; export default App;"
  );
  await fs.writeFile(path.join(root, 'index.css'), '@unocss;');
  const unoModule = pathToFileURL(requireExample.resolve('unocss')).href;
  await fs.writeFile(
    path.join(root, 'uno.config.mjs'),
    `import { presetUno } from ${JSON.stringify(unoModule)}; export default { presets: [presetUno()], content: { filesystem: ['*.vue'] } };`
  );
  const unoPostcssModule = requireExample.resolve('@unocss/postcss');
  await fs.writeFile(
    path.join(root, 'postcss.config.cjs'),
    `module.exports = { plugins: [require(${JSON.stringify(unoPostcssModule)})({ cwd: __dirname, configOrPath: require('node:path').join(__dirname, 'uno.config.mjs') })] };`
  );
  const vuePackagePath = process.env.FARM_VUE_PLUGIN_PACKAGE_PATH;
  const requireVue = vuePackagePath
    ? createRequire(path.join(vuePackagePath, 'package.json'))
    : requireExample;
  const vueVersion = requireVue('@vitejs/plugin-vue/package.json').version;
  expect(vueVersion).toMatch(/^6\./);
  const vue = (
    await import(pathToFileURL(requireVue.resolve('@vitejs/plugin-vue')).href)
  ).default;
  const build = () =>
    compile(
      root,
      [
        farmPostcssPlugin(),
        {
          name: 'assert-stylesheet-in-main-graph',
          updateModules: {
            executor(_param, context) {
              expect(
                context.viteGetModuleById(path.join(root, 'index.css'))
              ).toBeTruthy();
            }
          }
        }
      ],
      'development',
      [vue()]
    );
  expect(generatedContent(await build())).toMatch(/width:\s*50%/);
  const warmCompiler = await build();
  expect(generatedContent(warmCompiler)).toMatch(/width:\s*50%/);
  expect(warmCompiler.hasModule(path.join(root, 'index.css'))).toBe(true);
  const updateResults: string[] = [];
  servers.at(-1)?.hmrEngine?.onUpdateFinish((result) => {
    updateResults.push(result.mutableModules);
  });
  await writeVue(100);
  await expect
    .poll(() => updateResults.some((code) => /width:\s*100%/.test(code)), {
      timeout: 20_000
    })
    .toBe(true);
  // An independent compiler must consume updated on-disk caches without clearing .farm.
  await awaitCacheWrite(root);
  expect(generatedContent(await build())).toMatch(/width:\s*100%/);
}, 60_000);

it('Tailwind utilities survive a warm production startup and source edits after restart', async () => {
  vi.stubEnv('NODE_ENV', 'production');
  const root = await createRoot('farm-tailwind-warm-cache-');
  const source = path.join(root, 'App.js');
  await fs.writeFile(
    source,
    'export const template = \'<div class="p-4"></div>\';'
  );
  await fs.writeFile(
    path.join(root, 'index.js'),
    "export { template } from './App.js'; import './index.css';"
  );
  await fs.writeFile(
    path.join(root, 'index.css'),
    '@theme { --spacing: 0.25rem; } @tailwind utilities;'
  );
  const build = () => compile(root, tailwindcss(), 'production');
  expect(generatedContent(await build())).toContain('.p-4');
  expect(generatedContent(await build())).toContain('.p-4');
  // Rebuild just the CSS while the source module hits persistent cache, then
  // change the source on a further startup to verify restored watch edges.
  await fs.appendFile(
    path.join(root, 'index.css'),
    '\n/* changed stylesheet */'
  );
  expect(generatedContent(await build())).toContain('.p-4');
  await fs.writeFile(
    source,
    'export const template = \'<div class="p-8"></div>\';'
  );
  const changed = generatedContent(await build());
  expect(changed).toContain('.p-8');
  expect(changed).not.toContain('.p-4');
  await fs.writeFile(path.join(root, 'index.js'), "import './index.css';");
  await fs.rm(source);
  const removed = generatedContent(await build());
  expect(removed).not.toContain('.p-8');
  expect(removed).not.toContain('.p-4');
}, 60_000);

it('glob additions and deletions update importers during HMR and across warm starts', async () => {
  vi.stubEnv('NODE_ENV', 'production');
  const root = await createRoot('farm-glob-membership-cache-');
  const routes = path.join(root, 'routes');
  await fs.mkdir(routes);
  const firstRoute = path.join(routes, 'a.js');
  const secondRoute = path.join(routes, 'b.js');
  await fs.writeFile(firstRoute, "export default 'first';");
  await fs.writeFile(
    path.join(root, 'index.js'),
    "export const routes = import.meta.glob('./routes/*.js', { eager: true });"
  );
  const build = () => compile(root, [], 'production');
  expect(generatedContent(await build())).toContain('./routes/a.js');
  const warm = await build();
  expect(generatedContent(warm)).toContain('./routes/a.js');
  await fs.writeFile(secondRoute, "export default 'second';");
  const added = await warm.update([{ path: secondRoute, type: 'added' }]);
  expect(added.mutableModules).toContain('./routes/b.js');
  await fs.rm(firstRoute);
  const removed = await warm.update([{ path: firstRoute, type: 'removed' }]);
  expect(removed.mutableModules).not.toContain('./routes/a.js');
  expect(removed.mutableModules).toContain('./routes/b.js');
  await awaitCacheWrite(root);
  const restarted = generatedContent(await build());
  expect(restarted).toContain('./routes/b.js');
  expect(restarted).not.toContain('./routes/a.js');
}, 60_000);

function websocketModuleCode(message: string) {
  const match = message.match(/\bmutableModules:\s*("(?:\\.|[^"\\])*")/);
  return match ? (JSON.parse(match[1]) as string) : '';
}

it('the dev watcher delivers glob add/remove updates over a real WebSocket', async () => {
  vi.stubEnv('NODE_ENV', 'development');
  const root = await createRoot('farm-glob-websocket-');
  const routes = path.join(root, 'routes');
  await fs.mkdir(routes);
  const firstRoute = path.join(routes, 'a.js');
  const secondRoute = path.join(routes, 'b.js');
  await fs.writeFile(firstRoute, "export default 'a';");
  await fs.writeFile(
    path.join(root, 'index.js'),
    "export const routes = import.meta.glob('./routes/*.js', { eager: true });"
  );
  const server = await Server.createServer({
    root,
    configFile: false,
    server: {
      host: '127.0.0.1',
      port: 0,
      hmr: { port: 0 },
      open: false,
      allowedHosts: ['http://farm.test']
    },
    compilation: {
      input: { index: './index.js' },
      output: { path: '.farm', targetEnv: 'browser', showFileSize: false },
      progress: false,
      lazyCompilation: false,
      minify: false,
      sourcemap: false,
      persistentCache: {
        cacheDir: path.join(root, '.farm', 'cache'),
        namespace: 'websocket-regression'
      }
    }
  });
  const requireCore = createRequire(
    new URL('../../../packages/core/package.json', import.meta.url)
  );
  const WebSocket = requireCore('ws');
  let socket: any;
  try {
    await server.listen();
    expect(server.compiler).toBeDefined();
    const compiler = server.compiler;
    if (!compiler) throw new Error('server compiler missing');
    expect(generatedContent(compiler)).toContain('./routes/a.js');
    const address = server.httpServer?.address();
    if (!address || typeof address === 'string')
      throw new Error('dev server did not bind a TCP port');

    const hmrPath =
      (server.config.server?.hmr as { path?: string }).path ?? '/';
    socket = new WebSocket(
      `ws://127.0.0.1:${address.port}${hmrPath}`,
      'farm_hmr',
      {
        handshakeTimeout: 5000,
        headers: { Origin: 'http://farm.test' }
      }
    );
    const messages: string[] = [];
    socket.on('message', (message: Buffer) => {
      messages.push(message.toString());
    });
    await new Promise<void>((resolve, reject) => {
      socket.once('open', resolve);
      socket.once('error', reject);
    });
    await fs.writeFile(secondRoute, "export default 'b';");
    await expect
      .poll(
        () =>
          messages.some((message) =>
            websocketModuleCode(message).includes('./routes/b.js')
          ),
        { timeout: 20_000 }
      )
      .toBe(true);
    await expect
      .poll(() => compiler.compiling, { timeout: 20_000 })
      .toBe(false);
    const beforeRemoval = messages.length;
    await fs.rm(firstRoute);
    await expect
      .poll(
        () =>
          messages.slice(beforeRemoval).some((message) => {
            const code = websocketModuleCode(message);
            return (
              message.includes("type: 'farm-update'") &&
              code.includes('./routes/b.js') &&
              !code.includes('./routes/a.js')
            );
          }),
        { timeout: 20_000 }
      )
      .toBe(true);
    await expect
      .poll(() => compiler.compiling, { timeout: 20_000 })
      .toBe(false);
    const content = generatedContent(compiler);
    expect(content).toContain('./routes/b.js');
    expect(content).not.toContain('./routes/a.js');
    await awaitCacheWrite(root);
  } finally {
    socket?.terminate();
    await server.close();
  }
}, 60_000);
