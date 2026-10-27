// cspell:words statfs bavail
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertReviewedCandidatePackHooks } from './candidate-pack-hooks.mjs';
import { archiveFiles } from './release-archive.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const output = path.resolve(process.argv[2] ?? '');
const pnpm = process.argv[3];
if (!process.argv[2] || !pnpm || !fs.existsSync(pnpm) || fs.existsSync(output)) throw new Error('Specify a new evidence directory and installed pnpm CLI path; existing evidence is preserved');
const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
const names = process.argv.slice(4);
const selected = names.length ? plan.npm.primary.filter(pkg => names.includes(pkg.name)) : plan.npm.primary;
if (names.length && selected.length !== new Set(names).size) throw new Error('Unknown selected primary package');
const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
fs.mkdirSync(output, { recursive: true });
const receipt = { schemaVersion: 1, sourceCommit: head, status: 'running', published: false,
  scope: `${selected.length} primary npm archives only; native platform payloads require separate sealing`,
  planSha256: sha(Buffer.from(JSON.stringify(plan))), records: [] };
const save = () => fs.writeFileSync(path.join(output, 'receipt.json'), `${JSON.stringify(receipt, null, 2)}\n`);
save();
try {
  for (const pkg of selected) {
    const disk = fs.statfsSync(output);
    if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 9.75 * 1024 ** 3) throw new Error('Resource guard before next package');
    const directory = path.dirname(path.join(root, pkg.manifestPath));
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'package.json')));
    assertReviewedCandidatePackHooks(directory, manifest);
    const env = { ...process.env, npm_config_cache: path.join(output, 'cache'), npm_config_audit: 'false', npm_config_fund: 'false', npm_config_update_notifier: 'false' };
    delete env.npm_config_ignore_scripts;
    env.PATH = `${path.dirname(process.execPath)}${path.delimiter}${env.PATH}`;
    const result = spawnSync(process.execPath, [pnpm, 'pack', '--pack-destination', output], { cwd: directory, env, windowsHide: true, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 });
    const log = `${pkg.name.replaceAll(/[@/]/g, '_')}.log`;
    fs.writeFileSync(path.join(output, log), `${result.stdout ?? ''}\n${result.stderr ?? ''}`);
    if (result.status !== 0) throw new Error(`Normal pack failed for ${pkg.name}; see ${log}`);
    const filename = `${pkg.name.replace(/^@/, '').replace('/', '-')}-${pkg.version}.tgz`;
    const bytes = fs.readFileSync(path.join(output, filename));
    const files = archiveFiles(bytes);
    const packed = JSON.parse(files.get('package/package.json').toString('utf8'));
    if (packed.name !== pkg.name || packed.version !== pkg.version) throw new Error(`Packed identity drift: ${pkg.name}`);
    receipt.records.push({ name: pkg.name, version: pkg.version, file: filename, sha256: sha(bytes), integrity: `sha512-${crypto.createHash('sha512').update(bytes).digest('base64')}`,
      files: [...files].map(([file, content]) => ({ path: file.replace(/^package\//, ''), size: content.length })), log });
    save();
    console.log(JSON.stringify({ packed: receipt.records.length, total: selected.length, name: pkg.name }));
  }
  if (head !== execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim()) throw new Error('Source commit changed during packing');
  receipt.status = 'packed-not-yet-content-verified';
} catch (error) { receipt.status = 'incomplete'; receipt.error = error.message; process.exitCode = 1; }
receipt.ended = new Date().toISOString();
save();
