# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Farm is an extremely fast, Vite-compatible web build tool written in Rust (v2.x beta). It is a hybrid Rust + TypeScript monorepo using pnpm workspaces and Cargo workspace. The Rust compiler is exposed to Node.js via napi-rs.

## Key Commands

| Command | Purpose |
|---------|---------|
| `pnpm bootstrap` | First-time setup: install deps + build core packages |
| `pnpm run ready` | Full CI gate (install, clean, build, lint, type-check, test, e2e) |
| `pnpm run test` | TypeScript unit tests (vitest) |
| `pnpm run test-e2e` | E2E tests (vitest + Playwright) |
| `cargo test` | Rust unit tests |
| `cargo test -p farmfe_compiler` | Run tests for a specific Rust crate |
| `cargo test --profile ci-test` | Run Rust tests with CI-optimized profile |
| `pnpm run check` | Biome lint + format |
| `cargo clippy` | Rust linter |
| `cargo check --all --all-targets` | Type-check all Rust code without building |
| `pnpm run spell-check` | cspell across all files |

### Rust snapshot testing uses insta
- `INSTA_UPDATE=always cargo test` to update snapshots
- Or set `INSTA_UPDATE=always` env var on Windows

## Architecture

### Rust layer (bottom-up)

1. **`farmfe_core`** (`crates/core/`) — Central type system: `Config`, `Module`, `ModuleGraph`, the `Plugin` trait, caching infrastructure. Everything depends on it.
2. **`farmfe_toolkit`** (`crates/toolkit/`) — SWC-based transformation engine (parse, codegen, minify, transpile for JS/CSS/HTML).
3. **Builtin plugins** (`crates/plugin_*`) — 16+ crates implementing the `Plugin` trait, compiled statically into the compiler. Pipeline-critical ones: `plugin_resolve`, `plugin_script`, `plugin_css`, `plugin_html`, `plugin_runtime`, `plugin_tree_shake`. Optimization: `plugin_minify`, `plugin_partial_bundling`.
4. **`farmfe_compiler`** (`crates/compiler/`) — Orchestrator that wires all builtin plugins together based on config.
5. **`farmfe_node`** (`crates/node/`) — napi-rs cdylib bridging Rust to Node.js. The primary delivery vehicle.
6. **`rust-plugins/`** — External plugins compiled to cdylib and loaded dynamically at runtime (react, sass, tailwindcss, dts, replace-dirname). Other directories here are JS-only.

### TypeScript layer

- **`packages/core`** (`@farmfe/core`) — Main package: dev server, file watcher, compiler JS wrapper, plugin bridge. Wraps the native Rust binding.
- **`packages/cli`** (`@farmfe/cli`) — CLI entry point (`farm` bin), uses cac.
- **`packages/runtime`** (`@farmfe/runtime`) — Browser runtime (module system, HMR).
- **`js-plugins/`** — Official JS plugins (postcss, less, sass, svgr, dts, visualizer, tailwindcss, electron). Each uses Farm itself as its build tool (dogfooding).

### JS-Rust bridge

```
User Code → @farmfe/cli → @farmfe/core (resolveConfig, createCompiler)
  → Compiler (JS wrapper) → BindingCompiler (napi-rs native addon)
  → farmfe_node cdylib → Rust compiler pipeline → Results as JS objects
```

The native binding is built via `napi build --platform -p farmfe_node --manifest-path ../../crates/node/Cargo.toml` and outputs `packages/core/binding/binding.cjs` + `binding.d.ts`.

## Conventions

- **Rust**: edition 2021, toolchain pinned in `rust-toolchain.toml`, format with `rustfmt.toml`, lint with `cargo clippy`
- **TypeScript**: Biome for format+lint (`biome.json`), ESM source with dual CJS+ESM output, `tsconfig.base.json` as base
- **Git**: Conventional Commits (`feat:`, `fix:`, `chore:`), changesets for versioning (`npx changeset`), PR titles must match Conventional Commits
- **Package manager**: pnpm v9.4.0 (enforced)
- **Node**: >=20

## Do Not

- Force push or hard reset without explicit approval
- Edit `pnpm-lock.yaml` manually
- Edit generated files (`.d.ts` in `dist/`, binding files)
- Bypass hooks with `--no-verify`
