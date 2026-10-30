import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { runDockerCargoBuild, readableCache, containerBuild, containerCargoHome } from './v2-docker-cargo-cache.mjs';

function fixture(mode = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docker-cache-contract-'));
  const home = path.join(root, 'cargo'); fs.mkdirSync(home);
  const env = { FARM_CARGO_HOME: home, FARM_BUILD_IMAGE: 'ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine',
    FARM_BUILD_ABI: 'linux-x64-musl', FARM_BUILD_TARGET: 'x86_64-unknown-linux-musl', FARM_BUILD_PROFILE: 'ci', FARM_BUILD_ZIG: 'false' };
  if (mode.cargoHome === '/usr/local/cargo') Object.assign(env, { FARM_BUILD_IMAGE: 'ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian', FARM_BUILD_ABI: 'linux-x64-gnu', FARM_BUILD_TARGET: 'x86_64-unknown-linux-gnu' });
  const id = 'a'.repeat(64), calls = []; let nonce, explicitHome, started = false, stopped = false;
  function docker(executable, args) {
    assert.equal(executable, 'docker'); calls.push(args);
    if (args[0] === 'create') {
      nonce = args[args.indexOf('--label') + 1].split('=')[1];
      assert.equal(args[args.indexOf('--workdir') + 1], '/build');
      for (const key of ['FARM_BUILD_PROFILE', 'FARM_BUILD_ABI', 'FARM_BUILD_TARGET', 'FARM_BUILD_ZIG']) {
        const index = args.indexOf(`${key}=${env[key]}`);
        assert.ok(index > 0 && args[index - 1] === '--env', `${key} must be forwarded unchanged`);
      }
      explicitHome = args.filter((value, index) => args[index - 1] === '--env' && value.startsWith('CARGO_HOME='));
      assert.deepEqual(explicitHome, [`CARGO_HOME=${env.FARM_BUILD_IMAGE.endsWith('lts-debian') ? '/usr/local/cargo' : '/root/.cargo'}`]);
      assert.ok(args.includes(`FARM_EXPECTED_CARGO_HOME=${explicitHome[0].slice('CARGO_HOME='.length)}`));
      assert.deepEqual(args.slice(-4), [env.FARM_BUILD_IMAGE, 'sh', '-c', containerBuild]);
      return { status: 0, stdout: id };
    }
    if (args[0] === 'start') { started = true; return mode.timeout ? { status: null, error: { code: 'ETIMEDOUT' } } : { status: mode.buildExit ?? 0 }; }
    if (args[0] === 'inspect') return { status: 0, stdout: JSON.stringify([{ Id: id, Image: `sha256:${'c'.repeat(64)}`, Config: { Env: mode.missingCargoHome ? [] : [...(mode.inspectHome ? [`CARGO_HOME=${mode.inspectHome}`] : explicitHome), ...(mode.omitHome ? [] : ['HOME=/root'])], Labels: { 'farm.cargo-cache-owner': mode.wrongOwner ? 'other' : nonce } },
      State: { Running: Boolean(mode.timeout && started && !stopped), Status: !started ? 'created' : mode.timeout && !stopped ? 'running' : 'exited', ExitCode: mode.timeout && stopped ? 137 : mode.buildExit ?? 0 } }]) };
    if (args[0] === 'stop') { stopped = true; return { status: 0 }; }
    if (args[0] === 'cp') {
      assert.equal(args.length, 3); assert.ok(args[1].startsWith(`${id}:${mode.cargoHome ?? '/root/.cargo'}/`));
      assert.ok(args[2].startsWith(home + path.sep));
      if (mode.copyFailure) return { status: 1 };
      fs.writeFileSync(path.join(args[2], 'host-created-entry'), 'fixture only'); return { status: 0 };
    }
    if (args[0] === 'rm') { assert.equal(args[1], id); assert.equal(args.length, 2); return { status: 0 }; }
    assert.fail(`Unexpected Docker command: ${args[0]}`);
  }
  return { root, home, env, calls, docker };
}

test('Docker uses read-only cache seeds, host-owned copy export and a readability receipt without claiming a save', () => {
  const f = fixture();
  try {
    const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker });
    assert.equal(report.exitCode, 0); assert.equal(report.buildExitCode, 0);
    assert.equal(report.cacheExported, true); assert.equal(report.cleanupConfirmed, true);
    assert.equal(report.containerImageId, `sha256:${'c'.repeat(64)}`);
    assert.equal(report.saveConfirmed, false); assert.match(report.saveEvidence, /Actions logs/);
    assert.equal(report.readability.unreadable.length, 0);
    const created = f.calls.find(args => args[0] === 'create');
    assert.equal(created.filter(arg => arg.endsWith(',readonly')).length, 2);
    assert.ok(created.includes(containerBuild));
    assert.ok(created.includes(`type=bind,source=${f.root},target=/build`));
    assert.doesNotMatch(f.calls.flat().join(' '), /--privileged|--archive|sudo|chown|chmod/);
    assert.ok(fs.existsSync(path.join(f.root, '.ci-native-cache/linux-x64-musl.json')));
  } finally { fs.rmSync(f.root, { recursive: true }); }
});

