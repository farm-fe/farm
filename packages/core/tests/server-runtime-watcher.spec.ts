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
