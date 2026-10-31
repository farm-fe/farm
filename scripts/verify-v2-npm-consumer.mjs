import { resolveNpmCli } from './v2-npm-cli.mjs';
import { publicationManifestDigest } from './incremental-release-producer.mjs';
// cspell:words statfs bavail userconfig USERCONFIG
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { command } from './performance-report.mjs';
import { publicConsumerManifest, cleanupConsumerScratch, assertConsumerPackagePath } from './verify-published-v2-consumers.mjs';
import { candidateNpmEvidence, serveCandidateRegistry, verifyInstalledNpmLock, consumerHostAbi, candidateInstallArgs } from './v2-consumer-registry.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const [sealArg, outputArg] = process.argv.slice(2);
assert.ok(sealArg && outputArg, 'Specify seal directory and new consumer evidence directory');
const seal = path.resolve(sealArg), output = path.resolve(outputArg);
await fs.mkdir(output); // Never overwrite prior consumer evidence.
const sourcePlan = JSON.parse(await fs.readFile(path.join(root, 'docs/releases/v2-candidate.json')));
let plan = sourcePlan;
try {
  plan = JSON.parse(await fs.readFile(path.join(seal, 'candidate-plan.json'), 'utf8'));
  assert.deepEqual(plan.npm, sourcePlan.npm, 'Portable candidate npm family differs from source');
  assert.deepEqual(plan.cargo, sourcePlan.cargo, 'Portable candidate Cargo family differs from source');
  assert.equal(plan.abi, sourcePlan.abi);
} catch (error) { if (error.code !== 'ENOENT') throw error; }

const receipt = JSON.parse(await fs.readFile(path.join(seal, 'receipt.json')));
let releaseBinding = null;
try {
  const manifest = JSON.parse(await fs.readFile(path.join(seal, 'manifest.json'), 'utf8'));
  assert.equal(manifest.chain, 'release'); assert.equal(manifest.profile, 'release'); assert.equal(manifest.namespace, 'farm-release-v1');
  assert.equal(manifest.sourceSHA, receipt.testedCommit); assert.equal(manifest.sourceTree, receipt.testedTree);
  assert.match(process.env.GITHUB_RUN_ID ?? '', /^[1-9][0-9]*$/); assert.match(process.env.GITHUB_RUN_ATTEMPT ?? '', /^[1-9][0-9]*$/);
  releaseBinding = { manifestHash: publicationManifestDigest(manifest), sourceSHA: manifest.sourceSHA, sourceTree: manifest.sourceTree,
    producerRunId: manifest.producerRunId, producerAttempt: manifest.producerAttempt,
    consumerRunId: Number(process.env.GITHUB_RUN_ID), consumerAttempt: Number(process.env.GITHUB_RUN_ATTEMPT) };
} catch (error) { if (error.code !== 'ENOENT') throw error; }
if (process.env.GITHUB_SHA) assert.equal(receipt.testedCommit, process.env.GITHUB_SHA, 'Candidate producer SHA differs from this run');
const scratch = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-npm-candidate-consumer-'));
const result = { status: 'running', sourceCommit: receipt.sourceCommit, testedCommit: receipt.testedCommit, scratch,
  releaseBinding,
  installation: 'real npm install from exact sealed loopback registry plus verified immutable registry reuse',
  externalRegistryCacheLinks: 0, workspacePackageLinks: 0, nativeOverrides: false, stages: [],
  platform: { os: process.platform, arch: process.arch, abi: consumerHostAbi() }, scope: 'All selected primary packages install; only host native ABI executes. Other targets have static sealed archive/hash evidence, not runtime proof.' };
