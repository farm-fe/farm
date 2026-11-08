import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { expect, test } from 'vitest';
import { getCompiler, getOutputFilePath, getOutputResult } from './common.js';

test.each([false, true])(
  'default runtime retains live reexports with minify=%s',
  async (minify) => {
    const root = mkdtempSync(path.join(os.tmpdir(), 'farm-runtime-bindings-'));
    const namespace = `runtime_bindings_${minify}`;
    try {
      writeFileSync(
        path.join(root, 'source.js'),
        `export let count = 1;
         let empty = 2;
         export { count as "s-k", empty as "" };
         export const collision = 'source';
         export default 'excluded';
         export function update() { count = 3; empty = 4; }`
      );
      writeFileSync(
        path.join(root, 'named.js'),
        `export { "s-k" as "public-name", "" as alias, "s-k" as "" } from './source.js';`
      );
      writeFileSync(
        path.join(root, 'star.js'),
        `export * from './source.js';
         export const collision = 'explicit';`
      );
      writeFileSync(
        path.join(root, 'entry.js'),
        `import * as named from './named.js';
         import * as star from './star.js';
         export { update } from './source.js';
         export function read() {
           return [named['public-name'], named.alias, named[''],
             star.count, star['s-k'], star[''], star.collision,
             Object.prototype.hasOwnProperty.call(star, 'default')];
         }`
      );
      const compiler = await getCompiler(
        root,
        'live',
        [],
        { index: './entry.js' },
        undefined,
        {
          treeShaking: false,
          concatenateModules: false,
          minify,
          // Deliberately exercise default package resolution, not runtime.path.
          runtime: { namespace }
        }
      );
      await compiler.compile();
      await compiler.writeResourcesToDisk();
      const output = await getOutputResult(getOutputFilePath(root, 'live'));
      expect(output.read()).toEqual([1, 2, 1, 1, 1, 2, 'explicit', false]);
      output.update();
      expect(output.read()).toEqual([3, 4, 3, 3, 3, 4, 'explicit', false]);
    } finally {
      Reflect.deleteProperty(globalThis, namespace);
      rmSync(root, { recursive: true, force: true });
    }
  }
);
