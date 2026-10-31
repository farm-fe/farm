import { realpathSync } from 'node:fs';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it } from 'vitest';
import { Compiler } from '../src/compiler/index.js';
import { resolveConfig } from '../src/config/index.js';

it.each([true, false])(
  'resolves parent identities through a directory alias with symlinks=%s',
  async (symlinks) => {
    const owner = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-parent-alias-')
    );
    const physical = path.join(owner, 'physical');
    const alias = path.join(owner, 'alias');
    try {
      await fs.mkdir(physical);
      await fs.symlink(
        physical,
        alias,
        process.platform === 'win32' ? 'junction' : 'dir'
      );
      await fs.writeFile(
        path.join(physical, 'index.html'),
        '<html><body><script type="module" src="./entry.js"></script></body></html>'
      );
      await fs.writeFile(
        path.join(physical, 'entry.js'),
        "import {value} from './dep.js?variant'; console.log(value);"
      );
      await fs.writeFile(
        path.join(physical, 'dep.js'),
        "export const value='alias-before';"
      );
      const config = await resolveConfig(
        {
          root: alias,
          configFile: false,
          compilation: {
            lazyCompilation: false,
            persistentCache: false,
            resolve: { symlinks }
          }
        },
        'dev',
        'development'
      );
      const compiler = new Compiler(config);
      await compiler.compile();
      const identityRoot = symlinks ? await fs.realpath(physical) : alias;
      const expected = [path.join(identityRoot, 'entry.js')];
      const normalize = (files: string[]) =>
        files.map((file) =>
          path.normalize(symlinks ? realpathSync.native(file) : file)
        );
      expect(normalize(compiler.getParentFiles('dep.js?variant'))).toEqual(
        expected
      );
      expect(
        normalize(
          compiler.getParentFiles(path.join(alias, 'dep.js') + '?variant')
        )
      ).toEqual(expected);
      if (symlinks)
        expect(
          normalize(
            compiler.getParentFiles(path.join(physical, 'dep.js') + '?variant')
          )
        ).toEqual(expected);
      expect(() => compiler.getParentFiles('missing.js')).toThrow(
        /not present in the module graph/
      );
      // A rejected stale identity must not prevent subsequent valid updates.
      await fs.writeFile(
        path.join(physical, 'dep.js'),
        "export const value='alias-after';"
      );
      await compiler.update(
        [{ path: path.join(identityRoot, 'dep.js'), type: 'updated' }],
        true
      );
      expect(
        Object.values(compiler.resources()).some((bytes) =>
          bytes.toString().includes('alias-after')
        )
      ).toBe(true);
      expect(normalize(compiler.getParentFiles('dep.js?variant'))).toEqual(
        expected
      );
    } finally {
      await fs.rm(owner, { recursive: true, force: true });
    }
  }
);
