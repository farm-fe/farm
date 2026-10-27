// cspell:words manypkg
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

// This one-time migration consumes prerelease changesets after earlier commits
// already prepared their first stable versions. Future releases use the CLI unchanged.
export function freezeFirstStablePlan(plan, candidate, packages) {
  assert.equal(plan.preState?.mode, 'exit', 'Run the official Changesets pre exit first');
  const selected = new Map(candidate.npm.primary.map(row => [row.name, row]));
  const byName = new Map(packages.packages.map(pkg => [pkg.packageJson.name, pkg]));
  for (const release of plan.releases) {
    assert.ok(release.type === 'none' || selected.has(release.name), `Unreviewed version change: ${release.name}`);
  }
  const releases = [...selected.values()].map(row => {
    const pkg = byName.get(row.name);
    assert.ok(pkg && !pkg.packageJson.private, `Missing public candidate: ${row.name}`);
    assert.equal(pkg.packageJson.version, row.version, `Frozen version drift: ${row.name}`);
    assert.match(row.version, /^\d+\.\d+\.\d+$/, 'Only stable versions are permitted');
    const release = plan.releases.find(item => item.name === row.name);
    return { name: row.name, oldVersion: row.version, newVersion: row.version,
      type: release?.type ?? 'patch', changesets: release?.changesets ?? [] };
  });
  return { ...plan, releases };
}
export function consolidateStableChangelog(source, version) {
  if (!source.split(/\r?\n/).includes(`## ${version}`)) {
    const index = source.indexOf('\n');
    const note = `\n\n## ${version}\n\n### Stable release preparation\n\nRelease the already prepared package at the version in the reviewed V2 candidate manifest. This migration applies no additional version bump.\n`;
    source = index < 0 ? source + note : source.slice(0, index) + note + source.slice(index);
  }
  let seen = false;
  return source.split(/\r?\n/).map(line => {
    if (line !== `## ${version}`) return line;
    if (!seen) { seen = true; return line; }
    // Preserve earlier author-written release notes, without a second release identity.
    return `### Earlier ${version} preparation`;
  }).join('\n');
}

async function main() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  assert.equal(fs.realpathSync(process.cwd()), fs.realpathSync(root), 'Run only in the unified repository');
  const require = createRequire(fs.realpathSync(path.join(root, 'node_modules/@changesets/cli/package.json')));
  const packages = await require('@manypkg/get-packages').getPackages(root);
  const config = await require('@changesets/config').read(root, packages);
  const original = await require('@changesets/get-release-plan').default(root, undefined, config);
  const candidate = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const frozen = freezeFirstStablePlan(original, candidate, packages);
  const report = { schemaVersion: 1, migration: 'already-prepared-first-stable',
    changesets: original.changesets.map(item => item.id),
    versions: frozen.releases.map(row => ({ name: row.name, stable: row.newVersion,
      ordinaryCalculated: original.releases.find(item => item.name === row.name)?.newVersion ?? row.oldVersion })) };
  if (!process.argv.includes('--apply')) { console.log(JSON.stringify(report, null, 2)); return; }
  const evidence = path.resolve(process.argv[process.argv.indexOf('--apply') + 1] ?? '');
  assert.ok(evidence !== root && !fs.existsSync(evidence), 'A fresh external evidence directory is required');
  fs.mkdirSync(evidence, { recursive: true });
  fs.cpSync(path.join(root, '.changeset'), path.join(evidence, 'original-changesets'), { recursive: true });
  fs.writeFileSync(path.join(evidence, 'ordinary-plan.json'), JSON.stringify(original, null, 2));
  fs.writeFileSync(path.join(evidence, 'frozen-plan.json'), JSON.stringify(frozen, null, 2));
  await require('@changesets/apply-release-plan').default(frozen, packages, config);
  for (const row of candidate.npm.primary) {
    assert.equal(JSON.parse(fs.readFileSync(path.join(root, row.manifestPath))).version, row.version);
    const changelog = path.join(root, path.dirname(row.manifestPath), 'CHANGELOG.md');
    if (fs.existsSync(changelog)) fs.writeFileSync(changelog, consolidateStableChangelog(fs.readFileSync(changelog, 'utf8'), row.version));
  }
  assert.ok(!fs.existsSync(path.join(root, '.changeset/pre.json')), 'Stable exit was not applied');
  fs.writeFileSync(path.join(evidence, 'result.json'), JSON.stringify({ ...report, applied: true }, null, 2));
  console.log(JSON.stringify({ applied: true, packages: frozen.releases.length, changesets: original.changesets.length, evidence }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
