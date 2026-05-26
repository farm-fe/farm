# Design — `@farmfe/plugin-vue` Phase 2

## Context

Phase A adds `rust-plugins/vue/` with a native Farm Rust plugin that delegates SFC compilation to `fervid`. It currently handles `.vue` loading, main-module transformation, style virtual modules, custom-element filtering, Vue define flags, and package/CI/release wiring.

The Phase A task list explicitly defers granular HMR, preprocessor re-scoping, `inlineTemplate: false`, custom blocks, type-dependency invalidation, configurable component IDs, external `<script src>`/`<template src>`, full source-map fidelity, returning to a crates.io `fervid` release, and a reference example.

Phase 2 should be treated as a compatibility and developer-experience phase. Work should land in small slices and avoid taking ownership of work that belongs in `fervid`.

## Design goals

1. Keep plugin-vue primarily Rust-native and keep the JS surface minimal.
2. Preserve Phase A virtual style module behavior for existing users.
3. Prefer stable virtual-module contracts over ad hoc string formats.
4. Add tests around Farm-observable behavior instead of snapshotting unstable `fervid` output.
5. Gate or document features blocked by `fervid` rather than inventing incomplete compiler behavior in Farm.

## Architecture

### 1. Reference Vue example

Add `examples/vue/` as the first Phase 2 deliverable. It should use the normal Farm example shape, depend on `@farmfe/plugin-vue`, and cover:

- Vue 3 app bootstrapping.
- Vue Router navigation.
- Pinia state usage.
- A scoped native CSS block.
- A preprocessed style block, preferably `scss`, when the Sass plugin is available.

The example is the end-to-end acceptance target for Phase 2. CI already downloads Rust plugin artifacts into `rust-plugins/*/npm/<abi>` before running example tests, so the example can consume the local plugin package once the plugin-vue artifact exists.

### 2. Descriptor cache and HMR foundation

Introduce a descriptor cache keyed by resolved file path or stable module ID. The cache should store the minimum data needed to compare updates:

- source hash;
- style block count, languages, scoped flags, and content hashes;
- whether the file compiled as a custom element;
- imported type dependencies if/when `fervid` exposes them;
- generated virtual IDs for style and future custom/template blocks.

On file changes, the plugin should re-read and recompile the SFC, compare the new descriptor metadata with the cached entry, and choose the narrowest safe invalidation:

| Change kind | Preferred action | Fallback |
|---|---|---|
| style-only native CSS | invalidate affected style virtual modules | reload main SFC |
| style-only preprocessed CSS | invalidate affected custom style modules, then re-scope if supported | reload main SFC |
| template-only | invalidate template virtual module if split codegen exists | reload main SFC |
| script/setup change | invalidate main SFC | reload main SFC |
| custom block change | invalidate matching custom-block virtual module | reload main SFC |
| unknown descriptor change | reload main SFC | reload main SFC |

If Farm's current plugin hooks cannot express the needed update granularity, Phase 2 should document the gap and keep the cache as a foundation for a later hook addition.

### 3. Stable virtual module IDs

Phase A uses style IDs of the form:

`<module_id>?vue&type=style&idx=<N>&lang=<lang>&scoped=<bool>`

Phase 2 should keep that form compatible, but normalize future IDs around these query keys:

- `vue` marker: identifies SFC virtual modules;
- `type`: `style`, `template`, `custom`;
- `idx`: zero-based block index;
- `lang`: normalized block language;
- `scoped`: boolean for style blocks;
- `block`: custom block tag name, when `type=custom`.

Use the existing style registry pattern for style blocks and extend it to a generic SFC virtual module registry only when custom/template blocks are implemented.

### 4. Scoped preprocessor handling

Phase A deliberately emits `lang="scss"` and similar blocks as `ModuleType::Custom(lang)` so downstream CSS preprocessor plugins can compile them. The missing piece is re-applying Vue scope attributes after preprocessing.

Preferred design:

1. plugin-vue emits raw preprocessor style virtual modules with enough metadata to identify the owning SFC and scope ID;
2. downstream Sass/Less plugins compile the block into CSS;
3. plugin-vue receives or reclaims the compiled CSS through an existing post-transform/process hook;
4. plugin-vue applies the scope attribute rewrite to the compiled CSS;
5. final output continues through Farm's CSS pipeline.

If Farm does not expose a suitable hook ordering point, Phase 2 should not compile Sass inside plugin-vue. Instead, document the limitation and add tests that lock the current fallback behavior.

### 5. Custom blocks

`fervid` exposes non-style/script/template assets as `other_assets`. Phase 2 should expose these as virtual modules rather than silently dropping them.

Proposed custom-block ID:

`<module_id>?vue&type=custom&idx=<N>&block=<tag>&lang=<lang>`

The plugin should return `ModuleType::Custom(lang_or_tag)` so userland or future first-party plugins can claim blocks such as `<i18n>` or `<docs>`. If no plugin claims the custom block, Farm should fail or warn consistently with existing unknown-module behavior rather than injecting unused code.

### 6. Template split and `inlineTemplate: false`

Fervid currently emits one combined script/template module. Phase 2 should not emulate `compiler-sfc` template splitting unless `fervid` exposes per-block codegen.

Design the option shape and virtual ID now, but gate implementation:

- `inlineTemplate` remains effectively `true` by default.
- `inlineTemplate: false` should produce a clear unsupported diagnostic unless upstream support exists.
- Once available, template virtual IDs should use `type=template` and integrate with the HMR descriptor cache.

### 7. Type dependency invalidation

Type-only `defineProps<T>()` dependencies require compiler-level knowledge of imported type files. Phase 2 should add an internal representation for type dependencies only after `fervid` exposes them. Until then, the docs should state that changing an imported type may require touching the importing `.vue` file or restarting dev.

### 8. Component ID generator

Phase A accepts `fervid`'s scope ID algorithm. Phase 2 should add a public option only if `fervid` accepts an injected ID generator or equivalent stable option. If unsupported, keep `componentIdGenerator` documented as blocked and avoid a Farm-only divergent implementation.

### 9. Source maps and dependency cleanup

Phase 2 should periodically test whether a published `fervid` version builds against the repository's current `serde` and `swc_common` graph. When it does, replace the git pin with a crates.io semver dependency and regenerate `Cargo.lock` through Cargo.

Source-map work should remain validation-driven: add fixtures that assert source-map presence, file names, and basic mappings, but avoid byte-for-byte snapshots while `fervid` mapping output is partial.

## Testing strategy

- Add hook-level Rust tests for descriptor-cache metadata and virtual ID stability.
- Add tests for custom block virtual-module registration once implemented.
- Add tests for unsupported diagnostics where features are gated by `fervid`.
- Add `examples/vue/` to the example test path and run the single-example command during implementation.
- Continue running `cargo fmt -p farmfe_plugin_vue --check`, `cargo clippy --no-deps -p farmfe_plugin_vue --all-targets`, and `cargo test -p farmfe_plugin_vue` for Rust plugin changes.

## Documentation strategy

Update `website/docs/plugins/official-plugins/vue.mdx` with:

- a Phase 2 compatibility table;
- reference example link;
- HMR behavior and known fallbacks;
- custom block virtual ID format;
- scoped preprocessor support status;
- upstream-gated features and diagnostics.

## Risks

- Farm may not currently expose a hook that can re-scope CSS after Sass/Less compilation.
- Fervid may not expose enough descriptor detail for precise HMR, type dependency tracking, component ID generation, or template splitting.
- A Vue example can become flaky if it depends on locally built native artifacts without matching CI download paths.
- Implementing too many Phase B items in one PR could make review difficult; prefer independent slices.
