import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
import {fileURLToPath} from 'node:url';
for (const args of [[], ['--install']]) {
  test(`retired deployment ${args.join(' ') || 'build'} fails without source fallback`, () => {
    const result = spawnSync(process.execPath, [fileURLToPath(new URL('./farm-docs-vercel-build.mjs', import.meta.url)), ...args], {encoding:'utf8', windowsHide:true});
    assert.equal(result.status, 1);
    assert.match(result.stderr, /FARM_DOCS_SOURCE_DEPLOYMENT_RETIRED/);
    assert.match(result.stderr, /published v2 release set/);
    assert.equal(result.stdout, '');
  });
}
