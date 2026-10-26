import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { assertReviewedCandidatePackHooks, reviewedCorePrepack } from './candidate-pack-hooks.mjs';

const core = fileURLToPath(new URL('../packages/core/', import.meta.url));
test('allows only exact reviewed Core prepack with its reviewed source', () => {
  assert.doesNotThrow(() => assertReviewedCandidatePackHooks(core, {
    name: '@farmfe/core', scripts: { prepack: reviewedCorePrepack }
  }));
});
test('rejects other hooks, packages and appended commands before execution', () => {
  for (const manifest of [
    {name:'@farmfe/core',scripts:{prepack:reviewedCorePrepack+' && node unsafe.mjs'}},
    {name:'@farmfe/cli',scripts:{prepack:reviewedCorePrepack}},
    {name:'@farmfe/core',scripts:{prepack:reviewedCorePrepack,postpack:'node unknown.mjs'}},
    {name:'@farmfe/core',scripts:{prepare:reviewedCorePrepack}}
  ]) assert.throws(() => assertReviewedCandidatePackHooks(core, manifest), /Review candidate pack hook/);
});
test('rejects changed script body even with the exact hook; accepts Git CRLF checkout', () => {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'farm-reviewed-pack-hook-'));
  fs.mkdirSync(path.join(dir,'scripts'));
  for(const name of ['prepare-sdk-types.mjs','prepare-cjs-types.mjs']) {
    fs.writeFileSync(path.join(dir,'scripts',name),fs.readFileSync(path.join(core,'scripts',name),'utf8').replaceAll('\r\n','\n').replaceAll('\n','\r\n'));
  }
  const manifest={name:'@farmfe/core',scripts:{prepack:reviewedCorePrepack}};
  assert.doesNotThrow(()=>assertReviewedCandidatePackHooks(dir,manifest));
  fs.appendFileSync(path.join(dir,'scripts/prepare-cjs-types.mjs'),'\nthrow Error("unreviewed");\n');
  assert.throws(()=>assertReviewedCandidatePackHooks(dir,manifest),/Review changed candidate pack script/);
});
test('packages without pack hooks remain allowed', () => {
  assert.doesNotThrow(()=>assertReviewedCandidatePackHooks(core,{name:'@farmfe/cli'}));
});
