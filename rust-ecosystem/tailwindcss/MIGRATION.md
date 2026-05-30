# Tailwind Rust migration status

Upstream source audit reference: `tailwindlabs/tailwindcss` commit `ae96721fc5453919ffa41913262f5691c51a0d46`.

## Package status

| Package | Status | Notes |
| --- | --- | --- |
| `@tailwindcss/node` | In progress | Node-side compile orchestration has been split to `rust-ecosystem/tailwindcss-node` (compile, env, dependency tracing, instrumentation, path normalization, optimize, resolve, source-map, URL-rewrite). Remaining gaps are TS config loading fallback (`jiti`) and source-map remapping parity. |
| `tailwindcss` | In progress | Core-facing API now lives in `rust-ecosystem/tailwindcss` and accepts externally supplied config payloads. JS config-file loading and plugin compatibility are intentionally out of scope for this phase. |

## Migration checklist

### Shared / cross-crate

- [x] Audit upstream `@tailwindcss/node` package structure and map each source module to the Rust crate.
- [x] Audit upstream `tailwindcss` package structure and identify missing Rust workstreams.
- [x] Record the current migration status in the branch.
- [x] Move crate tests out of `src/` into `tests/`.
- [x] Add filesystem-backed fixtures and snapshot coverage for persisted inputs and outputs.
- [x] Explicitly scope out Node-specific cache loader behavior (`esm-cache.loader.mts`, `require-cache`) for Farm's Rust-only runtime boundary.
- [ ] Add a Rust-side TS/JS config loading strategy equivalent to the upstream `jiti` fallback.
- [ ] Port upstream source-map serialization/remapping parity from `@tailwindcss/node`.
- [x] Migrate upstream `@tailwindcss/node` test cases for `urls`, `source-maps`, and `instrumentation` into Rust integration tests with persisted fixtures/snapshots.
- [x] Split `tailwindcss-node` and `tailwindcss` into separate Rust crates.
- [x] Add explicit config pass-through API in `tailwindcss` core crate (no JS config file loading).
- [x] Keep plugin compatibility out of scope for the current Rust core migration.
- [ ] Design and implement a Rust AST/parser layer for `tailwindcss` core.
- [ ] Port candidate extraction, utilities, variants, and design-system generation from `tailwindcss` core.
- [ ] Replace the current `Compiler::build()` passthrough with full Tailwind candidate-driven CSS generation.
- [ ] Evaluate whether upstream `crates/oxide` scanner should be integrated or adapted for Farm.

### `tailwindcss` core crate (`rust-ecosystem/tailwindcss`)

- [x] Audit upstream `tailwindcss` core package structure and identify Rust migration scope.
- [x] Define `Features` bitflag type mirroring upstream compiler feature detection.
- [x] Define `Polyfills` bitflag type for CSS compatibility transforms.
- [x] Define `TailwindConfig` struct for externally supplied config (no JS config loading).
- [x] Define `Compiler`, `CompilerOptions`, and `compile()` as the public API entry point.
- [x] Add integration test with snapshot coverage for `Compiler` accepting external config.
- [x] Scope JS config-file loading and plugin compatibility out of this implementation.
- [x] Add a `scanner` module for extracting CSS utility class candidates from HTML/JS/CSS source text.
- [x] Add unit tests for `Features` bitflag operations (union, intersection, named constants).
- [x] Add unit tests for `Polyfills` bitflag operations.
- [x] Add snapshot tests for `scanner::extract_candidates()` with fixture inputs.
- [x] Migrate plugin's inline candidate regex into the shared `scanner` module.

## Workstream detail

### `@tailwindcss/node` (`rust-ecosystem/tailwindcss-node`)

#### Done
- Resolution and module loading scaffolding
- CSS `url()` rewriting
- JS/TS dependency tracing
- `DEBUG` environment handling
- Lightning CSS optimization pass
- Instrumentation/report formatting
- Source-map wrapper helpers

#### Remaining
- Source-map remapping parity after optimizer rewrites
- TS config loading fallback parity (explicitly scoped to non-JS-config-loading strategy for now)

#### Scoped out for Farm
- Cache loader and require-cache parity (Node.js module-loader concern; not required by the Rust crate boundary)

### `tailwindcss` (`rust-ecosystem/tailwindcss`)

#### Planned phases (original plan, `docs/superpowers/plans/2026-05-12-tailwindcss-rust-migration.md`)
1. Port the core AST, parser, walker, and source location model.
2. Port import/application transforms (`@import`, `@apply`) and theme/design-system primitives.
3. Port candidate parsing, utilities, variants, and plugin APIs.
4. Replace the temporary build passthrough with candidate expansion and source-map aware output.
5. Add upstream fixture parity for compiler outputs and candidate scanning.

