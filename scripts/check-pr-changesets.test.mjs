// cspell:words NOSYSTEM
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { verifyPreparedRelease, verifyChangesetsPreparation, runPreparedInventory, changesetBase, fetchChangesetBase, isNoPublishDiff, archival2368, isExactArchivalPullRequest } from './check-pr-changesets.mjs';
import { changesetsPublication } from './v2-release-inventory.mjs';
const digest = text => crypto.createHash('sha256').update(text).digest('hex');

test('ordinary main and unified PRs fetch their exact verified event baseline', () => {
  for (const ref of ['main', 'codex/v2-unified-release-20261025']) {
    const sha = 'a'.repeat(40);
    const event = { pull_request: { base: { ref, sha, repo: { full_name: 'farm-fe/farm' } } } };
    const environment = { GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_BASE_REF: ref };
    assert.deepEqual(changesetBase(event, environment), { ref, sha });
    const calls = [];
    const actual = fetchChangesetBase(event, environment, args => { calls.push(args); return args[0] === 'rev-parse' ? sha + '\n' : ''; });
    assert.equal(actual, sha);
    assert.deepEqual(calls, [['fetch', '--no-tags', 'origin', sha], ['rev-parse', '--verify', `${sha}^{commit}`], ['merge-base', '--is-ancestor', sha, 'HEAD']]);
  }
});

test('missing mismatched or unsafe PR baseline fails before any fetch', () => {
  const base = { ref: 'main', sha: 'a'.repeat(40), repo: { full_name: 'farm-fe/farm' } };
  const environment = { GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_BASE_REF: 'main' };
  for (const value of [null, {}, { ...base, sha: '' }, { ...base, ref: 'main; echo unsafe' }, { ...base, ref: 'other' }, { ...base, repo: { full_name: 'other/farm' } }]) {
    assert.throws(() => fetchChangesetBase({ pull_request: { base: value } }, environment, () => assert.fail('Invalid baseline cannot fetch')));
  }
  const event = { pull_request: { base } };
  assert.throws(() => fetchChangesetBase(event, environment, () => { throw new Error('Fetch failed'); }), /Fetch failed/);
  assert.throws(() => fetchChangesetBase(event, environment, args => args[0] === 'rev-parse' ? 'b'.repeat(40) : ''), /differs/);
});

test('dispatch uses its verified release PR and push uses before SHA without a silent main fallback', () => {
  const sha = 'a'.repeat(40), repository = 'farm-fe/farm';
  const event = { inputs: { release_base: sha }, pull_request: { base: { ref: 'main', sha, repo: { full_name: repository } } } };
  const environment = { GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_REPOSITORY: repository };
  assert.equal(changesetBase(event, environment).sha, sha);
  assert.throws(() => changesetBase({ ...event, inputs: {} }, environment), /differs/);
  const push = { GITHUB_EVENT_NAME: 'push', GITHUB_REPOSITORY: repository };
  assert.deepEqual(changesetBase({ before: sha }, push), { sha, ref: null });
  for (const before of [undefined, '', '0'.repeat(40)]) assert.throws(() => changesetBase({ before }, push));
  assert.throws(() => changesetBase(null, environment));
});

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-prepared-release-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const directory of ['docs/releases', 'docs/release-evidence', '.changeset', 'pkg']) fs.mkdirSync(path.join(root, directory), { recursive: true });
  const plan = { npm: { primary: [{ name: 'pkg', version: '2.0.0', manifestPath: 'pkg/package.json' }] } };
  const changelog = '# pkg\n\n## 2.0.0\n\nStable changes\n';
  const original = { '.changeset/pre.json': '{"mode":"pre","tag":"beta","changesets":["fix"]}', '.changeset/fix.md': '---\n"pkg": patch\n---\nFix\n' };
  const record = { schemaVersion: 2, migration: 'already-prepared-first-stable', applied: true,
    sourceCommit: 'a'.repeat(40), planSha256: digest(JSON.stringify(plan)), sourcePreSha256: digest(original['.changeset/pre.json']),
    versions: [{ name: 'pkg', stable: '2.0.0', changelog: { file: 'pkg/CHANGELOG.md', sha256: digest(changelog) } }],
    changesets: ['fix'], changesetSources: [{ id: 'fix', sha256: digest(original['.changeset/fix.md']) }] };
  const recordFile = path.join(root, 'docs/release-evidence/frozen-stable-versions.json');
  fs.writeFileSync(path.join(root, 'docs/releases/v2-candidate.json'), JSON.stringify(plan));
  fs.writeFileSync(path.join(root, 'pkg/package.json'), '{"name":"pkg","version":"2.0.0"}');
  fs.writeFileSync(path.join(root, 'pkg/CHANGELOG.md'), changelog);
  const save = () => fs.writeFileSync(recordFile, JSON.stringify(record)); save();
  return { root, record, save, options: { readGitFile: (_, file) => original[file], readGitChangesets: () => ['fix'], isAncestor: () => true } };
}
test('fully consumed stable preparation validates versions, changelogs and immutable source changesets', t => {
  const { root, options } = fixture(t);
  assert.deepEqual(verifyPreparedRelease(root, { ...options, readProducerRun: () => assert.fail('First stable must not query future Cargo runs'), readProducerArtifact: () => assert.fail('First stable must not download future Cargo artifacts') }), { status: 'verified-stable-preparation', versions: 1, consumed: 1, sourceCommit: 'a'.repeat(40) });
});
test('manifest, changelog, ancestor, omitted source and pending changeset drift cannot bypass Changesets', t => {
  for (const mutate of [
    x => fs.writeFileSync(path.join(x.root, 'pkg/package.json'), '{"version":"2.0.1"}'),
    x => fs.appendFileSync(path.join(x.root, 'pkg/CHANGELOG.md'), 'Unrecorded notes'),
    x => { x.options.isAncestor = () => false; },
    x => { x.options.readGitChangesets = () => ['fix', 'omitted']; },
    x => fs.writeFileSync(path.join(x.root, '.changeset/new.md'), 'Still pending'),
    x => { x.record.changesetSources[0].sha256 = 'b'.repeat(64); x.save(); }
  ]) {
    const value = fixture(t); mutate(value);
    assert.throws(() => verifyPreparedRelease(value.root, value.options));
  }
});

