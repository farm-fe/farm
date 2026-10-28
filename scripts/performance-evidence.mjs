import fs from 'node:fs/promises';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import { resourceSnapshot } from './performance-resources.mjs';

const directory = path.resolve('performance-results');
await fs.mkdir(directory, { recursive: true });
const raw = path.join(directory, 'raw.json');
if (process.argv[2] === 'init') {
  if (process.env.GITHUB_ENV) await fs.appendFile(process.env.GITHUB_ENV, `FARM_PERFORMANCE_STARTED_AT=${Date.now()}\n`);
  let revision = null;
  try { revision = execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim(); } catch { /* Identity is unknown, never substitute a claimed SHA. */ }
  const disk = await fs.statfs(os.tmpdir());
  await fs.writeFile(raw, JSON.stringify({ schemaVersion: 1, protocol: 'commerce-business-2048-rust-build-v2', status: 'partial', failureKind: 'setup-incomplete', publishable: false,
    metadata: { revision, artifactProducerSHA: process.env.GITHUB_SHA ?? null, os: process.platform, arch: process.arch, baseSHA: process.env.FARM_BASE_SHA ?? null },
    resources: resourceSnapshot({ diskFreeBytes: disk.bavail * disk.bsize }), records: [], errors: ['Setup or sampling has not completed.'] }, null, 2));
  for (const name of ['run.log', 'samples.csv', 'summary.md']) await fs.writeFile(path.join(directory, name), '');
} else if (process.argv[2] === 'finalize') {
  const report = JSON.parse(await fs.readFile(raw, 'utf8'));
  if (report.status === 'partial') {
    report.status = 'invalid';
    report.failureKind = 'setup-failure';
    report.errors.push('Job setup failed or sampling was interrupted; no complete measurement.');
    await fs.writeFile(raw, JSON.stringify(report, null, 2));
    await fs.writeFile(path.join(directory, 'summary.json'), JSON.stringify(report, null, 2));
    const markdown = '### Commerce performance\n\n**Invalid/incomplete measurement**; publishable: false. Inspect the setup/sampling step and raw artifact. No zero measurements or regression conclusions were produced.\n';
    await fs.writeFile(path.join(directory, 'summary.md'), markdown);
    if (process.env.GITHUB_STEP_SUMMARY) await fs.appendFile(process.env.GITHUB_STEP_SUMMARY, markdown);
  }
} else throw new Error('Expected init or finalize');
