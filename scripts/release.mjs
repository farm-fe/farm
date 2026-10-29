// cspell:words npmjs
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertStableReleaseState } from './check-stable-release-state.mjs';
import { archiveFiles } from './release-archive.mjs';
import { publicationNpm } from './v2-release-inventory.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
export function publicationOrder(plan, records, directory) {
  const selected = publicationNpm(plan);
  const byName = new Map(records.map(record => [record.name, record]));
  const primary = new Map(selected.primary.map(pkg => [pkg.name, pkg]));
  const ordered = selected.payloads.map(pkg => byName.get(pkg.name));
  const visiting = new Set();
  const visited = new Set();
  function visit(name) {
    if (visited.has(name)) return;
    if (visiting.has(name)) throw new Error(`Primary runtime dependency cycle requires a reviewed publication order: ${name}`);
    visiting.add(name);
    const record = byName.get(name);
    if (!record) throw new Error(`Missing sealed package: ${name}`);
    const files = archiveFiles(fs.readFileSync(path.join(directory, record.file)));
    const manifest = JSON.parse(files.get('package/package.json').toString('utf8'));
    for (const dep of Object.keys({ ...manifest.dependencies, ...manifest.optionalDependencies })) if (primary.has(dep)) visit(dep);
    visiting.delete(name); visited.add(name); ordered.push(record);
  }
  for (const name of primary.keys()) visit(name);
  if (ordered.some(record => !record) || ordered.length !== selected.primary.length + selected.payloads.length) throw new Error('Incomplete exact publication allowlist');
  return ordered;
}

