import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it } from 'vitest';
import { Compiler } from '../src/compiler/index.js';
import { resolveConfig } from '../src/config/index.js';

for (const scenario of [
  'html',
  'alias',
  'sequence',
  'new-html-dependency',
  'alias-html'
]) {
  it(`renders HTML update safely: ${scenario}`, async () => {
    const owner = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-html-direct-'));
    const root = await fs.realpath(owner);
    const entry = path.join(root, 'entry.js');
    const html = path.join(root, 'index.html');
    let compiler: Compiler | undefined;
    try {
      await fs.writeFile(entry, 'console.log("initial");');
      await fs.writeFile(
        html,
        '<html><body>Initial<script type="module" src="./entry.js"></script></body></html>'
      );
      compiler = new Compiler(
        await resolveConfig(
          {
            root,
            configFile: false,
            compilation: {
              input: { index: './index.html' },
              persistentCache: false,
              lazyCompilation: false
            }
          },
          'dev',
          'development'
        )
      );
      await compiler.compile();
      console.log(
        'DIRECT_FIXTURE',
        JSON.stringify({ scenario, owner, root, real: await fs.realpath(root) })
      );
      const update = async (
        file: string,
        type: 'updated' | 'added' = 'updated'
      ) => {
        if (!compiler) throw new Error('Compiler was not initialized');
        console.log('DIRECT_UPDATE', JSON.stringify({ file, type }));
        const result = await compiler.update([{ path: file, type }], true);
        await compiler.waitForCompileFinish();
        console.log('DIRECT_RESULT', JSON.stringify(result));
        return result;
      };
      if (scenario === 'sequence') {
        await update(root, 'added');
        await update(entry);
        await update(entry.replace(/^\/private\/var\//, '/var/'));
      } else if (scenario === 'alias') {
        await update(entry.replace(/^\/private\/var\//, '/var/'));
      } else if (scenario === 'new-html-dependency') {
        await fs.writeFile(
          path.join(root, 'other.html'),
          '<html><body>Other</body></html>'
        );
        await fs.writeFile(
          entry,
          'import "./other.html"; console.log("changed");'
        );
        // Multiple HTML documents in one pot are unsupported. Report that
        // diagnostic without aborting the native process, then recover.
        await expect(update(entry)).rejects.toThrow(
          'A html resource pot should only contains one html module'
        );
        await compiler.waitForCompileFinish();
        expect(compiler.compiling).toBe(false);
        await fs.writeFile(entry, 'console.log("recovered");');
        const result = await update(entry);
        expect(result.mutableModules).toContain('recovered');
        return;
      }
      await fs.writeFile(
        html,
        '<html><body>Changed<script type="module" src="./entry.js"></script></body></html>'
      );
      const result = await update(
        scenario === 'alias-html'
          ? html.replace(/^\/private\/var\//, '/var/')
          : html
      );
      expect(result.immutableModules).toContain('reload');
    } finally {
      await compiler?.waitForCompileFinish();
      await fs.rm(owner, { recursive: true, force: true });
    }
  });
}
