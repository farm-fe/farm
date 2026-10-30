// Cache metadata is captured once, before build, and reused without network access in post cleanup.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawnSync } = require('node:child_process');
const digest = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
function commandPlan(args, env = process.env) {
  const [command, ...parameters] = args;
  if (!['cargo', 'rustc', 'rustup'].includes(command)) throw Error('Unsupported cache metadata command');
  if (command === 'cargo' && !(parameters[0] === 'metadata' && parameters.slice(1).every((value, index, values) => ['--all-features', '--format-version', '--no-deps', '--locked'].includes(value) || (value === '1' && values[index - 1] === '--format-version')))) throw Error('Only reviewed Cargo metadata arguments are allowed');
  if (command === 'rustc' && parameters.join(' ') !== '-vV') throw Error('Only Rust compiler identity is allowed');
  if (command === 'rustup' && !(parameters.join(' ') === 'toolchain list --quiet' || (parameters.length === 4 && parameters[0] === 'run' && /^[A-Za-z0-9_.-]+$/.test(parameters[1]) && parameters[2] === 'rustc' && parameters[3] === '-vV'))) throw Error('Only Rust toolchain identity is allowed');
  if (!env.FARM_CARGO_HOME) throw Error('Explicit source Cargo home required');
  const sourceHome = fs.realpathSync(env.FARM_CARGO_HOME);
  if (command === 'cargo' && !parameters.includes('--locked')) parameters.push('--locked');
  return { command, args: parameters, env: { ...env, CARGO_HOME: sourceHome, CARGO_NET_RETRY: '3', CARGO_HTTP_TIMEOUT: '60' } };
}
function validateMetadata(metadata) {
  if (!Array.isArray(metadata.packages) || !Array.isArray(metadata.workspace_members) || !metadata.workspace_members.length) throw Error('Empty workspace metadata is not a cleanup input');
  const members = new Set(metadata.workspace_members);
  if (!metadata.packages.some(row => !members.has(row.id))) throw Error('Full dependency metadata required before target cleanup');
  for (const id of members) if (!metadata.packages.some(row => row.id === id && typeof row.manifest_path === 'string')) throw Error('Workspace member missing from metadata');
  if (new Set(metadata.packages.map(row => row.id)).size !== metadata.packages.length) throw Error('Duplicate package metadata');
  for (const row of metadata.packages) if (typeof row.id !== 'string' || typeof row.name !== 'string' || typeof row.version !== 'string' || typeof row.manifest_path !== 'string' || !Array.isArray(row.targets) || row.targets.some(target => typeof target.name !== 'string' || !Array.isArray(target.kind) || target.kind.some(kind => typeof kind !== 'string'))) throw Error('Invalid package metadata');
}
function metadataInputs(metadata, root) {
  const members = new Set(metadata.workspace_members);
  const files = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'rust-toolchain.toml'].map(file => path.join(root, file));
  for (const row of metadata.packages) {
    const file = fs.realpathSync(row.manifest_path), relative = path.relative(root, file);
    const outside = relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative);
    if (outside) {
      if (members.has(row.id)) throw Error('Workspace manifest outside repository');
      continue; // External registry/Git manifests do not belong to repository inputs.
    }
    files.push(file);
  }
  return [...new Set(files)].sort().map(file => ({ file, sha256: digest(file) }));
}
function captureMetadata({ root = process.cwd(), env = process.env, run = spawnSync } = {}) {
  if (!env.FARM_CARGO_METADATA) throw Error('Explicit metadata snapshot path required');
  const plan = commandPlan(['cargo', 'metadata', '--all-features', '--format-version', '1'], env);
  const result = run(plan.command, plan.args, { cwd: root, env: plan.env, encoding: 'utf8', windowsHide: true, timeout: 600000, maxBuffer: 64 * 1024 ** 2, shell: false });
  if (result.error || result.status !== 0) throw Error(`Locked cache metadata failed: ${result.error?.code ?? result.status}; ${(result.stderr ?? '').slice(-2000)}`);
  const metadata = JSON.parse(result.stdout); validateMetadata(metadata);
  const receipt = { schemaVersion: 1, root: fs.realpathSync(root), inputs: metadataInputs(metadata, fs.realpathSync(root)), metadata };
  fs.writeFileSync(env.FARM_CARGO_METADATA, JSON.stringify(receipt), { flag: 'wx' });
  return receipt;
}
function readMetadata(args, { root = process.cwd(), env = process.env } = {}) {
  commandPlan(args, env);
  if (!env.FARM_CARGO_METADATA) throw Error('Explicit metadata snapshot path required');
  const receipt = JSON.parse(fs.readFileSync(env.FARM_CARGO_METADATA, 'utf8'));
  if (receipt.schemaVersion !== 1 || receipt.root !== fs.realpathSync(root)) throw Error('Metadata snapshot identity differs');
  validateMetadata(receipt.metadata);
  const current = metadataInputs(receipt.metadata, receipt.root);
  if (JSON.stringify(current) !== JSON.stringify(receipt.inputs)) throw Error('Cargo inputs changed after metadata capture; do not save target cache');
  const metadata = receipt.metadata;
  return args.includes('--no-deps') ? { ...metadata, packages: metadata.packages.filter(row => metadata.workspace_members.includes(row.id)) } : metadata;
}
function executeCacheCommand(args, { root = process.cwd(), env = process.env, run = spawnSync } = {}) {
  try {
    if (args[0] === '--snapshot') {
      const receipt = captureMetadata({ root, env, run });
      return { status: 0, stdout: `Captured locked cache metadata: ${receipt.metadata.packages.length} packages\n`, stderr: '' };
    }
    if (args[0] === '--verify-snapshot') {
      const metadata = readMetadata(['cargo', 'metadata'], { root, env });
      return { status: 0, stdout: `Verified cache cleanup metadata: ${metadata.packages.length} packages\n`, stderr: '' };
    }
    if (args[0] === 'cargo') return { status: 0, stdout: JSON.stringify(readMetadata(args, { root, env })), stderr: '' };
    const plan = commandPlan(args, env);
    const result = run(plan.command, plan.args, { env: plan.env, encoding: 'utf8', windowsHide: true, timeout: 600000, maxBuffer: 1024 ** 2, shell: false });
    return { status: result.error?.code === 'ETIMEDOUT' ? 124 : result.error || result.status === null ? 1 : result.status,
      stdout: result.stdout ?? '', stderr: result.error ? `Cache identity command failed: ${result.error.code ?? 'UNKNOWN'}\n` : result.stderr ?? '' };
  } catch (error) { return { status: 1, stdout: '', stderr: `Cache metadata rejected: ${error.message}\n` }; }
}
module.exports = { commandPlan, captureMetadata, readMetadata, executeCacheCommand };
if (require.main === module) {
  const result = executeCacheCommand(process.argv.slice(2));
  process.stdout.write(result.stdout); process.stderr.write(result.stderr); process.exitCode = result.status;
}
