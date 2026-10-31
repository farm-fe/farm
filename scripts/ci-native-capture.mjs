// cspell:words RUSTFLAGS
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { nativeArtifactInventory } from './native-artifact-inventory.mjs';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const root = fileURLToPath(new URL('../', import.meta.url));
const buildVariables = ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CARGO_PROFILE_RELEASE_LTO', 'CARGO_PROFILE_RELEASE_CODEGEN_UNITS', 'FARM_BUILD_PROFILE', 'FARM_BUILD_ABI', 'FARM_BUILD_TARGET', 'FARM_BUILD_ZIG', 'FARM_CAPTURE_CORE_SCRIPT', 'FARM_CAPTURE_CORE_ARGS', 'FARM_CAPTURE_PLUGIN_ARGS', 'FARM_CAPTURE_CREATE_SCRIPT', 'FARM_CAPTURE_PLUGINS'];

export function captureRuntime({ env = process.env, run = spawnSync, cwd = root } = {}) {
  const errors = [];
  function version(executable, args) {
    const result = run(executable, args, { cwd, env, encoding: 'utf8', timeout: 15000, maxBuffer: 1024 * 1024, windowsHide: true, shell: false });
    if (result.status !== 0 || result.error) { errors.push({ tool: executable, status: result.status, code: result.error?.code ?? null }); return null; }
    return (result.stdout ?? '').trim();
  }
  const rustc = version('rustc', ['-vV']), cargo = version('cargo', ['-vV']);
  const activeToolchain = version('rustup', ['show', 'active-toolchain']);
  const selected = Object.fromEntries(buildVariables.map(name => [name, env[name] ?? null]));
  // Values can contain build paths; expose only their digest, never arbitrary env.
  const flagsHash = hash(JSON.stringify(selected));
  const permittedBuildVariables = new Set([...buildVariables, 'CARGO_HOME', 'CARGO_TARGET_DIR', 'RUSTUP_HOME',
    'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER']);
  const unboundBuildVariables = Object.keys(env).filter(name => /^(CARGO_|RUST|CC(?:_|$)|CXX(?:_|$)|AR(?:_|$)|LD(?:_|$))/.test(name)
    && !permittedBuildVariables.has(name)).sort();
  const target = env.FARM_BUILD_TARGET;
  const linkerKey = target ? `CARGO_TARGET_${target.toUpperCase().replaceAll('-', '_')}_LINKER` : null;
  const linker = linkerKey ? env[linkerKey] : null;
  const linkerVersion = linker && !/[\r\n]/.test(linker) ? version(linker, ['--version']) : null;
  return { schemaVersion: 1, phase: 'before-build', abi: env.FARM_BUILD_ABI ?? null, profile: env.FARM_BUILD_PROFILE ?? null, target: target ?? null,
    runtime: { node: process.version, os: process.platform, arch: process.arch, release: os.release() }, rustc, cargo, activeToolchain, flagsHash, unboundBuildVariables,
    linker: linkerVersion ? { executableHash: hash(linker), version: linkerVersion } : null, errors,
    measuredToolsComplete: Boolean(rustc && cargo && linkerVersion), completeMeasuredContext: false,
    reuseAuthorized: false, reason: linkerVersion ? 'Shadow tool capture only; final compiler feature resolution and authenticated producer proof remain mandatory' : 'Effective linker is not explicit and measured; do not guess a reusable environment' };
}

export function captureProducts({ cwd = root, env = process.env, inventory = nativeArtifactInventory(cwd) } = {}) {
  const abi = env.FARM_BUILD_ABI;
  if (!/^(?:linux|darwin|win32)-[a-z0-9-]+$/.test(abi ?? '')) throw new Error('Exact native ABI required');
  let owners = null;
  if (env.FARM_BUILD_PROFILE === 'release' && env.FARM_RELEASE_NATIVE_OWNERS) {
    owners = JSON.parse(env.FARM_RELEASE_NATIVE_OWNERS);
    const known = new Set(inventory.npm.payloads.map(row => row.owner));
    if (!Array.isArray(owners) || new Set(owners).size !== owners.length || owners.some(owner => typeof owner !== 'string' || !known.has(owner))) throw Error('Invalid native owner selection');
  }
  const expected = inventory.npm.payloads.filter(row => row.platform === abi && (!owners || owners.includes(row.owner)));
  if (!expected.length && owners === null) throw new Error('Empty native producer cohort');
  const records = [];
  for (const row of expected) {
    const owner = inventory.npm.primary.find(item => item.name === row.owner);
    if (!owner) throw new Error('Native owner missing');
    const directory = path.dirname(owner.manifestPath);
    const file = row.owner === '@farmfe/core' ? `packages/core/binding/farm.${abi}.node` : row.owner === 'create-farm' ? `packages/create-farm/create-farm.${abi}.node` : `${directory.replaceAll('\\', '/')}/npm/${abi}/index.farm`;
    const absolute = path.resolve(cwd, file), realRoot = fs.realpathSync.native(cwd), realFile = fs.realpathSync.native(absolute);
    const relative = path.relative(realRoot, realFile);
    if (!relative || path.isAbsolute(relative) || relative === '..' || relative.startsWith(`..${path.sep}`) || !fs.lstatSync(absolute).isFile()) throw new Error('Native product escaped its producer');
    const bytes = fs.readFileSync(absolute);
    records.push({ name: row.name, owner: row.owner, file, bytes: bytes.length, sha256: hash(bytes) });
  }
  return { schemaVersion: 1, kind: 'native-producer-shadow-record-not-reuse-proof', abi, profile: env.FARM_BUILD_PROFILE,
    originBuildSHA: env.GITHUB_SHA ?? null, runId: env.GITHUB_RUN_ID ?? null, runAttempt: env.GITHUB_RUN_ATTEMPT ?? null,
    jobKey: env.GITHUB_JOB ?? null, workflowRef: env.GITHUB_WORKFLOW_REF ?? null, repository: env.GITHUB_REPOSITORY ?? null,
    producerStatus: 'requires-authoritative-completed-job-result', records, reuseAuthorized: false, acceptanceInherited: false };
}