let registry;
const env = { ...process.env, npm_config_cache: path.join(scratch, '.npm-cache') };
for (const key of ['NPM_TOKEN', 'NODE_AUTH_TOKEN', 'GH_TOKEN', 'GITHUB_TOKEN', 'NAPI_RS_NATIVE_LIBRARY_PATH', 'FARM_LIB_CORE_PATH', 'npm_config_registry', 'NPM_CONFIG_REGISTRY', 'NPM_CONFIG_USERCONFIG', 'npm_config_userconfig']) delete env[key];
const deadline = Date.now() + 20 * 60 * 1000;
async function run(stage, args, timeout = 120000) {
  const disk = await fs.statfs(scratch); assert.ok(disk.bavail * disk.bsize >= 8 * 1024 ** 3, 'Candidate consumer disk reserve');
  assert.ok(Date.now() < deadline, 'Candidate consumer deadline exceeded');
  await command(process.execPath, args, { cwd: scratch, env, log: path.join(output, `${stage}.log`), timeoutMs: Math.min(timeout, deadline - Date.now()) });
  result.stages.push(stage);
}
try {
  const evidence = await candidateNpmEvidence(plan, receipt, seal);
  registry = await serveCandidateRegistry(evidence);
  const manifest = publicConsumerManifest({ ...plan, publication: undefined });
  await fs.writeFile(path.join(scratch, 'package.json'), JSON.stringify(manifest, null, 2));
  await fs.writeFile(path.join(scratch, 'empty-user.npmrc'), '');
  env.NPM_CONFIG_USERCONFIG = path.join(scratch, 'empty-user.npmrc');
  await fs.writeFile(path.join(scratch, '.npmrc'), `registry=${registry.url}\n@farmfe:registry=${registry.url}\nignore-scripts=true\naudit=false\nfund=false\n`);
  const npm = resolveNpmCli();
  await run('install', candidateInstallArgs(npm, registry.url), 10 * 60 * 1000);
  const lock = JSON.parse(await fs.readFile(path.join(scratch, 'package-lock.json')));
  verifyInstalledNpmLock(plan, manifest, lock, evidence, { registry: registry.url });
  await fs.copyFile(path.join(scratch, 'package-lock.json'), path.join(output, 'package-lock.json'));
  result.installedPrimary = Object.entries(manifest.dependencies).map(([name, version]) => ({ name, version, integrity: evidence.get(name).integrity }));
  result.nativeTargetEvidence = plan.npm.payloads.map(row => ({ name: row.name, version: row.version, integrity: evidence.get(row.name).integrity, proof: evidence.get(row.name).kind }));
  const hostAbi = consumerHostAbi();
  result.platform.abi = hostAbi;
  const hostCore = plan.npm.payloads.find(row => row.owner === '@farmfe/core' && row.platform === hostAbi);
  assert.ok(hostCore, `Unsupported consumer native target: ${hostAbi}`);
  const expectedNative = evidence.get(hostCore.name);
  await fs.writeFile(path.join(scratch, 'probe.mjs'), `import assert from 'node:assert/strict'; import path from 'node:path'; import fs from 'node:fs'; import {createRequire} from 'node:module'; import * as esm from '@farmfe/core'; import {defineDocsConfig} from '@farmfe/docs'; const require=createRequire(import.meta.url), cjs=require('@farmfe/core'); assert.equal(typeof esm.build,'function');assert.equal(typeof cjs.build,'function');assert.equal(typeof defineDocsConfig,'function'); const identity=await esm.compilerNativeIdentity(); const assertConsumerPackagePath=${assertConsumerPackagePath.toString()};const nativePath=assertConsumerPackagePath(process.cwd(),${JSON.stringify(hostCore.name)},identity.path,fs,path);if(${JSON.stringify(expectedNative.nativeSha256 ?? null)})assert.equal(identity.sha256,${JSON.stringify(expectedNative.nativeSha256 ?? null)}); const generator=require('create-farm');assert.equal(typeof generator.run,'function');fs.writeFileSync('runtime-proof.json',JSON.stringify({esmCjs:true,nativeIdentity:identity,nativePath,createFarm:true}));`);
  // create-farm is included explicitly for stable support/entry verification.
  if (!manifest.dependencies['create-farm']) throw new Error('Candidate consumer support set lacks create-farm');
  await run('runtime', ['probe.mjs']);
  await fs.copyFile(path.join(scratch, 'runtime-proof.json'), path.join(output, 'runtime-proof.json'));
  await run('create-farm-help', ['node_modules/create-farm/create-farm.js', '--help']);
  await fs.writeFile(path.join(scratch, 'consumer.mts'), `import {build} from '@farmfe/core';import {defineDocsConfig} from '@farmfe/docs';import {run} from 'create-farm';void run;defineDocsConfig({title:'Candidate',url:'https://example.invalid'});\n// @ts-expect-error wrong input must remain an error\nbuild(42);\n`);
  await run('types', ['node_modules/typescript/bin/tsc', '--noEmit', '--strict', '--skipLibCheck', '--module', 'NodeNext', '--moduleResolution', 'NodeNext', 'consumer.mts']);
  await fs.mkdir(path.join(scratch, 'docs'));
  await fs.writeFile(path.join(scratch, 'docs/quick.md'), '# Sealed candidate installed\n\nIndependent npm installation.\n');
  await fs.writeFile(path.join(scratch, 'farm.config.mjs'), `import {defineDocsConfig} from '@farmfe/docs';export default defineDocsConfig({title:'Candidate',url:'https://example.invalid',docs:{collections:[{id:'docs',dir:'docs',routeBase:'/docs/'}]}},{compilation:{persistentCache:false,output:{path:'dist'}}});`);
  await run('docs-build', ['node_modules/@farmfe/cli/bin/farm.mjs', 'build', '--config', 'farm.config.mjs'], 300000);
  const rendered = [];
  async function scan(directory) { for (const row of await fs.readdir(directory, { withFileTypes: true })) { const file = path.join(directory, row.name); if (row.isDirectory()) await scan(file); else if (row.name.endsWith('.html')) rendered.push(await fs.readFile(file, 'utf8')); } }
  await scan(path.join(scratch, 'dist'));
  assert.ok(rendered.some(html => html.includes('Sealed candidate installed')), 'Independent candidate Docs output missing');
  result.status = 'passed';
} catch (error) {
  result.status = 'failed'; result.error = error.message; result.cleanupConfirmed = error.cleanupConfirmed !== false;
  result.cleanupError = error.cleanupError ?? null; result.ownedPid = error.pid ?? null; process.exitCode = 1;
} finally {
  try { await registry?.close(); } catch (error) { result.status = 'failed'; result.cleanupConfirmed = false; result.cleanupError = `Candidate registry close failed: ${error.message}`; }
  await cleanupConsumerScratch(scratch, result);
  if (result.status !== 'passed') process.exitCode = 1;
  await fs.writeFile(path.join(output, 'result.json'), JSON.stringify(result, null, 2));
}
