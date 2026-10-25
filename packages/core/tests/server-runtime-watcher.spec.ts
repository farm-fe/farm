import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

const compiler = vi.hoisted(() => ({
  createInlineCompiler: vi.fn(async () => ({
    resolvedModulePaths: () => [],
    resolvedWatchPaths: () => []
  }))
}));
vi.mock('../src/compiler/index.js', () => compiler);

import Watcher from '../src/watcher/index.js';

describe('watcher using an externally managed runtime compiler', () => {
  for (const key of ['configFilePath', 'configFileDependencies', 'envFiles']) {
    it(`recognizes normalized ${process.platform} events for ${key}`, () => {
      const root = process.platform === 'win32' ? 'C:/site' : '/site';
      const config = {
        root,
        compilation: { output: {}, persistentCache: false },
        [key]:
          key === 'configFilePath'
            ? path.join(root, 'farm.config.mjs')
            : [path.join(root, 'settings.mjs')]
      };
      const watcher = new Watcher(config as any);
      expect(
        watcher.isConfigFilesChanged(
          key === 'configFilePath'
            ? `${root}/farm.config.mjs`
            : `${root}/settings.mjs`
        )
      ).toBe(true);
    });
  }
  it('does not create a discovery compiler when explicitly disabled', async () => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-runtime-watcher-')
    );
    const watcher = new Watcher({
      root,
      watch: {},
      compilation: { output: {}, persistentCache: false }
    } as any);
    try {
      await watcher.createWatcher(null);
      expect(compiler.createInlineCompiler).not.toHaveBeenCalled();
    } finally {
      await watcher.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });
});

describe('rapid edit and repair events', () => {
  for (const mode of ['ordinary', 'runtime'] as const) {
    it(`preserves a repair immediately after an observed edit (${mode})`, async () => {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'farm-watch-repair-')
      );
      const file = path.join(root, 'entry.ts');
      await fs.writeFile(file, 'original');
      const platform = Object.getOwnPropertyDescriptor(process, 'platform');
      if (!platform) throw new Error('Missing process platform descriptor');
      let watcher: Watcher;
      try {
        // Exercise Linux defaults on every CI host; file events remain real.
        Object.defineProperty(process, 'platform', { value: 'linux' });
        watcher = new Watcher({
          root,
          watch: {},
          compilation: { output: {}, persistentCache: false }
        } as any);
      } finally {
        Object.defineProperty(process, 'platform', platform);
      }
      let timer: ReturnType<typeof setTimeout> | undefined;
      const values: string[] = [];
      try {
        await watcher.createWatcher(mode === 'runtime' ? null : undefined);
        const internal = watcher.getInternalWatcher();
        if (!internal) throw new Error('Watcher was not created');
        await new Promise<void>((resolve) => internal.once('ready', resolve));
        const repaired = new Promise<void>((resolve, reject) => {
          timer = setTimeout(
            () =>
              reject(
                new Error(
                  `Repair event lost; observed ${JSON.stringify(values)}`
                )
              ),
            1500
          );
          watcher.on('change', async () => {
            try {
              const value = await fs.readFile(file, 'utf8');
              values.push(value);
              if (value === 'invalid') {
                await new Promise((resolve) => setTimeout(resolve, 15));
                await fs.writeFile(file, 'repaired');
              } else if (value === 'repaired') resolve();
            } catch (error) {
              reject(error);
            }
          });
        });
        await fs.writeFile(file, 'invalid');
        await repaired;
        expect(values).toEqual(['invalid', 'repaired']);
      } finally {
        clearTimeout(timer);
        await watcher.close();
        await fs.rm(root, { recursive: true, force: true });
      }
    });
  }
  it('preserves an explicitly disabled write-finish policy', async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-watch-policy-'));
    const watcher = new Watcher({
      root,
      watch: { awaitWriteFinish: false },
      compilation: { output: {}, persistentCache: false }
    } as any);
    try {
      await watcher.createWatcher();
      expect(watcher.resolvedWatchOptions.awaitWriteFinish).toBe(false);
    } finally {
      await watcher.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });
});
