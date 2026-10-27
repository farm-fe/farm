import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { verifyPreparedRelease } from './check-pr-changesets.mjs';
const digest = text => crypto.createHash('sha256').update(text).digest('hex');
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
  assert.deepEqual(verifyPreparedRelease(root, options), { status: 'verified-stable-preparation', versions: 1, consumed: 1, sourceCommit: 'a'.repeat(40) });
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
