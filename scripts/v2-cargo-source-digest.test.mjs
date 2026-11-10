import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import childProcess, { execFileSync, spawnSync } from 'node:child_process';
import { syncBuiltinESMExports } from 'node:module';
import { cargoPackageInputsDigest, nativeSourceDigest, npmInventory, root, changesetsPublication } from './v2-release-inventory.mjs';

test('pinned Cargo oracle binds fixtures and inherited files without changing version-policy fingerprints', () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-cargo-inputs-'));
  const directory = path.join(workspace, 'member');
  try {
    fs.mkdirSync(path.join(directory, 'src'), {recursive: true});
    fs.mkdirSync(path.join(directory, 'tests'));
    fs.writeFileSync(path.join(workspace, 'Cargo.toml'), '[workspace]\nmembers=["member"]\nresolver="2"\n[workspace.package]\nreadme="README.md"\nlicense-file="LICENSE"\n');
    fs.writeFileSync(path.join(directory, 'Cargo.toml'), '[package]\nname="farm-source-fixture"\nversion="0.1.0"\nedition="2021"\nreadme.workspace=true\nlicense-file.workspace=true\n');
    fs.writeFileSync(path.join(directory, 'src/lib.rs'), 'pub fn value() {}\n');
    fs.writeFileSync(path.join(directory, 'tests/profiles.json'), '{"profile":"first"}\n');
    fs.writeFileSync(path.join(workspace, 'LICENSE'), 'fixture license\n');
    fs.writeFileSync(path.join(workspace, 'README.md'), 'fixture readme\n');
    execFileSync('cargo', ['+nightly-2026-04-10', 'generate-lockfile', '--offline'], {cwd: workspace});
    const lock = fs.readFileSync(path.join(workspace, 'Cargo.lock'));
    const baseline = cargoPackageInputsDigest(directory, workspace);
    const versionSource = nativeSourceDigest(directory);
    for (const file of ['member/tests/profiles.json', 'LICENSE', 'README.md']) {
      const target = path.join(workspace, file), original = fs.readFileSync(target);
      fs.appendFileSync(target, ' ');
      assert.notEqual(cargoPackageInputsDigest(directory, workspace), baseline, file);
      assert.equal(nativeSourceDigest(directory), versionSource, 'existing version policy remains unchanged');
      fs.writeFileSync(target, original);
      assert.equal(cargoPackageInputsDigest(directory, workspace), baseline);
    }
    const added = path.join(directory, 'tests/new-fixture.json');
    fs.writeFileSync(added, '{}');
    assert.notEqual(cargoPackageInputsDigest(directory, workspace), baseline, 'untracked package input must be bound');
    fs.unlinkSync(added);
    assert.equal(cargoPackageInputsDigest(directory, workspace), baseline);
    const existing = path.join(directory, 'tests/profiles.json');
    const bytes = fs.readFileSync(existing);
    fs.unlinkSync(existing);
    assert.notEqual(cargoPackageInputsDigest(directory, workspace), baseline, 'removed package input must change evidence');
    fs.symlinkSync('../src/lib.rs', existing);
    assert.throws(() => cargoPackageInputsDigest(directory, workspace), /symlink requires review/);
    fs.unlinkSync(existing);
    fs.writeFileSync(existing, bytes);
    assert.equal(cargoPackageInputsDigest(directory, workspace), baseline);
    assert.deepEqual(fs.readFileSync(path.join(workspace, 'Cargo.lock')), lock);
    assert.throws(() => cargoPackageInputsDigest(directory, path.join(directory, 'src')), /outside workspace/);
  } finally { fs.rmSync(workspace, {recursive: true, force: true}); }
});


