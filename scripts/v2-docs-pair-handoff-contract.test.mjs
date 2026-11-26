import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

test('protected Docs handoff evaluates only offline inputs', () => {
  const file = new URL('./docs-pair-reviewed-handoff.py', import.meta.url);
  const source = readFileSync(file, 'utf8');
  assert.doesNotMatch(source, /(?:__main__|os\.environ|subprocess|urllib|requests|NPM_[A-Z_]+_TOKEN)/);
  execFileSync('python3', ['-B', 'scripts/docs-pair-reviewed-handoff.test.py'], { timeout: 15_000 });
});
