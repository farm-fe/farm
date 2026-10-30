// cspell:words RUSTFLAGS
import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const { commandPlan, executeCacheCommand } = createRequire(import.meta.url)('./cargo-cache-command.cjs');
test('target cache metadata uses real dependency sources without changing build profiles or flags', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-cache-command-'));
  try {
    const env = { FARM_CARGO_HOME: home, CARGO_HOME: '/isolated-target-home', RUSTFLAGS: 'original flags', RUSTUP_HOME: '/toolchains' };
    const result = commandPlan(['cargo', 'metadata', '--all-features', '--format-version', '1'], env);
    assert.equal(result.env.CARGO_HOME, fs.realpathSync(home));
    assert.equal(env.CARGO_HOME, '/isolated-target-home');
    assert.equal(result.env.RUSTFLAGS, env.RUSTFLAGS); assert.equal(result.env.RUSTUP_HOME, env.RUSTUP_HOME);
    assert.equal(result.args.at(-1), '--locked');
    for (const args of [['rustc', '-vV'], ['rustup', 'toolchain', 'list', '--quiet'], ['rustup', 'run', 'nightly-2026-04-10', 'rustc', '-vV']]) assert.equal(commandPlan(args, env).command, args[0]);
    for (const args of [['cargo', 'build'], ['cargo', 'test'], ['sh'], ['rustup', 'run', 'nightly', 'cargo', 'publish']]) assert.throws(() => commandPlan(args, env));
    assert.throws(() => commandPlan(['cargo', 'metadata'], {}), /Explicit source/);
  } finally { fs.rmSync(home, { recursive: true }); }
});

function snapshotFixture() {
  const cleanupRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'farm metadata snapshot '));
  const root = path.join(cleanupRoot, 'repository'); fs.mkdirSync(root);
  const externalManifest = path.join(cleanupRoot, 'registry/serde/Cargo.toml');
  fs.mkdirSync(path.dirname(externalManifest), { recursive: true }); fs.writeFileSync(externalManifest, 'external dependency');
  for (const file of ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'rust-toolchain.toml', 'crates/core/Cargo.toml']) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true }); fs.writeFileSync(path.join(root, file), 'fixture');
  }
  const env = { FARM_CARGO_HOME: root, FARM_CARGO_METADATA: path.join(root, 'metadata.json'), CARGO_HOME: path.join(root, 'isolated-target-home') };
  const metadata = { workspace_members: ['workspace'], packages: [
    { id: 'workspace', name: 'farmfe_core', version: '2.2.0', manifest_path: path.join(root, 'crates/core/Cargo.toml'), targets: [{ kind: ['rlib'], name: 'farmfe_core' }] },
    { id: 'dependency', name: 'serde', version: '1.0.0', manifest_path: externalManifest, targets: [{ kind: ['lib'], name: 'serde' }] },
  ] };
  return { root, env, metadata, cleanupRoot, externalManifest };
}

test('real command-format argument shapes return clean JSON for restore and post without a second executor call', () => {
  const f = snapshotFixture(); let calls = 0;
  const run = (command, args, options) => {
    calls++; assert.equal(command, 'cargo');
    assert.deepEqual(args, ['metadata', '--all-features', '--format-version', '1', '--locked']);
    assert.equal(options.env.CARGO_HOME, fs.realpathSync(f.root)); assert.equal(options.shell, false);
    assert.equal(options.timeout, 600000);
    return { status: 0, stdout: JSON.stringify(f.metadata) };
  };
  try {
    assert.equal(executeCacheCommand(['--snapshot'], { ...f, run }).status, 0);
    for (const [args, count] of [
      [['cargo', 'metadata', '--all-features', '--format-version', '1', '--no-deps'], 1],
      [['cargo', 'metadata', '--all-features', '--format-version', '1'], 2],
    ]) {
      const result = executeCacheCommand(args, { ...f, run });
      assert.equal(result.status, 0); assert.equal(result.stderr, '');
      assert.equal(JSON.parse(result.stdout).packages.length, count);
      assert.equal(f.env.CARGO_HOME, path.join(f.root, 'isolated-target-home'));
    }
    assert.equal(calls, 1);
    assert.equal(executeCacheCommand(['--verify-snapshot'], { ...f, run }).status, 0);
    fs.appendFileSync(path.join(f.root, 'Cargo.lock'), 'changed');
    const stale = executeCacheCommand(['--verify-snapshot'], { ...f, run });
    assert.equal(stale.status, 1); assert.match(stale.stderr, /inputs changed/);
    assert.equal(stale.stdout, ''); assert.equal(calls, 1);
  } finally { fs.rmSync(f.cleanupRoot, { recursive: true }); }
});

