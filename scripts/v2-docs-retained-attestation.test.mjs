// cspell:words binascii hashlib
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

test('Docs retained proof is an offline policy evaluator', () => {
  const source = readFileSync(new URL('./docs-retained-attestation.py', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /(?:__main__|os\.environ|NPM_[A-Z_]+_TOKEN)/);
  const imports = Array.from(source.matchAll(/^\s*import +(.+)$/gm), match => match[1]);
  assert.ok(imports.length > 0);
  assert.ok(imports.every(name => ['base64', 'binascii', 'hashlib', 'json', 're'].includes(name)));
  assert.doesNotMatch(source, /^\s*from /m);
});

test('check retained Docs attestation with offline fixtures', () => {
  execFileSync('python3', ['-B', 'scripts/docs-retained-attestation.test.py'], { timeout: 15_000 });
});