test('actual failed build code survives cache-export failure and a failure receipt still exists', () => {
  const f = fixture({ buildExit: 17, copyFailure: true });
  try {
    const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker });
    assert.equal(report.exitCode, 17); assert.equal(report.buildExitCode, 17);
    assert.equal(report.cacheExported, false); assert.equal(report.cleanupConfirmed, true);
    assert.ok(report.errors.some(message => message.includes('cp failed')));
    assert.equal(JSON.parse(fs.readFileSync(path.join(f.root, '.ci-native-cache/linux-x64-musl.json'))).buildExitCode, 17);
  } finally { fs.rmSync(f.root, { recursive: true }); }
});

test('timeout stops only a label-proven owned container and remains a failed outcome', () => {
  const f = fixture({ timeout: true });
  try {
    const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker, buildTimeoutMs: 1000 });
    assert.equal(report.timedOut, true); assert.notEqual(report.exitCode, 0);
    assert.equal(report.cleanupConfirmed, true);
    assert.equal(f.calls.filter(args => args[0] === 'stop').length, 1);
  } finally { fs.rmSync(f.root, { recursive: true }); }
  const unknown = fixture({ wrongOwner: true });
  try {
    const report = runDockerCargoBuild({ workspace: unknown.root, env: unknown.env, docker: unknown.docker });
    assert.notEqual(report.exitCode, 0); assert.equal(report.cleanupConfirmed, false);
    assert.ok(!unknown.calls.some(args => ['stop', 'rm', 'cp'].includes(args[0])));
  } finally { fs.rmSync(unknown.root, { recursive: true }); }
});

test('host readability failures are explicit; cache symlinks are never traversed', () => {
  const denied = readableCache(['/owned/cache'], { io: { existsSync: () => true,
    lstatSync: () => ({ isSymbolicLink: () => false }), accessSync: () => { throw Object.assign(new Error('denied'), { code: 'EACCES' }); } } });
  assert.equal(denied.unreadable[0].code, 'EACCES');
  const link = readableCache(['/owned/cache'], { io: { existsSync: () => true, lstatSync: () => ({ isSymbolicLink: () => true }) } });
  assert.equal(link.symlinks, 1); assert.equal(link.unreadable.length, 0);
});

test('seed and export follow the actual image Cargo home instead of assuming root HOME', () => {
  assert.equal(containerCargoHome(['CARGO_HOME=/usr/local/cargo', 'HOME=/root', 'SECRET=must-not-be-logged']), '/usr/local/cargo');
  assert.throws(() => containerCargoHome(['HOME=/root']), /Unsupported or unknown/);
  for (const home of ['/usr/local/cargo', '/root/.cargo']) {
    const f = fixture({ cargoHome: home });
    try {
      const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker });
      assert.equal(report.exitCode, 0); assert.equal(report.containerCargoHome, home);
      assert.ok(f.calls.filter(args => args[0] === 'cp').every(args => args[1].startsWith(`a`.repeat(64) + `:${home}/`)));
      assert.doesNotMatch(JSON.stringify(report), /SECRET|must-not-be-logged/);
    } finally { fs.rmSync(f.root, { recursive: true }); }
  }
  for (const env of [[], ['CARGO_HOME=/outside'], ['CARGO_HOME=/usr/local/cargo', 'CARGO_HOME=/root/.cargo']]) assert.throws(() => containerCargoHome(env));
});

test('Alpine metadata without implicit HOME uses explicit reviewed Cargo home before start', () => {
  const f = fixture({ omitHome: true });
  try {
    const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker });
    assert.equal(report.exitCode, 0); assert.equal(report.containerCargoHome, '/root/.cargo');
    assert.equal(report.buildExitCode, 0); assert.equal(report.cleanupConfirmed, true);
  } finally { fs.rmSync(f.root, { recursive: true }); }
  const missing = fixture({ missingCargoHome: true });
  try {
    const report = runDockerCargoBuild({ workspace: missing.root, env: missing.env, docker: missing.docker });
    assert.notEqual(report.exitCode, 0); assert.equal(report.buildExitCode, null);
    assert.equal(report.cleanupConfirmed, true);
    assert.ok(!missing.calls.some(args => ['start', 'cp'].includes(args[0])));
  } finally { fs.rmSync(missing.root, { recursive: true }); }
});

test('a different whitelisted home still cannot override the explicit image policy', () => {
  const f = fixture({ inspectHome: '/usr/local/cargo' });
  try {
    const report = runDockerCargoBuild({ workspace: f.root, env: f.env, docker: f.docker });
    assert.notEqual(report.exitCode, 0); assert.equal(report.buildExitCode, null);
    assert.equal(report.cleanupConfirmed, true);
    assert.ok(report.errors.some(message => message.includes('explicit image policy')));
    assert.ok(!f.calls.some(args => ['start', 'cp'].includes(args[0])));
  } finally { fs.rmSync(f.root, { recursive: true }); }
});

test('shell checks the fixed Cargo home again after sourcing Node setup without rewriting toolchain homes', () => {
  const setup = containerBuild.indexOf('. ./scripts/setup-native-container-node.sh');
  const check = containerBuild.indexOf('test "$(printenv CARGO_HOME)" = "$cargo_home"');
  const build = containerBuild.indexOf('./scripts/ci-build-rust-artifacts.sh');
  assert.ok(setup >= 0 && check > setup && build > check);
  assert.match(containerBuild, /readonly cargo_home expected_cargo_home/);
  assert.match(containerBuild, /test "\$cargo_home" = "\$expected_cargo_home"/);
  assert.doesNotMatch(containerBuild, /(?:export )?(?:HOME|RUSTUP_HOME|PATH)=/);
});
