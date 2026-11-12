// cspell:words cffaedfe feedfacf bebafeca
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { verifyInstalledDocsPackages } from './farm-docs-consumer-isolation.mjs';
import { addHomepageVerification, assertHomepageVerification } from './farm-docs-site-verification.mjs';

const repository = fileURLToPath(new URL('../', import.meta.url));
const registry = 'https://registry.npmjs.org';
const exact = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const fields = ['dependencies', 'optionalDependencies', 'peerDependencies'];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const fail = (code, message) => { throw new Error(`[${code}] ${message}`); };
const inside = (root, file) => file === root || (file.startsWith(root + path.sep));

export function deploymentTarget(platform = process.platform, arch = process.arch, glibc = process.report?.getReport().header.glibcVersionRuntime) {
  if (!['darwin', 'linux', 'win32'].includes(platform) || !['x64', 'arm64'].includes(arch)) fail('UNSUPPORTED_TARGET', `${platform}/${arch}`);
  return [platform, arch, platform === 'linux' ? (glibc ? 'gnu' : 'musl') : platform === 'win32' ? 'msvc' : ''].filter(Boolean).join('-');
}

// Manifests provide exact release versions only. No workspace code is installed.
export function releaseManifests(root = repository) {
  const manifests = new Map();
  for (const group of ['packages', 'rust-plugins']) {
    for (const item of fs.readdirSync(path.join(root, group), { withFileTypes: true })) {
      if (!item.isDirectory()) continue;
      const directory = path.join(root, group, item.name);
      const add = directory => {
        const file = path.join(directory, 'package.json');
        if (!fs.existsSync(file)) return;
        const pkg = json(file);
        if (pkg.name?.startsWith('@farmfe/')) manifests.set(pkg.name, pkg);
      };
      add(directory);
      if (fs.existsSync(path.join(directory, 'npm'))) {
        for (const platform of fs.readdirSync(path.join(directory, 'npm'))) add(path.join(directory, 'npm', platform));
      }
    }
  }
  return manifests;
}

// Keep this check dependency-free: the preflight runs before any install.
// Package-manager semver resolution remains authoritative. This grammar admits
// registry ranges (including third-party prereleases), never aliases or sources.
const numeric = '(?:0|[1-9]\\d*)';
const identifier = `(?:${numeric}|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)`;
const completeVersion = `v?${numeric}\\.${numeric}\\.${numeric}(?:-${identifier}(?:\\.${identifier})*)?(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?`;
const partialVersion = `v?(?:[xX*]|${numeric}(?:\\.(?:[xX*]|${numeric}(?:\\.(?:[xX*]|${numeric}))?))?)`;
const registryVersion = `(?:${completeVersion}|${partialVersion})`;
const comparator = `(?:[~^]|[<>]=?|=)?\\s*${registryVersion}`;
const rangeSet = `(?:${registryVersion}\\s+-\\s+${registryVersion}|${comparator}(?:\\s+${comparator})*)`;
const registryRange = new RegExp(`^\\s*${rangeSet}(?:\\s*\\|\\|\\s*${rangeSet})*\\s*$`);
function registrySelector(value) {
  if (typeof value !== 'string' || value.length > 512 || !registryRange.test(value)) fail('REGISTRY_ONLY', 'Dependency must use a public registry semver selector');
}

export function deploymentPlan(website, manifests, target) {
  const direct = { ...website.dependencies, ...website.devDependencies };
  const queue = Object.keys(direct).filter(name => name.startsWith('@farmfe/'));
  const selected = new Map();
  while (queue.length) {
    const name = queue.shift();
    if (selected.has(name)) continue;
    const pkg = manifests.get(name);
    if (!pkg || !exact.test(pkg.version)) fail('EXACT_RELEASE_REQUIRED', name);
    selected.set(name, pkg);
    for (const field of fields) {
      for (const [dependency, range] of Object.entries(pkg[field] ?? {})) {
        if (!dependency.startsWith('@farmfe/')) continue;
        const child = manifests.get(dependency);
        if (!child) fail('UNKNOWN_FARM_DEPENDENCY', `${name} -> ${dependency}`);
        if (child.os && !dependency.endsWith('-' + target)) continue;
        if (!range.startsWith('workspace:') && range !== child.version) fail('COHORT_MISMATCH', `${name} -> ${dependency}`);
        queue.push(dependency);
      }
    }
    const native = Object.keys(pkg.optionalDependencies ?? {}).filter(name => manifests.get(name)?.os);
    if (native.length && !native.includes(`${name}-${target}`)) fail('NATIVE_TARGET_MISSING', `${name}/${target}`);
  }
  for (const name of ['@farmfe/core', '@farmfe/cli']) {
    if (!selected.get(name)?.version.startsWith('2.')) fail('V2_REQUIRED', name);
  }
  for (const [name, range] of Object.entries(direct)) {
    if (!name.startsWith('@farmfe/')) registrySelector(range);
  }
  // Native packages are mandatory here: package-manager optional skipping cannot
  // silently turn an incomplete publication into a source build.
  const dependencies = { ...direct, ...Object.fromEntries([...selected].map(([name, pkg]) => [name, pkg.version])) };
  return { target, selected, manifests, manifest: { name: 'farm-docs-published-consumer', version: '0.0.0', private: true, type: 'module', packageManager: 'pnpm@9.4.0', dependencies } };
}

