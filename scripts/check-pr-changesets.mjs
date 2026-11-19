import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validationEvent, changesetsPublication } from './v2-release-inventory.mjs';
import { cargoChangesetId, validateCargoPreparation, readCargoArtifact } from './prepare-cargo-version-proposal.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const digest = text => crypto.createHash('sha256').update(text.replaceAll('\r\n', '\n')).digest('hex');

export function changesetBase(event, environment) {
  const kind = environment.GITHUB_EVENT_NAME;
  assert.ok(['pull_request', 'workflow_dispatch', 'push'].includes(kind), 'Changesets event baseline unavailable');
  assert.ok(event && typeof event === 'object', 'Changesets event baseline unavailable');
  assert.match(environment.GITHUB_REPOSITORY ?? '', /^[\w.-]+\/[\w.-]+$/);
  if (kind === 'push') {
    assert.match(event.before ?? '', /^[a-f0-9]{40}$/, 'Push baseline SHA required');
    assert.notEqual(event.before, '0'.repeat(40), 'New branch has no safe push baseline');
    return { sha: event.before, ref: null };
  }
  const base = event.pull_request?.base;
  assert.equal(base?.repo?.full_name, environment.GITHUB_REPOSITORY, 'PR base repository differs');
  assert.match(base?.sha ?? '', /^[a-f0-9]{40}$/, 'PR base SHA required');
  assert.match(base?.ref ?? '', /^[A-Za-z0-9][A-Za-z0-9._/-]{0,200}$/, 'PR base ref required');
  assert.ok(!base.ref.includes('..') && !base.ref.includes('//') && !base.ref.endsWith('.lock') && !/[/.]$/.test(base.ref), 'Invalid PR base ref');
  if (environment.GITHUB_BASE_REF) assert.equal(base.ref, environment.GITHUB_BASE_REF, 'PR base ref differs from runner');
  if (kind === 'workflow_dispatch') {
    assert.equal(base.ref, 'main', 'Dispatch requires the verified PR base');
    assert.equal(event.inputs?.base_sha ?? event.inputs?.release_base, base.sha, 'Dispatch base differs');
  }
  return { sha: base.sha, ref: base.ref };
}

export function fetchChangesetBase(event, environment, git = (args => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 }))) {
  const base = changesetBase(event, environment);
  git(['fetch', '--no-tags', 'origin', base.sha]);
  assert.equal(git(['rev-parse', '--verify', `${base.sha}^{commit}`]).trim(), base.sha, 'Fetched Changesets baseline differs');
  git(['merge-base', '--is-ancestor', base.sha, 'HEAD']);
  return base.sha;
}

// Only these audited, non-shipped surfaces are eligible. In particular, generic
// docs/, package manifests, release evidence and executable loaders are not.
const noPublishPaths = new Set([
  'packages/utils/tests/color.spec.ts',
  'website/docs/plugins/writing-plugins/rust-plugin.mdx'
]);

export function isNoPublishDiff(base, eventName, git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 })) {
  try {
    assert.equal(eventName, 'pull_request');
    assert.match(base, /^[a-f0-9]{40}$/);
    assert.equal(git(['rev-parse', '--verify', `${base}^{commit}`]).trim(), base);
    const head = git(['rev-parse', '--verify', 'HEAD^{commit}']).trim();
    assert.match(head, /^[a-f0-9]{40}$/);
    git(['merge-base', '--is-ancestor', base, head]);
    const diff = git(['diff', '--name-only', '--no-renames', '-z', base, head]);
    assert.ok(diff.length > 0 && diff.endsWith('\0'));
    const files = diff.slice(0, -1).split('\0');
    assert.equal(new Set(files).size, files.length);
    assert.ok(files.every(file => noPublishPaths.has(file)));
    // Inspect both sides, including deleted paths. Do not accept symlinks,
    // submodules, executable mode changes, or ambiguous tree output.
    const snapshots = [base, head].map(sha => {
      const tree = git(['ls-tree', '-r', '-z', sha, '--', ...files]);
      assert.ok(!tree || tree.endsWith('\0'));
      const entries = new Map();
      for (const row of tree ? tree.slice(0, -1).split('\0') : []) {
        const match = /^(100644) blob ([a-f0-9]{40})\t([^\0]+)$/.exec(row);
        assert.ok(match && files.includes(match[3]) && !entries.has(match[3]));
        entries.set(match[3], match[2]);
      }
      return entries;
    });
    assert.ok(files.every(file => snapshots.some(tree => tree.has(file))));
    assert.equal(git(['rev-parse', '--verify', 'HEAD^{commit}']).trim(), head);
    return true;
  } catch {
    // Classification is optional: failures never replace ordinary validation.
    return false;
  }
}

