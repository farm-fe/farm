import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const sourceCommit = process.argv[2];
if (!/^[a-f0-9]{40}$/.test(sourceCommit ?? '')) throw new Error('Expected the pre-preparation source commit');
execFileSync('git', ['merge-base', '--is-ancestor', sourceCommit, 'HEAD'], { cwd: root });
const hash = text => crypto.createHash('sha256').update(text.replaceAll('\r\n', '\n')).digest('hex');
const file = path.join(root, 'docs/release-evidence/frozen-stable-versions.json');
const record = JSON.parse(fs.readFileSync(file));
const candidate = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
record.schemaVersion = 2;
record.sourceCommit = sourceCommit;
record.planSha256 = crypto.createHash('sha256').update(JSON.stringify(candidate)).digest('hex');
record.changesetSources = record.changesets.map(id => {
  if (!/^[a-z0-9-]+$/.test(id)) throw new Error('Unexpected changeset ID');
  const text = execFileSync('git', ['show', `${sourceCommit}:.changeset/${id}.md`], { cwd: root, encoding: 'utf8' });
  return { id, sha256: hash(text) };
});
const pre = execFileSync('git', ['show', `${sourceCommit}:.changeset/pre.json`], { cwd: root, encoding: 'utf8' });
record.sourcePreSha256 = hash(pre);
for (const version of record.versions) {
  const pkg = candidate.npm.primary.find(row => row.name === version.name);
  if (!pkg || pkg.version !== version.stable) throw new Error('Prepared version drift');
  const changelog = path.join(path.dirname(pkg.manifestPath), 'CHANGELOG.md').replaceAll('\\', '/');
  if (!fs.existsSync(path.join(root, changelog))) throw new Error(`Missing stable changelog: ${pkg.name}`);
  version.changelog = { file: changelog, sha256: hash(fs.readFileSync(path.join(root, changelog), 'utf8')) };
}
fs.writeFileSync(file, JSON.stringify(record, null, 2) + '\n');
console.log(JSON.stringify({ schemaVersion: record.schemaVersion, sourceCommit, changesets: record.changesets.length, versions: record.versions.length }));
