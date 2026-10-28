import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validationEvent, changesetsPublication } from './v2-release-inventory.mjs';
import { cargoChangesetId, validateCargoPreparation } from './prepare-cargo-version-proposal.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const digest = text => crypto.createHash('sha256').update(text.replaceAll('\r\n', '\n')).digest('hex');

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
  const event = process.env.GITHUB_EVENT_PATH ? validationEvent() : null;
  const mode = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json'))).preparation?.mode ?? 'first-stable';
  assert.ok(['first-stable', 'changesets'].includes(mode), 'Unknown version preparation mode');
  const prepared = event?.pull_request?.base?.ref === 'main' && (event.number === 2329 || event.pull_request.head.ref === 'changeset-release/main');
  if (prepared || process.argv.includes('--verify-prepared')) {
    const result = (mode === 'changesets' ? verifyChangesetsPreparation : verifyPreparedRelease)(root, {
      readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
      readGitChangesets: sha => execFileSync('git', ['ls-tree', '--name-only', `${sha}:.changeset`], { cwd: root, encoding: 'utf8' }).split(/\r?\n/).filter(name => name.endsWith('.md') && name !== 'README.md').map(name => name.slice(0, -3)),
      isAncestor: sha => { try { execFileSync('git', ['merge-base', '--is-ancestor', sha, 'HEAD'], { cwd: root }); return true; } catch { return false; } },
      generatedDiff: sha => execFileSync('git', ['diff', '--name-only', sha, 'HEAD'], { cwd: root, encoding: 'utf8' }).trim().split(/\r?\n/).filter(Boolean),
      verifyCargoRecord: (cargoRecord, baseline, baseSHA) => {
        execFileSync('git', ['fetch', 'origin', cargoRecord.headSHA], { cwd: root, timeout: 120000 });
        const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version', '1'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 }));
        validateCargoPreparation(cargoRecord, baseline, metadata, { baseSHA,
          readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
          changedFiles: (base, head) => execFileSync('git', ['diff', '--name-only', base, head], { cwd: root, encoding: 'utf8' }).trim().split(/\r?\n/).filter(Boolean),
          readCurrent: file => fs.readFileSync(path.join(root, file), 'utf8') });
      }
    });
    // Also checks all 297 source versions/dependencies and Cargo manifest digests.
    execFileSync(process.execPath, [path.join(root, 'scripts/v2-release-preflight.mjs')], { cwd: root, stdio: 'inherit' });
    console.log(JSON.stringify(result));
  } else {
    // All ordinary PRs keep the normal Changesets check and its failure semantics.
    const pkg = JSON.parse(fs.readFileSync(path.join(root, 'node_modules/@changesets/cli/package.json')));
    const bin = typeof pkg.bin === 'string' ? pkg.bin : Object.values(pkg.bin)[0];
    execFileSync(process.execPath, [path.resolve(path.dirname(fs.realpathSync(path.join(root, 'node_modules/@changesets/cli/package.json'))), bin), 'status', '--since=origin/main'], { cwd: root, stdio: 'inherit' });
  }
}
