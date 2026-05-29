# Docs Gap Overall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide one general progress view for the first broad documentation cleanup pass across plugin docs, config/feature correctness, APIs, CLI/JavaScript API docs, and migration guides.

**Architecture:** Use the existing Docusaurus docs in `website/docs` as the user-facing source and repository implementation files as the source of truth. Keep this first pass concise: identify gaps, correct high-impact inaccuracies, add missing navigable pages, and leave deeper exhaustive reference work for follow-up plans.

**Tech Stack:** Docusaurus 3, MDX/Markdown, Farm core packages, Rust plugin packages, JS plugin packages, CLI package, existing website sidebar configuration.

---

## Overall Progress Tracker

- [x] A. Official plugin docs coverage and completeness
- [x] B. Config + feature docs correctness
- [ ] C. JS/Rust plugin API docs
- [ ] D. CLI + JavaScript API docs
- [ ] E. Migration guide cleanup
- [ ] F. Broad first pass across all of the above, concise rather than exhaustive

## Source-of-Truth Map

- Website docs root: `website/docs`
- Website navigation: `website/sidebars.js`
- Official Rust plugins: `rust-plugins/*`
- Official JS plugins: `js-plugins/*`
- Core config/API implementation and types: `packages/core`, `crates/*`
- CLI implementation: `packages/cli`
- Existing migration docs: `website/docs/migration`

## A. Official Plugin Docs Coverage and Completeness

**Goal:** Every official plugin package has a navigable docs page that is not just present, but useful: install command, basic config, options, source-backed behavior notes, examples or known limitations where applicable.

**2026-05-28 re-audit requirement:** Before continuing other website-gap tasks, compare `rust-plugins/*` and `js-plugins/*` against `website/docs/plugins/official-plugins` and `website/sidebars.js`. The earlier pass added pages for all current packages, but this section remains open until the completeness pass validates and improves high-priority pages such as Vue and TailwindCSS.

### A.0 Current plugin inventory from package manifests

Rust plugin packages:

- `@farmfe/plugin-auto-import` → `website/docs/plugins/official-plugins/auto-import.mdx`
- `@farmfe/plugin-compress` → `website/docs/plugins/official-plugins/compress.mdx`
- `@farmfe/plugin-dsv` → `website/docs/plugins/official-plugins/dsv.mdx`
- `@farmfe/plugin-dts` → `website/docs/plugins/official-plugins/dts.mdx`
- `@farmfe/plugin-icons` → `website/docs/plugins/official-plugins/icons.mdx`
- `@farmfe/plugin-image` → `website/docs/plugins/official-plugins/image.mdx`
- `@farmfe/plugin-mdx` → `website/docs/plugins/official-plugins/mdx.mdx`
- `@farmfe/plugin-modular-import` → `website/docs/plugins/official-plugins/modular-import.mdx`
- `@farmfe/plugin-react` → `website/docs/plugins/official-plugins/react.mdx`
- `@farmfe/plugin-react-components` → `website/docs/plugins/official-plugins/react-components.mdx`
- `@farmfe/plugin-replace-dirname` → `website/docs/plugins/official-plugins/replace-dirname.mdx`
- `@farmfe/plugin-sass` → `website/docs/plugins/official-plugins/sass.mdx`
- `@farmfe/plugin-strip` → `website/docs/plugins/official-plugins/strip.mdx`
- `@farmfe/plugin-svgr` → `website/docs/plugins/official-plugins/svgr.mdx`
- `@farmfe/plugin-tailwindcss` → `website/docs/plugins/official-plugins/tailwindcss.mdx`
- `@farmfe/plugin-url` → `website/docs/plugins/official-plugins/url.mdx`
- `@farmfe/plugin-virtual` → `website/docs/plugins/official-plugins/virtual.mdx`
- `@farmfe/plugin-vue` → `website/docs/plugins/official-plugins/vue.mdx`
- `@farmfe/plugin-wasm` → `website/docs/plugins/official-plugins/wasm.mdx`
- `@farmfe/plugin-worker` → `website/docs/plugins/official-plugins/worker.mdx`
- `@farmfe/plugin-yaml` → `website/docs/plugins/official-plugins/yaml.mdx`

JavaScript plugin packages:

- `@farmfe/js-plugin-babel` → `website/docs/plugins/official-plugins/js-babel.mdx`
- `@farmfe/js-plugin-copy` → `website/docs/plugins/official-plugins/js-copy.mdx`
- `@farmfe/js-plugin-dts` → `website/docs/plugins/official-plugins/js-dts.mdx`
- `@farmfe/js-plugin-electron` → `website/docs/plugins/official-plugins/js-electron.mdx`
- `@farmfe/js-plugin-less` → `website/docs/plugins/official-plugins/js-less.mdx`
- `@farmfe/js-plugin-postcss` → `website/docs/plugins/official-plugins/js-postcss.mdx`
- `@farmfe/js-plugin-qiankun` → `website/docs/plugins/official-plugins/js-qiankun.mdx`
- `@farmfe/js-plugin-react-compiler` → `website/docs/plugins/official-plugins/js-react-compiler.mdx`
- `@farmfe/js-plugin-sass` → `website/docs/plugins/official-plugins/js-sass.mdx`
- `@farmfe/js-plugin-svgr` → `website/docs/plugins/official-plugins/js-svgr.mdx`
- `@farmfe/js-plugin-tailwindcss` → `website/docs/plugins/official-plugins/js-tailwindcss.mdx`
- `@farmfe/js-plugin-visualizer` → `website/docs/plugins/official-plugins/js-visualizer.mdx`
- `@farmfe/js-plugin-vuetify` → `website/docs/plugins/official-plugins/js-vuetify.mdx`

