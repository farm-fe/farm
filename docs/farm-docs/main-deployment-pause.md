# Temporary project-scoped main Git deployment pause

The user explicitly authorized pausing this project's main-push production deployments while preserving the existing site and branch previews. This is not publication approval.

The latest connected Vercel bot report on PR #2326 (updated 2026-10-21T06:54:26Z, [deployment record](https://vercel.com/brightwus-projects/farm-docs/GzzoEzkauy9dDDrJSvbfNJgfkSpL), commit 89f6d489) identifies [project farm-docs](https://vercel.com/brightwus-projects/farm-docs), Root Directory website. Thus the project-level file is website/vercel.json, not a monorepo-wide root configuration. Direct dashboard access is unavailable; any later dashboard Root Directory change must be caught by refreshing the connected bot metadata after the PR update, before merging.

The only policy addition is git.deploymentEnabled.main=false. Vercel documents this as a commit-trigger deployment filter read from the project-root configuration; unspecified branches remain true. No catch-all rule is used, avoiding an overlapping true wildcard that could re-enable main. It does not delete or replace an existing deployment, change domains, disconnect Git, change tokens/permissions, or pause other branches. It controls Git automatic deployments only; it does not authorize a manual production deployment.

Official behavior: https://vercel.com/docs/project-configuration/git-configuration and project-root placement: https://vercel.com/docs/project-configuration/vercel-json .

The merge commit must include this file and the manual crate/npm publication gates. This decision is made before any build, rather than relying on the retired source builder failing. Refresh root evidence on the final PR head; after an eventual authorized merge, verify no main deployment was created and retain the previous production deployment. Removing this temporary rule requires the later formal publication/deployment decision.
