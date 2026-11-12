# Build the website with published Farm v2 packages

The Vercel project is `farm-docs`, with Root Directory `website` and output
`build-farm`. `website/vercel.json` keeps the existing install/build commands:

```sh
node ../scripts/farm-docs-vercel-build.mjs --install
node ../scripts/farm-docs-vercel-build.mjs
```

Use Node 22 and Corepack. The independent consumer pins pnpm 9.4.0. For a read-only
publication check from the repository root:

```sh
node scripts/farm-docs-vercel-build.mjs --preflight
```

## Package and source boundaries

- Read the website's direct dependencies and current repository package manifests
  to determine exact, stable Farm release versions. Neither `latest`, beta tags,
  nor the old draft release inventory chooses deployment versions.
- Follow runtime, optional and peer Farm dependencies, including the current
  machine's precompiled native packages. Core and CLI must be v2. Native packages
  are mandatory consumer dependencies, so an optional-install failure cannot be
  ignored. macOS, Linux and Windows targets use their own published packages.
- Query the official npm registry for every required exact version. Check names,
  versions, SHA-512 integrity, registry tarball URLs, Farm dependency declarations
  and platform metadata. Missing publication or changed dependency declarations
  stops before installation. Farm install lifecycle scripts are rejected.
- Vercel's install phase performs only that preflight. It never installs the
  monorepo. The build phase copies website sources to a fresh OS temporary
  directory outside the workspace, omitting existing output, node_modules,
  Vercel metadata and `.env` files. Local development commands stay unchanged.
- Generate an independent lock using normal pnpm resolution, verify its Farm
  integrity values and absence of local/workspace dependencies, then run a frozen
  install. All dependency lifecycle scripts are disabled in both commands.
  Neither the repository's workspace overrides nor its patched lock is used.
  The website explicitly declares `@emotion/is-prop-valid@1.4.0`, the
  optional framer-motion peer previously supplied by the workspace lock.
  Third-party semver prereleases are allowed; Farm versions remain exact stable
  releases. Lock resolution maps allow only integrity and full public-registry
  tarball URLs, rejecting local paths, custom fetchers and ambiguous YAML.
- Verify installed package versions and real paths remain within that consumer.
  Check native file formats and reject workspace-style local native payloads.
  Run the installed CLI's `--help`, then the website build. There is no Cargo,
  rustup, native compilation, locally packed package or source-build fallback.
- Copy output only after the build succeeds and the public third-party notice
  matches the source bytes. An existing output is preserved under
  `website/build-farm.previous-<timestamp>`. The temporary consumer is removed.
  `build-farm/farm-docs-build-receipt.json` records exact Farm versions, npm
  integrities, the generated lock digest and installed native payload hashes.

The independent lock is frozen within each build, not committed as a cross-build
lock. Ordinary website dependency ranges can resolve newer compatible releases
on a later build. The receipt is registry-consumption evidence, not a publisher
approval, native ABI certification or a substitute for real build/preview checks.
The older `farm-docs-registry-consumer.mjs` publisher-receipt audit remains separate
and unchanged; its historical 40-package draft is not this deployment's manifest.

## Validation and publication

```sh
node --test scripts/farm-docs-vercel-build.test.mjs \
  scripts/farm-docs-main-deploy-gate.test.mjs \
  scripts/farm-docs-consumer-isolation.test.mjs \
  scripts/farm-docs-registry-consumer.test.mjs
node scripts/farm-docs-notices.test.mjs --source-only
```

Orchestration tests use explicitly synthetic metadata, installs and build output.
They do not prove that packages are published or that the website builds. A real
preflight and real Node 22 build must pass after the npm release finishes, followed
by checking the actual Vercel deployment's routes, assets and notices.

`git.deploymentEnabled.main=false` is deliberately unchanged. This build script
produces static files only: it does not create deployments, promote previews,
modify project settings, merge a PR or re-enable main-push production deployment.
Confirm the current Vercel project/Root Directory and authorized deployment target
before performing a manual deployment. Keep the existing production site in place
while publication or validation is incomplete.