### A.1 Existing coverage pass

- [x] Add first coverage slice for:
  - `@farmfe/plugin-tailwindcss`
  - `@farmfe/js-plugin-tailwindcss`
  - `@farmfe/js-plugin-visualizer`
  - `@farmfe/js-plugin-electron`
- [x] Link first-slice pages from `website/sidebars.js`.
- [x] Link first-slice pages from `website/docs/plugins/official-plugins/overview.md`.
- [x] Add README-backed Rust plugin pages:
  - `@farmfe/plugin-svgr`
  - `@farmfe/plugin-wasm`
  - `@farmfe/plugin-worker`
  - `@farmfe/plugin-url`
  - `@farmfe/plugin-icons`
  - `@farmfe/plugin-image`
  - `@farmfe/plugin-compress`
  - `@farmfe/plugin-modular-import`
- [x] Add source-backed Rust plugin pages:
  - `@farmfe/plugin-auto-import`
  - `@farmfe/plugin-dts`
  - `@farmfe/plugin-mdx`
  - `@farmfe/plugin-replace-dirname`
- [x] Add remaining JS plugin pages:
  - `@farmfe/js-plugin-babel`
  - `@farmfe/js-plugin-copy`
  - `@farmfe/js-plugin-qiankun`
  - `@farmfe/js-plugin-react-compiler`
  - `@farmfe/js-plugin-vuetify`
- [x] Keep install, usage, and options examples short and sourced from package README files, `index.d.ts`, or `src/index.ts`.

### A.2 Completeness pass before continuing

- [x] Verify every package listed in A.0 appears in `website/sidebars.js`.
- [x] Verify every package listed in A.0 appears in `website/docs/plugins/official-plugins/overview.md`.
- [x] Overhaul `website/i18n/zh/docusaurus-plugin-content-docs/current/plugins/official-plugins/overview.md` so current zh docs match the current package inventory:
  - Remove phantom packages `@farmfe/js-plugin-vue` and `@farmfe/js-plugin-solid`.
  - Add missing Rust plugin entries for `vue`, `auto-import`, `tailwindcss`, `svgr`, `wasm`, `worker`, `url`, `dts`, `icons`, `image`, `mdx`, `compress`, `modular-import`, and `replace-dirname`.
  - Add missing JS plugin entries for `js-babel`, `js-copy`, `js-electron`, `js-qiankun`, `js-react-compiler`, `js-tailwindcss`, `js-visualizer`, and `js-vuetify`.
  - Fix copy-paste placeholder descriptions for `js-postcss`, `js-less`, `js-svgr`, `js-dts`, and `js-sass`.
- [x] For each Rust plugin page, compare against `rust-plugins/<name>/package.json`, `index.d.ts`, README files when present, and `src/lib.rs`.
- [x] For each JS plugin page, compare against `js-plugins/<name>/package.json`, README files when present, and `src/index.ts` or emitted type definitions.
- [x] Upgrade `@farmfe/plugin-vue` docs first:
  - Confirm the page describes the current experimental status and `fervid` limitations accurately.
  - Confirm options match `rust-plugins/vue/index.d.ts` and `rust-plugins/vue/src/lib.rs`.
  - Confirm examples mention needed peer dependencies and common SFC style/preprocessor pairing.
- [x] Upgrade `@farmfe/plugin-tailwindcss` docs next:
  - Confirm options match `rust-plugins/tailwindcss/index.d.ts` and `rust-plugins/tailwindcss/src/lib.rs`.
  - Clarify whether the Rust plugin uses Tailwind v4 CSS-first `@import "tailwindcss"` behavior, content scanning, config loading, and `node_modules` exclusion.
  - Document known limitations and the relationship to `@farmfe/js-plugin-tailwindcss`.
- [x] Upgrade `@farmfe/js-plugin-tailwindcss` docs next:
  - Confirm options match `js-plugins/tailwindcss/src/index.ts` and package dependencies.
  - Clarify Tailwind v4 package usage, filters, metadata flow, and when to prefer the Rust plugin.
- [x] Spot-check medium-risk pages whose implementations often have non-obvious options or limitations:
  - `@farmfe/plugin-react`
  - `@farmfe/plugin-sass`
  - `@farmfe/plugin-dts`
  - `@farmfe/plugin-react-components`
  - `@farmfe/js-plugin-dts`
  - `@farmfe/js-plugin-electron`
  - `@farmfe/js-plugin-vuetify`
