# Tasks — `@farmfe/plugin-vue` Phase 2

Implementation tasks for Phase 2 compatibility and developer-experience work. Keep each group independently committable; split blocked upstream work into separate follow-up changes rather than forcing it into this phase.

## 1. Reference example

- [x] 1.1. Add `examples/vue/` following the existing example project layout.
- [x] 1.2. Configure the example to use `@farmfe/plugin-vue` and local workspace dependencies.
- [x] 1.3. Add Vue Router navigation and Pinia state usage.
- [x] 1.4. Add a fixture component with scoped native CSS.
- [x] 1.5. Add a fixture component with preprocessed style when the relevant CSS preprocessor plugin is available.
- [x] 1.6. Validate with `node scripts/test-examples.mjs --skip-build-js-plugins --example vue` after the plugin artifact is available.

## 2. Virtual module contract

- [x] 2.1. Document the stable query keys for `?vue&type=style|template|custom` virtual modules.
- [x] 2.2. Add Rust tests that lock the existing style virtual ID format.
- [x] 2.3. Refactor the style registry only if custom/template virtual modules require a shared registry.

## 3. HMR foundation

- [x] 3.1. Add an internal descriptor-cache data model for source hash, style metadata, custom-element mode, and virtual IDs.
- [x] 3.2. Populate the descriptor cache during transform without changing production output.
- [x] 3.3. Add hook-level tests for cache replacement on repeated transforms.
- [x] 3.4. Investigate Farm update hooks and document whether style-only invalidation can be expressed today.
- [x] 3.5. Implement the narrowest safe invalidation supported by Farm hooks, with full-SFC reload as fallback.

## 4. Scoped preprocessor styles

- [x] 4.1. Identify whether Farm exposes a post-preprocessor CSS hook that plugin-vue can use for re-scoping.
- [x] 4.2. If supported, thread owner SFC and scope metadata through preprocessed style virtual modules.
- [x] 4.3. Re-apply Vue scope attributes after Sass/Less compilation without compiling preprocessors inside plugin-vue.
- [x] 4.4. If unsupported, add docs and tests that capture the current fallback behavior.

## 5. Custom blocks

- [x] 5.1. Inspect `fervid::CompileResult::other_assets` and map available metadata to Farm virtual modules.
- [x] 5.2. Emit `?vue&type=custom&idx=<N>&block=<tag>&lang=<lang>` virtual modules.
- [x] 5.3. Return an appropriate `ModuleType::Custom(...)` so user plugins can claim custom blocks.
- [x] 5.4. Add tests for custom block load behavior and unclaimed-block diagnostics.

## 6. Upstream-gated features

- [x] 6.1. Re-check whether `fervid` supports template-only codegen for `inlineTemplate: false`; implement only if available.
- [x] 6.2. Re-check whether `fervid` exposes type-dependency information for `defineProps<T>()`; implement invalidation only if available.
- [x] 6.3. Re-check whether `fervid` accepts a configurable component ID generator; expose `componentIdGenerator` only if available.
- [x] 6.4. Verify external `<script src>` and `<template src>` behavior; add tests or documented unsupported diagnostics.

## 7. Source maps and dependency cleanup

- [x] 7.1. Add source-map presence/basic mapping tests that avoid byte-exact snapshots.

## 8. Docs and validation

- [x] 8.1. Update `website/docs/plugins/official-plugins/vue.mdx` with Phase 2 support status and examples.
- [x] 8.2. Run `cargo fmt -p farmfe_plugin_vue --check` for Rust plugin changes.
- [x] 8.3. Run `cargo clippy --no-deps -p farmfe_plugin_vue --all-targets` for Rust plugin changes.
- [x] 8.4. Run `cargo test -p farmfe_plugin_vue` for Rust plugin changes.
- [x] 8.5. Run the affected Vue example validation when `examples/vue/` is added.
- [x] 8.6. Add a changeset if package behavior changes.
- [x] 8.7. Run final code review and security validation before completing implementation.
