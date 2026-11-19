# Docs deployment after verified publication

The website consumes exact public package versions. Merging a Changesets version
PR does not mean its new npm versions are available yet. The Vercel Git build and
npm release run independently; the former must not build workspace substitutes.

## Deferred Git build

`website/vercel.json` runs a dependency-free ignored-build command. Only a
production build on `main`, with a valid source SHA and an exact package version
returning HTTP 404, is deferred. Vercel interprets exit 0 as cancel and 1 as build.
Preview branches and ordinary documentation commits whose pins are available keep
the existing build path. Authentication/network errors are not treated as package
absence; the normal strict build preflight still fails visibly. The install/build
commands, published-package isolation, checksums and platform checks are unchanged.

## Verified release handoff, not automatic deployment

After the existing release finalization succeeds, a separate read-only job reports the exact
verified source SHA. It declines a source that is no longer current main. The job
uses no Vercel credential, sends no deployment request, and cannot publish npm.
A failure of this informational handoff does not undo a verified package release.

In the existing Vercel project, manually create or redeploy the reported exact
commit after public verification succeeds. Recheck main before starting and before
promoting; never promote a superseded release. Confirm the deployed commit and
website build receipt before declaring the site updated. The ignored command does
not itself schedule a retry; canceled Git builds need this explicit continuation.

Automatic exact-source redeployment remains a follow-up. A branch Deploy Hook is
not sufficient: main can advance between verification and deployment, and a late
older deployment must not replace newer production. Do not introduce an unbounded
token, hook URL, empty commit, global Git disable switch or source-build fallback
to hide that race. Any future automation must prove same-SHA retrigger behavior,
keep source association, prevent stale promotion, and isolate least-privilege
credentials from package installation and npm publication. No secret or project
setting is created by this change.

References: [Vercel ignore command](https://vercel.com/docs/project-configuration/vercel-json#ignorecommand),
[deploy hooks](https://vercel.com/docs/deploy-hooks), and
[exact Git commit deployments](https://vercel.com/docs/git#creating-a-deployment-from-a-git-reference).
Keep the ignored-build command and strict install preflight enabled on redeploy.

The reviewed workflow copies under `scripts/release-provenance-templates/` are
updated byte-for-byte alongside the caller. The signer still loads those exact
files from its job source and verifies the caller against them. No template index,
verifier policy, accepted source identity or signature rule is changed.
