import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const suitePolicy = Object.freeze({
  schemaVersion: 1,
  protocol: 'farm-suite-discovery-v1',
  vitestVersion: '2.0.4',
  inputs: Object.freeze({
    'vitest.config.ts': 'ee2ef1a2efbdefbe54e83a6196375d843dc975af46fbf19155c6aa3650aea2ff',
    'scripts/build.mjs': 'a6d70ae72dc4beb58382145e62cd89992ea249f0c9b6f591a0dcb57ddf0d256f',
    'scripts/test-examples.mjs': 'b7f630a19fb96762d60e5a8086bcd8544fe466dc43c426e7acaae5b50a5e880d',
    'scripts/test-e2e.mjs': '8d8633626f881329ea8cd3ee991bb1b0dd0fdcf4185fdab4242094bba04e287a',
    'scripts/test-e2e-worker.mjs': 'fbda6480e244ee079e9812a7409eee180c360845b6d7c1c3454b8a734a9908f8',
    'scripts/check-e2e-specs.mjs': 'eda8df47273fdc6882dc271400e5b29f08efc1ca31655884f5b0649ab91c8ab2',
    'scripts/test-create-farm-candidates.mjs': '104791a813dde8e38cb3ed2a6b113a43513d5ee46d8bb075892c373ae92d8d0a'
  })
});

export const suiteKinds = Object.freeze(['ts', 'examples-build', 'examples-browser', 'create-farm']);
export const runtimePrerequisites = Object.freeze(['packages/runtime', 'packages/runtime-plugin-hmr', 'packages/runtime-plugin-import-meta']);
const templates = ['react', 'vue3', 'solid', 'svelte', 'lit', 'vanilla', 'preact', 'vue2'];
const exclusions = new Set(['node_modules', 'dist', 'cypress', '.idea', '.git', '.cache', '.output', '.temp']);
const configNames = /^(karma|rollup|webpack|vite|vitest|jest|ava|babel|nyc|cypress|tsup|build|eslint|prettier)\.config\./;
const digest = value => crypto.createHash('sha256').update(typeof value === 'string' ? value.replaceAll('\r\n', '\n') : JSON.stringify(value)).digest('hex');
const unique = values => [...new Set(values)].sort();
const object = (files, file) => file in files ? JSON.parse(files[file]) : null;

export function canonicalSuiteFile(file) {
  assert.equal(typeof file, 'string');
  assert.ok(file && !file.startsWith('/') && !/[\\:\u0000-\u001f\u007f]/.test(file), 'Unsafe suite path');
  assert.equal(file.normalize('NFC'), file, 'Suite path normalization differs');
  assert.ok(file.split('/').every(part => part && part !== '.' && part !== '..'), 'Non-canonical suite path');
  return file;
}

