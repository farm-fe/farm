import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { captureNativeProvenance } from './native-provenance.mjs';
import { captureSourceProvenance } from './source-provenance.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
function command(binary, args) {
  try {
    return execFileSync(binary, args, {
      cwd: root, encoding: 'utf8', windowsHide: true,
      timeout: 30000, stdio: ['ignore', 'pipe', 'pipe'],
    }).trim();
  } catch (error) {
    return { unavailable: true, error: error.message };
  }
}

export async function captureEnvironment() {
  const lock = await fs.readFile(path.join(root, 'pnpm-lock.yaml'));
  const packageFiles = [
    'packages/core/package.json', 'packages/cli/package.json',
    'examples/commerce-lab/package.json',
    ...['react', 'worker', 'auto-import', 'svgr', 'sass', 'yaml', 'dsv', 'mdx', 'tailwindcss', 'compress']
      .map(name => `rust-plugins/${name}/package.json`),
    ...['svgr', 'sass', 'tailwindcss'].map(name => `js-plugins/${name}/package.json`),
  ];
  const packages = {};
  for (const relative of packageFiles) {
    const pkg = JSON.parse(await fs.readFile(path.join(root, relative), 'utf8'));
    packages[pkg.name] = { version: pkg.version, source: relative };
  }
  const cpus = os.cpus();
  return {
    timestamp: new Date().toISOString(),
    platform: os.platform(), architecture: os.arch(), release: os.release(),
    cpu: { model: cpus[0]?.model ?? null, logicalCores: cpus.length },
    totalMemoryBytes: os.totalmem(),
    node: process.version,
    packageManager: JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8')).packageManager ?? null,
    pnpm: process.platform === 'win32'
      ? command('cmd.exe', ['/d', '/s', '/c', 'pnpm --version'])
      : command('pnpm', ['--version']),
    rustc: command('rustc', ['--version', '--verbose']),
    commit: command('git', ['rev-parse', 'HEAD']),
    workingTree: command('git', ['status', '--porcelain']),
    lockSha256: createHash('sha256').update(lock).digest('hex'),
    packages,
    native: await captureNativeProvenance(),
    source: await captureSourceProvenance(),
    provenance: 'Workspace source versions plus selected native file hashes; does not establish npm consumption acceptance.',
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  process.stdout.write(`${JSON.stringify(await captureEnvironment(), null, 2)}\n`);
