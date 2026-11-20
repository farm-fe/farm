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

- Read the website's direct dependencies and committed
  `website/farm-release-lock.json` to select the independently approved Farm
  release. Workspace manifest version bumps do not change deployment pins.
  Website dependency declaration changes require an explicit lock refresh.
- Follow runtime, optional and peer Farm dependencies, including the current
  machine's precompiled native packages. Core and CLI must be v2. Native packages
  are mandatory consumer dependencies, so an optional-install failure cannot be
  ignored. The lock verifies Linux x64 GNU, macOS ARM64 and Windows x64 MSVC;
  other targets fail closed. Source CI keeps its existing workspace/native path.
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
  integrities, the committed deployment lock source/digest, generated pnpm lock
  digest and installed native payload hashes.

Farm versions and integrity are committed across builds. The generated pnpm
lock is frozen within each build; third-party website ranges can still resolve
newer compatible releases on a later build. The receipt is registry-consumption evidence, not a publisher
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

`git.deploymentEnabled.main=true` enables Git-triggered builds from `main` for
this project. Branch previews remain enabled. The project's existing production
branch and domain settings decide where a successful deployment is served; this
repository change does not modify those settings. The local build script only
produces static files and does not deploy, promote, merge or publish packages.

### Independent package upgrades

The npm release workflow and Vercel Git integration still run independently.
A workspace Changesets version bump no longer changes the website's published
consumer. The committed Farm deployment lock keeps the known public cohort until
a separate website-upgrade PR deliberately refreshes it after publication.
Neither `latest` nor an automatic fallback chooses a version. Missing or changed
locked metadata still fails before installation; failed Git deployments are not
automatically retried when a package appears later.

See [the independent upgrade procedure](../releases/docs-deployment-lock.md).
This adds no polling, deploy hook, token, production settings or release-workflow
coupling. Local development and source CI still use workspace versions.

### Public site ownership verification

The deployment builder adds the public Algolia site-verification meta tag only to
the generated homepage `index.html`, after compilation and before replacing any
existing output. Farm's minifier omits optional head/body tags. For the known
published template with its unique Farm root and head-only prefix, the marker
goes immediately after the html start tag, inside the implicit head. All other
bytes stay unchanged. The builder validates one matching marker in the head; malformed,
duplicate or conflicting markup fails without replacing the previous output.
Repeated builds produce the same marker once. This does not alter the published
Farm Docs template, other routes, search credentials or index settings. Direct
`farm build` is not this deployment finalization path.