// One reviewed archival migration, not a permanent path exemption. Before and
// after tree entries bind every byte, mode and absence; after adoption the old
// before-state no longer matches. No production or release inputs are exempt.
export const archival2368 = Object.freeze({
  ".gitattributes": [
    "100644 blob c936a7034a2f720379ba72cda17838c7d5b59fd8\t.gitattributes",
    "100644 blob 0da1911ef32d6ab136e150f8a4f8cb758477ecd4\t.gitattributes"
  ],
  "cspell.json": [
    "100644 blob c70fed0ca933faa37ed4b5ab2ced61d4ecc87df4\tcspell.json",
    "100644 blob ce6b97b45f297a804ae10433b4f02b654d864476\tcspell.json"
  ],
  "docs/patches/vuetify-4.2.5-finite-data-table-slots.patch": [
    null,
    "100644 blob 9e9b3f46e4a62d1f6f1542215b72a7978baa25db\tdocs/patches/vuetify-4.2.5-finite-data-table-slots.patch"
  ],
  "docs/patches/vuetify-finite-child-witness.patch": [
    null,
    "100644 blob 05f58b80092ec1e8b2656034c5ee2994a399a5ac\tdocs/patches/vuetify-finite-child-witness.patch"
  ],
  "docs/releases/source-migration-checkpoint.md": [
    null,
    "100644 blob 62a2ade438662a5245fc477a4e898f66471f3c13\tdocs/releases/source-migration-checkpoint.md"
  ],
  "docs/source-migration/README.md": [
    null,
    "100644 blob 388429e0450deb017eff318da4cb27574df6d927\tdocs/source-migration/README.md"
  ],
  "docs/source-migration/create-farm-index.js.snapshot": [
    null,
    "100644 blob df67f9abfc6951291eb6313cf3ba6fe198841113\tdocs/source-migration/create-farm-index.js.snapshot"
  ],
  "docs/vuetify-delivery/README.md": [
    null,
    "100644 blob e99213152b02b8694ab6b61f6e841e63e53979c0\tdocs/vuetify-delivery/README.md"
  ],
  "docs/vuetify-delivery/manifest.json": [
    null,
    "100644 blob bcbd63b79c2d5bd4d28c75ed63d0c11dd24faefb\tdocs/vuetify-delivery/manifest.json"
  ],
  "docs/vuetify-delivery/vuetify-4.2.5-complete-source.patch": [
    null,
    "100644 blob aa87d79110412e8f17477b8eac7d9d0d63865d0f\tdocs/vuetify-delivery/vuetify-4.2.5-complete-source.patch"
  ],
  "packages/utils/tests/color.spec.ts": [
    "100644 blob 0e9d9e6838ca6ee49d3d20af87903a1f91537a12\tpackages/utils/tests/color.spec.ts",
    "100644 blob 0e41bc440d46b63bbba9226f115b834f320d85d0\tpackages/utils/tests/color.spec.ts"
  ],
  "scripts/native-crash-debugger.cs": [
    null,
    "100644 blob 7aa8f5f9bb6664b9faa94968c92b059725460a92\tscripts/native-crash-debugger.cs"
  ],
  "scripts/v2-source-migration-patch.test.mjs": [
    null,
    "100644 blob f70d3170c90d5c374210428366fbee92d965e5e8\tscripts/v2-source-migration-patch.test.mjs"
  ],
  "website/docs/plugins/writing-plugins/rust-plugin.mdx": [
    "100644 blob 8a305eb9a0384a16dc87c8306cd9bcc39028fb8e\twebsite/docs/plugins/writing-plugins/rust-plugin.mdx",
    "100644 blob f5750bca3ea322155c6e59e674fadf45e0be2c10\twebsite/docs/plugins/writing-plugins/rust-plugin.mdx"
  ]
});

