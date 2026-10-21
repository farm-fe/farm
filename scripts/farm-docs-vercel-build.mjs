import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const shared = ['utils', 'runtime', 'runtime-plugin-hmr', 'runtime-plugin-import-meta', 'plugin-tools'];
const plugins = ['replace-dirname', 'react', 'mdx', 'ssr-css', 'syntax-highlight', 'sass', 'tailwindcss', 'docs', 'docs-snapshot', 'docs-seo', 'docs-snippets'];
export const buildStages = [
  ...shared.map(name => ({ id: name, directory: `packages/${name}`, script: 'build' })),
  { id: 'core:native', directory: 'packages/core', script: 'build:rs' },
  ...plugins.map(name => ({ id: `native:${name}`, directory: `rust-plugins/${name}`, script: 'build' })),
  ...['core', 'cli', 'docs', 'docs-theme-default'].map(name => ({ id: `${name}:public`, directory: `packages/${name}`, script: 'build' })),
  { id: 'artifacts:verify', directory: '.', verifyArtifacts: true },
  { id: 'website', directory: 'website', script: 'build:farm' },
  { id: 'notices', directory: '.', nodeScript: 'scripts/farm-docs-notices.test.mjs' }
];

export async function runStages(stages, run) {
  for (const stage of stages) {
    console.log(`[farm-docs-deploy] ${stage.id}: starting`);
    await run(stage);
    console.log(`[farm-docs-deploy] ${stage.id}: completed`);
  }
}

function command(executable, args, { cwd = root, env = process.env } = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, { cwd, env, stdio: 'inherit', windowsHide: true });
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`[farm-docs-deploy] ${executable} ${args.join(' ')} failed: ${signal ?? code}`));
    });
  });
}

function packageCommand(executable, args, options) {
  // All commands and package-script arguments here are fixed repository values.
  if (process.platform === 'win32') return command(process.env.ComSpec ?? 'cmd.exe', ['/d', '/s', '/c', [executable, ...args].join(' ')], options);
  return command(executable, args, options);
}

export function parseToolchain(text) {
  const channel = text.match(/^channel\s*=\s*"([^"]+)"/m)?.[1];
  const components = text.match(/^components\s*=\s*(\[[^\]]*\])/m)?.[1];
  if (!channel || !components) throw new Error('[farm-docs-deploy] Cannot read the repository Rust toolchain');
  return { channel, components: JSON.parse(components) };
}

async function prepareRust(env) {
  const cargoBin = path.join(env.CARGO_HOME ?? path.join(os.homedir(), '.cargo'), 'bin');
  env.PATH = cargoBin + path.delimiter + env.PATH;
  const probe = spawnSync('rustup', ['--version'], { env, encoding: 'utf8', windowsHide: true });
  if (probe.error?.code === 'ENOENT') {
    if (process.platform !== 'linux') throw new Error('[farm-docs-deploy] Install rustup before running the deployment build on this platform');
    const response = await fetch('https://sh.rustup.rs');
    if (!response.ok) throw new Error(`[farm-docs-deploy] rustup installer HTTP ${response.status}`);
    const installer = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-rustup-')), 'rustup-init.sh');
    fs.writeFileSync(installer, Buffer.from(await response.arrayBuffer()));
    await command('sh', [installer, '-y', '--profile', 'minimal', '--default-toolchain', 'none', '--no-modify-path'], { env });
  } else if (probe.error || probe.status !== 0) {
    throw probe.error ?? new Error(`[farm-docs-deploy] rustup probe failed: ${probe.stderr}`);
  }
  const { channel, components } = parseToolchain(fs.readFileSync(path.join(root, 'rust-toolchain.toml'), 'utf8'));
  env.RUSTUP_TOOLCHAIN = channel;
  await command('rustup', ['toolchain', 'install', channel, '--profile', 'minimal', ...components.flatMap(name => ['--component', name])], { env });
  await command('cargo', ['--version'], { env });
  await command('rustc', ['--version'], { env });
}

async function verifyArtifacts() {
  const require = createRequire(path.join(root, 'package.json'));
  const coreDirectory = path.join(root, 'packages/core/binding') + path.sep;
  const core = fs.realpathSync(require('./packages/core/binding/resolve-binding.cjs'));
  if (!core.startsWith(coreDirectory)) throw new Error('[farm-docs-deploy] Core resolved outside this checkout binding directory');
  require('./packages/core/binding/binding.cjs');
  const binaries = [{ name: 'core', file: core }];
  for (const name of plugins) {
    const directory = path.join(root, 'rust-plugins', name);
    const module = await import(pathToFileURL(path.join(directory, 'scripts/index.js')).href);
    const file = fs.realpathSync(module.default);
    if (!file.startsWith(directory + path.sep)) throw new Error(`[farm-docs-deploy] ${name} resolved outside this checkout plugin directory`);
    binaries.push({ name, file });
  }
  for (const { name, file } of binaries) {
    const bytes = fs.readFileSync(file);
    if (!bytes.length) throw new Error(`[farm-docs-deploy] Empty native artifact: ${name}`);
    console.log(`[farm-docs-deploy] native artifact ${name}: ${file} sha256=${createHash('sha256').update(bytes).digest('hex')}`);
  }
  await command(process.execPath, ['packages/cli/bin/farm.mjs', '--help']);
}

async function main() {
  if (process.argv.includes('--plan')) {
    console.log(JSON.stringify(buildStages, null, 2));
    return;
  }
  if (Number(process.versions.node.split('.')[0]) !== 22) throw new Error('[farm-docs-deploy] Node.js 22 is required; see website/package.json');
  const env = { ...process.env, ELECTRON_SKIP_BINARY_DOWNLOAD: '1', PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: '1', CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2' };
  if (!env.NODE_OPTIONS?.includes('--max-old-space-size')) env.NODE_OPTIONS = `${env.NODE_OPTIONS ?? ''} --max-old-space-size=2048`.trim();
  if (process.argv.includes('--install')) {
    // Vercel starts in website/. Install at the workspace root with its pinned pnpm.
    await packageCommand('corepack', ['pnpm', 'install', '--frozen-lockfile'], { env });
    return;
  }
  if (process.argv.includes('--verify-artifacts')) { await verifyArtifacts(); return; }
  await prepareRust(env);
  await runStages(buildStages, stage => stage.verifyArtifacts ? verifyArtifacts() : stage.nodeScript
    ? command(process.execPath, [stage.nodeScript], { env })
    : packageCommand('npm', ['run', stage.script], { cwd: path.join(root, stage.directory), env }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
