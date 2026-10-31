import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assertSourceLicense } from './pack-v2-npm-candidates.mjs';
export function auditSourceLicenses(root, plan) {
  return plan.npm.primary.map(pkg => {
    try {
      const directory = path.dirname(path.join(root, pkg.manifestPath));
      assertSourceLicense(directory, JSON.parse(fs.readFileSync(path.join(root, pkg.manifestPath))));
      return { name: pkg.name, manifest: pkg.manifestPath, status: 'passed' };
    } catch (error) { return { name: pkg.name, manifest: pkg.manifestPath, status: 'blocked', reason: error.message }; }
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const records = auditSourceLicenses(root, plan);
  const report = { status: records.every(row => row.status === 'passed') ? 'passed' : 'blocked', scope: 'Selected npm source license declarations/full text only; not a transitive ownership audit', records };
  console.log(JSON.stringify(report, null, 2));
  if (report.status !== 'passed') process.exitCode = 1;
}
