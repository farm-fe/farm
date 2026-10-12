import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';
import { afterEach, expect, test, vi } from 'vitest';

// These tests exercise the JS adapter and config layers; no native compiler is constructed.
vi.mock('../binding/index.js', () => ({
  Compiler: class {},
  Resolver: class {},
  JsFileWatcher: class {},
  bindingPath: ''
}));
vi.mock('../src/index.js', async () => ({
  ...(await import('../src/utils/path.js')),
  ...(await import('../src/utils/share.js')),
  Logger: class {
    warn() {}
    info() {}
    debug() {}
    error() {}
  },
  logger: { warn() {}, info() {}, debug() {}, error() {} },
  Server: class {}
}));

import { resolveUserConfig } from '../src/config/resolve-config.js';
import { VitePluginAdapter } from '../src/plugin/js/vite-plugin-adapter.js';
import { createViteDevServerAdapter } from '../src/plugin/js/vite-server-adapter.js';
import { Logger } from '../src/utils/logger.js';
import { getAliasEntries } from '../src/utils/plugin-utils.js';

const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0))
    fs.rmSync(root, { recursive: true, force: true });
  vi.unstubAllEnvs();
});

test('production mode selects production env files (#2183)', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-env-issues-'));
  roots.push(root);
  vi.stubEnv('FARM_ISSUE_2183', '');
  delete process.env.FARM_ISSUE_2183;
  fs.writeFileSync(
    path.join(root, '.env.development'),
    'FARM_ISSUE_2183=development'
  );
  fs.writeFileSync(
    path.join(root, '.env.production'),
    'FARM_ISSUE_2183=production'
  );
  const config = await resolveUserConfig({
    root,
    mode: 'production',
    compilation: { mode: 'production' }
  });
  expect(config.env?.FARM_ISSUE_2183).toBe('production');
  expect(config.env?.PROD).toBe(true);
  expect(config.envFiles).toContain(
    path.join(root, '.env.production').replaceAll('\\', '/')
  );
  expect(config.envFiles).not.toContain(
    path.join(root, '.env.development').replaceAll('\\', '/')
  );
});

test('Vite HTML pre output reaches the next HTML hook (#1816)', async () => {
  const config = { compilation: { mode: 'production' as const } };
  const pre = new VitePluginAdapter(
    {
      name: 'pre-html',
      transformIndexHtml: {
        order: 'pre',
        handler: (html) => html.replace('{title}', '<title>Farm</title>')
      }
    },
    config,
    [],
    'production'
  );
  const post = new VitePluginAdapter(
    {
      name: 'post-html',
      transformIndexHtml: {
        order: 'post',
        handler: (html) => {
          expect(html).toContain('<title>Farm</title>');
          return html.replace('</head>', '<meta name="after-pre"></head>');
        }
      }
    },
    config,
    [],
    'production'
  );
  let htmlResource: any = {
    name: 'index.html',
    bytes: [...Buffer.from('<html><head>{title}</head><body></body></html>')],
    resourceType: 'html',
    origin: { type: 'Module', value: 'index.html' }
  };
  expect(pre.transformHtml?.order).toBe(0);
  expect(post.transformHtml?.order).toBe(2);
  htmlResource = await pre.transformHtml?.executor({ htmlResource }, {} as any);
  htmlResource = await post.transformHtml?.executor(
    { htmlResource },
    {} as any
  );
  expect(Buffer.from(htmlResource.bytes).toString()).toContain(
    '<meta name="after-pre">'
  );
});

test('cached-module hook does not prevent Vite Vue 6 adapters (#2252)', async () => {
  const transform = vi.fn((code: string) => ({
    code: `${code}\nexport const compiled = true;`
  }));
  const adapter = new VitePluginAdapter(
    { name: 'vite:vue', shouldTransformCachedModule: () => true, transform },
    { compilation: { mode: 'development' } },
    [],
    'development'
  );
  const result = await adapter.transform?.executor(
    {
      moduleId: '/App.vue',
      resolvedPath: '/App.vue',
      query: [],
      content: 'export default {};',
      moduleType: 'js'
    } as any,
    {} as any
  );
  expect(transform).toHaveBeenCalledOnce();
  expect(result?.content).toContain('export const compiled = true;');
});

test('inspector can call printUrls with the original server receiver (#2157/#1918)', () => {
  const server = {
    marker: 'farm-server',
    printUrls() {
      return this.marker;
    }
  };
  const adapter = createViteDevServerAdapter(
    'vite-plugin-vue-inspector',
    {},
    server,
    new Logger()
  );
  expect(adapter.printUrls()).toBe('farm-server');
});

test('object and array aliases are normalized without alias.reduce errors (#2124)', () => {
  expect(getAliasEntries({ '@': '/src' })).toEqual([
    { find: '@', replacement: '/src' }
  ]);
  expect(getAliasEntries([{ find: '@', replacement: '/src' }])).toEqual([
    { find: '@', replacement: '/src' }
  ]);
});

test('standalone consumer receives glob types without Vite types (#1996)', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-client-types-'));
  roots.push(root);
  const coreDir = fileURLToPath(new URL('..', import.meta.url));
  const packageDir = path.join(root, 'node_modules', '@farmfe');
  fs.mkdirSync(packageDir, { recursive: true });
  fs.symlinkSync(coreDir, path.join(packageDir, 'core'), 'junction');
  const source = path.join(root, 'consumer.mts');
  fs.writeFileSync(
    source,
    `import '@farmfe/core/client';
const eager: Record<string, { default: string }> = import.meta.glob<{ default: string }>('./*.ts', { eager: true });
const lazy: Record<string, () => Promise<{ default: string }>> = import.meta.glob<{ default: string }>('./*.ts');
// @ts-expect-error glob patterns must be strings
import.meta.glob(42);
void eager; void lazy;
`
  );
  const program = ts.createProgram([source], {
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    types: [],
    noEmit: true,
    strict: true
  });
  expect(
    ts
      .getPreEmitDiagnostics(program)
      .map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n'))
  ).toEqual([]);
});