test('metadata failure never registers a valid empty cleanup snapshot', () => {
  for (const result of [{ status: 28, stderr: 'network failed' }, { status: null, error: { code: 'ETIMEDOUT' } }, { status: 0, stdout: JSON.stringify({ packages: [], workspace_members: [] }) }]) {
    const f = snapshotFixture();
    try {
      const captured = executeCacheCommand(['--snapshot'], { ...f, run: () => result });
      assert.equal(captured.status, 1); assert.equal(captured.stdout, '');
      assert.equal(fs.existsSync(f.env.FARM_CARGO_METADATA), false);
      assert.equal(executeCacheCommand(['--verify-snapshot'], f).status, 1);
    } finally { fs.rmSync(f.cleanupRoot, { recursive: true }); }
  }
});

test('identity command exit codes, timeout and native paths are preserved without a shell', () => {
  const f = snapshotFixture();
  try {
    for (const [result, status] of [[{ status: 17, stdout: '', stderr: 'identity failed' }, 17], [{ status: null, error: { code: 'ETIMEDOUT' } }, 124]]) {
      const value = executeCacheCommand(['rustc', '-vV'], { ...f, run: (command, args, options) => {
        assert.equal(command, 'rustc'); assert.deepEqual(args, ['-vV']); assert.equal(options.shell, false);
        assert.equal(options.env.CARGO_HOME, fs.realpathSync(f.root)); return result;
      } });
      assert.equal(value.status, status);
    }
    // This fixture runs on Windows too: the native real path is not a Git Bash /c alias.
    assert.equal(commandPlan(['rustc', '-vV'], f.env).env.CARGO_HOME, fs.realpathSync(f.root));
  } finally { fs.rmSync(f.cleanupRoot, { recursive: true }); }
});

test('actual Node CLI emits only metadata JSON in restore and post with native paths containing spaces', () => {
  const f = snapshotFixture();
  try {
    const snapshot = executeCacheCommand(['--snapshot'], { ...f, run: () => ({ status: 0, stdout: JSON.stringify(f.metadata) }) });
    assert.equal(snapshot.status, 0);
    const script = fileURLToPath(new URL('./cargo-cache-command.cjs', import.meta.url));
    for (const noDependencies of [true, false]) {
      const args = [script, 'cargo', 'metadata', '--all-features', '--format-version', '1', ...(noDependencies ? ['--no-deps'] : [])];
      const result = spawnSync(process.execPath, args, { cwd: f.root, env: { ...process.env, ...f.env }, encoding: 'utf8', shell: false, windowsHide: true, timeout: 10000 });
      assert.equal(result.status, 0, result.stderr); assert.equal(result.stderr, '');
      assert.equal(JSON.parse(result.stdout).packages.length, noDependencies ? 1 : 2);
    }
  } finally { fs.rmSync(f.cleanupRoot, { recursive: true }); }
});

test('repository path and patch manifests are inputs even when they are not workspace members', () => {
  const f = snapshotFixture();
  const patch = path.join(f.root, 'patches/serde/Cargo.toml');
  fs.mkdirSync(path.dirname(patch), { recursive: true }); fs.writeFileSync(patch, 'local patch');
  f.metadata.packages.push({ id: 'local-patch', name: 'patched_dependency', version: '1.0.0', manifest_path: patch, targets: [{ kind: ['lib'], name: 'patched_dependency' }] });
  const run = () => ({ status: 0, stdout: JSON.stringify(f.metadata) });
  try {
    assert.equal(executeCacheCommand(['--snapshot'], { ...f, run }).status, 0);
    const receipt = JSON.parse(fs.readFileSync(f.env.FARM_CARGO_METADATA, 'utf8'));
    assert.ok(receipt.inputs.some(input => input.file === fs.realpathSync(patch)));
    assert.ok(!receipt.inputs.some(input => input.file === fs.realpathSync(f.externalManifest)));
    fs.appendFileSync(f.externalManifest, 'not a repository input');
    assert.equal(executeCacheCommand(['--verify-snapshot'], f).status, 0);
    fs.appendFileSync(patch, 'changed');
    const result = executeCacheCommand(['--verify-snapshot'], f);
    assert.equal(result.status, 1); assert.match(result.stderr, /inputs changed/);
  } finally { fs.rmSync(f.cleanupRoot, { recursive: true }); }
});
