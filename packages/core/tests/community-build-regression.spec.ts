import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { afterEach, expect, test } from 'vitest';
import { getCompiler } from './common.js';

const fixtureRoots: string[] = [];
afterEach(() => {
  for (const root of fixtureRoots.splice(0))
    rmSync(root, { recursive: true, force: true });
});

test.each([
  [false, false],
  [false, true],
  [true, false],
  [true, true]
])(
  'community update/delete and wrapper bindings execute with minify=%s concatenate=%s',
  async (minify, concatenateModules) => {
    const root = mkdtempSync(path.join(tmpdir(), 'farm-community-build-'));
    fixtureRoots.push(root);
    writeFileSync(
      path.join(root, 'state.ts'),
      `let count = 0; for (let i = 0; i < 3; i++) count++; const object = { a: 1, b: 2 }; delete object.a; export { count, object };`
    );
    writeFileSync(
      path.join(root, 'bindings.ts'),
      `const module = 'm'; let exports = 1; var farmRequire = 2; function farmDynamicRequire() { return 3; } export const value = [module, exports, farmRequire, farmDynamicRequire()];`
    );
    writeFileSync(
      path.join(root, 'index.ts'),
      `import { count, object } from './state'; import { value } from './bindings'; console.log(JSON.stringify({ count, keys: Object.keys(object), value }));`
    );
    const compiler = await getCompiler(
      root,
      'regression',
      [],
      { index: './index.ts' },
      undefined,
      {
        minify,
        concatenateModules,
        runtime: {
          path: fileURLToPath(new URL('../../runtime', import.meta.url))
        }
      }
    );
    await compiler.compile();
    compiler.writeResourcesToDisk();
    const result = execFileSync(
      process.execPath,
      [path.join(root, 'dist/regression/index.mjs')],
      { encoding: 'utf8' }
    );
    expect(JSON.parse(result.trim())).toEqual({
      count: 3,
      keys: ['b'],
      value: ['m', 1, 2, 3]
    });
  },
  30000
);
