import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
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
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const event = process.env.GITHUB_EVENT_PATH ? JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH)) : null;
  const prepared = event?.number === 2329 && event.pull_request?.base?.ref === 'main';
  if (prepared || process.argv.includes('--verify-prepared')) {
    const result = verifyPreparedRelease(root, {
      readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
      readGitChangesets: sha => execFileSync('git', ['ls-tree', '--name-only', `${sha}:.changeset`], { cwd: root, encoding: 'utf8' }).split(/\r?\n/).filter(name => name.endsWith('.md') && name !== 'README.md').map(name => name.slice(0, -3)),
      isAncestor: sha => { try { execFileSync('git', ['merge-base', '--is-ancestor', sha, 'HEAD'], { cwd: root }); return true; } catch { return false; } }
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
