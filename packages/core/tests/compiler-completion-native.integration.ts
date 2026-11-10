import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, test } from 'vitest';
import { Compiler, resolveConfig } from '../src/index.js';

// Explicit integration runner only; the plugin and NAPI binding must come from
// the same verified native cohort. Never silently fall back to installed bytes.
test('real async auto-import discovery failure settles and retries with retained DTS', async () => {
  const plugin = process.env.FARM_COMPLETION_AUTO_IMPORT;
  if (!plugin)
    throw new Error('Explicit same-cohort auto-import plugin is required');
  expect(process.env.NAPI_RS_NATIVE_LIBRARY_PATH).toBeTruthy();
  const root = await fs.realpath(
    await fs.mkdtemp(path.join(os.tmpdir(), 'farm-completion-native-'))
  );
  const entry = path.join(root, 'index.ts');
  const invalid = path.join(root, 'discovery', 'later.ts');
  let compiler: Compiler | undefined;
  try {
    await fs.mkdir(path.join(root, 'discovery'));
    await fs.writeFile(
      path.join(root, 'discovery', 'valid.ts'),
      'export const retained = 1;'
    );
    await fs.writeFile(entry, 'globalThis.marker = 1;');
    const config = await resolveConfig(
      {
        root,
        configFile: false,
        compilation: {
          input: { index: './index.ts' },
          output: { path: 'dist', targetEnv: 'browser', showFileSize: false },
          progress: false,
          persistentCache: false,
          lazyCompilation: false
        }
      },
      'start',
      'development'
    );
    config.rustPlugins = [
      ...(config.rustPlugins ?? []),
      [
        plugin,
        JSON.stringify({
          dirs: ['discovery/.*\\.ts$'],
          dts: 'auto-imports.d.ts'
        })
      ]
    ];
    compiler = new Compiler(config);
    await compiler.compile();
    console.info('native-completion: initial compile');
    const declarations = await fs.readFile(
      path.join(root, 'auto-imports.d.ts'),
      'utf8'
    );
    expect(declarations).toContain('retained');
    expect(Object.keys(compiler.resources()).length).toBeGreaterThan(0);
    expect(compiler.hasModule(entry)).toBe(true);
    expect(compiler.hasModule(invalid)).toBe(false);
    // This invalid export is discovered only by update_finished, not graph build.
    await fs.writeFile(invalid, 'export const later = ;');
    await fs.writeFile(entry, 'globalThis.marker = 2;');
    const binding = (compiler as any)._bindingCompiler;
    const original = binding.update.bind(binding);
    const nativeResults: any[] = [];
    binding.update = (...args: any[]) =>
      original(...args).then((result: any) => {
        nativeResults.push(result);
        return result;
      });
    await expect(
      compiler.update([{ path: entry, type: 'updated' }], false)
    ).rejects.toThrow();
    await compiler.waitForCompileFinish();
    console.info('native-completion: readiness settled');
    expect(compiler.compiling).toBe(false);
    expect(nativeResults).toHaveLength(1);
    expect(nativeResults[0].dynamicResourcesMap == null).toBe(true);
    expect(nativeResults[0].added).toEqual([]);
    expect(nativeResults[0].removed).toEqual([]);
    expect(compiler.hasModule(entry)).toBe(true);
    expect(compiler.hasModule(invalid)).toBe(false);
    expect(
      await fs.readFile(path.join(root, 'auto-imports.d.ts'), 'utf8')
    ).toBe(declarations);
    console.info('native-completion: correcting source');
    await fs.writeFile(invalid, 'export const later = 3;');
    await fs.writeFile(entry, 'globalThis.marker = 3;');
    await compiler.update([{ path: entry, type: 'updated' }], false);
    await compiler.waitForCompileFinish();
    console.info('native-completion: readiness settled');
    expect(compiler.compiling).toBe(false);
    const next = await fs.readFile(
      path.join(root, 'auto-imports.d.ts'),
      'utf8'
    );
    expect(next).toContain('retained');
    expect(next).toContain('later');
    await fs.writeFile(entry, 'globalThis.marker = later;');
    await compiler.update([{ path: entry, type: 'updated' }], false);
    await compiler.waitForCompileFinish();
    console.info('native-completion: readiness settled');
    expect(compiler.hasModule(invalid)).toBe(true);
  } finally {
    console.info('native-completion: cleanup');
    if (compiler) await compiler.flushCache();
    await fs.rm(root, { recursive: true, force: true });
  }
}, 30_000);
