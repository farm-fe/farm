# Docs Gap Overall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide one general progress view for the first broad documentation cleanup pass across plugin docs, config/feature correctness, APIs, CLI/JavaScript API docs, and migration guides.

**Architecture:** Use the existing Docusaurus docs in `website/docs` as the user-facing source and repository implementation files as the source of truth. Keep this first pass concise: identify gaps, correct high-impact inaccuracies, add missing navigable pages, and leave deeper exhaustive reference work for follow-up plans.

**Tech Stack:** Docusaurus 3, MDX/Markdown, Farm core packages, Rust plugin packages, JS plugin packages, CLI package, existing website sidebar configuration.

---

## Overall Progress Tracker

- [x] A. Official plugin docs coverage
- [ ] B. Config + feature docs correctness
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

## A. Official Plugin Docs Coverage

**Goal:** Every official plugin package has at least one navigable docs page or an intentional tracking note.

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

## B. Config + Feature Docs Correctness

**Goal:** High-traffic config and feature docs match current option names, anchors, and behavior.

- [ ] Audit `website/docs/config/configuring-farm.md` against current config loading behavior.
- [ ] Audit `website/docs/config/compilation-options.md` against current exposed `UserConfig`/compilation option types.
- [ ] Fix known broken anchors reported by Docusaurus build:
  - `#output-targetenv`
  - `#output-format`
  - `#output-librarybundletype`
  - `#externals`
  - `#outputexternalglobals`
- [ ] Audit feature pages that link to config anchors:
  - `website/docs/features/css.md`
  - `website/docs/features/library.md`
  - `website/docs/features/script.md`
  - `website/docs/advanced/polyfill.md`
  - `website/docs/advanced/ssr.md`
  - `website/docs/tutorials/build.md`
- [ ] Keep this pass focused on correctness, working links, and concise examples.

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

- [ ] Run `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website`.
- [ ] Record pre-existing warnings separately from warnings introduced by this work.
- [ ] Prioritize fixes in this order:
  1. Broken docs build or broken navigation.
  2. Incorrect install/import/config examples.
  3. Missing official package coverage.
  4. Broken anchors in current English docs.
  5. Translation/versioned-doc drift that blocks build confidence.
- [ ] Defer exhaustive API tables, full translation updates, and large conceptual rewrites to follow-up plans unless a short correction is enough.

## Validation

- [x] `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website` succeeds after installing dependencies with `corepack pnpm install --filter farm-docs --ignore-scripts`.
- [ ] Re-run `corepack pnpm build` after each docs coverage batch.
- [ ] Check changed docs pages are listed in `website/sidebars.js` or intentionally discoverable from another page.
- [ ] Check every new relative link resolves in the Docusaurus build.

## Known Pre-Existing Build Warnings

The current website build succeeds but reports pre-existing warnings unrelated to the first plugin-docs slice:

- Blog post `blog/index.md` lacks a truncation marker.
- Browserslist database is outdated.
- Sass legacy JS API deprecation warnings are emitted by the build toolchain.
- CSS minimizer warnings appear for generated `styles.*.css`.
- Multiple current and versioned docs have broken anchors, especially config-option anchors and zh writing-plugin anchors.