test('oracle validation rejects unknown generated members and source escape', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-oracle-schema-'));
  const original = childProcess.execFileSync;
  try {
    fs.writeFileSync(path.join(directory, 'Cargo.toml'), '[package]\nname="fixture"\nversion="0.1.0"\n');
    fs.writeFileSync(path.join(directory, 'Cargo.lock'), 'fixture lock');
    for (const [caseName, expected] of [['generated', /Unknown generated/], ['kind', /Unknown Cargo package input kind/], ['outside', /outside workspace/], ['wrong-lock', /did not isolate/]]) {
      childProcess.execFileSync = (command, args) => {
        assert.equal(command, 'cargo');
        const config = args.find(value => value.startsWith('resolver.lockfile-path='));
        const isolated = JSON.parse(config.slice(config.indexOf('=') + 1));
        const files = {
          'Cargo.toml': {kind: 'generate', path: path.join(directory, 'Cargo.toml')},
          'Cargo.toml.orig': {kind: 'copy', path: path.join(directory, 'Cargo.toml')},
          'Cargo.lock': {kind: 'generate', path: isolated},
        };
        if (caseName === 'generated') files['future-generated'] = {kind: 'generate', path: path.join(directory, 'Cargo.toml')};
        if (caseName === 'kind') files['future-copy'] = {kind: 'unknown', path: path.join(directory, 'Cargo.toml')};
        if (caseName === 'outside') files['outside'] = {kind: 'copy', path: path.join(directory, '..', 'outside')};
        if (caseName === 'wrong-lock') files['Cargo.lock'].path = path.join(directory, 'Cargo.lock');
        return JSON.stringify({files});
      };
      syncBuiltinESMExports();
      assert.throws(() => cargoPackageInputsDigest(directory, directory), expected);
    }
  } finally {
    childProcess.execFileSync = original;
    syncBuiltinESMExports();
    fs.rmSync(directory, {recursive: true, force: true});
  }
});

test('actual preflight rejects missing or tampered package-input evidence in an owned candidate fixture', () => {
  const folder = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-candidate-inputs-'));
  try {
    const checkedIn = path.join(root, 'docs/releases/v2-candidate.json');
    const original = JSON.parse(fs.readFileSync(checkedIn));
    const candidate = structuredClone(original);
    candidate.npm = npmInventory();
    for (const row of candidate.cargo) {
      const directory = path.dirname(path.join(root, row.manifestPath));
      row.sourceSha256 = nativeSourceDigest(directory);
      row.packageInputsSha256 = cargoPackageInputsDigest(directory);
    }
    const identity = plan => [...plan.npm.primary, ...plan.npm.payloads, ...plan.cargo].map(({name, version}) => [name, version]);
    assert.deepEqual(identity(candidate), identity(original));
    const program = `import fs from 'node:fs';
      const read = fs.readFileSync;
      const fixture = read(process.argv[1]);
      fs.readFileSync = (file, ...args) => String(file) === ${JSON.stringify(checkedIn)} ? fixture : read(file, ...args);
      await import(${JSON.stringify(new URL('./v2-release-preflight.mjs', import.meta.url).href)});`;
    for (const mode of ['valid', 'missing', 'tampered']) {
      const value = structuredClone(candidate);
      if (mode === 'missing') delete value.cargo[0].packageInputsSha256;
      if (mode === 'tampered') value.cargo[0].packageInputsSha256 = '0'.repeat(64);
      const file = path.join(folder, mode + '.json'); fs.writeFileSync(file, JSON.stringify(value));
      const env = {...process.env}; delete env.FARM_RELEASE_CANDIDATE_PLAN;
      const result = spawnSync(process.execPath, ['--input-type=module', '--eval', program, file], {cwd: root, encoding: 'utf8', env});
      if (mode === 'valid') {
        assert.equal(result.status, 0, result.stderr);
        const report = JSON.parse(result.stdout); assert.equal(report.sourceInventory, 'consistent'); assert.equal(report.readyToPublish, false);
      } else {
        assert.equal(result.status, 1, result.stdout);
        assert.match(result.stderr, /Cargo candidate package inputs drifted/);
        assert.doesNotMatch(result.stdout, /"sourceInventory": "consistent"/);
      }
    }
    assert.deepEqual(JSON.parse(fs.readFileSync(checkedIn)), original, 'checked-in candidate was never refreshed');
    const baseline = structuredClone(original), family = structuredClone(original);
    for (const row of family.cargo) row.packageInputsSha256 = 'f'.repeat(64);
    const changed = family.npm.primary.find(row => row.name === '@farmfe/js-plugin-vuetify');
    changed.version = '0.0.3';
    const selection = changesetsPublication(family, baseline, [{name: changed.name, newVersion: changed.version}]);
    assert.equal(selection.cargo.length, 0, 'package completeness does not alter existing version policy');
    assert.deepEqual(selection.npm.primary.map(row => row.name), [changed.name]);
  } finally { fs.rmSync(folder, {recursive: true, force: true}); }
});
