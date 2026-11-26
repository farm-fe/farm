// cspell:words noprofile norc
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';

test('actual Docs writer command is statically disabled', () => {
  const result = spawnSync('python3', ['-B', 'scripts/docs-pair-protected-writer.py', '--help'], {
    env: { PATH: process.env.PATH }, encoding: 'utf8', timeout: 15_000
  });
  assert.equal(result.status, 1);
  assert.equal(result.stdout, '');
  assert.match(result.stderr, /^Protected Docs publication stopped/);
});

for (const step of ['docs-pair-protected-writer', 'docs-pair-npm-transport']) {
  test(`exercise ${step} with fake network and npm clients`, () => {
    execFileSync('python3', ['-B', `scripts/${step}.test.py`], { timeout: 15_000 });
  });
}


test('writer registration grants no authority and only reports disabled on protected main', () => {
  const file = new URL('../.github/workflows/docs-pair-publish.yaml', import.meta.url);
  const workflow = YAML.parse(readFileSync(file, 'utf8'));
  const onlyStep = `printf '%s\\n' 'Docs package publishing is disabled. This workflow registers an identity only.'`;
  assert.deepEqual(workflow, {
    name: 'Docs package writer registration (disabled)',
    on: { workflow_dispatch: null },
    permissions: {},
    jobs: {
      registration: {
        name: 'Report that the Docs writer is disabled',
        if: [
          "github.event_name == 'workflow_dispatch'",
          "github.repository == 'farm-fe/farm'",
          "github.repository_id == '507542208'",
          "github.ref == 'refs/heads/main'",
          'github.ref_protected',
          "github.run_attempt == '1'",
          'github.sha == github.workflow_sha',
          "github.workflow_ref == 'farm-fe/farm/.github/workflows/docs-pair-publish.yaml@refs/heads/main'"
        ].join(' && '),
        'runs-on': 'ubuntu-24.04',
        'timeout-minutes': 1,
        steps: [{ name: 'Report registration only', shell: 'bash', run: onlyStep }]
      }
    }
  });
  assert.equal(execFileSync('/bin/bash', ['--noprofile', '--norc', '-c', workflow.jobs.registration.steps[0].run], {
    env: {}, encoding: 'utf8', timeout: 10_000
  }), 'Docs package publishing is disabled. This workflow registers an identity only.\n');
});
