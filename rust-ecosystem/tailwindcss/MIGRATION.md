# Tailwind Rust migration status

Upstream source audit reference: `tailwindlabs/tailwindcss` commit `ae96721fc5453919ffa41913262f5691c51a0d46`.

## Package status

| Package | Status | Notes |
| --- | --- | --- |
| `@tailwindcss/node` | In progress | Farm already ports the compile, env, dependency tracing, instrumentation, path normalization, optimize, resolve, source-map, and URL-rewrite modules into `rust-ecosystem/tailwindcss`. Remaining gaps are Node-specific cache loader shims, TS config loading fallback (`jiti`), and parity gaps around source-map remapping. |
| `tailwindcss` | Planned | Farm does not yet have a Rust port for the core compiler pipeline (`ast`, `css-parser`, `candidate`, `utilities`, `variants`, `design-system`, `theme`, plugin API, and the full `compile/build` candidate expansion path). |

## Migration checklist

- [x] Audit upstream `@tailwindcss/node` package structure and map each source module to the Rust crate.
- [x] Audit upstream `tailwindcss` package structure and identify missing Rust workstreams.
- [x] Record the current migration status in the branch.
- [x] Move crate tests out of `src/` into `tests/`.
- [x] Add filesystem-backed fixtures and snapshot coverage for persisted inputs and outputs.
- [ ] Port Node-specific cache loader behavior (`esm-cache.loader.mts`, `require-cache`) or explicitly scope it out for Farm.
- [ ] Add a Rust-side TS/JS config loading strategy equivalent to the upstream `jiti` fallback.
- [ ] Port upstream source-map serialization/remapping parity from `@tailwindcss/node`.
- [ ] Design and implement a Rust AST/parser layer for `tailwindcss` core.
- [ ] Port candidate extraction, utilities, variants, and design-system generation from `tailwindcss` core.
- [ ] Replace the current `Compiler::build()` passthrough with full Tailwind candidate-driven CSS generation.
- [ ] Evaluate whether upstream `crates/oxide` scanner should be integrated or adapted for Farm.

## Workstream detail

### `@tailwindcss/node`

#### Done
- Resolution and module loading scaffolding
- CSS `url()` rewriting
- JS/TS dependency tracing
- `DEBUG` environment handling
- Lightning CSS optimization pass
- Instrumentation/report formatting
- Source-map wrapper helpers

#### Remaining
- Cache loader and require-cache parity
- TS config loading fallback parity
- Source-map remapping parity after optimizer rewrites
- Broader fixture parity against upstream package tests

### `tailwindcss`

#### Planned phases
1. Port the core AST, parser, walker, and source location model.
2. Port import/application transforms (`@import`, `@apply`) and theme/design-system primitives.
3. Port candidate parsing, utilities, variants, and plugin APIs.
4. Replace the temporary build passthrough with candidate expansion and source-map aware output.
5. Add upstream fixture parity for compiler outputs and candidate scanning.
