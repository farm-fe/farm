import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';

vi.mock('@farmfe/core', () => ({
  logger: {},
  Resolver: class {
    resolve() {
      return false;
    }
  }
}));

import tailwindcss from './index.js';

const roots: string[] = [];
afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => fs.rm(root, { recursive: true, force: true }))
  );
});

it('restores candidate sources and generates utilities when sources hit the cache after restart', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-tailwind-cache-'));
  roots.push(root);
  const source = path.join(root, 'App.tsx');
  const stylesheet = path.join(root, 'index.css');
  await fs.writeFile(source, '<div className="p-4" />');
  const metadata = new Map<string, string[]>();
  const context = {
    readMetadata: <T>(name: string) => metadata.get(name) as T | undefined,
    writeMetadata: (name: string, candidates: string[]) =>
      metadata.set(name, candidates),
    readMetadataByScope: () => [...metadata.values()],
    addWatchFile: vi.fn()
  };
  const config = { root, compilation: { minify: false } };
  const [coldScan] = tailwindcss();
  await coldScan.configResolved?.(config as never);
  const scan = coldScan.transform as {
    executor: (...args: any[]) => Promise<any>;
  };
  await scan.executor(
    {
      resolvedPath: source,
      content: '<div className="p-4" />',
      moduleId: source
    },
    context
  );
  // The compiler persists source metadata and plugin data independently of cached module transforms.
  const cache = await coldScan.writePluginCache?.executor(
    undefined as never,
    context as never
  );
  const [warmScan, warmGenerate] = tailwindcss();
  await warmScan.configResolved?.(config as never);
  await warmScan.pluginCacheLoaded?.executor(cache as never, context as never);
  const generate = warmGenerate.freezeModule as {
    executor: (...args: any[]) => Promise<any>;
  };
  const result = await generate.executor(
    {
      moduleId: stylesheet,
      resolvedPath: stylesheet,
      content: '@theme { --spacing: 0.25rem; } @tailwind utilities;'
    },
    context
  );
  expect(result.content).toContain('.p-4');
  expect(context.addWatchFile).toHaveBeenCalledWith(stylesheet, source);
});

it.each(['', '?vue&type=template'])(
  'does not retain utilities or watch paths from a deleted cached source %s',
  async (query) => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-tailwind-removed-cache-')
    );
    roots.push(root);
    const source = path.join(root, 'Removed.js');
    const stylesheet = path.join(root, 'index.css');
    await fs.writeFile(source, '<div class="p-4" />');
    const metadata = new Map<string, string[]>();
    const context = {
      readMetadata: <T>(name: string) => metadata.get(name) as T | undefined,
      writeMetadata: (name: string, candidates: string[]) =>
        metadata.set(name, candidates),
      readMetadataByScope: () => [...metadata.values()],
      addWatchFile: vi.fn()
    };
    const config = { root, compilation: { minify: false } };
    const [coldScan] = tailwindcss();
    await coldScan.configResolved?.(config as never);
    const scan = coldScan.transform as {
      executor: (...args: any[]) => Promise<any>;
    };
    await scan.executor(
      {
        resolvedPath: source + query,
        content: '<div class="p-4" />',
        moduleId: source
      },
      context
    );
    const cache = await coldScan.writePluginCache?.executor(
      undefined as never,
      context as never
    );
    await fs.rm(source);
    const [warmScan, warmGenerate] = tailwindcss();
    await warmScan.configResolved?.(config as never);
    await warmScan.pluginCacheLoaded?.executor(
      cache as never,
      context as never
    );
    const generate = warmGenerate.freezeModule as {
      executor: (...args: any[]) => Promise<any>;
    };
    const result = await generate.executor(
      {
        moduleId: stylesheet,
        resolvedPath: stylesheet,
        content: '@theme { --spacing: 0.25rem; } @tailwind utilities;'
      },
      context
    );
    expect(result.content).not.toContain('.p-4');
    expect(context.addWatchFile).not.toHaveBeenCalledWith(stylesheet, source);
  }
);

it('keeps candidates from transformed virtual sources without a physical file', async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), 'farm-tailwind-virtual-cache-')
  );
  roots.push(root);
  const source = path.join(root, 'virtual-generated.js');
  const stylesheet = path.join(root, 'index.css');
  const metadata = new Map<string, string[]>();
  const context = {
    readMetadata: <T>(name: string) => metadata.get(name) as T | undefined,
    writeMetadata: (name: string, candidates: string[]) =>
      metadata.set(name, candidates),
    readMetadataByScope: () => [...metadata.values()],
    addWatchFile: vi.fn()
  };
  const [scanPlugin, generatePlugin] = tailwindcss();
  await scanPlugin.configResolved?.({
    root,
    compilation: { minify: false }
  } as never);
  const scan = scanPlugin.transform as {
    executor: (...args: any[]) => Promise<any>;
  };
  await scan.executor(
    { resolvedPath: source, content: '<div class="p-8" />', moduleId: source },
    context
  );
  const generate = generatePlugin.freezeModule as {
    executor: (...args: any[]) => Promise<any>;
  };
  const result = await generate.executor(
    {
      moduleId: stylesheet,
      content: '@theme { --spacing: 0.25rem; } @tailwind utilities;'
    },
    context
  );
  expect(result.content).toContain('.p-8');
});
