# Proposal — `@farmfe/plugin-vue` Phase 2

## Summary

Extend the Phase A `@farmfe/plugin-vue` Rust plugin from basic Vue 3 SFC compilation into a practical development workflow: a reference example, clearer virtual-module semantics, HMR foundations, scoped preprocessor handling, custom-block routing, source-map validation, and dependency cleanup when upstream `fervid` permits it.

## Motivation

Phase A proves that Farm can load and transform Vue SFCs through `fervid`, emit style virtual modules, and integrate with CI/release. The remaining gap is developer experience and compatibility with common Vue projects. Vue users expect reliable dev-server updates, scoped styles that continue to work after preprocessors, custom block extensibility, and an end-to-end example that shows the supported path.

## Goals

- Add a Vue reference example that exercises router, Pinia, scoped CSS, and preprocessed styles through `@farmfe/plugin-vue`.
- Define the plugin-side architecture for descriptor caching and HMR invalidation.
- Preserve scoped CSS semantics for preprocessed style blocks without moving Sass/Less compilation into plugin-vue.
- Route custom SFC blocks through virtual modules so external Farm plugins can opt into them.
- Track source-map and `fervid` dependency stabilization work explicitly.
- Update plugin docs and tests for Phase 2 behavior.

## Non-goals

- Replacing `fervid` or vendoring Vue compiler internals.
- Implementing unsupported `fervid` compiler features in Farm as large JS fallbacks.
- Shipping every Vue compiler-sfc option in one change.
- Changing existing Phase A behavior for projects that only use basic SFC compilation.

## Success criteria

1. `examples/vue/` builds in the existing example test pipeline.
2. HMR design has concrete cache, invalidation, and fallback behavior documented in `rust-plugins/vue` tests or docs.
3. Scoped preprocessor behavior is either implemented or explicitly gated with documented fallback behavior.
4. Custom block virtual IDs are stable and documented.
5. Phase 2 docs clarify which features are implemented, gated, or blocked by upstream `fervid`.