test('ordinary Changesets receipt is independent of first stable and rejects stale heads or changed source', t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-normal-preparation-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const write = (file, value) => { const target = path.join(directory, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, typeof value === 'string' ? value : JSON.stringify(value)); };
  fs.mkdirSync(path.join(directory, '.changeset'));
  const baseline = { npm: { primary: [{ name: 'pkg', version: '1.0.0', manifestPath: 'pkg/package.json' }], payloads: [] }, cargo: [] };
  const plan = structuredClone(baseline); plan.npm.primary[0].version = '1.0.1';
  const official = { changesets: [{ id: 'fix' }], releases: [{ name: 'pkg', newVersion: '1.0.1' }] };
  plan.publication = changesetsPublication(plan, baseline, official.releases);
  const source = '---\n"pkg": patch\n---\n\nFix.\n';
  const record = { mode: 'changesets', schemaVersion: 1, baseSHA: 'a'.repeat(40), sourceHeadSHA: 'b'.repeat(40), officialPlanSha256: digest(JSON.stringify(official)), planSha256: digest(JSON.stringify(plan)), consumed: [{ id: 'fix', sha256: digest(source) }] };
  write('docs/releases/v2-candidate.json', plan); write('docs/release-evidence/changesets-preparation.json', record); write('docs/release-evidence/changesets-version-plan.json', official);
  write('pkg/package.json', { name: 'pkg', version: '1.0.1' }); write('pkg/CHANGELOG.md', '# pkg\n\n## 1.0.1\n');
  const options = { readGitFile: (_, file) => file.endsWith('v2-candidate.json') ? JSON.stringify(baseline) : source, isAncestor: () => true, generatedDiff: () => ['docs/releases/v2-candidate.json'] };
  assert.equal(verifyChangesetsPreparation(directory, options).status, 'verified-changesets-preparation');
  assert.throws(() => verifyChangesetsPreparation(directory, { ...options, isAncestor: () => false }), /no longer belongs/);
  assert.throws(() => verifyChangesetsPreparation(directory, { ...options, generatedDiff: () => ['pkg/src/index.ts'] }), /Source changed/);
  write('pkg/package.json', { name: 'pkg', version: '1.0.2' });
  assert.throws(() => verifyChangesetsPreparation(directory, options));
});


test('generic CI dispatch uses the exact verified base without release-only inputs',()=>{
  const sha='a'.repeat(40),repository='farm-fe/farm';
  const event={inputs:{base_sha:sha},pull_request:{base:{ref:'main',sha,repo:{full_name:repository}}}};
  assert.equal(changesetBase(event,{GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REPOSITORY:repository}).sha,sha);
  assert.throws(()=>changesetBase({...event,inputs:{base_sha:'b'.repeat(40)}},{GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REPOSITORY:repository}));
});


