# GitHub Bug Fixes Implementation Plan

> **For agentic workers:** Use subagent-driven-development for scoped tasks and dispatching-parallel-agents only for disjoint files. Execute continuously; the user has authorized the triage, isolated workspace, fixes and regression tests.

**Goal:** Resolve the bugs identified in the 2026-10-12 issue triage and prove existing fixes with regression tests.

**Architecture:** Start from committed HEAD 2000ef8a in a managed worktree. Keep compiler graph/cache changes separate from browser HMR and declaration generation. Reproduce each reported behavior before implementation, then retain tests and record results in the triage document.

**Tech Stack:** Rust workspace, TypeScript, Vitest, Farm compiler fixtures, browser E2E.

## 1. Isolation and baseline

- [x] Complete native worktree registration and create codex/github-bug-fixes branch.
- [x] Copy triage and this plan into worktree docs.
- [x] Install dependencies with pnpm using repository lockfile; skip Electron/Playwright binary downloads during install.
- [x] Run focused existing tests before changes; record environmental failures separately.

## 2. Resolver and module groups

Files: crates/plugin_resolve/src/resolver/exports.rs and its existing test suite; crates/plugin_partial_bundling/src/analyze_module_graph.rs; crates/plugin_partial_bundling/tests/module_group_graph.rs.

- [x] Add resolver tests for ./modules/a.json.b, full suffix matching, literal special characters, and imports patterns. Run cargo test -p farmfe_plugin_resolve and confirm expected failures.
- [x] Match literal prefix/suffix and require the complete suffix; rerun resolver suite.
- [x] Add graph tests for a module serving as an entry and dynamic dependency in both entry orders, self/cyclic dynamic imports and repeated entries; confirm baseline failure.
- [x] Ensure group identity and visited keys agree without dropping required Entry/DynamicImport groups; verify resource consumers before choosing canonical entry groups versus separate groups.
- [x] Run cargo test -p farmfe_plugin_partial_bundling and applicable compiler fixtures.

## 3. Browser HMR lifecycle

Files: packages/runtime-plugin-hmr/src/hmr-client.ts; new packages/runtime-plugin-hmr test file included by root Vitest.

- [x] Write tests for close during page navigation, disconnect while ping is pending, normal server restart, and page restoration.
- [x] Confirm old client reloads during navigation.
- [x] Cancel reconnect/reload while document is leaving; preserve server restart recovery and BFCache restoration behavior.
- [x] Run focused Vitest and build runtime-plugin-hmr. Validate Firefox navigation with a browser test where available.

## 4. DTS configuration

Files: js-plugins/dts/src/context.ts; js-plugins/dts regression tests.

- [x] Build a temporary TS project using isolatedDeclarations and generate declarations through actual plugin context, with diagnostics enabled.
- [x] Confirm the forced allowJs causes incompatible options on baseline.
- [x] Preserve explicit user options and prevent forced allowJs from conflicting with isolatedDeclarations; retain ordinary JS declaration behavior.
- [x] Run focused Vitest, type checks and npm run build in js-plugins/dts.

## 5. Compiler caches and virtual resources

Files: crates/compiler/src/build/module_cache.rs; crates/plugin_script/src/lib.rs; crates/swc_transformer_import_glob/src/lib.rs; relevant cache hooks and resource naming files identified by tracing.

- [x] Reproduce #1975 by warm-cache build, deleting a glob-matched route and rebuilding; include adding a new matched file and unchanged files.
- [x] Fix glob membership invalidation with directory/pattern dependency evidence or a deliberately scoped cache exclusion; verify warm restart and dev updates.
- [x] Reproduce UnoCSS/PostCSS Vue class changes and Tailwind two consecutive starts with persistent cache. Trace cache-hit plugin state and virtual stylesheet regeneration before changing behavior.
- [x] Add real output assertions for class changes and repeated builds, then implement the verified root-cause fixes.
- [x] Reproduce dynamic import of a NUL-prefixed virtual module; sanitize output naming while preserving internal ID and collision resistance; assert safe HTML/files and successful dynamic loading.

## 6. Existing fix and adapter regression coverage

- [x] Run no_expr_tpl fixture for #1833 and retain additional cases if needed.
- [x] Verify @farmfe/core/client types in a TypeScript consumer without Vite types for #1996.
- [x] Validate Vue 6 shouldTransformCachedModule startup and actual transform for #2252.
- [x] Validate default re-export parsing and runtime exports for #2131.
- [x] Confirm v2 CLI manifest has no inquirer dependency; document v1 advisory fix as separate branch/release work (#2316).
- [x] Investigate production env selection (#2183), Vite HTML pre/normal/post propagation (#1816), Vue inspector printUrls (#2157/#1918) and object/array aliases (#2124); add focused reproduction tests and fix confirmed failures.

## 7. Documentation and acceptance

- [x] Correct compilation.externals to external in English and Chinese tutorials/2-start.md.
- [x] Correct all @farmfe/visualizer installation/import references in js-plugins/visualizer/README.md to the manifest package name; use public defineConfig API.
- [x] Add scoped changesets for published package fixes.
- [x] Perform spec compliance review followed by code quality review, repair findings and rerun affected checks.
- [x] Build core using package npm scripts, build affected plugins/examples, run regression suites. Do not run cargo check/clippy or full ready unless requested.
- [x] Update docs with exact tests, outcomes, remaining limitations and final issue disposition. Keep goal active until all requested fixes and cases are verified.

## Final acceptance — 2026-10-12

All scoped implementation and regression work is complete in codex/github-bug-fixes. Tests: core/compiler libraries 34+58; affected Rust plugin runners all pass; actual compiler/Node runtime7; mangle/tree-shake5 runners; JS38; native cache/server4 plus HTML/env1; retained real Firefox baseline/fix evidence. Core/native, affected Rust/JS plugins and three relevant examples built through npm scripts. Exact commands and issue dispositions are in docs/github-issues-bug-triage-2026-10-12.md.

Runtime, tree-shake, watched-resource hashes, synthetic runtime update identity, external-HMR roots and ordered watcher delivery were added as directly necessary fixes exposed by retained end-to-end paths. No tests disable normal minify/mangle/splitting to pass. The original v1 CLI advisory is documented as a separate release branch; the current v2 CLI does not depend on inquirer. Existing core whole-test-tsconfig diagnostics are not reported as passing. No full ready/check/clippy was run. Publication is handled separately after implementation; no GitHub issue changes were made during triage.
