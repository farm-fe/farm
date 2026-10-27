import fs from 'node:fs/promises';
import path from 'node:path';

const directory = path.resolve('performance-results');
await fs.mkdir(directory, { recursive: true });
const raw = path.join(directory, 'raw.json');
if (process.argv[2] === 'init') {
  await fs.writeFile(raw, JSON.stringify({ schemaVersion: 1, status: 'partial', publishable: false, revision: process.env.GITHUB_SHA, records: [], errors: ['Setup or sampling has not completed.'] }, null, 2));
  for (const name of ['run.log', 'samples.csv', 'summary.md']) await fs.writeFile(path.join(directory, name), '');
} else if (process.argv[2] === 'finalize') {
  const report = JSON.parse(await fs.readFile(raw, 'utf8'));
  if (report.status === 'partial') {
    report.status = 'invalid';
    report.errors.push('Job setup failed or sampling was interrupted; no complete measurement.');
    await fs.writeFile(raw, JSON.stringify(report, null, 2));
    await fs.writeFile(path.join(directory, 'summary.json'), JSON.stringify(report, null, 2));
    const markdown = '### Commerce performance\n\n**Invalid/incomplete measurement**; publishable: false. Inspect the setup/sampling step and raw artifact. No zero measurements or regression conclusions were produced.\n';
    await fs.writeFile(path.join(directory, 'summary.md'), markdown);
    if (process.env.GITHUB_STEP_SUMMARY) await fs.appendFile(process.env.GITHUB_STEP_SUMMARY, markdown);
  }
} else throw new Error('Expected init or finalize');