test('no-publish classification is narrow, exact and fails closed', () => {
  const base = 'a'.repeat(40), head = 'b'.repeat(40), blob = 'c'.repeat(40);
  const testPath = 'packages/utils/tests/color.spec.ts';
  const docsPath = 'website/docs/plugins/writing-plugins/rust-plugin.mdx';
  const probe = ({ paths = [testPath], diff, mode = '100644', fail, drift = false, tree } = {}) => {
    let heads = 0;
    return isNoPublishDiff(base, 'pull_request', args => {
      if (args[0] === fail) throw new Error('unavailable');
      if (args[0] === 'rev-parse') return args[2].startsWith(base) ? base : drift && heads++ ? base : head;
      if (args[0] === 'merge-base') return '';
      if (args[0] === 'diff') {
        assert.deepEqual(args, ['diff', '--name-only', '--no-renames', '--no-ext-diff', '--ignore-submodules=none', '-z', base, head]);
        return diff ?? paths.join('\0') + '\0';
      }
      if (args[0] === 'ls-tree') return tree ?? paths.map(file => `${mode} blob ${blob}\t${file}\0`).join('');
      assert.fail('Unexpected Git operation');
    });
  };
  for (const eventName of [undefined, 'push', 'workflow_dispatch', 'pull_request_target']) {
    assert.equal(isNoPublishDiff(base, eventName, () => assert.fail('must not invoke Git')), false);
  }
  assert.equal(probe(), true);
  assert.equal(probe({ paths: [testPath, docsPath] }), true);
  for (const paths of [
    [testPath, 'packages/utils/src/color.ts'],
    [testPath, 'packages/utils/src/moved.ts'],
    ['packages/utils/src/color.ts', 'packages/utils/tests/moved.spec.ts'],
    ['packages/create-farm/index.js'], ['packages/utils/package.json'],
    ['scripts/check-pr-changesets.mjs'], ['scripts/check-pr-changesets.test.mjs'],
    ['docs/releases/v2-candidate.json'], ['.changeset/empty.md'],
    ['website/docs/../package.json'], [testPath, testPath]
  ]) assert.equal(probe({ paths }), false);
  for (const diff of ['', testPath, '\0', testPath + '\0\0', testPath + '\n\0']) assert.equal(probe({ diff }), false);
  for (const mode of ['100755', '120000', '160000']) assert.equal(probe({ mode }), false);
  for (const fail of ['rev-parse', 'merge-base', 'diff', 'ls-tree']) assert.equal(probe({ fail }), false);
  for (const tree of ['', 'malformed\0', `100644 blob ${blob}\t${testPath}`]) assert.equal(probe({ tree }), false);
  assert.equal(probe({ drift: true }), false);
  assert.equal(isNoPublishDiff('main', 'pull_request', () => assert.fail('must not invoke Git')), false);
});