export function isExactArchivalPullRequest(base, event, environment, git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 })) {
  try {
    assert.equal(environment.GITHUB_EVENT_NAME, 'pull_request');
    assert.equal(environment.GITHUB_REPOSITORY, 'farm-fe/farm');
    const pr = event.pull_request;
    assert.equal(pr.number, 2368);
    assert.equal(pr.base.ref, 'main');
    assert.equal(pr.base.sha, base);
    for (const repository of [pr.base.repo, pr.head.repo]) {
      assert.equal(repository.id, 507542208);
      assert.equal(repository.full_name, 'farm-fe/farm');
    }
    assert.match(base, /^[a-f0-9]{40}$/);
    assert.match(pr.head.sha, /^[a-f0-9]{40}$/);
    assert.equal(git(['rev-parse', '--verify', `${base}^{commit}`]).trim(), base);
    const head = git(['rev-parse', '--verify', 'HEAD^{commit}']).trim();
    assert.match(head, /^[a-f0-9]{40}$/);
    git(['merge-base', '--is-ancestor', base, head]);
    git(['merge-base', '--is-ancestor', pr.head.sha, head]);
    const paths = Object.keys(archival2368).sort();
    const diff = git(['diff', '--name-only', '--no-renames', '-z', base, head]);
    assert.deepEqual(diff.split('\0'), [...paths, '']);
    for (const [index, sha] of [base, head].entries()) {
      const expected = paths.map(file => archival2368[file][index]).filter(Boolean).join('\0') + '\0';
      assert.equal(git(['ls-tree', '-r', '-z', sha, '--', ...paths]), expected);
    }
    assert.equal(git(['rev-parse', '--verify', 'HEAD^{commit}']).trim(), head);
    return true;
  } catch {
    return false;
  }
}

