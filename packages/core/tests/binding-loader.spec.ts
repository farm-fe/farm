import { readFileSync } from 'node:fs';
import { createServer as createHttpServer } from 'node:http';
import { runInNewContext } from 'node:vm';
import { createServer } from 'vite';
import { expect, test } from 'vitest';

const file = new URL('../binding/resolve-binding.cjs', import.meta.url);
const source = readFileSync(file, 'utf8');

test('native path resolver can be parsed by the Vite SSR test loader', async () => {
  const hmrServer = createHttpServer();
  const server = await createServer({
    configFile: false,
    server: { middlewareMode: true, hmr: { server: hmrServer } },
    optimizeDeps: { noDiscovery: true },
    appType: 'custom'
  });
  try {
    await expect(
      server.ssrTransform(source, null, file.pathname)
    ).resolves.toBeTruthy();
  } finally {
    await server.close();
  }
});

function resolve(platform: string, override?: string) {
  const module = { exports: undefined };
  const requests: string[] = [];
  const require = Object.assign(
    (id: string) => {
      if (id === 'fs') {
        return {
          existsSync: () => {
            if (override)
              throw new Error('override must bypass platform probing');
            return false;
          }
        };
      }
      if (id === 'path')
        return { join: (...parts: string[]) => parts.join('/') };
      throw new Error(`Unexpected require: ${id}`);
    },
    {
      resolve: (id: string) => {
        requests.push(id);
        return `resolved:${id}`;
      }
    }
  );
  runInNewContext(source, {
    module,
    require,
    __dirname: '/candidate/binding',
    process: {
      platform,
      arch: 'x64',
      env: override ? { NAPI_RS_NATIVE_LIBRARY_PATH: override } : {},
      report: { getReport: () => ({ header: { glibcVersionRuntime: '2.39' } }) }
    }
  });
  return { value: module.exports, requests };
}

test('explicit native path bypasses platform probing and exports its resolved identity', () => {
  expect(resolve('unsupported-platform', '/isolated/native.node')).toEqual({
    value: 'resolved:/isolated/native.node',
    requests: ['/isolated/native.node']
  });
});

test.each([
  ['win32', '@farmfe/core-win32-x64-msvc'],
  ['darwin', '@farmfe/core-darwin-x64'],
  ['linux', '@farmfe/core-linux-x64-gnu']
])('ordinary %s native resolution is unchanged', (platform, packageName) => {
  expect(resolve(platform)).toEqual({
    value: `resolved:${packageName}`,
    requests: [packageName]
  });
});