test('one archival PR binds exact repository, event, paths, bytes, modes and both snapshots', () => {
  const base = 'a'.repeat(40), head = 'b'.repeat(40);
  const repository = { id: 507542208, full_name: 'farm-fe/farm' };
  const event = { pull_request: { number: 2368, base: { sha: base, ref: 'main', repo: repository }, head: { sha: head, repo: repository } } };
  const environment = { GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_SHA: head };
  const paths = Object.keys(archival2368).sort();
  const snapshots = [0, 1].map(index => paths.map(file => archival2368[file][index]).filter(Boolean).join('\0') + '\0');
  const probe = ({ input = event, env = environment, files = paths, trees = snapshots, fail, drift = false } = {}) => {
    let heads = 0;
    return isExactArchivalPullRequest(base, input, env, args => {
      if (args[0] === fail) throw Error('unavailable');
      if (args[0] === 'rev-parse') return args[2].startsWith(base) ? base : drift && heads++ ? base : head;
      if (args[0] === 'merge-base') return '';
      if (args[0] === 'diff') {
        assert.deepEqual(args, ['diff', '--name-only', '--no-renames', '--no-ext-diff', '--ignore-submodules=none', '-z', base, head]);
        return files.join('\0') + '\0';
      }
      if (args[0] === 'ls-tree') return trees[args[3] === base ? 0 : 1];
      assert.fail('Unexpected Git operation');
    });
  };
  assert.equal(probe(), true);
  for (const value of [undefined, base, 'main']) assert.equal(probe({ env: { ...environment, GITHUB_SHA: value } }), false);
  for (const kind of ['push', 'workflow_dispatch', 'pull_request_target', undefined]) assert.equal(probe({ env: { ...environment, GITHUB_EVENT_NAME: kind } }), false);
  for (const alter of [
    pr => { pr.number = 2331; }, pr => { pr.base.ref = 'other'; }, pr => { pr.base.sha = head; },
    pr => { pr.head.repo.id = 1; }, pr => { pr.head.repo.full_name = 'fork/farm'; },
    pr => { pr.base.repo.id = 1; }, pr => { pr.base.repo.full_name = 'fork/farm'; },
    pr => { pr.head.sha = 'main'; }
  ]) { const input = structuredClone(event); alter(input.pull_request); assert.equal(probe({ input }), false); }
  assert.equal(probe({ env: { ...environment, GITHUB_REPOSITORY: 'fork/farm' } }), false);
  for (const path of ['packages/create-farm/index.js', 'packages/utils/package.json', 'pnpm-lock.yaml', '.github/workflows/ci.yaml', 'scripts/check-pr-changesets.mjs', '.changeset/empty.md', 'unrelated.md']) assert.equal(probe({ files: [...paths, path].sort() }), false);
  for (const path of paths) assert.equal(probe({ files: paths.filter(file => file !== path) }), false);
  assert.equal(probe({ files: [...paths, paths[0]] }), false);
  for (const index of [0, 1]) {
    for (const entry of snapshots[index].split('\0').filter(Boolean)) {
      for (const changed of [entry.replace('100644', '100755'), entry.replace('100644', '120000'), entry.replace(/blob [a-f0-9]{40}/, 'blob ' + 'f'.repeat(40)), '']) {
        const trees = [...snapshots]; trees[index] = trees[index].replace(entry, changed); assert.equal(probe({ trees }), false);
      }
    }
  }
  // Adoption/replay: baseline already containing the archive is not eligible.
  assert.equal(probe({ trees: [snapshots[1], snapshots[1]] }), false);
  for (const fail of ['rev-parse', 'merge-base', 'diff', 'ls-tree']) assert.equal(probe({ fail }), false);
  assert.equal(probe({ drift: true }), false);
});


test('prepared hosted inventory fetches locked inputs first with no API or publication credentials', () => {
  const calls = [];
  const directory = path.join(os.tmpdir(), 'farm-prepared-inventory');
  const environment = { GITHUB_ACTIONS: 'true', GH_TOKEN: 'read', GITHUB_TOKEN: 'read',
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: 'oidc', NPM_TOKEN: 'npm', CARGO_REGISTRY_TOKEN: 'cargo',
    GIT_CONFIG_PARAMETERS: 'ambient', PATH: '/bin', HOME: '/home/runner', CARGO_HOME: '/cargo' };
  runPreparedInventory({ directory, environment, run: (...args) => calls.push(args) });
  const env = { PATH: '/bin', HOME: '/home/runner', CARGO_HOME: '/cargo',
    GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_TERMINAL_PROMPT: '0' };
  const options = { cwd: directory, stdio: 'inherit', env };
  assert.deepEqual(calls, [
    ['python3', [path.join(directory, 'scripts/prepare-cargo-package-inputs.py'), '--fetch'], options],
    [process.execPath, [path.join(directory, 'scripts/v2-release-preflight.mjs')], options],
  ]);
});

test('local prepared validation stays offline and a failed hosted fetch never proceeds', () => {
  const calls = [];
  runPreparedInventory({ environment: {}, run: (command, args) => calls.push([command, args]) });
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], process.execPath);
  assert.match(calls[0][1][0], /v2-release-preflight[.]mjs$/);
  const failed = [];
  assert.throws(() => runPreparedInventory({ environment: { GITHUB_ACTIONS: 'true' }, run: command => {
    failed.push(command); throw new Error('Locked Cargo fetch failed');
  } }), /Locked Cargo fetch failed/);
  assert.deepEqual(failed, ['python3']);
});

test('the Changesets job does not persist its checkout token for dependency fetches', () => {
  const workflow = fs.readFileSync(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  const job = workflow.split('\n  changesets-check:\n')[1]?.split('\n  ts-test:\n')[0];
  assert.match(job, /uses: actions\/checkout@v4\s+with:\s+fetch-depth: 0\s+persist-credentials: false/);
  assert.match(job, /permissions:\s+contents: read\s+actions: read\s+pull-requests: read/);
});
