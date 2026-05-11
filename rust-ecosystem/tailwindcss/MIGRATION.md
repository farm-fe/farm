# Tailwind Rust migration status

Upstream source audit reference: `tailwindlabs/tailwindcss` commit `ae96721fc5453919ffa41913262f5691c51a0d46`.

## Package status

| Package | Status | Notes |
| --- | --- | --- |
| `@tailwindcss/node` | In progress | Node-side compile orchestration has been split to `rust-ecosystem/tailwindcss-node` (compile, env, dependency tracing, instrumentation, path normalization, optimize, resolve, source-map, URL-rewrite). Remaining gaps are TS config loading fallback (`jiti`) and source-map remapping parity. |
| `tailwindcss` | In progress | Core-facing API now lives in `rust-ecosystem/tailwindcss` and accepts externally supplied config payloads. JS config-file loading and plugin compatibility are intentionally out of scope for this phase. |

## Migration checklist

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

#### Planned phases
1. Port the core AST, parser, walker, and source location model.
2. Port import/application transforms (`@import`, `@apply`) and theme/design-system primitives.
3. Port candidate parsing, utilities, variants, and plugin APIs.
4. Replace the temporary build passthrough with candidate expansion and source-map aware output.
5. Add upstream fixture parity for compiler outputs and candidate scanning.