export async function publicMetadata(name, version) {
  let response;
  try { response = await fetch(`${registry}/${encodeURIComponent(name)}/${version}`, { signal: AbortSignal.timeout(30000) }); }
  catch { fail('REGISTRY_UNAVAILABLE', `${name}@${version}`); }
  if (!response.ok) fail(response.status === 404 ? 'RELEASE_NOT_PUBLISHED' : 'REGISTRY_UNAVAILABLE', `${name}@${version}: HTTP ${response.status}`);
  return response.json();
}

export async function preflight(plan, getMetadata = publicMetadata) {
  const results = await Promise.all([...plan.selected].map(async ([name, local]) => {
    const meta = await getMetadata(name, local.version);
    if (meta.name !== name || meta.version !== local.version) fail('REGISTRY_VERSION_MISMATCH', name);
    const sri = meta.dist?.integrity?.match(/^sha512-([A-Za-z0-9+/]+={0,2})$/);
    if (!sri || Buffer.from(sri[1], 'base64').length !== 64 || Buffer.from(sri[1], 'base64').toString('base64') !== sri[1]) fail('REGISTRY_INTEGRITY_REQUIRED', name);
    if (!meta.dist?.tarball?.startsWith(registry + '/')) fail('REGISTRY_ONLY', `${name}: tarball`);
    for (const field of fields) {
      const actual = Object.entries(meta[field] ?? {}).filter(([name]) => name.startsWith('@farmfe/'));
      const expected = Object.entries(local[field] ?? {}).filter(([name]) => name.startsWith('@farmfe/'));
      assert.deepEqual(actual.map(([name]) => name).sort(), expected.map(([name]) => name).sort(), `Published ${field} changed: ${name}`);
      for (const [dependency, range] of Object.entries(meta[field] ?? {})) {
        registrySelector(range);
        if (dependency.startsWith('@farmfe/') && range !== plan.manifests.get(dependency)?.version) fail('COHORT_MISMATCH', `${name} -> ${dependency}`);
      }
    }
    for (const key of ['preinstall', 'install', 'postinstall', 'prepare']) {
      if (meta.scripts?.[key]) fail('FARM_INSTALL_SCRIPT_FORBIDDEN', `${name}: ${key}`);
    }
    for (const field of ['os', 'cpu', 'libc']) assert.deepEqual(meta[field], local[field], `Published platform metadata changed: ${name}/${field}`);
    return { name, version: meta.version, integrity: meta.dist.integrity };
  }));
  return results.sort((a, b) => a.name.localeCompare(b.name));
}

function lockScalar(value) {
  if (value.startsWith('"')) {
    try { return JSON.parse(value); } catch { fail('REGISTRY_ONLY', 'Unsupported lock scalar escaping'); }
  }
  if (value.startsWith("'")) {
    if (!/^'(?:[^']|'')*'$/.test(value)) fail('REGISTRY_ONLY', 'Malformed quoted lock scalar');
    return value.slice(1, -1).replaceAll("''", "'");
  }
  if (!value || /\s/.test(value)) fail('REGISTRY_ONLY', 'Unsupported lock scalar');
  return value;
}

function assertRegistryTarball(value) {
  if (typeof value !== 'string' || !value.startsWith(registry + '/') || /[\\\s]/.test(value)) fail('REGISTRY_ONLY', 'Lock tarballs must use the public npm registry URL');
  let url;
  try { url = new URL(value); } catch { fail('REGISTRY_ONLY', 'Invalid lock URL'); }
  if (url.origin !== registry || url.username || url.password) fail('REGISTRY_ONLY', 'Lock tarballs must come from the public npm registry');
}