// The include/default-exclude interpretation is tied to the reviewed config and
// locked Vitest version above. A policy change makes discovery incomplete.
export function discoverSuiteInventory(files, policy = suitePolicy, sourceIdentity = null, exampleDirectories = []) {
  assert.ok(files && typeof files === 'object');
  const paths = Object.keys(files).sort(); paths.forEach(canonicalSuiteFile);
  assert.ok(paths.length <= 30000, 'Suite discovery exceeds bound');
  assert.ok(paths.every(file => typeof files[file] === 'string') && paths.reduce((total, file) => total + Buffer.byteLength(files[file]), 0) <= 32 * 1024 ** 2, 'Suite discovery bytes exceed bound');
  const fullOnlyReasons = [], items = [], excluded = [], owners = new Map();
  const policyInputs = Object.fromEntries(Object.keys(policy.inputs).sort().map(file => [file, file in files ? digest(files[file]) : null]));
  for (const [file, hash] of Object.entries(policy.inputs)) {
    if (policyInputs[file] !== hash) fullOnlyReasons.push({ code: 'unreviewed-discovery-input', file });
  }
  if (object(files, 'package.json')?.devDependencies?.vitest !== policy.vitestVersion) fullOnlyReasons.push({ code: 'unreviewed-vitest-version' });
  for (const file of paths.filter(file => /^(packages|js-plugins|examples|rust-plugins)\/[^/]+\/package\.json$/.test(file))) {
    const pkg = object(files, file), directory = file.slice(0, -'/package.json'.length);
    if (!pkg?.name) { fullOnlyReasons.push({ code: 'owner-name-missing', file }); continue; }
    owners.set(directory, { manifest: file, name: pkg.name });
  }
  const byName = new Map();
  for (const [directory, owner] of owners) byName.set(owner.name, [...(byName.get(owner.name) ?? []), directory]);
  for (const [name, directories] of byName) if (directories.length > 1) fullOnlyReasons.push({ code: 'ambiguous-package-name', name, directories: directories.sort() });
  const ownerFor = file => owners.get(file.split('/').slice(0, 2).join('/')) ?? null;
  for (const file of paths.filter(file => file.endsWith('.spec.ts'))) {
    const parts = file.split('/'), reason = !/^(packages|js-plugins)\/.*\.spec\.ts$/.test(file) ? 'outside-vitest-includes'
      : parts.some(part => exclusions.has(part)) || configNames.test(parts.at(-1)) ? 'vitest-default-exclude' : null;
    if (reason) { excluded.push({ kind: 'ts', file, reason }); continue; }
    const owner = ownerFor(file);
    if (!owner) fullOnlyReasons.push({ code: 'spec-owner-missing', file });
    items.push({ id: 'ts:' + file, kind: 'ts', file, owner, sourceDigest: digest(files[file]), requiresActualCases: true });
  }
  assert.ok(Array.isArray(exampleDirectories) && exampleDirectories.every(name => typeof name === 'string' && !name.includes('/') && canonicalSuiteFile('examples/' + name)), 'Invalid Examples directory inventory');
  const directories = unique([...exampleDirectories, ...paths.filter(file => file.startsWith('examples/') && file.split('/').length >= 3).map(file => file.split('/')[1])]);
  const docs = object(files, 'packages/docs/package.json');
  for (const name of directories) {
    const directory = 'examples/' + name, manifest = directory + '/package.json', pkg = object(files, manifest);
    const dependencies = { ...pkg?.dependencies, ...pkg?.devDependencies }, native = [];
    if (name.startsWith('rust-plugin-')) native.push(name.slice('rust-plugin-'.length));
    for (const dep of Object.keys(dependencies)) if (dep.startsWith('@farmfe/plugin-') && 'rust-plugins/' + dep.slice('@farmfe/plugin-'.length) + '/package.json' in files) native.push(dep.slice('@farmfe/plugin-'.length));
    const publicPrerequisites = [];
    if (dependencies['@farmfe/docs']) {
      if (!docs?.dependencies) fullOnlyReasons.push({ code: 'docs-prerequisites-unavailable', file: manifest });
      for (const dep of Object.keys(docs?.dependencies ?? {})) if (dep.startsWith('@farmfe/plugin-')) native.push(dep.slice('@farmfe/plugin-'.length));
      publicPrerequisites.push('docs', 'docs-theme-default');
    }
    const prerequisites = { runtime: [...runtimePrerequisites], native: unique(native), public: publicPrerequisites, dependencyProofComplete: false };
    if (pkg) {
      if (typeof pkg.scripts?.build !== 'string') fullOnlyReasons.push({ code: 'example-build-script-missing', file: manifest });
      items.push({ id: 'examples-build:' + directory, kind: 'examples-build', directory, owner: ownerFor(manifest), prerequisites, command: ['npm', 'run', 'build'] });
    } else excluded.push({ kind: 'examples-build', directory, reason: 'build-runner-skips-missing-manifest', prerequisites });
    const spec = directory + '/e2e.spec.mjs', html = directory + '/index.html';
    const browser = spec in files ? 'explicit-spec' : html in files && !['issues1433', 'nestjs'].includes(name) ? 'default-start-preview-smoke' : null;
    if (browser) {
      if (!ownerFor(manifest)) fullOnlyReasons.push({ code: 'browser-owner-missing', file: spec in files ? spec : html });
      if (spec in files && !/\b(startAndTest|watchAndTest)\s*\(|from\s+['"]playwright-chromium['"]/.test(files[spec])) fullOnlyReasons.push({ code: 'browser-api-contract-missing', file: spec });
      items.push({ id: 'examples-browser:' + directory, kind: 'examples-browser', directory, file: spec in files ? spec : html, owner: ownerFor(manifest), mode: browser, prerequisites, requiresActualCases: true });
    } else excluded.push({ kind: 'examples-browser', directory, reason: ['issues1433', 'nestjs'].includes(name) ? 'default-excluded-without-explicit-spec' : 'no-spec-or-html-entry' });
  }
  for (const template of templates) items.push({ id: 'create-farm:' + template, kind: 'create-farm', template, runner: 'scripts/test-create-farm-candidates.mjs', requiresActualCases: true });
  const fileEnumerationComplete = !fullOnlyReasons.some(row => row.code.startsWith('unreviewed-'));
  const ownerMappingComplete = fileEnumerationComplete && fullOnlyReasons.length === 0;
  const result = { schemaVersion: 1, protocol: policy.protocol, mode: 'discovery-only', executionEnabled: false, selectionReady: false,
    fileEnumerationComplete, ownerMappingComplete, dependencyMappingComplete: false, discoveryComplete: false, fullOnlyReasons, policyInputs,
    items: items.sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0), excluded, sourceIdentity,
    discoveryInputsHash: digest(Object.fromEntries(paths.map(file => [file, digest(files[file])]))), sourceDigestSemantics: 'LF-normalized source text, not archive bytes',
    note: 'File-level discovery does not prove case execution, impact authorization or complete dynamic dependency closure.' };
  assert.equal(new Set(result.items.map(row => row.id)).size, result.items.length, 'Duplicate suite identity');
  return { ...result, inventoryHash: digest(result) };
}

export function readGitSuiteFiles(root, readGit = (args, options = {}) => execFileSync('git', args, { cwd: root, timeout: 60000, maxBuffer: 32 * 1024 ** 2, ...options })) {
  const commit = readGit(['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  const tree = readGit(['rev-parse', 'HEAD^{tree}'], { encoding: 'utf8' }).trim();
  assert.match(commit, /^[a-f0-9]{40}$/); assert.match(tree, /^[a-f0-9]{40}$/);
  assert.equal(readGit(['rev-parse', commit + '^{tree}'], { encoding: 'utf8' }).trim(), tree, 'Captured commit and HEAD tree differ');
  const metadata = readGit(['ls-tree', '-r', '-z', commit]);
  assert.ok(Buffer.isBuffer(metadata), 'Discovery metadata must be bytes');
  const entries = new TextDecoder('utf-8', { fatal: true }).decode(metadata).split('\0').filter(Boolean).map(entry => {
    const split = entry.indexOf('\t'); assert.ok(split > 0, 'Invalid discovery metadata');
    const [mode, type, oid] = entry.slice(0, split).split(' '), file = entry.slice(split + 1);
    return { mode, type, oid, file };
  });
  assert.ok(entries.length <= 30000, 'Discovery metadata exceeds bound');
  const spelling = new Map();
  for (const row of entries.filter(row => /^(packages|js-plugins|examples|rust-plugins|e2e)\//.test(row.file) || row.file in suitePolicy.inputs || row.file === 'package.json')) {
    canonicalSuiteFile(row.file);
    assert.ok(row.type === 'blob' && ['100644', '100755'].includes(row.mode), 'Relevant discovery symlink or submodule rejected');
    const parts = row.file.split('/');
    for (let index = 1; index <= parts.length; index++) {
      const prefix = parts.slice(0, index).join('/'), key = prefix.toLowerCase();
      assert.ok(!spelling.has(key) || spelling.get(key) === prefix, 'Discovery path case collision'); spelling.set(key, prefix);
    }
  }
  const exampleDirectories = unique(entries.filter(row => row.file.startsWith('examples/') && row.file.split('/').length >= 3).map(row => row.file.split('/')[1]));
  const records = entries.filter(row => row.file === 'package.json' || row.file in suitePolicy.inputs || row.file.endsWith('.spec.ts')
    || /^(packages|js-plugins|examples|rust-plugins)\/[^/]+\/package\.json$/.test(row.file)
    || /^examples\/[^/]+\/(index\.html|e2e\.spec\.mjs)$/.test(row.file) || row.file.startsWith('e2e/') && /\.(mjs|cjs|js|ts|json)$/.test(row.file));
  assert.ok(records.length > 0 && records.length <= 5000, 'Discovery file count exceeds bound');
  for (const row of records) {
    canonicalSuiteFile(row.file);
    assert.ok(row.type === 'blob' && ['100644', '100755'].includes(row.mode) && /^[a-f0-9]{40}$/.test(row.oid), 'Discovery requires ordinary committed files');
  }
  const data = readGit(['cat-file', '--batch'], { input: records.map(row => row.oid + '\n').join('') });
  assert.ok(Buffer.isBuffer(data) && data.length <= 32 * 1024 ** 2, 'Discovery bytes exceed bound');
  const files = {}; let cursor = 0;
  for (const row of records) {
    const end = data.indexOf(10, cursor), header = data.subarray(cursor, end).toString('ascii').split(' ');
    assert.ok(end >= cursor && header[0] === row.oid && header[1] === 'blob' && /^\d+$/.test(header[2]), 'Discovery blob identity differs');
    const length = Number(header[2]), start = end + 1;
    assert.ok(Number.isSafeInteger(length) && length >= 0 && start + length < data.length && data[start + length] === 10, 'Incomplete discovery blob');
    files[row.file] = new TextDecoder('utf-8', { fatal: true }).decode(data.subarray(start, start + length));
    cursor = start + length + 1;
  }
  assert.equal(cursor, data.length, 'Unexpected discovery bytes');
  assert.equal(readGit(['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), commit, 'Discovery HEAD moved during capture');
  return { files, exampleDirectories, sourceIdentity: { commit, tree, trackedInputsOnly: true } };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const outputIndex = process.argv.indexOf('--output');
  assert.ok(outputIndex >= 0 && process.argv[outputIndex + 1], 'Inventory output required');
  const root = fileURLToPath(new URL('../', import.meta.url));
  const { files, sourceIdentity, exampleDirectories } = readGitSuiteFiles(root);
  const inventory = discoverSuiteInventory(files, suitePolicy, sourceIdentity, exampleDirectories);
  const output = process.argv[outputIndex + 1]; fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(inventory, null, 2) + '\n');
  console.log(JSON.stringify({ protocol: inventory.protocol, fileEnumerationComplete: inventory.fileEnumerationComplete, ownerMappingComplete: inventory.ownerMappingComplete, dependencyMappingComplete: false, discoveryComplete: false, selectionReady: false, executionEnabled: false, items: inventory.items.length, fullOnlyReasons: inventory.fullOnlyReasons.map(row => row.code) }));
}

export function makeSuiteExecutionPlan(inventory, kind, selection = { mode: 'all' }) {
  assert.ok(suiteKinds.includes(kind), 'Unknown suite kind');
  assert.equal(inventory.inventoryHash, digest(Object.fromEntries(Object.entries(inventory).filter(([key]) => key !== 'inventoryHash'))), 'Inventory digest differs');
  assert.equal(inventory.fileEnumerationComplete, true, 'Discovery file enumeration incomplete');
  const known = inventory.items.filter(row => row.kind === kind).map(row => row.id);
  assert.ok(['all', 'selected'].includes(selection.mode), 'Unknown selection mode');
  if (selection.mode === 'selected') {
    assert.equal(inventory.fullOnlyReasons.length, 0, 'Ambiguous or dynamic selection requires all suites');
    assert.equal(inventory.selectionReady, true, 'Selective execution is not authorized');
  }
  const ids = selection.mode === 'all' ? known : selection.ids;
  assert.ok(Array.isArray(ids) && ids.length > 0 && new Set(ids).size === ids.length && ids.every(id => known.includes(id)), 'Empty duplicate or unknown suite selection');
  const result = { schemaVersion: 1, protocol: inventory.protocol, inventoryHash: inventory.inventoryHash, kind, mode: selection.mode, ids: [...ids].sort(), executionEnabled: false };
  return { ...result, planHash: digest(result) };
}

export function validateSuiteExecution(inventory, plan, results) {
  const expected = makeSuiteExecutionPlan(inventory, plan.kind, { mode: plan.mode, ids: plan.ids });
  assert.deepEqual(plan, expected, 'Suite plan differs from discovered selection');
  assert.ok(Array.isArray(results) && results.length === plan.ids.length, 'Missing suite terminal result');
  assert.equal(new Set(results.map(row => row.id)).size, results.length, 'Duplicate suite terminal result');
  for (const row of results) {
    assert.ok(plan.ids.includes(row.id), 'Unexpected suite terminal result');
    assert.equal(row.inventoryHash, inventory.inventoryHash); assert.equal(row.planHash, plan.planHash);
    assert.equal(row.status, 'executed-success'); assert.equal(row.skipped, false);
    const item = inventory.items.find(item => item.id === row.id);
    if (item.requiresActualCases) assert.ok(Number.isSafeInteger(row.executedCases) && row.executedCases > 0, 'No-tests success is not execution evidence');
  }
  return { coverageVerified: true, impactAuthorizationVerified: false, publicationAuthorized: false, skipAuthorized: false, reuseAuthorized: false };
}
