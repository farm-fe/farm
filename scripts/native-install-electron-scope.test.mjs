import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { parse } from 'yaml';

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const native = parse(read('.github/workflows/rust-build-user.yaml'));
const primary = parse(read('.github/workflows/ci-primary-platform.yaml'));
const key = 'ELECTRON_SKIP_BINARY_DOWNLOAD';

test('only the native dependency install skips the unrelated Electron binary', () => {
  assert.equal(native.env?.[key], undefined);
  const job = native.jobs.build;
  assert.equal(job.env?.[key], undefined);
  const installs = job.steps.filter((step) => step.name === 'Install Dependencies');
  assert.equal(installs.length, 1);
  assert.equal(installs[0].env?.[key], '1');
  assert.equal(installs[0].with.command, 'pnpm i --frozen-lockfile');
  for (const step of job.steps.filter((step) => step !== installs[0])) {
    assert.equal(step.env?.[key], undefined);
    assert.doesNotMatch(step.run ?? '', /ELECTRON_SKIP_BINARY_DOWNLOAD|test-examples|test-e2e|electron-forge/);
  }
  assert.doesNotMatch(read('.github/workflows/rust-build-release.yaml'), /ELECTRON_SKIP_BINARY_DOWNLOAD/);
});

test('Examples installs its own dependencies before building the real Electron example', () => {
  assert.doesNotMatch(read('.github/workflows/ci-primary-platform.yaml'), /ELECTRON_SKIP_BINARY_DOWNLOAD/);
  const steps = primary.jobs['examples-test'].steps;
  const checkout = steps.findIndex((step) => step.uses?.startsWith('actions/checkout@'));
  const install = steps.findIndex((step) => step.name === 'Install Dependencies');
  const build = steps.findIndex((step) => step.run === 'node scripts/test-examples.mjs');
  assert.ok(checkout >= 0 && checkout < install && install < build);
  assert.equal(steps[install].with.command, 'pnpm i --frozen-lockfile');
  assert.equal(JSON.parse(read('examples/electron/package.json')).scripts.build, 'farm build && electron-forge package');
  for (const step of native.jobs.build.steps.filter((step) => step.uses?.startsWith('actions/upload-artifact@'))) {
    assert.doesNotMatch(step.with.path, /node_modules|examples\/electron/);
    assert.notEqual(step.with.path.trim(), '.');
  }
});

test('native build entrypoints and Forge checksum policy retain their distinct scope', () => {
  assert.equal(JSON.parse(read('packages/plugin-tools/package.json')).scripts.build, 'tsc -b');
  for (const file of ['scripts/formal-native-build.mjs', 'scripts/ci-build-rust-artifacts.sh']) {
    assert.doesNotMatch(read(file), /electron|test-examples|test-e2e/);
  }
  const policy = read('examples/electron/electron-download.cjs');
  assert.match(policy, /require\('electron\/checksums.json'\)/);
  assert.match(policy, /unsafelyDisableChecksums: false/);
  assert.match(read('examples/electron/forge.config.cjs'), /require\('\.\/electron-download.cjs'\)/);
});
