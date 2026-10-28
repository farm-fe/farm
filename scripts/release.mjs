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
  const existing = new Set();
  for (const record of ordered) {
    const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(record.name)}/${encodeURIComponent(record.version)}`, { signal: AbortSignal.timeout(30000) });
    if (response.status !== 404 && !response.ok) throw new Error(`npm registry check failed: ${response.status}`);
    const metadata = response.status === 404 ? null : await response.json();
    if (verifyExistingNpm(record, fs.readFileSync(path.join(path.resolve(directory), record.file)), metadata)) existing.add(record.name);
  }
  for (const record of ordered) {
    if (existing.has(record.name)) continue;
    // The exact reviewed archive is published; source prepublish hooks cannot rebuild it.
    execFileSync(process.execPath, [npm, 'publish', path.join(path.resolve(directory), record.file), '--access', 'public', '--tag', 'latest', '--ignore-scripts'], { cwd: root, stdio: 'inherit' });
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await release();
