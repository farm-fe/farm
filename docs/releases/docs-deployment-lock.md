# Independent website package upgrades

Vercel builds the current website content against the separately reviewed
`website/farm-release-lock.json`, not the current workspace package versions.
Changesets can prepare the next release without breaking the production website
or ordinary Git Previews. Local development and Ubuntu/Windows source Docs CI
retain `workspace:*` and the existing source/native provenance checks.

The initial lock comes from the last ready production source
`eb8be2c8ce4bbf8774dc3c6877c43bf84e79bca1`: Docs/theme 0.0.1 and core/CLI 2.0.0.
It records the full Farm dependency graph, exact public registry SHA512 integrity,
platform metadata and dependency edges. All 93 records were checked against the
public registry. Each supported consumer selects 29 packages with matching native
payloads. Supported deployment-validation targets are Linux x64 GNU (Vercel),
macOS ARM64 and Windows x64 MSVC; other targets fail rather than falling back.
This lock is for Farm packages, not a complete frozen third-party pnpm lock.
Third-party registry ranges retain the previous isolated resolution behavior.

The regular build still verifies public metadata, exact topology, integrity,
registry-only lock resolution, installed package isolation and native binary
format before compiling the website. A missing package or changed registry
integrity fails visibly. There is no workspace/native-build fallback and no
credentials are required. The public build receipt includes the deployment lock
source and digest, as well as the installed package and native hashes.

## Upgrade separately after publication

1. Select an exact, reviewed source commit whose complete Farm release cohort is
   already public. Do not change package versions or the release workflow here.
2. Run `node scripts/farm-docs-refresh-deployment-lock.mjs <full-source-SHA>`.
   It reads that commit's manifests and verifies every exact public package before
   writing the lock. It never publishes, installs packages or contacts Vercel.
3. Review the dependency/integrity diff in a standalone website-upgrade PR. Changes
   to website dependency or devDependency declarations require this explicit
   refresh too; the build rejects a stale lock. Content-only edits do not.
4. With Node 22 and Corepack available, run
   `node --test scripts/farm-docs-deployment-lock.test.mjs scripts/farm-docs-vercel-build.test.mjs`
   and `node scripts/farm-docs-vercel-build.mjs --preflight`, then
   `node scripts/farm-docs-vercel-build.mjs` for a real published-consumer build.
   Review the built website and Git Preview before merging the upgrade.

Vercel Git main/Preview behavior is unchanged. No GitHub Actions deployment
controller, protected environment, new token, deploy hook, production setting,
package version or npm publication workflow is introduced. A website code change
that uses a new Farm API still needs a compatible lock upgrade and a real build;
independent versions do not imply forward API compatibility.