- [x] Keep the pass concise: prefer source-backed option tables, minimal config examples, and "Known limitations" notes over large conceptual rewrites.
- [x] After updating plugin pages, run `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website` and confirm no new broken links are introduced.

## B. Config + Feature Docs Correctness

**Goal:** High-traffic config and feature docs match current option names, anchors, and behavior.

- [x] Audit `website/docs/config/configuring-farm.md` against current config loading behavior.
- [x] Audit `website/docs/config/compilation-options.md` against current exposed `UserConfig`/compilation option types.
- [x] Replace stale `@farmfe/js-plugin-vue` current-doc examples with current Vue guidance:
  - `website/docs/using-plugins.mdx`
  - `website/docs/features/script.md`
  - `website/docs/config/compilation-options.md`
  - `website/i18n/zh/docusaurus-plugin-content-docs/current/using-plugins.md`
  - `website/i18n/zh/docusaurus-plugin-content-docs/current/features/script.md`
  - `website/i18n/zh/docusaurus-plugin-content-docs/current/config/compilation-options.md`
- [x] Leave `version-1.x` `@farmfe/js-plugin-vue` references unchanged because they document historical v1 behavior.
- [x] Fix known broken anchors reported by Docusaurus build:
  - `#output-targetenv`
  - `#output-format`
  - `#output-librarybundletype`
  - `#externals`
  - `#outputexternalglobals`
- [x] Audit feature pages that link to config anchors:
  - `website/docs/features/css.md`
  - `website/docs/features/library.md`
  - `website/docs/features/script.md`
  - `website/docs/advanced/polyfill.md`
  - `website/docs/advanced/ssr.md`
  - `website/docs/tutorials/build.md`
- [x] Keep this pass focused on correctness, working links, and concise examples.

## C. JS/Rust Plugin API Docs

**Goal:** Plugin author docs describe current hooks, types, and recommended patterns.

- [ ] Audit `website/docs/api/js-plugin-api.md` against current JS plugin type definitions.
- [ ] Audit `website/docs/api/rust-plugin-api.md` against current Rust plugin traits and hook signatures.
- [ ] Audit writing guides:
  - `website/docs/plugins/writing-plugins/js-plugin.mdx`
  - `website/docs/plugins/writing-plugins/rust-plugin.mdx`
  - `website/docs/plugins/writing-plugins/runtime-plugin.md`
- [ ] Fix broken writing-guide anchors currently reported for zh/current and zh/1.x pages where the English source change can prevent future drift.
- [ ] Add concise hook lifecycle notes only where they remove ambiguity.

## D. CLI + JavaScript API Docs

**Goal:** CLI and JavaScript API pages reflect current exported commands and programmatic APIs.

- [ ] Audit `website/docs/cli/cli-api.md` against `packages/cli`.
- [ ] Audit `website/docs/api/javascript-api.mdx` against current JavaScript exports from Farm packages.
- [ ] Verify examples use current import paths and config helpers.
- [ ] Remove stale commands or mark legacy-only commands when they still apply to versioned docs.
- [ ] Keep examples minimal and runnable.

## E. Migration Guide Cleanup

**Goal:** Migration pages are easy to follow and do not conflict with current 2.x docs.

- [ ] Audit `website/docs/migration/from-vite.md` for outdated Vite comparison notes and broken links.
- [ ] Audit `website/docs/migration/v1-to-v2.md` for stale option names and missing 2.x redirects.
- [ ] Separate current migration guidance from version-specific historical guidance.
- [ ] Prefer short tables for renamed options, changed defaults, and replacement commands.

## F. Broad First Pass

**Goal:** Complete a concise sweep that improves navigability and correctness without attempting exhaustive reference rewrites.

- [x] Run `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website`.
- [x] Record pre-existing warnings separately from warnings introduced by this work.
- [ ] Prioritize fixes in this order:
  1. Broken docs build or broken navigation.
  2. Incorrect install/import/config examples.
  3. Missing official package coverage.
  4. Broken anchors in current English docs.
  5. Translation/versioned-doc drift that blocks build confidence.
- [ ] Defer exhaustive API tables, full translation updates, and large conceptual rewrites to follow-up plans unless a short correction is enough.

## Validation

- [x] `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website` succeeds after installing dependencies with `corepack pnpm install --filter farm-docs --ignore-scripts`.
- [x] Re-run `corepack pnpm build` after each docs coverage batch.
- [x] Check changed docs pages are listed in `website/sidebars.js` or intentionally discoverable from another page.
- [x] Check every new relative link resolves in the Docusaurus build.

## Known Pre-Existing Build Warnings

The current website build succeeds but reports pre-existing warnings unrelated to the first plugin-docs slice:

- Blog post `blog/index.md` lacks a truncation marker.
- Browserslist database is outdated.
- Sass legacy JS API deprecation warnings are emitted by the build toolchain.
- CSS minimizer warnings appear for generated `styles.*.css`.
- No broken config-option anchors were reported by the 2026-05-29 build after the config/feature docs pass.
