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
  const selected = Object.fromEntries(buildVariables.map(name => [name, env[name] ?? null]));
  // Values can contain build paths; expose only their digest, never arbitrary env.
  const flagsHash = hash(JSON.stringify(selected));
  const target = env.FARM_BUILD_TARGET;
  const linkerKey = target ? `CARGO_TARGET_${target.toUpperCase().replaceAll('-', '_')}_LINKER` : null;
  const linker = linkerKey ? env[linkerKey] : null;
  const linkerVersion = linker && !/[\r\n]/.test(linker) ? version(linker, ['--version']) : null;
  return { schemaVersion: 1, phase: 'before-build', abi: env.FARM_BUILD_ABI ?? null, profile: env.FARM_BUILD_PROFILE ?? null, target: target ?? null,
    runtime: { node: process.version, os: process.platform, arch: process.arch, release: os.release() }, rustc, cargo, flagsHash,
    linker: linkerVersion ? { executableHash: hash(linker), version: linkerVersion } : null, errors,
    measuredToolsComplete: Boolean(rustc && cargo && linkerVersion), completeMeasuredContext: false,
    reuseAuthorized: false, reason: linkerVersion ? 'Shadow tool capture only; final compiler feature resolution and authenticated producer proof remain mandatory' : 'Effective linker is not explicit and measured; do not guess a reusable environment' };
}

export function captureProducts({ cwd = root, env = process.env, inventory = nativeArtifactInventory(cwd) } = {}) {
  const abi = env.FARM_BUILD_ABI;
  if (!/^(?:linux|darwin|win32)-[a-z0-9-]+$/.test(abi ?? '')) throw new Error('Exact native ABI required');
  const expected = inventory.npm.payloads.filter(row => row.platform === abi);
  if (!expected.length) throw new Error('Empty native producer cohort');
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

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2], abi = process.env.FARM_BUILD_ABI;
  if (!/^(?:linux|darwin|win32)-[a-z0-9-]+$/.test(abi ?? '')) throw new Error('Exact native ABI required');
  const directory = path.join(root, '.ci-native-proof', abi); fs.mkdirSync(directory, { recursive: true });
  try {
    const record = mode === 'runtime' ? captureRuntime() : mode === 'products' ? captureProducts() : null;
    if (!record) throw new Error('Unknown native capture phase');
    if (mode === 'products') {
      const receipt = path.join(root, '.ci-native-cache', `${abi}.json`);
      if (fs.existsSync(receipt)) record.container = JSON.parse(fs.readFileSync(receipt));
      const runtime = path.join(directory, 'runtime.json');
      record.runtimeCapture = fs.existsSync(runtime) ? JSON.parse(fs.readFileSync(runtime)) : null;
    }
    fs.writeFileSync(path.join(directory, `${mode}.json`), JSON.stringify(record, null, 2) + '\n');
  } catch (error) {
    fs.writeFileSync(path.join(directory, `${mode}-failure.json`), JSON.stringify({ schemaVersion: 1, reuseAuthorized: false, error: error.message.slice(0, 1000) }) + '\n');
    process.exitCode = 1;
  }
}
