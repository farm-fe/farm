import { resolveNpmCli } from './v2-npm-cli.mjs';
// cspell:words statfs bavail userconfig USERCONFIG
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { command } from './performance-report.mjs';
import { publicationNpm } from './v2-release-inventory.mjs';
import { candidateNpmEvidence, verifyInstalledNpmLock } from './v2-consumer-registry.mjs';

export function publicConsumerManifest(plan) {
  const names = [...new Set(['@farmfe/core', '@farmfe/cli', '@farmfe/docs', '@farmfe/docs-theme-default', 'create-farm', ...publicationNpm(plan).primary.map(row => row.name)])];
  const dependencies = Object.fromEntries(names.map(name => {
    const pkg = plan.npm.primary.find(row => row.name === name);
    assert.ok(pkg && /^\d+\.\d+\.\d+$/.test(pkg.version), `Missing stable public consumer dependency: ${name}`);
    return [name, pkg.version];
  }));
  return { name: 'farm-public-registry-consumer', version: '0.0.0', private: true, type: 'module', dependencies,
    devDependencies: { typescript: '5.6.3', '@types/node': '18.19.67' } };
}
export async function cleanupConsumerScratch(scratch, report, remove = fs.rm) {
  if (report.cleanupConfirmed === false) {
    report.scratchRemoved = false;
    report.scratchRetainedReason = 'Owned process cleanup is unconfirmed; retain inputs and diagnostics';
    report.status = 'failed';
    return;
  }
  try { await remove(scratch, { recursive: true, force: true }); report.scratchRemoved = true; }
  catch (error) { report.status = 'failed'; report.scratchRemoved = false; report.scratchRetainedReason = `Scratch cleanup failed: ${error.message}`; }
}
export function assertPublicRustLock(receipt, lock) {
  assert.ok(!/name = "(?:wax|tardar)"/.test(lock), 'Removed dependency returned in public registry consumer');
  for (const record of receipt.rust) {
    const block = lock.split('[[package]]').find(part => part.includes(`name = "${record.name}"\n`) && part.includes(`version = "${record.version}"\n`));
    assert.ok(block?.includes('source = "registry+https://github.com/rust-lang/crates.io-index"') && block.includes(`checksum = "${record.sha256}"`), `Public Rust registry source/hash mismatch: ${record.name}`);
  }
}
async function main() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const directory = process.env.FARM_RELEASE_CANDIDATE_DIRECTORY;
  assert.ok(directory, 'Expected verified sealed candidate directory');
  const plan = JSON.parse(await fs.readFile(path.join(root, 'docs/releases/v2-candidate.json')));
  const receipt = JSON.parse(await fs.readFile(path.join(directory, 'receipt.json')));
  const attempt = Number(process.env.GITHUB_RUN_ATTEMPT ?? Date.now()); assert.ok(Number.isSafeInteger(attempt) && attempt > 0);
  const parent = path.join(directory, 'public-consumer'); await fs.mkdir(parent, { recursive: true });
  const output = path.join(parent, `attempt-${attempt}`); await fs.mkdir(output); // Preserve previous attempts.
  const scratch = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-public-consumer-'));
  const report = { status: 'running', publicRegistry: true, workspacePatches: false, scratch, stages: [], startedAt: new Date().toISOString() };
  const reportFile = path.join(output, 'result.json');
  const deadline = Date.now() + 100 * 60 * 1000;
  const env = { ...process.env, CARGO_HOME: path.join(scratch, 'cargo-home'), CARGO_TARGET_DIR: path.join(scratch, 'target'), CARGO_REGISTRIES_CRATES_IO_PROTOCOL: 'sparse', npm_config_cache: path.join(scratch, '.npm-cache') };
  for (const key of ['NPM_TOKEN', 'NODE_AUTH_TOKEN', 'CARGO_REGISTRY_TOKEN', 'GH_TOKEN', 'GITHUB_TOKEN', 'CARGO_NET_OFFLINE', 'NAPI_RS_NATIVE_LIBRARY_PATH', 'FARM_LIB_CORE_PATH']) delete env[key];
  async function run(stage, executable, args, cwd = scratch, timeout = 120000) {
    const disk = await fs.statfs(scratch);
    assert.ok(os.freemem() >= 3.4 * 1024 ** 3 && disk.bavail * disk.bsize >= 9.75 * 1024 ** 3, 'Public consumer resource guard');
    assert.ok(Date.now() < deadline, 'Public consumer time budget exhausted');
    await command(executable, args, { cwd, env, log: path.join(output, `${stage}.log`), timeoutMs: Math.min(timeout, deadline - Date.now()) });
    report.stages.push(stage); await fs.writeFile(reportFile, JSON.stringify(report, null, 2));
  }
  try {
    await fs.writeFile(path.join(scratch, 'package.json'), JSON.stringify(publicConsumerManifest(plan), null, 2));
    await fs.writeFile(path.join(scratch, 'empty-user.npmrc'), '');
    env.NPM_CONFIG_USERCONFIG = path.join(scratch, 'empty-user.npmrc');
    await fs.writeFile(path.join(scratch, '.npmrc'), 'registry=https://registry.npmjs.org/\n@farmfe:registry=https://registry.npmjs.org/\nignore-scripts=true\n');
    const npm = resolveNpmCli();
    await run('npm-install', process.execPath, [npm, 'install', '--ignore-scripts', '--registry=https://registry.npmjs.org'], scratch, 600000);
    const lock = JSON.parse(await fs.readFile(path.join(scratch, 'package-lock.json')));
    for (const [name, pkg] of Object.entries(lock.packages ?? {})) {
      assert.ok(!pkg.link, `Linked package in public registry consumer: ${name}`);
      if (pkg.resolved) assert.equal(new URL(pkg.resolved).hostname, 'registry.npmjs.org');
    }
    verifyInstalledNpmLock(plan, publicConsumerManifest(plan), lock, await candidateNpmEvidence(plan, receipt, directory), { registry: 'https://registry.npmjs.org/' });
    await fs.copyFile(path.join(scratch, 'package-lock.json'), path.join(output, 'package-lock.json'));
    await fs.writeFile(path.join(scratch, 'probe.mjs'), `import assert from 'node:assert/strict'; import { createRequire } from 'node:module'; import fs from 'node:fs'; import path from 'node:path'; import * as core from '@farmfe/core'; import { defineDocsConfig } from '@farmfe/docs'; const require=createRequire(import.meta.url); assert.equal(typeof core.build,'function'); assert.equal(typeof require('@farmfe/core').build,'function'); assert.equal(typeof defineDocsConfig,'function'); for(const name of ['@farmfe/core','@farmfe/docs']) assert.ok(fs.realpathSync(require.resolve(name)).startsWith(path.join(process.cwd(),'node_modules')+path.sep));`);
    await run('esm-cjs', process.execPath, ['probe.mjs']);
    await fs.writeFile(path.join(scratch, 'probe.mts'), `import {build} from '@farmfe/core'; import {defineDocsConfig} from '@farmfe/docs'; defineDocsConfig({title:'Registry',url:'https://example.invalid'});\n// @ts-expect-error wrong argument must remain an error\nbuild(42);\n// @ts-expect-error title must remain typed\ndefineDocsConfig({title:42,url:'https://example.invalid'});\n`);
    await run('types', process.execPath, ['node_modules/typescript/bin/tsc', '--noEmit', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', '--skipLibCheck', '--strict', 'probe.mts']);
    await fs.mkdir(path.join(scratch, 'docs'));
    await fs.writeFile(path.join(scratch, 'docs/quick.md'), '# Public registry verified\n\nIndependent stable installation.\n');
    await fs.writeFile(path.join(scratch, 'farm.config.mjs'), `import {defineDocsConfig} from '@farmfe/docs'; export default defineDocsConfig({title:'Registry verification',url:'https://example.invalid',docs:{collections:[{id:'docs',dir:'docs',routeBase:'/docs/'}]}},{compilation:{persistentCache:false,output:{path:'dist'}}});`);
    await run('docs-build', process.execPath, ['node_modules/@farmfe/cli/bin/farm.mjs', 'build', '--config', 'farm.config.mjs'], scratch, 300000);
    const html = [];
    async function scan(dir) { for (const item of await fs.readdir(dir, { withFileTypes: true })) { const file = path.join(dir, item.name); if (item.isDirectory()) await scan(file); else if (item.name.endsWith('.html')) html.push(await fs.readFile(file, 'utf8')); } }
    await scan(path.join(scratch, 'dist')); assert.ok(html.some(text => text.includes('Public registry verified')), 'Docs registry build lacks expected rendered content');
    const sdk = path.join(scratch, 'sdk'); await fs.cp(path.join(root, 'scripts/release-fixtures/sdk-consumer'), sdk, { recursive: true });
    await fs.rm(path.join(sdk, 'Cargo.lock'), { force: true });
    await fs.mkdir(env.CARGO_HOME);
    const cargo = process.env.FARM_CARGO_EXECUTABLE ?? 'cargo';
    await run('sdk', cargo, ['+nightly-2026-04-10', 'test', '--release', '--jobs', '1'], sdk, 90 * 60 * 1000);
    const cargoLock = (await fs.readFile(path.join(sdk, 'Cargo.lock'), 'utf8')).replaceAll('\r\n', '\n');
    assertPublicRustLock(receipt, cargoLock); await fs.writeFile(path.join(output, 'Cargo.lock'), cargoLock);
    assert.match(await fs.readFile(path.join(output, 'sdk.log'), 'utf8'), /test result: ok\. 3 passed; 0 failed/);
    report.status = 'passed';
  } catch (error) { report.status = 'failed'; report.error = error.message; report.cleanupConfirmed = error.cleanupConfirmed !== false; report.cleanupError = error.cleanupError ?? null; report.ownedPid = error.pid ?? null; process.exitCode = 1; }
  finally {
    // Scratch belongs exclusively to this command, not a user workspace/cache.
    await cleanupConsumerScratch(scratch, report);
    if (report.status !== 'passed') process.exitCode = 1;
    report.finishedAt = new Date().toISOString(); await fs.writeFile(reportFile, JSON.stringify(report, null, 2));
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
