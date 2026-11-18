# Main Git deployment policy

The temporary main deployment pause is lifted at the user's request. The
project-root `website/vercel.json` explicitly sets
`git.deploymentEnabled.main=true`. Unspecified branches remain enabled, so branch
previews are unchanged. No wildcard or global Git integration switch is added.

The configuration belongs to the existing `farm-docs` project with Root Directory
`website`. Its existing Production Branch and domain settings determine production
routing. This repository change does not edit Vercel project settings, domains,
credentials, permissions or the npm publication workflow. It does not merge a PR
or perform a manual production deployment. Once merged into main, the normal Git
integration may create a production deployment from that merge commit.

## Publish order and failure behavior

The npm release workflow (`.github/workflows/incremental-release.yaml`) and Vercel
Git deployment both start independently on main pushes. There is no ordering
barrier between them. Vercel's install and build commands retain the exact-version
public npm preflight and the isolated published-package builder.

If a referenced version has not been published, the preflight fails before any
install/build command or output replacement. A failed build is not promoted, so
the existing production deployment remains in service. The builder never falls
back to local sources, older Farm releases or native compilation.

npm publication completion does not automatically retry a failed Vercel build.
After the exact-package preflight succeeds, redeploy the same main commit in
Vercel. A later main push also triggers a fresh attempt. No release-completion
workflow, polling loop, token or deploy hook is introduced by this change.

## Validation

Run the main-deploy gate and published-builder tests. The missing-publication
regression covers both Vercel phases with an existing output and verifies that it
is preserved without running commands. These tests do not simulate Vercel's
production routing. After merging, check the real deployment status and source
commit, then verify the production routes and notices. Keep the previous site
until a successful production deployment is ready.

Deployment details: [registry deployment](registry-deployment.md).
Official Git-filter behavior:
https://vercel.com/docs/project-configuration/git-configuration
Project-root configuration:
https://vercel.com/docs/project-configuration/vercel-json

## Historical record (superseded)

The former pause policy below is retained for context. Its merge prerequisites
and pause instructions are historical, not the current deployment policy.

### Temporary project-scoped main Git deployment pause

The user explicitly authorized pausing this project's main-push production deployments while preserving the existing site and branch previews. This is not publication approval.

The latest connected Vercel bot report on PR #2326 (updated 2026-10-21T06:54:26Z, [deployment record](https://vercel.com/brightwus-projects/farm-docs/GzzoEzkauy9dDDrJSvbfNJgfkSpL), commit 89f6d489) identifies [project farm-docs](https://vercel.com/brightwus-projects/farm-docs), Root Directory website. Thus the project-level file is website/vercel.json, not a monorepo-wide root configuration. Direct dashboard access is unavailable; any later dashboard Root Directory change must be caught by refreshing the connected bot metadata after the PR update, before merging.

The only policy addition is git.deploymentEnabled.main=false. Vercel documents this as a commit-trigger deployment filter read from the project-root configuration; unspecified branches remain true. No catch-all rule is used, avoiding an overlapping true wildcard that could re-enable main. It does not delete or replace an existing deployment, change domains, disconnect Git, change tokens/permissions, or pause other branches. It controls Git automatic deployments only; it does not authorize a manual production deployment.

Official behavior: https://vercel.com/docs/project-configuration/git-configuration and project-root placement: https://vercel.com/docs/project-configuration/vercel-json .

The merge commit must include this file and the manual crate/npm publication gates. This decision is made before any build, rather than relying on the retired source builder failing. Refresh root evidence on the final PR head; after an eventual authorized merge, verify no main deployment was created and retain the previous production deployment. Removing this temporary rule requires the later formal publication/deployment decision.
