import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { evaluateNodeEngines } from './v2-node-engines.mjs';
const workflow = fs.readFileSync(new URL('../.github/workflows/release.yaml', import.meta.url), 'utf8');
test('all stable publication jobs use the supported pinned Node version', () => {
  const versions = [...workflow.matchAll(/node-version:\s*([0-9.]+)/g)].map(match => match[1]);
  assert.equal(versions.length, 3);
  assert.deepEqual([...new Set(versions)], ['22.20.0']);
  const tools = JSON.parse(fs.readFileSync(new URL('../packages/plugin-tools/package.json', import.meta.url)));
  const rows = evaluateNodeEngines([{ name: tools.name, engine: tools.engines.node, optional: false }], ['20.18.0', ...versions]);
  assert.equal(rows[0].satisfiesInstalledRanges, false);
  assert.ok(rows.slice(1).every(row => row.satisfiesInstalledRanges));
});
test('Node repair retains manual, main-only, explicit publication confirmation', () => {
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /confirm_release:[\s\S]*?default: false/);
  assert.match(workflow, /github.event_name == 'workflow_dispatch' && github.ref == 'refs\/heads\/main' && inputs.confirm_release == true/);
  assert.doesNotMatch(workflow, /^  push:/m);
});