export function verifyExistingNpm(record, bytes, metadata) {
  if (metadata === null) return false;
  if (metadata.name !== record.name || metadata.version !== record.version) throw new Error('Existing npm identity mismatch');
  const integrity = `sha512-${crypto.createHash('sha512').update(bytes).digest('base64')}`;
  if (metadata.dist?.integrity !== integrity) throw new Error(`Immutable npm version differs from sealed bytes: ${record.name}@${record.version}`);
  return true;
}
function registryVersion(value) {
  if (typeof value !== 'string') return null;
  const match = value.match(/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/);
  if (!match) return null;
  const core = match.slice(1, 4).map(Number), prerelease = match[4] ?? null;
  if (!core.every(Number.isSafeInteger) || prerelease?.split('.').some(part => /^\d+$/.test(part) && part.length > 1 && part.startsWith('0'))) return null;
  return { core, prerelease };
}
export function latestPublicationDecision(current, target) {
  const wanted = registryVersion(target);
  if (!wanted || wanted.prerelease) throw new Error('Stable latest target must be an exact stable version');
  if (current === null) return 'advance';
  const observed = registryVersion(current);
  if (!observed) throw new Error('Registry latest tag is not a valid version');
  for (let index = 0; index < 3; index++) {
    if (observed.core[index] > wanted.core[index]) return 'preserve-higher';
    if (observed.core[index] < wanted.core[index]) return 'advance';
  }
  // Only the target is required to be stable. A same-core prerelease is older;
  // build metadata has no precedence. No general range/parser dependency is needed.
  return observed.prerelease ? 'advance' : 'verified';
}
export function verifyLatestTarget(record, current) {
  const action = latestPublicationDecision(current, record.version);
  if (action === 'advance') throw new Error(`Registry latest tag has not reached the reviewed version: ${record.name}@${record.version}`);
  return action;
}
export async function ensureLatestTarget(record, { readLatest, writeLatest }) {
  // Read again immediately before any tag write. An already higher latest is
  // preserved; the exact old release remains addressable by its immutable version.
  const before = await readLatest(record);
  const action = latestPublicationDecision(before, record.version);
  if (action === 'preserve-higher' || action === 'verified') return { action, observed: before };
  await writeLatest(record);
  const observed = await readLatest(record);
  return { action: verifyLatestTarget(record, observed), observed };
}
export async function release({ directory = process.env.FARM_RELEASE_CANDIDATE_DIRECTORY, sdkProof = process.env.FARM_RELEASE_SDK_PROOF } = {}) {
  assertStableReleaseState(root);
  if (!directory || !sdkProof) throw new Error('Stable publication requires the sealed candidate directory and independent SDK proof; source workspace publishing is disabled.');
  const receiptPath = path.join(path.resolve(directory), 'receipt.json');
  execFileSync(process.execPath, [path.join(root, 'scripts/v2-release-preflight.mjs'), '--publication', '--candidate-seal', receiptPath, '--sdk-proof', path.resolve(sdkProof)], { cwd: root, stdio: 'inherit' });
  const receipt = JSON.parse(fs.readFileSync(receiptPath));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const npm = path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
  const ordered = publicationOrder(plan, receipt.records, path.resolve(directory));
  // Check every immutable destination before the first upload. Retrying after a
  // partial publication reuses only byte-identical registry versions.
  const existing = new Set(), states = ordered.map(record => ({ name: record.name, version: record.version, sha256: record.sha256, status: 'pending' }));
  const stateDirectory = path.join(root, '.release-publication-status');
  fs.mkdirSync(stateDirectory, { recursive: true });
  const persist = () => fs.writeFileSync(path.join(stateDirectory, 'npm.json'), JSON.stringify({ schemaVersion: 1, sourceCommit: receipt.sourceCommit, packages: states }, null, 2) + '\n');
  const registryMetadata = async (name, version) => {
    const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(name)}/${encodeURIComponent(version)}`, { signal: AbortSignal.timeout(30000) });
    if (response.status !== 404 && !response.ok) throw new Error(`npm registry check failed: ${response.status}`);
    const metadata = response.status === 404 ? null : await response.json();
    if (metadata && metadata.name !== name) throw new Error('Registry package identity mismatch');
    return metadata;
  };
  for (const record of ordered) {
    const state = states.find(row => row.name === record.name); state.status = 'preflight'; persist();
    try {
      const metadata = await registryMetadata(record.name, record.version);
      if (verifyExistingNpm(record, fs.readFileSync(path.join(path.resolve(directory), record.file)), metadata)) existing.add(record.name);
      const latest = await registryMetadata(record.name, 'latest');
      state.latestBefore = latest?.version ?? null;
      state.latestAction = latestPublicationDecision(state.latestBefore, record.version);
      state.status = existing.has(record.name) ? 'immutable-reuse-verified' : 'upload-pending'; persist();
    } catch (error) { state.status = 'failed'; state.error = error.message.slice(0, 500); persist(); throw error; }
  }
  for (const record of ordered) {
    const state = states.find(row => row.name === record.name);
    try {
      if (!existing.has(record.name)) {
        // Never let npm publish itself move latest backwards. First publish the
        // exact archive under a release-version tag, then reconcile latest.
        execFileSync(process.execPath, [npm, 'publish', path.join(path.resolve(directory), record.file), '--access', 'public', '--tag', `farm-release-${record.version}`, '--ignore-scripts'], { cwd: root, stdio: 'inherit' });
        state.status = 'uploaded'; persist();
      }
      const metadata = await registryMetadata(record.name, record.version);
      if (!verifyExistingNpm(record, fs.readFileSync(path.join(path.resolve(directory), record.file)), metadata)) throw new Error('Published immutable npm version unavailable');
      const latest = await ensureLatestTarget(record, {
        readLatest: async row => (await registryMetadata(row.name, 'latest'))?.version ?? null,
        writeLatest: row => execFileSync(process.execPath, [npm, 'dist-tag', 'add', `${row.name}@${row.version}`, 'latest'], { cwd: root, stdio: 'inherit' })
      });
      state.latestAction = latest.action; state.latestObserved = latest.observed; state.status = 'verified'; persist();
    } catch (error) { state.status = 'failed'; state.error = error.message.slice(0, 500); persist(); throw error; }
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await release();
