// cspell:words statfs bavail
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { rustRegistryReuse, copyVerifiedRegistryArchives } from './v2-rust-registry-reuse.mjs';

const defaultRoot = fileURLToPath(new URL('../', import.meta.url));
export function sealRustPackages({ root = defaultRoot, output, env = process.env, sourceCommit, resources } = {}) {
  if (!output || fs.existsSync(output)) throw new Error('Specify a fresh output directory; preserve previous evidence');
  output = path.resolve(output); fs.mkdirSync(output, { recursive: true });
  const receipt = { schemaVersion: 1, sourceCommit: sourceCommit ?? null, status: 'not-started', compiled: false, published: false, reusedRegistry: [], records: [] };
  let selected = [], stage = 'read-plan';
  try {
    const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
    receipt.planSha256 = crypto.createHash('sha256').update(JSON.stringify(plan)).digest('hex');
    receipt.sourceCommit ??= execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
    if (!/^[a-f0-9]{40}$/.test(receipt.sourceCommit)) throw new Error('Exact source commit required');
    stage = 'registry-reuse-evidence';
    const reused = rustRegistryReuse(plan, { readEvidence: file => JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence', file), 'utf8')) });
    receipt.reusedRegistry = reused;
    selected = plan.cargo.filter(row => !reused.some(item => item.name === row.name && item.version === row.version));
    const disk = fs.statfsSync(output);
    const observed = resources ?? { freeRamGiB: os.freemem() / 1024 ** 3, freeDiskGiB: disk.bavail * disk.bsize / 1024 ** 3 };
    receipt.freeRamGiB = observed.freeRamGiB; receipt.freeDiskGiB = observed.freeDiskGiB;
    if (!Number.isFinite(receipt.freeRamGiB) || !Number.isFinite(receipt.freeDiskGiB)) throw new Error('Resource observations unavailable');
    if (receipt.freeRamGiB < 3.4 || receipt.freeDiskGiB < 9.75) receipt.status = 'resource-wait';
    else {
      stage = 'reuse-registry-archives';
      const cache = path.join(env.CARGO_HOME ?? path.join(os.homedir(), '.cargo'), 'registry/cache');
      receipt.reusedRegistry = copyVerifiedRegistryArchives(reused, cache, output);
      // Official multi-package preparation resolves unpublished SDK dependencies
      // together. --no-verify disables compilation, never source normalization.
      const args = ['+nightly-2026-04-10', 'package', '--no-verify', '--offline', '--locked', '--allow-dirty', ...selected.flatMap(row => ['-p', row.name])];
      const cargo = env.FARM_CARGO_EXECUTABLE ?? (process.platform === 'win32' ? path.join(os.homedir(), '.cargo/bin/cargo.exe') : 'cargo');
      stage = 'cargo-package';
      const result = selected.length ? spawnSync(cargo, args, { cwd: root, env: { ...env, CARGO_TARGET_DIR: path.join(output, 'target') },
        encoding: 'utf8', windowsHide: true, timeout: 120000, maxBuffer: 16 * 1024 ** 2 }) : { status: 0, stdout: 'All SDK archives reused from verified public registry; no package command executed.', stderr: '' };
      fs.writeFileSync(path.join(output, 'stdout.txt'), result.stdout ?? '');
      fs.writeFileSync(path.join(output, 'diagnostic.txt'), result.stderr ?? '');
      receipt.command = selected.length ? ['cargo', ...args] : null; receipt.exitCode = result.status;
      receipt.status = result.status === 0 ? 'packaged-not-compiled' : 'package-failed';
      receipt.error = result.error?.message;
      for (const row of selected) {
        const file = path.join(output, 'target/package', `${row.name}-${row.version}.crate`);
        if (!fs.existsSync(file)) continue;
        receipt.records.push({ name: row.name, version: row.version, file: path.relative(output, file).replaceAll('\\', '/'),
          sha256: crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'), bytes: fs.statSync(file).size });
      }
    }
  } catch (error) {
    receipt.status = 'package-failed'; receipt.error = error.message.slice(0, 2000); receipt.failureStage = stage;
  } finally { fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2)); }
  return receipt;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const output = process.argv[2] ? path.resolve(process.argv[2]) : null;
  const receipt = sealRustPackages({ output });
  console.log(JSON.stringify({ status: receipt.status, prepared: receipt.records.length, reused: receipt.reusedRegistry.length, output }));
  if (receipt.status !== 'packaged-not-compiled') process.exitCode = 1;
}
