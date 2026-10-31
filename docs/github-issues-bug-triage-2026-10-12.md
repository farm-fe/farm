# GitHub issue triage and bug-fix report

Date: 2026-10-12. Repository: farm-fe/farm. Baseline: `2000ef8a790a1b565d1687cfe70564270a826e15` (v2).

The initial paginated review found 169 open non-PR issues, including 82 titles mentioning bugs or failures. These are search results, not 82 confirmed defects. The fixes and regressions below cover the issues selected for this investigation.

## Findings and disposition

| Issue | Finding, change, and verification |
| --- | --- |
| [#2321](https://github.com/farm-fe/farm/issues/2321) | Package export wildcards interpreted literal prefixes/suffixes as regular expressions and allowed partial suffix matches. Match literal prefixes and complete suffixes; cover JSON-like substrings, special characters, exports, and imports. The final resolver suite passes 46 unit/integration tests. Related existing PR: #2323. |
| [#2272](https://github.com/farm-fe/farm/issues/2272), [#2130](https://github.com/farm-fe/farm/issues/2130) | Entry modules also used as dynamic dependencies were deduplicated by ModuleId while edges used ModuleGroupId. Preserve distinct Entry/DynamicImport group identities. Partial bundling passes 19 unit and 2 graph integration tests; actual Node loading covers entry order and static/dynamic imports. Related existing PR: #2275. |
| [#2125](https://github.com/farm-fe/farm/issues/2125) | Actual or encoded NUL-prefixed virtual IDs could enter output filenames and generated HTML/runtime references. Sanitize at the resource naming boundary, preserving internal IDs, ordinary Windows 0assets directories, and collision handling. Actual HTML, emitted files, and Node dynamic loading pass. |
| [#2266](https://github.com/farm-fe/farm/issues/2266) | Socket-close recovery could reload Firefox during navigation. Cancel recovery, ping, timers, and reload while leaving the page; preserve canceled-navigation recovery, restored-page reconnection, and normal server restart behavior. Ten unit tests pass. Retained Firefox 141 evidence demonstrates the failing baseline and fixed navigation. |
| [#1975](https://github.com/farm-fe/farm/issues/1975) | Persistent glob caches reused expanded AST/dependencies without checking pattern membership. Persist and revalidate matching sets; rebuild importers after additions/deletions. Cold/warm cache, compiler updates, restart, and actual chokidar/WebSocket notifications pass. |
| [#2033](https://github.com/farm-fe/farm/issues/2033), related Rust Tailwind cache paths | Restore cached candidate sources, prune deleted physical/query sources, and retain virtual sources. Track Rust sources that initially have no utilities. Actual Tailwind cold/warm startup, stylesheet edits, p-4 to p-8 changes, and source deletion pass, including absence assertions for stale selectors. |
| [#2196](https://github.com/farm-fe/farm/issues/2196), [#2252](https://github.com/farm-fe/farm/issues/2252) | Actual Vue plugin 6.0.1 and UnoCSS 0.62.2 pass cold/warm startup, real server HMR from w-50% to w-100%, and restart with existing .farm caches. This verifies current v2 behavior, not a backport to the old reported version. UnoCSS reloads configuration relative to process cwd; the isolated fixture uses the project cwd, as the CLI does, and restores it during cleanup. |
| [#1783](https://github.com/farm-fe/farm/issues/1783) | DTS forced allowJs:true even with isolatedDeclarations. Preserve explicit/inherited options and choose compatible defaults. Six real declaration-output/configuration regressions and the complete plugin build pass. |
| [#2131](https://github.com/farm-fe/farm/issues/2131) | A valid entry variable alone did not guarantee correct default reexports. Fix missing interop-helper flags and preserve public names during export mangling. Direct, aliased, and mixed reexports execute correctly with normal minification/mangling. |
| [#1833](https://github.com/farm-fe/farm/issues/1833) | v2 already supports template strings without interpolation. The complete import-glob fixture runner passes, including no_expr_tpl. Dynamic interpolation was not added as a new feature. |
| [#1996](https://github.com/farm-fe/farm/issues/1996) | A standalone TypeScript consumer imports only @farmfe/core/client, without Vite types. Eager/lazy generics and invalid-argument diagnostics pass. |
| [#2183](https://github.com/farm-fe/farm/issues/2183), [#1816](https://github.com/farm-fe/farm/issues/1816) | Actual native production output contains the production environment value and HTML pre/post transformations, with default minification enabled. |
| [#2157](https://github.com/farm-fe/farm/issues/2157), [#1918](https://github.com/farm-fe/farm/issues/1918), [#2124](https://github.com/farm-fe/farm/issues/2124) | JS adapter regressions verify printUrls with the original receiver and object/array alias normalization. Full browser flows for the old third-party inspector/Solid plugin versions were not executed. |
| [#2316](https://github.com/farm-fe/farm/issues/2316) | The v2 CLI depends only on cac; source searches and pnpm --filter @farmfe/cli why inquirer find no inquirer dependency. The v1 advisory and existing PR #2319 belong to a separate release branch; no v1 backport is included. |
| [#2268](https://github.com/farm-fe/farm/issues/2268), [#2208](https://github.com/farm-fe/farm/issues/2208) | Correct English/Chinese tutorials to compilation.external, and use @farmfe/js-plugin-visualizer plus public defineConfig in the visualizer README. Website translations retain their intended language. |

## Supporting fixes exposed by complete execution paths

- Skip bootstrap when runtime.plugins is empty. Share one registry, cache, and dynamic loader across inline and standalone runtimes. Initialize workers before accessing a missing window.
- Terminate repeated All-exports work for dynamic self-imports and A-to-B-to-A cycles. Construct dynamic resource maps before temporarily taking an entry resource. Tests retain bounded compilation timeouts and actual Node identity assertions.
- Preserve built-in runtime synthetic IDs during updates using an exact package boundary; make suffix insertion idempotent instead of resolving them under the project directory.
- Expire modules when watched inputs change even if their own source hash does not. Include watched content fingerprints in generated-resource and development CSS-to-JS cache keys. Canonical ordering and timestamp-only changes preserve reuse.
- Keep HMR roots internal when an absolute project path matches an external package pattern. Ordinary imports still honor external configuration.
- Route Added/Removed watcher results through the HMR engine and capture importers before deletion. Broadcast batches in order; replace obsolete same-path membership events, recheck queue indexes after asynchronous stat, and retain failed batches for retry.
- Register PostCSS dependency/dir-dependency messages in production as well as development. Directory messages are expanded into physical file dependencies; directory paths are not passed to a file-content cache reader.

Retained regressions first reproduced the relevant failures. Cache invalidation hooks were checked against their callers: returning true from handle_persistent_cached_module invalidates the cached module.

The 56 updated existing mangle/tree-shaking snapshots were audited as runtime guard/bootstrap changes. A new mixed-default-reexport fixture verifies preserved public named/default exports. Tests do not disable normal minification, mangling, or splitting to pass.

## Validation

Implementation used the isolated managed worktree on branch `codex/github-bug-fixes`, starting from the committed baseline. Unrelated uncommitted v2 work in the original checkout was preserved. Scoped changesets accompany the published-package fixes.

| Check | Result |
| --- | --- |
| Core/compiler library tests | 34 + 58 tests pass. |
| Resolver, partial bundling, script, script_meta, import_glob, and Rust Tailwind runners | All pass, including the template fixture, cache restoration, and external-HMR regression. |
| Actual compiler/Node virtual_dynamic_import | Seven tests pass: virtual outputs, entry identity, reexports, worker initialization, self-imports, and cycles. |
| Compiler mangle_exports/tree_shake | Five runner tests pass after audited snapshot updates and rerunning without UPDATE. |
| DTS, adapter, HMR client/queue/watcher, PostCSS, JS Tailwind | Seven files and 38 Vitest tests pass. |
| Native cache/styles/server WebSocket | Four tests pass with the final binding and actual Vue plugin 6.0.1. |
| Native HTML/environment | One test passes. |
| Firefox navigation | Real failing-baseline and fixed-client evidence is retained in docs/issue-evidence/hmr-firefox.json; the runnable harness is packages/runtime-plugin-hmr/tests/firefox-navigation.cjs. |
| Core native binding | Standard npm run build:rs:debug passes with default SWC-plugin support, including the final build without temporary diagnostics. |
| Package/example builds | npm run build passes for core including CJS, runtime, utils, HMR/import-meta/plugin-tools/CLI, DTS/PostCSS/JS Tailwind, replace-dirname/React/Rust Tailwind, and the import-meta/tailwind-next/tailwindcss-rust-plugin examples. |
| TypeScript/format/diff | Affected production TypeScript checks, 19 scoped Biome files, and diff checks pass. The whole core test tsconfig still has existing baseline diagnostics; a full repository type-check pass is not claimed. |
| Review | Spec and quality reviews completed. Runtime, worker, HMR ordering, and queue-race findings were fixed and reviewed again. |

Multi-package build verification was used. Full ready, cargo check, and cargo clippy were not run.

### Reproduce the selected tests

Run from the repository root after building the local prerequisites:

```powershell
cargo test -j 2 -p farmfe_core -p farmfe_compiler --lib
cargo test -j 2 -p farmfe_plugin_resolve -p farmfe_plugin_partial_bundling -p farmfe_plugin_script -p farmfe_plugin_script_meta -p farmfe_swc_transformer_import_glob -p farmfe_plugin_tailwindcss
cargo test -j 2 -p farmfe_compiler --test virtual_dynamic_import
cargo test -j 2 -p farmfe_compiler --test mangle_exports --test tree_shake
pnpm exec vitest run js-plugins/dts/tests/context.spec.ts packages/core/tests/issue-regressions.spec.ts packages/core/tests/hmr-membership.spec.ts packages/core/tests/hmr-watcher.spec.ts packages/runtime-plugin-hmr/src/hmr-client.spec.ts js-plugins/postcss/src/index.spec.ts js-plugins/tailwindcss/src/index.spec.ts --maxWorkers 1 --minWorkers 1
node node_modules/vitest/vitest.mjs run --config packages/core/vitest.integration.config.ts
$env:FARM_VUE_PLUGIN_PACKAGE_PATH = Join-Path ([System.IO.Path]::GetTempPath()) 'farm-vue-6-0-1-github-bugs/node_modules/@vitejs/plugin-vue'
node node_modules/vitest/vitest.mjs run --config js-plugins/postcss/vitest.integration.config.ts
```

The exact Vue 6.0.1 package was installed temporarily without changing the repository lockfile. If that path is unavailable, omit the environment variable: the fixture uses the existing Vue 6 dependency in examples/vite-adapter-vue and asserts the major version. The Firefox harness header documents its temporary Playwright Firefox dependency and arguments.

### Verification boundaries

- WebSocket cases prove notification delivery and updated module membership/content, not complete browser application behavior for those glob routes.
- Firefox navigation uses a real browser. beforeunload can affect Firefox BFCache; a real BFCache cache hit is not claimed.
- Old third-party plugin versions and the v1 CLI release branch are outside this v2 patch.
- No GitHub issue was closed as part of this investigation.
