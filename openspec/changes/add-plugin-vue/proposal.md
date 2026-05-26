# Proposal — Add `@farmfe/plugin-vue` (Rust plugin powered by fervid)

## Summary

Add a first-party Farm Rust plugin, `@farmfe/plugin-vue`, that compiles Vue 3 Single-File Components by delegating to the [`fervid`](https://github.com/phoenix-ru/fervid) compiler. Replaces the need for users to integrate `unplugin-vue` (which orchestrates `vue/compiler-sfc` from JS) and brings Vue support inline with Farm's other native Rust plugins (`react`, `sass`, `replace-dirname`).

## Motivation

- Farm currently has no first-party Vue SFC plugin. Users have to fall back to `unplugin-vue`, an ~2.2k LoC TS orchestrator that calls out to `vue/compiler-sfc` on the Node side — slow and inconsistent with Farm's "compiler-in-Rust" architecture.
- A Rust-native compiler ([`fervid`](https://github.com/phoenix-ru/fervid)) exists and exposes a stable `compile(source, CompileOptions) -> CompileResult` entry point. Fervid's repo also ships a `fervid_farmfe` proof-of-concept (~145 LoC) that already proves the Farm hook plumbing.
- Bringing Vue support natively into the Farm ecosystem unlocks faster dev/prod builds for Vue projects, removes the JS-orchestrator round-trip, and lets us evolve the integration in lockstep with Farm's own plugin trait.

## Goals

- Ship `@farmfe/plugin-vue` as a Farm Rust plugin under `rust-plugins/vue/`, crate `farmfe_plugin_vue`, following the conventions of `rust-plugins/react`.
- Phase A — working v0 covering the subset of `unplugin-vue`'s options surface that `fervid` supports today: `.vue` load + main module transform, `isProduction`/`ssr`/`sourceMap`/`include`/`exclude`/`customElement`/`features.{optionsAPI,prodDevtools,prodHydrationMismatchDetails,propsDestructure}`, scoped CSS & `v-bind()` for native CSS, virtual style sub-modules for downstream CSS preprocessor plugins (`@farmfe/plugin-sass`, etc.), and auto-injected `__VUE_*__` define flags.
- Phase B — close the gap with `unplugin-vue`: granular HMR (template-only / style-only), preprocessor re-scoping after sass/less, custom-block dispatch, `inlineTemplate: false`, type-only `defineProps<…>()` cross-file deps, and `componentIdGenerator`. Each item is an independent follow-up.
- CI/release pipelines build, upload, download and verify the new artifact next to the existing rust plugins.
- Docs page under `website/docs/plugins/official-plugins/vue.mdx` with options table, define-flag behaviour, and the Phase A/B feature matrix.

## Non-goals

- **Not** re-implementing or vendoring fervid. We depend on it as a published crate (`fervid = "0.2"`); upstream improvements that genuinely need fervid changes will be filed against fervid, not patched in-tree.
- **Not** preserving the `compiler` option (pin `vue/compiler-sfc` version) — fervid *is* the compiler, the option is intentionally dropped.
- **Not** preserving `unplugin-vue`'s exact scope-id algorithm (`hash(filepath)`). We accept fervid's `compute_scope_id(source)` in v0 and only surface `componentIdGenerator` if upstream support lands.
- **Not** adding a new `examples/vue` reference project in this change; that lands as a separate follow-up to keep the diff reviewable.
- **Not** doing CSS preprocessor invocation (sass/less/stylus) inside the plugin — those are emitted as `ModuleType::Custom(lang)` virtual sub-modules so the existing Farm CSS plugin chain claims them.

## Success criteria

1. `cargo check`, `cargo clippy --no-deps -p farmfe_plugin_vue`, and `cargo fmt -- --check` are green on the new crate.
2. `cargo test -p farmfe_plugin_vue` passes a hook-level integration suite covering: non-vue files ignored, basic SFC compiles, scoped style virtual module registers and is served as `ModuleType::Css`, `<style lang="scss">` virtual module is served as `ModuleType::Custom("scss")`, `.ce.vue` triggers custom-element mode, `exclude` filter is honoured, the `config` hook injects the three `__VUE_*__` define flags, `resolve.dedupe` skip for Node target, and user-supplied defines are preserved.
3. `.github/workflows/{rust-build,ci,release}.yaml` build, upload, download and verify `plugin-vue/npm/<abi>/index.farm` for every published ABI.
4. `website/docs/plugins/official-plugins/vue.mdx` documents the options surface, the auto-injected define flags, the style-virtual-module format, and the Phase A / Phase B feature matrix.
5. A changeset publishes `@farmfe/plugin-vue` at version `0.0.1`.
