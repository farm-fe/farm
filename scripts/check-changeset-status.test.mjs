import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

const source = fs.readFileSync(new URL('./check-changeset-status.mjs', import.meta.url), 'utf8').replace(/^import .*;\r?\n/gm, '');
function status({ files = [], pre } = {}) {
  const stop = new Error('exit');
  let code;
  const messages = [];
  try {
    vm.runInNewContext(source, {
      path,
      readdirSync: () => ['README.md', ...files],
      existsSync: () => pre !== undefined,
      readFileSync: () => JSON.stringify(pre),
      console: { log: message => messages.push(message) },
      process: { exit: value => { code = value; throw stop; } }
    });
  } catch (error) { if (error !== stop) throw error; }
  return { code, messages };
}

test('prerelease exit requests stable version generation even when beta consumed every changeset', () => {
  const result = status({ files: ['fix.md'], pre: { mode: 'exit', changesets: ['fix'] } });
  assert.equal(result.code, 0);
  assert.match(result.messages[0], /generate stable versions/);
  assert.equal(status({ pre: { mode: 'exit', changesets: [] } }).code, 0);
});

test('active prerelease only considers unconsumed changesets', () => {
  assert.equal(status({ files: ['fix.md'], pre: { mode: 'pre', changesets: ['fix'] } }).code, 1);
  assert.equal(status({ files: ['fix.md', 'new.md'], pre: { mode: 'pre', changesets: ['fix'] } }).code, 0);
});

test('ordinary release keeps the existing changeset/no-changeset status contract', () => {
  assert.equal(status().code, 1);
  assert.equal(status({ files: ['fix.md'] }).code, 0);
});
