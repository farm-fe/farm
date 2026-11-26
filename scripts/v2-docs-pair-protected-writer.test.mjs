// cspell:words pipefail
// cspell:words noprofile norc
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';

for (const step of ['docs-pair-protected-writer', 'docs-pair-npm-transport']) {
  test('exercise ' + step + ' with fake network and npm clients', () => {
    execFileSync('python3', ['-B', 'scripts/' + step + '.test.py'], { timeout: 15_000 });
  });
}

const file = new URL('../.github/workflows/docs-pair-publish.yaml', import.meta.url);
const raw = readFileSync(file, 'utf8');
const workflow = YAML.parse(raw);
const job = workflow.jobs['publish-pair'];
const preflight = workflow.jobs.preflight;

test('actual Docs writer and preflight reject invocation outside GitHub Actions', () => {
  for (const args of [
    ['--preflight'],
    ['--node', '/usr/bin/node', '--npm-cli', '/tmp/npm-cli.js', '--report', '/tmp/docs-pair-publish-journal.jsonl']
  ]) {
    const result = spawnSync('python3', ['-B', 'scripts/docs-pair-protected-writer.py', ...args], {
      env: { PATH: process.env.PATH }, encoding: 'utf8', timeout: 15_000
    });
    assert.equal(result.status, 1);
    assert.equal(result.stdout, '');
    assert.match(result.stderr, /^Protected Docs publication stopped/);
  }
});

test('writer accepts only reviewed first dispatch on the fixed protected main source', () => {
  assert.deepEqual(Object.keys(workflow.on), ['workflow_dispatch']);
  assert.deepEqual(Object.keys(workflow.jobs), ['preflight', 'publish-pair']);
  assert.deepEqual(Object.keys(workflow.on.workflow_dispatch.inputs), [
    'source_sha', 'source_tree', 'producer_run', 'artifact_id', 'artifact_sha256', 'tag', 'confirm'
  ]);
  for (const input of Object.values(workflow.on.workflow_dispatch.inputs)) {
    assert.equal(input.required, true);
    assert.equal(input.type, 'string');
  }
  assert.equal(job.if, [
    "github.event_name == 'workflow_dispatch'",
    "github.repository == 'farm-fe/farm'",
    "github.repository_id == '507542208'",
    "github.ref == 'refs/heads/main'",
    'github.ref_protected',
    "github.run_attempt == '1'",
    'github.sha == github.workflow_sha',
    'github.sha == inputs.source_sha',
    "inputs.tag == 'farm-docs-bootstrap'",
    "inputs.confirm == 'first-and-only'",
    "github.workflow_ref == 'farm-fe/farm/.github/workflows/docs-pair-publish.yaml@refs/heads/main'"
  ].join(' && '));
  assert.deepEqual(workflow.concurrency, {group: 'farm-docs-pair-publish-2.0.0', 'cancel-in-progress': false});
  assert.deepEqual(job.needs, ['preflight']);
  assert.equal(preflight.needs, undefined);
  assert.equal(preflight.if, job.if);
  assert.equal(preflight.environment, undefined);
  assert.equal(preflight.permissions, undefined);
  assert.equal(preflight['runs-on'], 'ubuntu-24.04');
  assert.equal(job['runs-on'], 'ubuntu-24.04');
  assert.equal(job['timeout-minutes'], 80);
  assert.equal(job.environment, undefined);
  assert.equal(job.permissions, undefined);
  assert.deepEqual(workflow.permissions, { contents: 'read', actions: 'read' });
});

test('only the reviewed writer step receives credentials; checkout and artifacts stay narrow', () => {
  const [checkout, tools, publish, evidence, ...others] = job.steps;
  const [preflightCheckout, validate, inspect, ...unexpected] = preflight.steps;
  assert.deepEqual(unexpected, []);
  assert.deepEqual(preflightCheckout, checkout);
  assert.deepEqual(inspect.env, { GH_TOKEN: '$'+'{{ github.token }}' });
  assert.equal(inspect.run, 'python3 -B scripts/docs-pair-protected-writer.py --preflight');
  assert.deepEqual(others, []);
  assert.equal(checkout.uses, 'actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683');
  assert.deepEqual(checkout.with, { ref: '${{ github.sha }}', 'persist-credentials': false });
  assert.deepEqual(publish.env, { GH_TOKEN: '${{ github.token }}', NPM_BYPASS_2FA_TOKEN: '${{ secrets.NPM_BYPASS_2FA_TOKEN }}' });
  assert.equal(raw.match(/secrets\./g)?.length, 1);
  assert.doesNotMatch(raw, /(?:id-token|attestations):|npm (?:publish|dist-tag)/);
  assert.match(publish.run, /^tool_dir=.*\npython3 -B scripts\/docs-pair-protected-writer\.py /);
  assert.match(publish.run, /--report "\$RUNNER_TEMP\/docs-pair-publish-journal\.jsonl"/);
  assert.match(tools.run, /npm\/-\/npm-11\.21\.2\.tgz/);
  assert.match(tools.run, /node-linux-x64\/-\/node-linux-x64-22\.22\.2\.tgz/);
  assert.match(tools.run, /openssl dgst -sha512/);
  assert.match(tools.run, /curl -q --no-location --max-filesize 100663296/);
  assert.equal(tools.env, undefined);
  assert.equal(evidence.env, undefined);
  assert.equal(evidence.if, 'always()');
  assert.equal(evidence.uses, 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02');
  assert.equal(evidence.with.overwrite, false);
  assert.deepEqual(evidence.with.path.trim().split('\n'), [
    '${{ runner.temp }}/docs-pair-publish-journal.jsonl',
    '${{ runner.temp }}/docs-pair-retained.json'
  ]);
  for (const step of [validate, inspect, tools, publish]) {
    execFileSync('/bin/bash', ['--noprofile', '--norc', '-n'], { input: step.run, timeout: 10_000 });
  }
});

test('preflight rejects incorrect dispatch identifiers, checkout tree, tag and confirmation', () => {
  const [source, tree] = execFileSync('git', ['rev-parse', 'HEAD', 'HEAD^{tree}'], {
    encoding: 'utf8', timeout: 10_000
  }).trim().split('\n');
  const valid = {
    PATH: process.env.PATH,
    SOURCE_SHA: source,
    SOURCE_TREE: tree,
    PRODUCER_RUN: '123',
    ARTIFACT_ID: '456',
    ARTIFACT_SHA256: 'a'.repeat(64),
    STAGING_TAG: 'farm-docs-bootstrap',
    CONFIRM: 'first-and-only'
  };
  const run = override => spawnSync('/bin/bash', ['--noprofile', '--norc', '-eo', 'pipefail', '-c',
    preflight.steps[1].run], { env: {...valid, ...override}, encoding: 'utf8', timeout: 10_000 });
  assert.equal(run({}).status, 0);
  for (const override of [{ SOURCE_SHA: '0'.repeat(40) }, { SOURCE_TREE: '0'.repeat(40) },
    { PRODUCER_RUN: '0123' }, { ARTIFACT_ID: '0' }, { ARTIFACT_SHA256: 'invalid' },
    { STAGING_TAG: 'latest' }, { CONFIRM: 'true' }]) {
    assert.equal(run(override).status, 1, JSON.stringify(override));
  }
});