#### Phase log (incremental work beyond the original plan)

The Rust port has been delivered in narrow, individually-verifiable phases.
Each phase ends with `cargo test -p farmfe_ecosystem_tailwindcss --tests`
green; the test total is recorded below so regressions are visible.

| Phase | Scope | Tests after |
| ----- | ----- | ----------- |
| 11 | Utility submodules: `segment`, `escape`, `to_key_path`, `brace_expansion`, `compare`, `compare_breakpoints`, `math_operators`, `decode_arbitrary_value`, `value_parser`, `infer_data_type`, `is_color`, `is_valid_arbitrary`, `replace_shadow_colors`, `dimensions`, `selector_parser` (incl. `:not/:where/:has/:is` recursion), `attribute_selector_parser`. | 254 |
| 12 | `optimize_ast` recursively merges adjacent same-name same-params `@`-rules. | 267 |
| 13 | `ParsedCandidate.negative`; arbitrary-value normalisation (`_` → space, preserving `\_` and `url(...)`); paren-arbitrary shorthand `bg-(--var) → var(--var)`; paren-aware variant splitter. | 267 → 293 |
| 14 | Full `Theme` parity: `ThemeOptions` bitflags, namespace wildcards, `resolve` / `resolve_value` / `resolve_with`, `ignoredThemeKeyMap`, prefix, keyframes. | 293 |
| 15 | `variants.rs` complete rewrite: at-rule wrapping (`sm/md/lg/xl/2xl` + `max-*`, `dark/print/motion-*/portrait/landscape/contrast-*`, `min-[]/max-[]`, `@container` + `@sm`–`@7xl` + `@min-[]/@max-[]/@[…]`, `supports-[]`); `data-/aria-/has-/not-` functional variants; `group-*` / `peer-*` (named + arbitrary `&`); arbitrary `[&_p]` variants. | 336 |
| 16 | `FunctionalHandler` infrastructure on `UtilityRegistry`; `Theme::with_defaults()` seeded with v4 tokens; ~50 functional utility handlers (spacing, sizing, color, radius, border, opacity, z, order, grow, shrink, basis); longest-prefix functional-key lookup. | 372 |
| 17 | `@apply` variant support: `@apply hover:flex` → nested `&:hover { display: flex }`; `@apply md:flex` → nested `@media { & { … } }`; `@apply flex!` preserves `!important`. | 377 |
| 18 | `Compiler::build` weaves user CSS AST + `@apply` substitution + `@tailwind` / `@import "tailwindcss"` marker inlining. The compiler now stores the parsed AST; `inline_tailwind_markers` recursively replaces markers with generated utilities. | 382 |
| 19 | User-defined `@utility name { … }` blocks and `@custom-variant name (selector \| @media …);` rules walked from the user AST in `DesignSystem::build` and registered into `UtilityRegistry.user_static_utilities` / `VariantRegistry.user_variants`. User entries take precedence over built-ins. `@utility` / `@custom-variant` stripped from output. | 385 |
| 20 | `@theme { … }` block parsing (with `reference / inline / static / default` modifier flags). User theme custom properties materialise as `:root, :host { … }` prepended to inlined utilities (skipping `reference` entries and namespace resets). `@theme` rules stripped from output. `DesignSystem::empty()` constructor added. | 389 |
| 21 | `@source` directive parsing: `@source "glob"`, `@source not "glob"`, `@source inline("…")`, `@source not inline("…")` collected into `DesignSystem::sources()`, exposed via `Compiler::sources()` for host-side scanner integration, and stripped from compiled output. Malformed forms silently dropped. | 394 |
| 22 | Verification + plugin hardening: confirmed the rust plugin (`farmfe_plugin_tailwindcss`) end-to-end — 12 plugin tests pass (6 unit + 6 integration). Extracted `has_tailwind_directive()` helper in the plugin so directive detection is a single named pass with a guard against `@applyXyz`-style false positives. Plan doc (`docs/superpowers/plans/2026-05-12-tailwindcss-rust-migration.md`) synced with the live phase log. | 394 |
| 23 | Phase 16.x utility follow-up: added lower-frequency list-style-type (`list-disc`), line-clamp, columns, and aspect-ratio utility support. `Theme::with_defaults()` now seeds `--aspect-video`. | 398 |
| 24 | Phase 16.x grid utility follow-up: added grid placement and track utilities (`col-*`, `col-span-*`, `col-start/end-*`, row equivalents, `auto-cols-*`, `auto-rows-*`, `grid-flow-*`, `grid-cols-*`, `grid-rows-*`) with upstream-derived coverage. | 405 |

Run-of-show for the next phases is tracked in
`docs/superpowers/plans/2026-05-12-tailwindcss-rust-migration.md`.
