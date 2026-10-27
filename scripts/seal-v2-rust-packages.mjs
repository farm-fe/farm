// cspell:words statfs bavail
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const output = path.resolve(process.argv[2] ?? '');
if (!process.argv[2] || fs.existsSync(output)) throw new Error('Specify a fresh output directory; preserve previous evidence');
const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
const macro = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/published-macro-compat.json')));
const pluginMacro = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/published-plugin-macro-compat.json')));
if (macro.status !== 'passed' || macro.registryPackage.name !== 'farmfe_macro_cache_item' || macro.registryPackage.version !== '2.1.0') throw new Error('Published macro compatibility evidence is required');
if (pluginMacro.status !== 'passed' || pluginMacro.registryPackage.name !== 'farmfe_macro_plugin' || pluginMacro.registryPackage.version !== '2.0.0') throw new Error('Published plugin macro compatibility evidence is required');
const reused = [macro.registryPackage, pluginMacro.registryPackage];
const selected = plan.cargo.filter(row => !reused.some(item => item.name === row.name));
fs.mkdirSync(output, { recursive: true });
const disk = fs.statfsSync(output);
const receipt = { schemaVersion: 1, sourceCommit: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
  planSha256: crypto.createHash('sha256').update(JSON.stringify(plan)).digest('hex'),
  status: 'not-started', compiled: false, published: false, reusedRegistry: reused, records: [],
  freeRamGiB: os.freemem() / 1024 ** 3, freeDiskGiB: disk.bavail * disk.bsize / 1024 ** 3 };
if (receipt.freeRamGiB < 3.4 || receipt.freeDiskGiB < 9.75) receipt.status = 'resource-wait';
else {
  const cache = path.join(process.env.CARGO_HOME ?? path.join(os.homedir(), '.cargo'), 'registry/cache');
  for (const reused of receipt.reusedRegistry) {
    const name = `${reused.name}-${reused.version}.crate`;
    const source = fs.readdirSync(cache).filter(directory => directory.startsWith('index.crates.io-')).map(directory => path.join(cache, directory, name)).find(file => fs.existsSync(file));
    if (!source) throw new Error(`Missing verified immutable registry archive: ${name}`);
    if (crypto.createHash('sha256').update(fs.readFileSync(source)).digest('hex') !== reused.sha256) throw new Error(`Immutable registry archive differs: ${name}`);
    fs.copyFileSync(source, path.join(output, name));
    reused.file = name;
  }
  // Official multi-package preparation resolves unpublished selected dependencies
  // together. --no-verify disables compilation, never source normalization.
  const args = ['+nightly-2026-04-10', 'package', '--no-verify', '--offline', '--locked', '--allow-dirty', ...selected.flatMap(row => ['-p', row.name])];
  const cargo = process.env.FARM_CARGO_EXECUTABLE ?? (process.platform === 'win32' ? path.join(os.homedir(), '.cargo/bin/cargo.exe') : 'cargo');
  const result = spawnSync(cargo, args, { cwd: root, env: { ...process.env, CARGO_TARGET_DIR: path.join(output, 'target') },
    encoding: 'utf8', windowsHide: true, timeout: 120000, maxBuffer: 16 * 1024 ** 2 });
  fs.writeFileSync(path.join(output, 'stdout.txt'), result.stdout ?? '');
  fs.writeFileSync(path.join(output, 'diagnostic.txt'), result.stderr ?? '');
  receipt.command = ['cargo', ...args]; receipt.exitCode = result.status;
  receipt.status = result.status === 0 ? 'packaged-not-compiled' : 'package-failed';
  receipt.error = result.error?.message;
  for (const row of selected) {
    const file = path.join(output, 'target/package', `${row.name}-${row.version}.crate`);
    if (!fs.existsSync(file)) continue;
    receipt.records.push({ name: row.name, version: row.version, file: path.relative(output, file).replaceAll('\\', '/'),
      sha256: crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'), bytes: fs.statSync(file).size });
  }
}
fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify({ status: receipt.status, prepared: receipt.records.length, selected: selected.length, reused: receipt.reusedRegistry.length, output }));
if (!['packaged-not-compiled'].includes(receipt.status)) process.exitCode = 1;