export function proofDirectory(cwd, abi, mode) {
  if (!/^(?:linux|darwin|win32)-[a-z0-9-]+$/.test(abi ?? '') || !['runtime', 'products'].includes(mode)) throw Error('Exact native capture phase and ABI required');
  return path.join(cwd, mode === 'runtime' ? '.ci-native-proof' : '.ci-native-host-proof', abi);
}

function prepareProofDirectory(cwd, abi, mode, io = fs) {
  const directory = proofDirectory(cwd, abi, mode);
  io.mkdirSync(directory, {recursive: true});
  if (io.lstatSync(directory).isSymbolicLink() || io.realpathSync.native(directory) !== path.join(io.realpathSync.native(cwd), mode === 'runtime' ? '.ci-native-proof' : '.ci-native-host-proof', abi)) throw Error('Native proof directory redirects');
  return directory;
}

function writeImmutableProof(file, body, io) {
  if (io.existsSync(file)) {
    const stat = io.lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink() || !io.readFileSync(file).equals(Buffer.from(body))) throw Error('Existing native evidence differs');
    return;
  }
  io.writeFileSync(file, body, {flag: 'wx'});
}

export function captureProductProof({cwd = root, env = process.env, inventory, io = fs} = {}) {
  const abi = env.FARM_BUILD_ABI, directory = prepareProofDirectory(cwd, abi, 'products', io);
  const record = captureProducts({cwd, env, ...(inventory ? {inventory} : {})});
  const receipt = path.join(cwd, '.ci-native-cache', `${abi}.json`);
  if (io.existsSync(receipt)) record.container = JSON.parse(io.readFileSync(receipt));
  // Container-created runtime evidence is read only. Product/source evidence
  // must never require writing into the container-owned bind-mount directory.
  const runtime = path.join(proofDirectory(cwd, abi, 'runtime'), 'runtime.json');
  const runtimeBytes = io.existsSync(runtime) ? io.readFileSync(runtime) : null;
  record.runtimeCapture = runtimeBytes ? JSON.parse(runtimeBytes) : null;
  if (runtimeBytes) writeImmutableProof(path.join(directory, 'runtime.json'), runtimeBytes, io);
  writeImmutableProof(path.join(directory, 'products.json'), JSON.stringify(record, null, 2) + '\n', io);
  return record;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2], abi = process.env.FARM_BUILD_ABI;
  if (!/^(?:linux|darwin|win32)-[a-z0-9-]+$/.test(abi ?? '')) throw new Error('Exact native ABI required');
  try {
    if (mode === 'products') captureProductProof();
    else {
      const directory = prepareProofDirectory(root, abi, mode);
      fs.writeFileSync(path.join(directory, 'runtime.json'), JSON.stringify(captureRuntime(), null, 2) + '\n');
    }
  } catch (error) {
    try {
      const directory = prepareProofDirectory(root, abi, mode);
      fs.writeFileSync(path.join(directory, `${mode}-failure.json`), JSON.stringify({ schemaVersion: 1, reuseAuthorized: false, error: error.message.slice(0, 1000) }) + '\n');
    } catch { /* Preserve the original failure even when a diagnostic cannot be written. */ }
    console.error(JSON.stringify({phase: mode, code: ['EACCES', 'EPERM', 'ENOENT', 'EEXIST', 'EINVAL'].includes(error.code) ? error.code : 'capture-failed'}));
    process.exitCode = 1;
  }
}
