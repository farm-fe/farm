# Tailwind Rust Migration Design

## Problem

Farm already has a JavaScript Tailwind plugin, but the long-term target is a Rust-first implementation that moves the Tailwind compilation path into the Rust ecosystem crates. The migration should align with the upstream `tailwindcss` and `@tailwindcss/node` package boundaries, while intentionally excluding the Tailwind plugin system and config-file loading.

The design must preserve the working Rust scanner and focus on refactoring the remaining compile and optimize flow so that the Farm Rust plugin becomes a thin integration layer over Rust ecosystem crates.

## Goals

- Align the Rust surface with the upstream `tailwindcss` and `@tailwindcss/node` module responsibilities.
- Reuse the existing Rust scanner instead of rewriting it.
- Accept Tailwind config as structured input data only.
- Avoid reading `tailwind.config.js` or `tailwind.config.ts`.
- Exclude Tailwind plugin-system compatibility.
- Drive the migration with TDD by migrating and extending tests before implementation.

## Non-Goals

- Supporting the Tailwind plugin system.
- Loading Tailwind config from JavaScript or TypeScript files.
- Rewriting the existing Rust scanner.
- Preserving Node-specific APIs such as `clearRequireCache` as-is.

## Architecture

### `rust-ecosystem\tailwindcss`

This crate is the core Tailwind surface. It owns compile/build state and externally supplied config data, but it does not know about Farm hooks, watch graphs, or Node-specific cache behavior.

Responsibilities:

- Maintain the compiler state returned by `compile`.
- Expose feature detection for `@apply`, `theme()`, and utility imports.
- Build final CSS from aggregated candidates.
- Produce source maps only when explicitly enabled.
- Store externally supplied Tailwind config as structured data.

Expected stable API shape:

- `compile(css, CompilerOptions) -> Compiler`
- `Compiler::build(candidates) -> String`
- `Compiler::build_source_map() -> Option<_>`
- `Compiler::dependencies() -> &[String]`
- `Compiler::config() -> Option<&TailwindConfig>`

### `rust-ecosystem\tailwindcss-node`

This crate is the orchestration layer that mirrors the upstream `@tailwindcss/node` package. It should compose resolution, stylesheet loading, dependency tracing, URL rewriting, source-map handling, instrumentation, and optimize behavior into reusable Rust APIs.

Responsibilities:

- Resolve CSS and JS dependencies through explicit resolver hooks.
- Expand `@import` chains and collect dependencies.
- Rewrite relative URLs when requested.
- Expose the compile entrypoint used by the Farm plugin.
- Expose optimize behavior matching the JavaScript plugin pipeline.
- Provide `Features`, instrumentation, `DEBUG` handling, and source-map utilities used by integration layers.

Expected stable API areas:

- `compile`
- `optimize`
- `Features`
- `instrumentation`
- `env`
- `source_maps`

### `rust-plugins\tailwindcss`

This crate is the Farm integration layer. It should stay thin and implement the Farm-specific behavior that exists today in the JavaScript plugin's `Root.generate()` flow.

Responsibilities:

- Scan non-CSS modules for candidates by reusing the existing Rust scanner.
- Detect Tailwind CSS roots.
- Cache compiler state per CSS root.
- Rebuild when tracked dependencies change.
- Aggregate candidates and pass them to the compiler.
- Register watch files with Farm.
- Return optimized CSS and source maps back through Farm hooks.

## API and Behavior Parity

The JavaScript plugin currently relies on more than `compile` and `optimize`. The Rust design should cover these APIs or their equivalent semantics:

- `compile`
- `optimize`
- `Features`
- `Instrumentation`
- `env.DEBUG`
- source-map wrapping equivalent to `toSourceMap`

The design should **not** preserve the literal Node API for `clearRequireCache`, but it **must** preserve the rebuild-invalidates-compiler behavior that the API currently supports in the JavaScript flow.

The scanner remains out of scope for migration because a Rust implementation already exists and should be reused directly.

## Data Flow

1. A non-CSS module enters the Farm plugin and is scanned for Tailwind candidates with the existing Rust scanner.
2. A CSS module that matches Tailwind root criteria enters the plugin's root-generation flow.
3. The plugin checks tracked build dependencies to decide whether the cached compiler can be reused.
4. If a rebuild is required, the plugin calls `tailwindcss-node::compile(...)` to create fresh compiler state.
5. The plugin aggregates collected candidates and calls `compiler.build(candidates)`.
6. If source maps are enabled, the plugin reads `build_source_map()` and passes it through the Rust source-map utility layer.
7. The plugin runs `tailwindcss-node::optimize(...)`.
8. The final CSS and source map are returned to Farm.

## Error Handling

- Core and node crates should expose explicit `Result`-based failure paths for compilation and orchestration errors.
- "Not a Tailwind root" is a valid no-op outcome.
- "Detected as a Tailwind root but failed during compile or optimize" is an error and should be surfaced to Farm rather than silently ignored.
- The Rust Farm plugin should not keep the current silent-failure pattern of logging and returning `Ok(None)` after a compile failure, because that hides real migration gaps and weakens TDD feedback.

## Testing Strategy

The migration should be implemented with TDD and layered tests.

### Phase 1: Core crate tests

Migrate and expand tests that lock down:

- feature detection for `@apply`, `theme()`, and `@import "tailwindcss"`
- config pass-through behavior
- candidate-sensitive `build(candidates)` output
- source-map enablement behavior

Goal: `Compiler` becomes real compile/build state instead of a string wrapper.

### Phase 2: Node crate tests

Migrate and expand tests that lock down:

- `@import` expansion
- dependency collection
- CSS and JS resolver behavior
- URL rewriting
- source-map output wrapping
- instrumentation behavior
- `DEBUG` resolution behavior
- optimize behavior, including the two-pass flow and media-query fixup

Goal: `tailwindcss-node` can reproduce the JavaScript compile/optimize pipeline without Farm integration.

### Phase 3: Farm plugin tests

Add focused integration tests that lock down:

- pass-through behavior for non-CSS modules
- Tailwind-root detection
- `compile -> build -> optimize` execution order
- candidate aggregation affecting final CSS
- dependency changes triggering rebuild
- watch-file registration
- source-map propagation through Farm hooks

The plugin test suite should validate integration semantics only. It should not repeat lower-level compile or optimize detail that is already covered in ecosystem-crate tests.

## Recommended Migration Order

1. Reuse the existing Rust scanner unchanged.
2. Migrate failing tests for `tailwindcss` core compile/build behavior.
3. Implement the missing core compile/build behavior until those tests pass.
4. Migrate failing tests for `tailwindcss-node` compile/optimize behavior.
5. Implement the missing node orchestration and optimize behavior until those tests pass.
6. Add Farm plugin integration tests for root caching, rebuild invalidation, candidate aggregation, and hook output.
7. Implement the Farm-specific glue as the final step.

## Design Rationale

This split keeps upstream Tailwind behavior aligned with upstream package boundaries while preventing Farm-specific lifecycle concerns from leaking into the ecosystem crates. It also keeps the TDD feedback loop narrow: core failures stay in core tests, node failures stay in node tests, and Farm integration failures stay in plugin tests.

That separation is the best fit for the target outcome: upstream capability parity first, Farm integration second, without bringing back config-file loading or plugin-system support.
