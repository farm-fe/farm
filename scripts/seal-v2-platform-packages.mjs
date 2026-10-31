// cspell:words statfs bavail
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { execFileSync, spawnSync } from 'node:child_process';
import { archiveFiles, sha256 } from './release-archive.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const [cohortFile, outputArg, pnpm] = process.argv.slice(2);
const owners = process.argv.slice(5);
if (!cohortFile || !outputArg || !pnpm) throw new Error('Specify cohort receipt, new evidence directory and installed pnpm CLI');
const output = path.resolve(outputArg);
if (fs.existsSync(output)) throw new Error('Preserve existing platform evidence');
const cohortBytes = fs.readFileSync(cohortFile);
const cohort = JSON.parse(cohortBytes);
if (cohort.abi !== '0.5.4') throw new Error('Unexpected producer ABI');
const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, 'native-producer.json'), cohortBytes);
const receipt = { schemaVersion: 1, sourceCommit: head, published: false, status: 'running', planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [] };
const save = () => fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2));
save();
try {
  for (const pkg of plan.npm.payloads.filter(row => row.platform === 'win32-x64-msvc' && (!owners.length || owners.includes(row.owner)))) {
    const owner = plan.npm.primary.find(row => row.name === pkg.owner);
    const producer = cohort.artifacts.find(row => row.file.startsWith(path.dirname(owner.manifestPath) + '/'));
    if (!producer) continue; // An absent producer stays missing; do not invent provenance.
    const disk = fs.statfsSync(output);
    if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 9.75 * 1024 ** 3) throw new Error('Resource guard');
    const directory = path.dirname(path.join(root, pkg.manifestPath));
    const manifest = JSON.parse(fs.readFileSync(path.join(root, pkg.manifestPath)));
    if (path.basename(manifest.main) !== manifest.main) throw new Error('Unsafe native package main');
    const native = fs.readFileSync(path.join(directory, manifest.main));
    if (sha256(native) !== producer.sha256) throw new Error(`Native bytes drifted: ${pkg.name}`);
    const result = spawnSync(process.execPath, [pnpm, 'pack', '--pack-destination', output], { cwd: directory, windowsHide: true, encoding: 'utf8', timeout: 120000, maxBuffer: 4 * 1024 ** 2, env: { ...process.env, PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH}`, npm_config_update_notifier: 'false' } });
    fs.writeFileSync(path.join(output, `${pkg.name.replaceAll(/[@/]/g, '_')}.log`), `${result.stdout}\n${result.stderr}`);
    if (result.status !== 0) throw new Error(`Platform pack failed: ${pkg.name}`);
    const file = `${pkg.name.replace(/^@/, '').replace('/', '-')}-${pkg.version}.tgz`;
    const bytes = fs.readFileSync(path.join(output, file));
    const packed = JSON.parse(archiveFiles(bytes).get('package/package.json').toString('utf8'));
    if (packed.name !== pkg.name || packed.version !== pkg.version) throw new Error('Packed platform identity drift');
    receipt.records.push({ name: pkg.name, version: pkg.version, file, sha256: sha256(bytes), nativeSha256: producer.sha256, abi: cohort.abi, producer: { kind: 'local-build', sourceCommit: producer.producerSha, receipt: 'native-producer.json', receiptSha256: sha256(cohortBytes) } });
    save();
    console.log(JSON.stringify({ packed: receipt.records.length, name: pkg.name }));
  }
  receipt.status = 'packed-not-yet-content-verified';
} catch (error) { receipt.status = 'incomplete'; receipt.error = error.message; process.exitCode = 1; }
save();
