// cspell:words hashlib noprofile norc pipefail
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { parse } from 'yaml';

const workflowPath = '.github/workflows/docs-pair-bootstrap.yaml';
const source = fs.readFileSync(new URL('../' + workflowPath, import.meta.url), 'utf8');
const workflow = parse(source);
const job = workflow.jobs.freeze;
const scripts = job.steps.filter(step => typeof step.run === 'string').map(step => step.run);
const sha = 'a'.repeat(40);
const tree = 'b'.repeat(40);

function isolated(t) {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-freeze-'));
  t.after(() => fs.rmSync(cwd, { recursive: true, force: true }));
  return cwd;
}

function bash(body, cwd, values = {}) {
  return spawnSync('/bin/bash', ['--noprofile', '--norc', '-e', '-u', '-o', 'pipefail', '-c', body],
    { cwd, env: { PATH: '/usr/bin:/bin', HOME: cwd, ...values }, encoding: 'utf8', timeout: 5000 });
}

test('freeze is manual, protected, first-attempt, and has only immutable artifact outputs', () => {
  assert.deepEqual(Object.keys(workflow.on), ['workflow_dispatch']);
  assert.deepEqual(Object.keys(workflow.on.workflow_dispatch.inputs).sort(), ['source_sha', 'source_tree']);
  assert.deepEqual(workflow.permissions, {});
  assert.deepEqual(Object.keys(workflow.jobs), ['freeze']);
  assert.equal(job['runs-on'], 'ubuntu-24.04');
  for (const field of ['github.ref_protected', "github.run_attempt == '1'", 'github.workflow_sha == github.sha', 'github.sha == inputs.source_sha']) {
    assert.ok(job.if.includes(field));
  }
  assert.ok(!job.permissions && !job.environment);
  assert.doesNotMatch(source, /secrets\.|id-token:|attestations:|npm publish|publish-pair|GITHUB_TOKEN|NODE_AUTH_TOKEN/);
  const actions = job.steps.filter(step => step.uses);
  assert.equal(actions.length, 1);
  const [artifact] = actions;
  assert.match(artifact.uses, /^actions\/upload-artifact@[a-f0-9]{40}$/);
  assert.equal(artifact.name, 'Upload exact original Docs archives');
  assert.equal(artifact.with.overwrite, false);
  assert.equal(artifact.with['if-no-files-found'], 'error');
  assert.deepEqual(artifact.with.path.trim().split('\n').map(line => line.split('/').at(-1)).sort(),
    ['farmfe-docs-2.0.0.tgz', 'farmfe-docs-theme-default-2.0.0.tgz', 'frozen-pair.json']);
  for (const shell of scripts) assert.equal(spawnSync('/bin/bash', ['-n'], { input: shell }).status, 0);
});

test('bad identity never writes workflow environment or begins the public fetch', t => {
  const cwd = isolated(t);
  const control = { GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_REF: 'refs/heads/main',
    GITHUB_REF_PROTECTED: 'true', GITHUB_REPOSITORY_ID: '507542208', GITHUB_WORKFLOW_SHA: sha, GITHUB_SHA: sha,
    GITHUB_WORKFLOW_REF: 'farm-fe/farm/' + workflowPath + '@refs/heads/main', SOURCE_SHA: sha, SOURCE_TREE: tree,
    GITHUB_RUN_ATTEMPT: '1', GITHUB_ENV: path.join(cwd, 'workflow.env'), RUNNER_TEMP: cwd };
  for (const changed of [
    { GITHUB_EVENT_NAME: 'pull_request' }, { GITHUB_REPOSITORY: 'fork/farm' }, { GITHUB_REF: 'refs/heads/feature' },
    { GITHUB_REF_PROTECTED: 'false' }, { GITHUB_REPOSITORY_ID: '1' }, { GITHUB_WORKFLOW_SHA: tree },
    { GITHUB_WORKFLOW_REF: control.GITHUB_WORKFLOW_REF.replace('main', 'feature') }, { SOURCE_SHA: tree },
    { SOURCE_TREE: 'not-a-tree' }, { GITHUB_RUN_ATTEMPT: '2' },
  ]) {
    assert.equal(bash(scripts[0], cwd, { ...control, ...changed }).status, 1, Object.keys(changed)[0]);
    assert.deepEqual(fs.readdirSync(cwd), [], 'rejected identities must stop before setup');
  }
});

test('upload gate checks actual bytes, exact pair order, and no substituted files', t => {
  const cwd = isolated(t);
  const names = ['docs-theme-default', 'docs'];
  const measure = (algorithm, body, format) => createHash(algorithm).update(body).digest(format);
  const files = names.map(name => path.join(cwd, 'farmfe-' + name + '-2.0.0.tgz'));
  const originals = names.map(name => Buffer.from('synthetic frozen artifact for ' + name));
  const expected = { schemaVersion: 1, kind: 'docs-frozen-pair-v1', sourceSHA: sha, sourceTree: tree,
    packages: names.map((name, i) => ({ name: '@farmfe/' + name, version: '2.0.0', archiveBytes: originals[i].length,
      archiveSha256: measure('sha256', originals[i], 'hex'), integrity: 'sha512-' + measure('sha512', originals[i], 'base64') })) };
  const receipt = path.join(cwd, 'frozen-pair.json');
  const write = value => fs.writeFileSync(receipt, JSON.stringify(value));
  files.forEach((file, i) => fs.writeFileSync(file, originals[i]));
  const verify = () => bash(scripts.at(-1), cwd, { PACK_DIR: cwd, SOURCE_SHA: sha, SOURCE_TREE: tree });
  write(expected);
  assert.equal(verify().status, 0);
  for (const changed of [{ sourceTree: sha }, { packages: expected.packages.slice(1) }, { packages: [...expected.packages].reverse() }]) {
    write({ ...expected, ...changed });
    assert.equal(verify().status, 1);
  }
  write(expected);
  fs.writeFileSync(files[1], Buffer.alloc(originals[1].length, 7));
  assert.equal(verify().status, 1, 'equal size cannot replace original bytes');
  fs.writeFileSync(files[1], originals[1]);
  fs.writeFileSync(path.join(cwd, 'unapproved'), 'not part of the pair');
  assert.equal(verify().status, 1);
  fs.unlinkSync(path.join(cwd, 'unapproved'));
  fs.unlinkSync(files[1]);
  fs.symlinkSync(files[0], files[1]);
  assert.equal(verify().status, 1);
});
