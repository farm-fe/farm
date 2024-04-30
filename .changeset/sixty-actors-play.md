---
'@farmfe/core': major
---

Description:

This PR introduces a new function, resolvePlugins, which combines the functionality of resolveFarmPlugins and resolveAsyncPlugins. This new function first resolves any promises and flattens the array of plugins, and then processes the resulting plugins as in resolveFarmPlugins. This allows the use of async and nested plugins in the configuration.

Changes:

Added a new function resolvePlugins in src/plugin/index.ts that handles async and nested plugins.
Replaced calls to resolveFarmPlugins with resolvePlugins in the codebase.

BREAKING CHANGE:

This change should not introduce any breaking changes. The new function resolvePlugins is backwards compatible with resolveFarmPlugins.
