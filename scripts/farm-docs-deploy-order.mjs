import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { deploymentPlan, releaseManifests, publicMetadata, preflight } from './farm-docs-vercel-build.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const sha = /^[a-f0-9]{40}$/;

// Vercel's ignore command uses 0 to cancel and 1 to build. Only an exact
// missing public release is deferred; other errors must reach strict preflight.
export async function ignoreReleaseBuild({ env = process.env, getMetadata = publicMetadata, plan, releasePins } = {}) {
  if (env.VERCEL_ENV !== 'production' || env.VERCEL_GIT_COMMIT_REF !== 'main') return false;
  if (!sha.test(env.VERCEL_GIT_COMMIT_SHA ?? '')) return false;
  plan ??= deploymentPlan(JSON.parse(fs.readFileSync(path.join(root, 'website/package.json'), 'utf8')), releaseManifests(root), 'linux-x64-gnu');
  releasePins ??= new Map(Object.values(JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json'), 'utf8')).npm).flat().map(pkg => [pkg.name, pkg.version]));
  const selected = [...plan.selected];
  const results = await Promise.allSettled(selected.map(([name, pkg]) => getMetadata(name, pkg.version)));
  const errors = results.filter(row => row.status === 'rejected').map(row => row.reason);
  // Do not let a concurrent 404 hide an authentication, network or metadata error.
  if (errors.some(error => !String(error?.message).startsWith('[RELEASE_NOT_PUBLISHED] '))) return false;
  const available = selected.filter((_, index) => results[index].status === 'fulfilled');
  const metadata = new Map(selected.flatMap(([name], index) => results[index].status === 'fulfilled' ? [[name, results[index].value]] : []));
  // A successful HTTP response can still be corrupt. Validate every available
  // member before classifying other members as an unpublished release.
  await preflight({ ...plan, selected: new Map(available) }, name => metadata.get(name));
  if (errors.length) return selected.every(([name, pkg], index) => results[index].status === 'fulfilled' || releasePins.get(name) === pkg.version);
  return false;
}

// This is a handoff only. No credential, deployment POST or branch hook is used.
// Automatic deployment requires a separately reviewed stale-promotion strategy.
export function deploymentHandoff(env, currentMain) {
  assert.equal(env.GITHUB_REPOSITORY, 'farm-fe/farm');
  assert.equal(env.GITHUB_EVENT_NAME, 'push');
  assert.equal(env.GITHUB_REF, 'refs/heads/main');
  assert.match(env.GITHUB_SHA ?? '', sha);
  assert.equal(env.GITHUB_WORKFLOW_SHA, env.GITHUB_SHA);
  assert.equal(env.FARM_RELEASE_FINALIZED, 'success');
  assert.match(currentMain, sha);
  if (currentMain !== env.GITHUB_SHA) return { status: 'superseded', source: env.GITHUB_SHA, currentMain };
  return { status: 'manual-exact-source-redeploy', source: env.GITHUB_SHA, repository: env.GITHUB_REPOSITORY,
    instruction: 'Public release verification succeeded. In the existing Vercel project, create/redeploy this exact commit SHA. Recheck main before starting and before promoting; do not promote a superseded source. No deployment was requested automatically.' };
}

async function main() {
  if (process.argv[2] === '--ignore') {
    try {
      const defer = await ignoreReleaseBuild();
      console.log(defer ? 'Exact release pins are not public yet; defer this main build until verified publication.' : 'Keep ordinary build and strict public-registry preflight.');
      process.exitCode = defer ? 0 : 1;
    } catch {
      console.log('Release readiness could not be established; retain strict build preflight.');
      process.exitCode = 1;
    }
  } else if (process.argv[2] === '--handoff') {
    const response = await fetch('https://api.github.com/repos/farm-fe/farm/git/ref/heads/main', { signal: AbortSignal.timeout(30000), redirect: 'error', headers: { Accept: 'application/vnd.github+json' } });
    assert.ok(response.ok, 'Cannot verify current main; no deployment handoff authorized');
    const result = deploymentHandoff(process.env, (await response.json()).object?.sha);
    const text = `## Docs deployment handoff\n\n${JSON.stringify(result, null, 2)}\n`;
    console.log(text);
    if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, text);
  } else throw new Error('Use --ignore or --handoff');
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) main().catch(() => { console.error('Docs deployment handoff unavailable. No deployment was requested.'); process.exitCode = 1; });
