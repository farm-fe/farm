// cspell:words printenv esac
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const label = 'farm.cargo-cache-owner';
const imageCargoHomes = new Map([
  ['ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian', '/usr/local/cargo'],
  ['ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine', '/root/.cargo'],
]);
const images = new Set(imageCargoHomes.keys());
// The read-only seed is copied into the container, never mutated by root on the
// host. docker cp without --archive copies back with the invoking host user's
// ownership: https://docs.docker.com/reference/cli/docker/container/cp/
export const containerBuild = `set -eu
cargo_home="$(printenv CARGO_HOME)"
expected_cargo_home="$(printenv FARM_EXPECTED_CARGO_HOME)"
readonly cargo_home expected_cargo_home
test "$cargo_home" = "$expected_cargo_home"
case "$cargo_home" in /usr/local/cargo|/root/.cargo) ;; *) exit 64 ;; esac
mkdir -p "$cargo_home/git" "$cargo_home/registry"
cp -R /cargo-cache-seed/git/. "$cargo_home/git/"
cp -R /cargo-cache-seed/registry/. "$cargo_home/registry/"
. ./scripts/setup-native-container-node.sh
test "$(printenv CARGO_HOME)" = "$cargo_home"
./scripts/ci-build-rust-artifacts.sh`;

export function containerCargoHome(environment) {
  assert.ok(Array.isArray(environment), 'Image Cargo environment metadata is unavailable');
  const selected = new Map();
  for (const entry of environment) {
    if (typeof entry !== 'string' || !/^CARGO_HOME=/.test(entry)) continue;
    const equal = entry.indexOf('='), name = entry.slice(0, equal);
    assert.ok(!selected.has(name), 'Ambiguous image Cargo environment');
    selected.set(name, entry.slice(equal + 1));
  }
  const home = selected.get('CARGO_HOME');
  assert.ok(['/usr/local/cargo', '/root/.cargo'].includes(home), 'Unsupported or unknown image Cargo home');
  return home;
}

export function readableCache(roots, { io = fs } = {}) {
  const result = { checkedEntries: 0, symlinks: 0, unreadable: [] };
  for (const root of roots) {
    function visit(file) {
      try {
        const stat = io.lstatSync(file); result.checkedEntries++;
        // A cache archiver stores the link itself; do not follow arbitrary source
        // repository symlinks out of the explicitly selected cache directories.
        if (stat.isSymbolicLink()) { result.symlinks++; return; }
        io.accessSync(file, fs.constants.R_OK);
        if (stat.isDirectory()) for (const entry of io.readdirSync(file)) visit(path.join(file, entry));
      } catch (error) {
        if (result.unreadable.length < 30) result.unreadable.push({ root: path.basename(root), entry: path.relative(root, file), code: error.code ?? 'UNKNOWN' });
      }
    }
    if (io.existsSync(root)) visit(root);
  }
  return result;
}

