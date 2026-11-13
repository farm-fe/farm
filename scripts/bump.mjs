import { execSync } from 'child_process';
import fs from 'node:fs';
import { snapshotUnpublishedManifests, restoreUnchangedManifestFormatting } from './changesets-manifest-format.mjs';
import { applyCargoProposal, synchronizeNativePayloads } from './prepare-cargo-version-proposal.mjs';

if (fs.existsSync('.changeset/pre.json')) throw new Error('Stable Changesets preparation cannot consume an active prerelease');
const baseline = JSON.parse(fs.readFileSync('docs/releases/v2-candidate.json'));
const manifests = snapshotUnpublishedManifests(process.cwd(), baseline);
fs.mkdirSync('docs/release-evidence', { recursive: true });
applyCargoProposal(JSON.parse(process.env.FARM_CARGO_PROPOSAL ?? 'null'));
execSync('npx changeset status --output docs/release-evidence/changesets-version-plan.json', { stdio: 'inherit' });
execSync('npx changeset version', { stdio: 'inherit' });
// Changesets owns the owner version; platform wrappers follow that exact delta.
// Synchronize before regenerating the lock or taking any inventory snapshot.
synchronizeNativePayloads(process.cwd(), baseline);
execSync('pnpm install --no-frozen-lockfile', { stdio: 'inherit' });
restoreUnchangedManifestFormatting(process.cwd(), manifests);