function assertLockResolution(value) {
  // pnpm 9 registry entries use inline resolution maps. Non-registry fetchers
  // add type/directory/repo/commit fields; reject them and unknown YAML shapes.
  if (!value.startsWith('{') || !value.endsWith('}')) fail('REGISTRY_ONLY', 'Unsupported lock resolution map');
  const body = value.slice(1, -1), parts = [];
  let start = 0, quote = '';
  for (let i = 0; i < body.length; i++) {
    const char = body[i];
    if (quote) {
      if (quote === '"' && char === '\\') { i++; continue; }
      if (char === quote) {
        if (quote === "'" && body[i + 1] === "'") { i++; continue; }
        quote = '';
      }
    } else if (char === '"' || char === "'") quote = char;
    else if (char === ',') { parts.push(body.slice(start, i)); start = i + 1; }
  }
  if (quote) fail('REGISTRY_ONLY', 'Malformed lock resolution quoting');
  parts.push(body.slice(start));
  const keys = new Set();
  for (const part of parts) {
    const field = part.trim().match(/^(integrity|tarball):\s*(.+)$/);
    if (!field || keys.has(field[1])) fail('REGISTRY_ONLY', 'Non-registry or ambiguous lock resolution field');
    keys.add(field[1]);
    const scalar = lockScalar(field[2].trim());
    if (field[1] === 'tarball') assertRegistryTarball(scalar);
    else if (typeof scalar !== 'string' || !/^sha(?:1|256|384|512)-[A-Za-z0-9+/]+={0,2}$/.test(scalar)) fail('REGISTRY_ONLY', 'Lock resolution needs registry integrity');
  }
  if (!keys.has('integrity')) fail('REGISTRY_ONLY', 'Lock resolution needs registry integrity');
}