export function verifyPreparedRelease(directory, { readGitFile, readGitChangesets, isAncestor } = {}) {
  const candidate = JSON.parse(fs.readFileSync(path.join(directory, 'docs/releases/v2-candidate.json')));
  const record = JSON.parse(fs.readFileSync(path.join(directory, 'docs/release-evidence/frozen-stable-versions.json')));
  assert.equal(record.schemaVersion, 2);
  assert.equal(record.migration, 'already-prepared-first-stable'); assert.equal(record.applied, true);
  assert.match(record.sourceCommit, /^[a-f0-9]{40}$/);
  assert.equal(record.planSha256, crypto.createHash('sha256').update(JSON.stringify(candidate)).digest('hex'), 'Prepared release manifest changed');
  assert.ok(isAncestor(record.sourceCommit), 'Preparation source is not an ancestor');
  assert.ok(!fs.existsSync(path.join(directory, '.changeset/pre.json')), 'Still in prerelease state');
  assert.equal(fs.readdirSync(path.join(directory, '.changeset')).filter(name => name.endsWith('.md') && name !== 'README.md').length, 0, 'Unconsumed changesets');
  assert.equal(record.versions.length, candidate.npm.primary.length);
  assert.equal(new Set(record.versions.map(row => row.name)).size, record.versions.length);
  for (const pkg of candidate.npm.primary) {
    const row = record.versions.find(version => version.name === pkg.name);
    assert.equal(row?.stable, pkg.version, `Prepared version mismatch: ${pkg.name}`);
    assert.match(pkg.version, /^\d+\.\d+\.\d+$/);
    assert.equal(JSON.parse(fs.readFileSync(path.join(directory, pkg.manifestPath))).version, pkg.version);
    const expected = path.join(path.dirname(pkg.manifestPath), 'CHANGELOG.md').replaceAll('\\', '/');
    assert.equal(row.changelog?.file, expected, 'Unexpected changelog path');
    const changelog = fs.readFileSync(path.join(directory, expected), 'utf8');
    assert.equal(digest(changelog), row.changelog.sha256, `Stable changelog changed: ${pkg.name}`);
    assert.ok(changelog.split(/\r?\n/).includes(`## ${pkg.version}`), `Missing stable version notes: ${pkg.name}`);
  }
  assert.ok(record.changesets.length > 0);
  assert.equal(new Set(record.changesets).size, record.changesets.length, 'Duplicate consumed changeset');
  assert.deepEqual([...record.changesets].sort(), readGitChangesets(record.sourceCommit).sort(), 'Original changeset set is incomplete');
  assert.deepEqual(record.changesetSources.map(row => row.id).sort(), [...record.changesets].sort());
  const pre = readGitFile(record.sourceCommit, '.changeset/pre.json');
  assert.equal(digest(pre), record.sourcePreSha256, 'Original prerelease record changed');
  const original = JSON.parse(pre);
  assert.equal(original.mode, 'pre'); assert.equal(original.tag, 'beta');
  for (const id of original.changesets) assert.ok(record.changesets.includes(id), 'Consumed beta changeset omitted');
  for (const row of record.changesetSources) {
    assert.match(row.id, /^[a-z0-9-]+$/);
    assert.equal(digest(readGitFile(record.sourceCommit, `.changeset/${row.id}.md`)), row.sha256, 'Consumed source changeset changed');
  }
  return { status: 'verified-stable-preparation', versions: record.versions.length, consumed: record.changesets.length, sourceCommit: record.sourceCommit };
}
export function verifyChangesetsPreparation(directory, { readGitFile, isAncestor, generatedDiff, verifyCargoRecord } = {}) {
  const plan = JSON.parse(fs.readFileSync(path.join(directory, 'docs/releases/v2-candidate.json')));
  const record = JSON.parse(fs.readFileSync(path.join(directory, 'docs/release-evidence/changesets-preparation.json')));
  assert.equal(record.mode, 'changesets'); assert.equal(record.schemaVersion, 1);
  assert.ok(!fs.existsSync(path.join(directory, '.changeset/pre.json')));
  assert.equal(fs.readdirSync(path.join(directory, '.changeset')).filter(file => file.endsWith('.md') && file !== 'README.md').length, 0);
  assert.ok(isAncestor(record.sourceHeadSHA), 'Prepared head no longer belongs to this version PR');
  const allowed = new Set(['docs/releases/v2-candidate.json', 'docs/releases/v2-release-intent.json', 'docs/release-evidence/changesets-preparation.json', '.github/workflows/v2-stable-release.yaml', 'scripts/release-fixtures/sdk-consumer/Cargo.toml']);
  for (const row of plan.npm.primary) allowed.add(`${path.dirname(row.manifestPath).replaceAll('\\', '/')}/license-components.json`);
  assert.ok(generatedDiff(record.sourceHeadSHA).every(file => allowed.has(file)), 'Source changed after version preparation');
  const official = JSON.parse(fs.readFileSync(path.join(directory, 'docs/release-evidence/changesets-version-plan.json')));
  assert.equal(digest(JSON.stringify(official)), record.officialPlanSha256);
  assert.ok(official.changesets.length > 0 && official.releases.length > 0);
  const cargoRecord = official.changesets.some(row => row.id === cargoChangesetId)
    ? JSON.parse(fs.readFileSync(path.join(directory, 'docs/release-evidence/cargo-version-proposal.json'))) : null;
  const baseline = JSON.parse(readGitFile(record.baseSHA, 'docs/releases/v2-candidate.json'));
  if (cargoRecord) {
    assert.equal(typeof verifyCargoRecord, 'function', 'Cargo proposal verifier required');
    verifyCargoRecord(cargoRecord, baseline, record.baseSHA);
    for (const name of cargoRecord.requiredNpm) {
      const release = official.releases.find(row => row.name === name);
      assert.ok(release && release.newVersion !== baseline.npm.primary.find(row => row.name === name)?.version, `Cargo native consumer version missing: ${name}`);
    }
  }
  for (const change of official.changesets) {
    assert.match(change.id, /^[a-z0-9-]+$/);
    const original = change.id === cargoChangesetId && cargoRecord ? cargoRecord.changeset : readGitFile(record.baseSHA, `.changeset/${change.id}.md`);
    assert.equal(digest(original), record.consumed.find(row => row.id === change.id)?.sha256, 'Consumed Changeset source differs');
  }
  const expected = changesetsPublication(plan, baseline, official.releases);
  for (const row of expected.npm.primary) {
    assert.equal(JSON.parse(fs.readFileSync(path.join(directory, row.manifestPath))).version, row.version);
    const text = fs.readFileSync(path.join(directory, path.dirname(row.manifestPath), 'CHANGELOG.md'), 'utf8');
    assert.ok(text.split(/\r?\n/).includes(`## ${row.version}`), `Official changelog version missing: ${row.name}`);
  }
  assert.equal(digest(JSON.stringify(plan)), record.planSha256);
  assert.deepEqual(plan.publication.npm, expected.npm); assert.deepEqual(plan.publication.cargo, expected.cargo);
  assert.equal(plan.publication.releaseIdentity, expected.releaseIdentity);
  return { status: 'verified-changesets-preparation', packages: expected.counts.total };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const event = process.env.GITHUB_EVENT_PATH ? (process.env.GITHUB_EVENT_NAME === 'workflow_dispatch' ? validationEvent() : JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'))) : null;
  const mode = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json'))).preparation?.mode ?? 'first-stable';
  assert.ok(['first-stable', 'changesets'].includes(mode), 'Unknown version preparation mode');
  const prepared = event?.pull_request?.base?.ref === 'main' && (event.number === 2329 || event.pull_request.head.ref === 'changeset-release/main');
  const mainProof = process.env.GITHUB_EVENT_NAME === 'push' && process.env.GITHUB_REF === 'refs/heads/main'
    ? await (await import('./automatic-release-source.mjs')).resolveAutomaticRelease() : null;
  if (mainProof?.releaseReady === true) {
    assert.ok(['verified-prepared-migration','verified-version-only'].includes(mainProof.mode));
    console.log(JSON.stringify({status:'verified-main-version-consumption',sourceSHA:mainProof.sourceSHA,receiptHash:mainProof.receiptHash}));
  } else if (prepared || process.argv.includes('--verify-prepared')) {
    const result = (mode === 'changesets' ? verifyChangesetsPreparation : verifyPreparedRelease)(root, {
      readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
      readGitChangesets: sha => execFileSync('git', ['ls-tree', '--name-only', `${sha}:.changeset`], { cwd: root, encoding: 'utf8' }).split(/\r?\n/).filter(name => name.endsWith('.md') && name !== 'README.md').map(name => name.slice(0, -3)),
      isAncestor: sha => { try { execFileSync('git', ['merge-base', '--is-ancestor', sha, 'HEAD'], { cwd: root }); return true; } catch { return false; } },
      generatedDiff: sha => execFileSync('git', ['diff', '--name-only', sha, 'HEAD'], { cwd: root, encoding: 'utf8' }).trim().split(/\r?\n/).filter(Boolean),
      verifyCargoRecord: (cargoRecord, baseline, baseSHA) => {
        execFileSync('git', ['fetch', 'origin', cargoRecord.headSHA], { cwd: root, timeout: 120000 });
        const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version', '1'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 }));
        validateCargoPreparation(cargoRecord, baseline, metadata, { baseSHA,
          repository: process.env.GITHUB_REPOSITORY,
          readProducerArtifact: readCargoArtifact,
          readProducerRun: id => JSON.parse(execFileSync('gh', ['api', `repos/${process.env.GITHUB_REPOSITORY}/actions/runs/${id}`], { cwd: root, encoding: 'utf8', timeout: 30000 })),
          isAncestor: (base, head) => { execFileSync('git', ['merge-base', '--is-ancestor', base, head], { cwd: root }); return true; },
          readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
          changedFiles: (base, head) => execFileSync('git', ['diff', '--name-only', base, head], { cwd: root, encoding: 'utf8' }).trim().split(/\r?\n/).filter(Boolean),
          readCurrent: file => fs.readFileSync(path.join(root, file), 'utf8') });
      }
    });
    // Also checks all 297 source versions/dependencies and Cargo manifest digests.
    execFileSync(process.execPath, [path.join(root, 'scripts/v2-release-preflight.mjs')], { cwd: root, stdio: 'inherit' });
    console.log(JSON.stringify(result));
  } else {
    // All non-eligible changes retain the normal Changesets failure semantics.
    const pkg = JSON.parse(fs.readFileSync(path.join(root, 'node_modules/@changesets/cli/package.json')));
    const bin = typeof pkg.bin === 'string' ? pkg.bin : Object.values(pkg.bin)[0];
    const base = fetchChangesetBase(event, process.env);
    if (isNoPublishDiff(base, process.env.GITHUB_EVENT_NAME) || isExactArchivalPullRequest(base, event, process.env)) {
      console.log(JSON.stringify({ status: 'verified-no-publish-diff', base }));
    } else {
      execFileSync(process.execPath, [path.resolve(path.dirname(fs.realpathSync(path.join(root, 'node_modules/@changesets/cli/package.json'))), bin), 'status', `--since=${base}`], { cwd: root, stdio: 'inherit' });
    }
  }
}
