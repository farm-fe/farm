import { execSync } from 'child_process';
import fs from 'node:fs';

if (fs.existsSync('.changeset/pre.json')) throw new Error('Stable Changesets preparation cannot consume an active prerelease');
fs.mkdirSync('docs/release-evidence', { recursive: true });
execSync('npx changeset status --output docs/release-evidence/changesets-version-plan.json', { stdio: 'inherit' });
execSync('npx changeset version', { stdio: 'inherit' });
execSync('pnpm install --no-frozen-lockfile', { stdio: 'inherit' });