// pnpm emits ordinary one-line YAML scalars. Decode quoted scalars before
// checking protocol boundaries; a package key ending in "-link:" is not a URL.
// Reject YAML features this deliberately small scanner cannot interpret rather
// than allowing a hidden or escaped source through the pre-install check.
export function assertRegistryLock(lock) {
  if (/[\u0000-\u0009\u000B\u000C\u000E-\u001F\u007F-\u009F]/.test(lock) || /\r(?!\n)/.test(lock)) fail('REGISTRY_ONLY', 'Control characters in lock');
  const scalars = [];
  for (const line of lock.split(/\r?\n/)) {
    const resolution = line.match(/^\s*resolution:\s*(.*)$/);
    if (resolution) assertLockResolution(resolution[1].trim());
    let plain = '';
    for (let i = 0; i < line.length;) {
      const char = line[i];
      if (char === '#' && (i === 0 || /\s/.test(line[i - 1]))) break;
      if (char !== '"' && char !== "'") { plain += char; i++; continue; }
      const start = i++;
      let closed = false;
      while (i < line.length) {
        if (char === '"' && line[i] === '\\') { i += 2; continue; }
        if (line[i] === char) {
          if (char === "'" && line[i + 1] === "'") { i += 2; continue; }
          i++; closed = true; break;
        }
        i++;
      }
      if (!closed) fail('REGISTRY_ONLY', 'Multiline or malformed lock scalar');
      const token = line.slice(start, i);
      try {
        const decoded = char === '"' ? JSON.parse(token) : token.slice(1, -1).replaceAll("''", "'");
        if (/^\s*:/.test(line.slice(i)) && decoded === 'resolution') fail('REGISTRY_ONLY', 'Unsupported quoted resolution key');
        scalars.push(decoded);
      }
      catch { fail('REGISTRY_ONLY', 'Unsupported lock scalar escaping'); }
      // Preserve a quoted key's presence when checking standalone ':' syntax.
      plain += '__quoted_scalar__';
    }
    if (/(?:^|[\s,{\[])\?|(?:^|[{,\[])\s*:/.test(plain)) fail('REGISTRY_ONLY', 'Explicit lock YAML mapping keys are unsupported');
    if (/[}\]]\s*:/.test(plain)) fail('REGISTRY_ONLY', 'Collection-valued lock mapping keys are unsupported');
    if (!resolution && /(?:^|[\s,{])resolution\s*:/.test(plain)) fail('REGISTRY_ONLY', 'Unsupported inline resolution key');
    if (/(?:^|[\s,{])(?:[&*!]|<<:)|:\s*[|>][+-]?\s*$/.test(plain)) fail('REGISTRY_ONLY', 'Unsupported lock YAML feature');
    // Remove YAML mapping separators, not protocol colons (which have an
    // immediate target). For example engines: {npm: '>=6'} contains a key.
    scalars.push(...plain.split(/:(?=\s|$|[{}\[\],])/));
  }
  const source = /(?:^|[\s,:({[\]}@])(?:workspace:|link:|file:|portal:|patch:|npm:|github:|gitlab:|bitbucket:|git\+|git:|git@|ssh:)/i;
  const urls = /(?:^|[\s,:({[\]}@])(https?:\/\/[^\s,{}\[\]]+)/gi;
  for (const scalar of scalars) {
    if (/[\u0000-\u001F\u007F-\u009F]/.test(scalar)) fail('REGISTRY_ONLY', 'Decoded control characters in lock');
    if (source.test(scalar)) fail('REGISTRY_ONLY', 'Generated consumer lock contains a source dependency');
    for (const match of scalar.matchAll(urls)) {
      assertRegistryTarball(match[1]);
    }
  }
}

function isolatedEnvironment(environment) {
  for (const key of ['NODE_PATH', 'NAPI_RS_NATIVE_LIBRARY_PATH', 'NAPI_RS_FORCE_WASI']) if (environment[key]) fail('BINDING_OVERRIDE_FORBIDDEN', key);
  if ((environment.NODE_OPTIONS ?? '').trim().split(/\s+/).filter(Boolean).some(option => !/^--max[-_]old[-_]space[-_]size=\d+$/.test(option) && !['--no-warnings', '--trace-warnings', '--enable-source-maps'].includes(option))) fail('BINDING_OVERRIDE_FORBIDDEN', 'NODE_OPTIONS');
  return { ...environment, NODE_ENV: 'production', NODE_OPTIONS: environment.NODE_OPTIONS || '--max-old-space-size=4096', npm_config_registry: registry, npm_config_ignore_scripts: 'true', ELECTRON_SKIP_BINARY_DOWNLOAD: '1', PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: '1' };
}

function execute(command, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { ...options, stdio: 'inherit', windowsHide: true });
    child.once('error', reject);
    child.once('exit', code => code === 0 ? resolve() : reject(new Error(`[DOCS_COMMAND_FAILED] ${command}: exit ${code}`)));
  });
}

function copyWebsite(source, destination) {
  const excluded = new Set(['node_modules', 'build-farm', '.farm', '.git', '.vercel', 'package.json', 'pnpm-lock.yaml', 'vercel.json']);
  fs.cpSync(source, destination, { recursive: true, filter: file => {
    const relative = path.relative(source, file);
    if (relative && (excluded.has(relative.split(path.sep)[0]) || relative.startsWith('build-farm.previous-') || relative.split(path.sep).some(part => part.startsWith('.env')))) return false;
    if (fs.lstatSync(file).isSymbolicLink()) fail('SOURCE_SYMLINK_FORBIDDEN', relative);
    return true;
  } });
}

export async function buildPublishedDocs({ root = repository, phase = 'build', target = deploymentTarget(), environment = process.env } = {}, hooks = {}) {
  if (!['install', 'build', 'preflight'].includes(phase)) fail('UNKNOWN_PHASE', phase);
  const env = isolatedEnvironment(environment);
  const website = path.join(root, 'website');
  const plan = deploymentPlan(json(path.join(website, 'package.json')), releaseManifests(root), target);
  const packages = await preflight(plan, hooks.getMetadata);
  console.log(`Published Farm v2 preflight: ${packages.length} exact packages for ${target}.`);
  // Vercel's install phase must never install the monorepo. The build phase
  // performs its entire install/build in one disposable independent directory.
  if (phase !== 'build') return { phase, target, packages, commandsRun: 0 };
  const run = hooks.execute ?? execute;
  const consumer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-published-')));
  try {
    if (inside(fs.realpathSync(root), consumer)) fail('WORKSPACE_FORBIDDEN', 'Temporary consumer must be outside checkout');
    for (let parent = consumer; ; parent = path.dirname(parent)) {
      if (fs.existsSync(path.join(parent, 'pnpm-workspace.yaml')) || (fs.existsSync(path.join(parent, 'package.json')) && json(path.join(parent, 'package.json')).workspaces)) fail('WORKSPACE_FORBIDDEN', parent);
      if (parent === path.dirname(parent)) break;
    }
    copyWebsite(website, consumer);
    fs.writeFileSync(path.join(consumer, 'package.json'), JSON.stringify(plan.manifest, null, 2) + '\n');
    fs.writeFileSync(path.join(consumer, '.npmrc'), `registry=${registry}/\nlink-workspace-packages=false\nprefer-workspace-packages=false\nauto-install-peers=false\nignore-scripts=true\n`);
    const options = { cwd: consumer, env };
    await run('corepack', ['pnpm', 'install', '--lockfile-only', '--ignore-scripts', '--ignore-workspace'], options);
    const lock = fs.readFileSync(path.join(consumer, 'pnpm-lock.yaml'), 'utf8');
    assertRegistryLock(lock);
    for (const pkg of packages) if (!lock.includes(pkg.integrity)) fail('LOCK_INTEGRITY_MISMATCH', pkg.name);
    await run('corepack', ['pnpm', 'install', '--frozen-lockfile', '--ignore-scripts', '--ignore-workspace'], options);
    const resolved = await verifyInstalledDocsPackages(consumer, { repository: root, env, packages: packages.map(pkg => ({ ...pkg, specifier: plan.selected.get(pkg.name).exports?.['./package.json'] ? pkg.name + '/package.json' : pkg.name })) });
    for (const pkg of resolved.packages) {
      const local = plan.selected.get(pkg.name);
      if (local.os) {
        const bytes = fs.readFileSync(pkg.entry);
        const signature = bytes.subarray(0, 4).toString('hex');
        const valid = target.startsWith('linux') ? signature === '7f454c46' : target.startsWith('win32') ? signature.startsWith('4d5a') : ['cffaedfe', 'feedfacf', 'cafebabe', 'bebafeca'].includes(signature);
        if (!valid) fail('NATIVE_PAYLOAD_FORMAT', pkg.name);
      }
      if (!local.os && pkg.name.startsWith('@farmfe/plugin-') && fs.existsSync(path.join(path.dirname(pkg.manifest), 'npm'))) fail('UNEXPECTED_LOCAL_NATIVE', pkg.name);
      if (pkg.name === '@farmfe/core' && fs.readdirSync(path.join(path.dirname(pkg.manifest), 'binding')).some(file => file.endsWith('.node'))) fail('UNEXPECTED_LOCAL_NATIVE', pkg.name);
    }
    const cli = resolved.packages.find(pkg => pkg.name === '@farmfe/cli');
    const cliFile = path.join(path.dirname(cli.manifest), 'bin/farm.mjs');
    await run(process.execPath, [cliFile, '--help'], options);
    await run(process.execPath, [cliFile, 'build', '--config', 'farm.config.ts'], options);
    const built = path.join(consumer, 'build-farm');
    const homepage = path.join(built, 'index.html');
    const html = fs.readFileSync(homepage, 'utf8');
    assert.ok(html.length > 0, 'Website index output is empty');
    assert.match(html, /THIRD_PARTY_NOTICES\.txt/);
    assert.deepEqual(fs.readFileSync(path.join(built, 'THIRD_PARTY_NOTICES.txt')), fs.readFileSync(path.join(website, 'static/THIRD_PARTY_NOTICES.txt')), 'Published notices changed');
    // Site-only finalization after real compilation, before any existing output
    // is moved. Published Farm package templates and other pages stay untouched.
    fs.writeFileSync(homepage, addHomepageVerification(html));
    assertHomepageVerification(fs.readFileSync(homepage, 'utf8'));
    const receipt = { schemaVersion: 1, registry, target, packages, lockSha256: hash(lock), nativePackages: resolved.packages.filter(pkg => plan.selected.get(pkg.name).os).map(({ name, version, entrySha256 }) => ({ name, version, sha256: entrySha256 })), commandsRun: 4 };
    const output = path.join(website, 'build-farm');
    // Copy only after build and notice validation succeed. Preserve an existing
    // output as a recoverable sibling instead of deleting the user's files.
    if (fs.existsSync(output)) fs.renameSync(output, path.join(website, `build-farm.previous-${Date.now()}`));
    fs.cpSync(built, output, { recursive: true });
    fs.writeFileSync(path.join(output, 'farm-docs-build-receipt.json'), JSON.stringify(receipt, null, 2) + '\n');
    console.log(`Published Farm v2 website built: ${output}`);
    return receipt;
  } finally {
    fs.rmSync(consumer, { recursive: true, force: true });
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length > 1 || (args.length && !['--install', '--preflight'].includes(args[0]))) fail('ARGUMENTS', 'Use [--install|--preflight]');
  if (Number(process.versions.node.split('.')[0]) !== 22) fail('NODE_VERSION', 'Node 22 is required for the published docs build');
  await buildPublishedDocs({ phase: args[0] === '--install' ? 'install' : args[0] === '--preflight' ? 'preflight' : 'build' });
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) main().catch(error => { console.error(error.message); process.exitCode = 1; });
