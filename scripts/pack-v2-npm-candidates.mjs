// cspell:words statfs bavail
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertReviewedCandidatePackHooks } from './candidate-pack-hooks.mjs';
import { archiveFiles, inspectNpmArchive, sha256, nativePlatformMatches } from './release-archive.mjs';
import { verifyWorkspaceConversion, verifyLicenseComponents } from './v2-normal-pack-contract.mjs';
import { verifyCandidateSeal } from './verify-v2-candidate-seal.mjs';
import { publicationNpm, validationEvent } from './v2-release-inventory.mjs';
import { resolvePnpmCli, probePnpmCli } from './v2-pnpm-cli.mjs';

export function assertSourceLicense(directory, manifest) {
  verifyLicenseComponents(directory, manifest, fileURLToPath(new URL('../', import.meta.url)));
  const text = fs.readFileSync(path.join(directory, 'LICENSE'), 'utf8');
  assert.ok(text.trim(), `Missing license text: ${manifest.name}`);
  assert.ok(typeof manifest.license === 'string', `Missing license declaration: ${manifest.name}`);
  if (manifest.license === 'MIT') assert.match(text, /MIT License|Permission is hereby granted, free of charge/);
  if (manifest.license === 'ISC') assert.ok(!text.includes('MIT License') && /permission to use, copy, modify, and\/or distribute/i.test(text), `ISC declaration differs from legal text: ${manifest.name}`);
}
export function assertNativeAssembly(plan, assembly, artifactIndex, sourceSHA) {
  const required = publicationNpm(plan).payloads;
  assert.equal(assembly.sourceCommit, sourceSHA); assert.equal(assembly.count, required.length);
  assert.equal(new Set(assembly.records.map(row => row.name)).size, assembly.count);
  for (const pkg of required) {
    const row = assembly.records.find(item => item.name === pkg.name);
    assert.ok(row && row.version === pkg.version && row.platform === pkg.platform, `Missing exact native cohort: ${pkg.name}`);
    const artifacts = artifactIndex.filter(item => item.name === row.artifact && !item.expired);
    assert.equal(artifacts.length, 1, `Ambiguous native artifact: ${pkg.name}`);
    assert.ok(Number.isSafeInteger(artifacts[0].id) && artifacts[0].id > 0);
  }
}
export function packNpmCandidates({ root, plan, assembly, artifactIndex, sourceCommit, testedCommit, testedTree, runId, output, pnpm }) {
  const selected = publicationNpm(plan);
  assert.match(sourceCommit, /^[a-f0-9]{40}$/); assert.match(testedCommit, /^[a-f0-9]{40}$/); assert.match(testedTree, /^[a-f0-9]{40}$/);
  assert.ok(Number.isSafeInteger(runId) && runId > 0);
  pnpm = resolvePnpmCli({ repositoryRoot: root, entry: pnpm });
  assertNativeAssembly(plan, assembly, artifactIndex, testedCommit);
  fs.mkdirSync(output); // Preserve partial runs; never overwrite evidence.
  const receipt = { schemaVersion: 1, sourceCommit, testedCommit, testedTree, planSha256: sha256(Buffer.from(JSON.stringify(plan))), published: false, status: 'incomplete', records: [], errors: [] };
  const save = () => fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2));
  const guard = () => { const disk = fs.statfsSync(output); assert.ok(os.freemem() >= 3.4 * 1024 ** 3 && disk.bavail * disk.bsize >= 8 * 1024 ** 3, 'Candidate pack resource guard'); };
  function legalCopy(directory, destination) {
    for (const name of ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'licenses', 'license-components.json']) {
      const source = path.join(directory, name);
      if (fs.existsSync(source)) fs.cpSync(source, path.join(destination, name), { recursive: true, filter: file => { assert.ok(!fs.lstatSync(file).isSymbolicLink(), 'Legal source symlink'); return true; } });
    }
  }
  function pack(directory, pkg, extra = {}) {
    guard();
    const stage = path.join(output, `pack-${receipt.records.length}`); fs.mkdirSync(stage);
    const log = path.join(stage, 'pack.log'); const fd = fs.openSync(log, 'wx');
    try { execFileSync(process.execPath, [pnpm, 'pack', '--pack-destination', stage], { cwd: directory, stdio: ['ignore', fd, fd], timeout: 120000, windowsHide: true }); }
    finally { fs.closeSync(fd); }
    const files = fs.readdirSync(stage).filter(file => file.endsWith('.tgz')); assert.equal(files.length, 1);
    const source = path.join(stage, files[0]), bytes = fs.readFileSync(source);
    const manifest = JSON.parse(archiveFiles(bytes).get('package/package.json'));
    assert.equal(manifest.name, pkg.name); assert.equal(manifest.version, pkg.version);
    const inspection = inspectNpmArchive(bytes, pkg);
    assert.deepEqual(inspection.issues, [], `Packed candidate contract failed: ${pkg.name}`);
    verifyWorkspaceConversion(JSON.parse(fs.readFileSync(path.join(directory, 'package.json'))), manifest,
      new Map([...plan.npm.primary, ...plan.npm.payloads].map(item => [item.name, item.version])));
    const file = `${pkg.name.replaceAll('@', '').replaceAll('/', '-')}-${pkg.version}.tgz`;
    fs.copyFileSync(source, path.join(output, file), fs.constants.COPYFILE_EXCL);
    receipt.records.push({ name: pkg.name, version: pkg.version, file, sha256: sha256(bytes), ...extra }); save();
  }
  try {
    // Fail all source/legal/native prerequisites before starting normal packing.
    for (const pkg of selected.primary) {
      const directory = path.dirname(path.join(root, pkg.manifestPath));
      const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'package.json')));
      assertSourceLicense(directory, manifest); assertReviewedCandidatePackHooks(directory, manifest);
    }
    for (const pkg of selected.payloads) {
      const row = assembly.records.find(item => item.name === pkg.name);
      const bytes = fs.readFileSync(path.join(root, row.file));
      assert.equal(sha256(bytes), row.sha256); assert.ok(nativePlatformMatches(bytes, pkg.platform), `Wrong platform: ${pkg.name}`);
    }
    for (const pkg of selected.primary) pack(path.dirname(path.join(root, pkg.manifestPath)), pkg);
    for (const pkg of selected.payloads) {
      const row = assembly.records.find(item => item.name === pkg.name);
      const owner = plan.npm.primary.find(item => item.name === pkg.owner);
      const directory = path.dirname(path.join(root, owner.manifestPath));
      const manifest = JSON.parse(fs.readFileSync(path.join(root, pkg.manifestPath)));
      assert.equal(manifest.name, pkg.name); assert.equal(path.basename(manifest.main), manifest.main);
      manifest.version = pkg.version;
      if (pkg.platform.endsWith('-gnu')) manifest.libc = ['glibc'];
      if (pkg.platform.endsWith('-musl')) manifest.libc = ['musl'];
      const stage = path.join(output, `platform-${receipt.records.length}`); fs.mkdirSync(stage);
      legalCopy(directory, stage);
      manifest.files = [...new Set([...(manifest.files ?? []), manifest.main, 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'licenses', 'license-components.json'])];
      fs.copyFileSync(path.join(root, row.file), path.join(stage, manifest.main));
      fs.writeFileSync(path.join(stage, 'package.json'), JSON.stringify(manifest, null, 2));
      const artifact = artifactIndex.find(item => item.name === row.artifact && !item.expired);
      pack(stage, pkg, { nativeSha256: row.sha256, ...(pkg.owner === 'create-farm' ? { nativeInterface: 'node-api-4' } : { abi: plan.abi ?? '0.5.4' }), producer: { kind: 'github-actions', sourceCommit: testedCommit, runId, artifactId: artifact.id } });
    }
    const verified = verifyCandidateSeal(plan, receipt, output, root);
    fs.writeFileSync(path.join(output, 'verification.json'), JSON.stringify(verified, null, 2));
    assert.equal(verified.candidateArchivesVerified, true, 'Candidate archive/legal/native verification failed');
    receipt.status = 'sealed';
  } catch (error) { receipt.errors.push(error.message); save(); throw error; }
  save(); return receipt;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [assemblyFile, artifactFile, destination] = process.argv.slice(2);
  if (assemblyFile === '--check-package-manager' && process.argv.length === 3) console.log(JSON.stringify(probePnpmCli()));
  else {
  assert.ok(assemblyFile && artifactFile && destination, 'Expected assembly/index/output');
  const pnpm = resolvePnpmCli();
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const event = validationEvent();
  const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  const testedCommit = git('rev-parse', 'HEAD'); assert.equal(testedCommit, process.env.GITHUB_SHA);
  const testedTree = git('rev-parse', 'HEAD^{tree}'); assert.equal(testedTree, git('rev-parse', `${event.pull_request.head.sha}^{tree}`));
  const pages = JSON.parse(fs.readFileSync(artifactFile));
  const artifactIndex = Array.isArray(pages) ? pages.flatMap(page => page.artifacts) : pages.artifacts;
  packNpmCandidates({ root, plan, assembly: JSON.parse(fs.readFileSync(assemblyFile)), artifactIndex, sourceCommit: event.pull_request.head.sha, testedCommit, testedTree, runId: Number(process.env.GITHUB_RUN_ID), output: path.resolve(destination), pnpm });
  }
}