export function runDockerCargoBuild({ workspace = process.cwd(), env = process.env, docker = spawnSync, buildTimeoutMs = 90 * 60 * 1000 } = {}) {
  const root = fs.realpathSync(workspace), home = fs.realpathSync(env.FARM_CARGO_HOME);
  assert.ok(!root.includes(',') && !home.includes(','), 'Docker bind paths may not contain mount-option delimiters');
  const image = env.FARM_BUILD_IMAGE, abi = env.FARM_BUILD_ABI;
  assert.ok(images.has(image), 'Only the existing reviewed native images are supported');
  assert.ok(['linux-x64-gnu', 'linux-x64-musl'].includes(abi), 'Explicit Docker ABI required');
  assert.ok(['ci', 'release'].includes(env.FARM_BUILD_PROFILE), 'Explicit native profile required');
  assert.equal(env.FARM_BUILD_TARGET, abi === 'linux-x64-gnu' ? 'x86_64-unknown-linux-gnu' : 'x86_64-unknown-linux-musl', 'Docker ABI/target differs');
  assert.equal(image, abi === 'linux-x64-gnu' ? 'ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian' : 'ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine', 'Docker ABI/image differs');
  assert.ok(['true', 'false'].includes(env.FARM_BUILD_ZIG ?? 'false'));
  assert.ok(Number.isSafeInteger(buildTimeoutMs) && buildTimeoutMs > 0 && buildTimeoutMs <= 90 * 60 * 1000);
  const nonce = randomUUID(), name = `farm-cache-${nonce}`;
  const report = { schemaVersion: 1, abi, image, profile: env.FARM_BUILD_PROFILE, target: env.FARM_BUILD_TARGET,
    buildExitCode: null, clientExitCode: null, timedOut: false, cacheExported: false, cleanupConfirmed: false,
    saveConfirmed: false, saveEvidence: 'Host readability only; actual cache action post-save must be confirmed in Actions logs.', errors: [] };
  const receiptDirectory = path.join(root, '.ci-native-cache'); fs.mkdirSync(receiptDirectory, { recursive: true });
  const receiptFile = path.join(receiptDirectory, `${abi}.json`);
  let id, containerConfig, containerImage;
  const invoke = (args, options = {}) => docker('docker', args, { encoding: 'utf8', timeout: 120000, windowsHide: true, ...options });
  const successful = result => !result.error && result.status === 0;
  function checked(args, options) {
    const result = invoke(args, options);
    assert.ok(successful(result), `Docker ${args[0]} failed (${result.error?.code ?? result.status})`);
    return result.stdout?.trim() ?? '';
  }
  function inspect() {
    const value = JSON.parse(checked(['inspect', id]))[0];
    assert.equal(value.Id, id, 'Container identity changed');
    assert.equal(value.Config?.Labels?.[label], nonce, 'Container ownership label changed');
    containerConfig = value.Config;
    containerImage = value.Image;
    if (report.containerImageId) assert.equal(containerImage, report.containerImageId, 'Container image identity changed');
    return value.State;
  }
  try {
    for (const part of ['git', 'registry']) fs.mkdirSync(path.join(home, part), { recursive: true });
    const args = ['create', '--name', name, '--label', `${label}=${nonce}`,
      '--mount', `type=bind,source=${path.join(home, 'git')},target=/cargo-cache-seed/git,readonly`,
      '--mount', `type=bind,source=${path.join(home, 'registry')},target=/cargo-cache-seed/registry,readonly`,
      '--mount', `type=bind,source=${root},target=/build`, '--workdir', '/build'];
    for (const key of ['FARM_BUILD_PROFILE', 'FARM_BUILD_ABI', 'FARM_BUILD_TARGET', 'FARM_BUILD_ZIG']) args.push('--env', `${key}=${env[key] ?? 'false'}`);
    // Alpine installs rustup in root's default home without declaring HOME or
    // CARGO_HOME in image metadata. Set the reviewed path explicitly instead of
    // inferring a runtime HOME from an incomplete Config.Env array.
    const cargoHome = imageCargoHomes.get(image);
    args.push('--env', `CARGO_HOME=${cargoHome}`, '--env', `FARM_EXPECTED_CARGO_HOME=${cargoHome}`);
    args.push(image, 'sh', '-c', containerBuild);
    id = checked(args, { timeout: 10 * 60 * 1000 }); assert.match(id, /^[a-f0-9]{64}$/);
    inspect();
    assert.match(containerImage ?? '', /^sha256:[a-f0-9]{64}$/, 'Actual container image ID is unavailable');
    report.containerImageId = containerImage;
    report.containerCargoHome = containerCargoHome(containerConfig.Env);
    assert.equal(report.containerCargoHome, cargoHome, 'Container Cargo home differs from explicit image policy');
    report.containerCargoHomeSource = 'Project explicit per-image Cargo home policy; Config.Env verification plus shell checks before build. Image ID is recorded separately, not a registry digest or inferred default';
    const started = invoke(['start', '--attach', id], { stdio: 'inherit', timeout: buildTimeoutMs });
    report.clientExitCode = started.status;
    report.timedOut = started.error?.code === 'ETIMEDOUT';
    let state = inspect();
    if (state.Running) {
      // Only this newly created, label-confirmed container is stopped. No global
      // container cleanup, process kill, ownership or permission changes occur.
      checked(['stop', '--time', '15', id], { timeout: 30000 });
      state = inspect();
    }
    assert.equal(state.Running, false, 'Owned build container did not stop');
    assert.equal(state.Status, 'exited', 'Container did not reach a completed build state');
    if (started.error && !report.timedOut) report.errors.push(`Docker start client failed: ${started.error.code ?? 'UNKNOWN'}`);
    if (!started.error && started.status !== 0 && state.ExitCode === 0) report.errors.push('Docker start failed without a matching build failure');
    assert.ok(Number.isInteger(state.ExitCode), 'Actual container build exit code is unavailable');
    report.buildExitCode = state.ExitCode;
    assert.equal(containerCargoHome(containerConfig.Env), report.containerCargoHome, 'Container Cargo home changed before cache export');
    // No -a/--archive: host ownership is required, not root ownership from source.
    for (const part of ['git', 'registry']) checked(['cp', `${id}:${report.containerCargoHome}/${part}/.`, path.join(home, part)], { timeout: 5 * 60 * 1000 });
    report.cacheExported = true;
    report.readability = readableCache([path.join(home, 'git'), path.join(home, 'registry'), path.join(root, 'target')]);
    assert.equal(report.readability.unreadable.length, 0, 'Host cache/target still has unreadable entries; do not claim cache save');
  } catch (error) { report.errors.push(error.message); }
  finally {
    if (!id) {
      // A timed-out create may have completed. Probe the unique name once; do not
      // create it again or touch a pre-existing container with another label.
      const found = invoke(['inspect', name]);
      if (successful(found)) {
        try { const value = JSON.parse(found.stdout)[0]; if (value.Config?.Labels?.[label] === nonce && /^[a-f0-9]{64}$/.test(value.Id)) id = value.Id; } catch { /* Unknown creation outcome remains a failed receipt. */ }
      }
    }
    if (id && /^[a-f0-9]{64}$/.test(id)) {
      try {
        const state = inspect();
        if (state.Running) { checked(['stop', '--time', '15', id], { timeout: 30000 }); assert.equal(inspect().Running, false); }
        checked(['rm', id]); report.cleanupConfirmed = true;
      } catch (error) { report.errors.push(`Owned container cleanup: ${error.message}`); }
    }
    report.exitCode = report.buildExitCode !== null && report.buildExitCode !== 0 ? report.buildExitCode
      : report.timedOut ? 124 : report.errors.length || !report.cleanupConfirmed || !report.cacheExported ? 1 : 0;
    fs.writeFileSync(receiptFile, JSON.stringify(report, null, 2) + '\n');
    console.log('[native Cargo cache receipt] ' + JSON.stringify(report));
  }
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) process.exitCode = runDockerCargoBuild().exitCode;
