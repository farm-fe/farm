# TailwindCSS Rust Migration Design Spec

> **Status:** Draft / Design Phase
> **Branch:** `copilot/refactor-js-ecosystem-to-rust`
> **Created:** 2026-05-12

## 1. Goal

Replace the JS implementation of the tailwindcss *compiler* (CSS AST parsing, candidate-driven utility generation) with a pure-Rust implementation in the `rust-ecosystem/tailwindcss` crate, while leveraging the upstream `tailwindcss-oxide` crate (already vendored at `tailwindcss/crates/oxide/`) for candidate scanning.

After this migration, `rust-plugins/tailwindcss` will be the primary tailwindcss plugin — no JS fallback required.

### Explicit non-goals

- No Tailwind plugin system / JS plugin compat (`@plugin`)
- No JS/TS config file loading (config is externally supplied as `serde_json::Value`)
- No IntelliSense / `getClassList()` / class sorting
- No v3 compat layer (`compat/`)
- Scanner rewrite (use upstream `tailwindcss-oxide` directly)

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  rust-plugins/tailwindcss  (Farm plugin — minimal changes)      │
│                                                                 │
│  Scanner now uses tailwindcss_oxide::Scanner directly           │
│  instead of farmfe_ecosystem_tailwindcss::scanner               │
│                                                                 │
│  Transform phase: scan files → accumulate candidates            │
│  FreezeModule phase: compile CSS → build(candidates) → optimize │
└──────────┬─────────────────────────────────┬────────────────────┘
           │                                 │
           ▼                                 ▼
┌──────────────────────┐   ┌──────────────────────────────────────┐
│ tailwindcss-oxide    │   │ farmfe_ecosystem_tailwindcss_node     │
│ (vendored upstream)  │   │ (unchanged except Compiler wiring)    │
│                      │   │                                      │
│ · Scanner            │   │ · compile.rs → @import resolution    │
│ · scan_content()     │   │ · optimize.rs → lightningcss pass   │
│ · extractor          │   │ · resolve.rs → module resolution     │
└──────────────────────┘   └──────────────┬───────────────────────┘
                                          │
                                          ▼
                               ┌──────────────────────────────────┐
                               │ farmfe_ecosystem_tailwindcss      │
                               │ (core compiler — HEAVY REWRITE)  │
                               │                                  │
                               │ · ast.rs         CSS AST types  │
                               │ · parser.rs      CSS → AST       │
                               │ · candidate.rs   candidate parse │
                               │ · compiler.rs    top-level API   │
                               │ · design_system.rs               │
                               │ · theme.rs       @theme handling │
                               │ · utilities.rs   built-in utils  │
                               │ · variants.rs    built-in vars   │
                               │ · apply.rs       @apply expand   │
                               │ · functions.rs   theme() etc.    │
                               │ · walk.rs        AST traversal   │
                               │ · selector_parser.rs             │
                               │ · value_parser.rs                │
                               └──────────────────────────────────┘
```

## 3. Crate Changes

### 3.1 `farmfe_ecosystem_tailwindcss` (core crate)

**Dependency changes:**
```toml
[dependencies]
tailwindcss-oxide = { path = "../../tailwindcss/crates/oxide" }
serde_json = "1"
# regex removed — no longer needed for scanner
```

**Module changes:**
- **DELETE** `src/scanner.rs` — replaced by `tailwindcss-oxide::Scanner`
- **REWRITE** `src/compiler.rs` — real `Compiler::build()` implementation
- **ADD** `src/ast.rs` — CSS AST types
- **ADD** `src/parser.rs` — CSS parser
- **ADD** `src/candidate.rs` — candidate parsing
- **ADD** `src/design_system.rs` — DesignSystem builder
- **ADD** `src/theme.rs` — `@theme` block processing
- **ADD** `src/utilities.rs` — built-in utility definitions
- **ADD** `src/variants.rs` — built-in variant definitions
- **ADD** `src/apply.rs` — `@apply` substitution
- **ADD** `src/functions.rs` — `theme()` / `--spacing()` / etc.
- **ADD** `src/walk.rs` — AST traversal
- **ADD** `src/selector_parser.rs` — selector parsing
- **ADD** `src/value_parser.rs` — CSS value parsing
- **UPDATE** `src/lib.rs` — updated exports

### 3.2 `farmfe_ecosystem_tailwindcss_node` (orchestration crate)

**Changes:** Minimal — only update `compile.rs` to wire into the new `Compiler::build()`.

### 3.3 `rust-plugins/tailwindcss` (Farm plugin)

**Changes:**
- Replace `extract_candidates()` call with `tailwindcss_oxide::Scanner::scan_content()`
- Add `tailwindcss-oxide` dependency (or re-export from core crate)
- Remove manual regex-based candidate scanning

## 4. Detailed Module Design

### 4.1 `ast.rs` — CSS AST

Mirrors `packages/tailwindcss/src/ast.ts`.

```rust
/// A single CSS AST node.
pub enum AstNode {
    Rule(StyleRule),
    AtRule(AtRule),
    Declaration(Declaration),
    Comment(String),
    Context(Context),  // Internal-only: not rendered
}

pub struct StyleRule {
    pub selector: String,
    pub nodes: Vec<AstNode>,
}

pub struct AtRule {
    pub name: String,
    pub prelude: String,
    pub nodes: Vec<AstNode>,
}

pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

/// Context nodes carry metadata through the AST.
pub enum Context {
    /// Marks a declaration as belonging to a @reference scope.
    Reference,
    /// Carries the current design system reference.
    DesignSystem(/* internal type */),
}

/// Serialise an AST to a CSS string.
pub fn to_css(nodes: &[AstNode]) -> String;

/// Apply optimizations: remove empty rules, deduplicate declarations.
pub fn optimize_ast(nodes: Vec<AstNode>) -> Vec<AstNode>;
```

Key behavior to preserve from upstream:
- `optimize_ast` removes empty `StyleRule`s but **preserves** empty `@charset`, `@layer`, `@custom-media`, `@namespace`, `@import`
- `optimize_ast` deduplicates exact-match declarations within the same rule
- `optimize_ast` does NOT emit `color-mix()` fallbacks inside `@keyframes`

### 4.2 `parser.rs` — CSS Parser

Mirrors `packages/tailwindcss/src/css-parser.ts`.

```rust
/// Parse a CSS string into an AST.
pub fn parse(css: &str) -> Vec<AstNode>;
```

Parsing strategy: a hand-written recursive descent parser (no parser generator dependency) since CSS syntax is simple and predictable. The upstream JS parser is `css-parser.ts` (~600 lines) — porting this directly produces manageable Rust.

### 4.3 `walk.rs` — AST Traversal

Mirrors `packages/tailwindcss/src/walk.ts`.

```rust
pub enum WalkAction<T> {
    /// Continue normal traversal.
    Continue,
    /// Skip children of this node.
    Skip,
    /// Stop traversal entirely.
    Stop,
    /// Replace this node with new node(s).
    Replace(Vec<T>),
    /// Replace and skip children of the new node(s).
    ReplaceSkip(Vec<T>),
}

pub fn walk<T, F>(nodes: Vec<T>, visitor: &mut F) -> Vec<T>
where
    F: FnMut(T, WalkPath) -> (WalkAction<T>, T);
```

### 4.4 `candidate.rs` — Candidate Parsing

Mirrors `packages/tailwindcss/src/candidate.ts`.

```rust
/// Parse a raw candidate string like "hover:bg-red-500/50" into
/// structured components.
pub struct ParsedCandidate {
    pub variant: Option<VariantComponent>,
    pub negated: bool,               // for "not-" variants
    pub utility: UtilityComponent,
    pub important: Option<ImportantSide>, // Prefix or Suffix
    pub modifier: Option<String>,    // /50 or /[50%]
}

pub struct VariantComponent {
    pub kind: VariantKind,           // Static, Arbitrary, Compound
    pub value: String,
}

pub struct UtilityComponent {
    pub kind: UtilityKind,           // Static, Functional, ArbitraryValue, ArbitraryProperty
    pub base: String,                // e.g., "bg-red-500" or "color"
    pub value: Option<String>,       // e.g., "red-500" or "var(--color)"
    pub type_hint: Option<String>,   // e.g., "color" in bg-[color:var(--x)]
}

pub enum ImportantSide { Prefix, Suffix }

pub fn parse_candidate(raw: &str) -> Option<ParsedCandidate>;
```

### 4.5 `design_system.rs` — Design System

Mirrors `packages/tailwindcss/src/design-system.ts`.

```rust
pub struct DesignSystem {
    pub theme: Theme,
    pub utilities: UtilityRegistry,
    pub variants: VariantRegistry,
}

impl DesignSystem {
    /// Build a design system from parsed CSS directives.
    pub fn build(nodes: &[AstNode], sources: Vec<Source>) -> Self;

    /// Compile candidates to AST nodes using this design system.
    pub fn compile_candidates(
        &self,
        candidates: &[String],
    ) -> Vec<AstNode>;
}
```

### 4.6 `theme.rs` — Theme Processing

Mirrors `packages/tailwindcss/src/theme.ts`.

```rust
pub struct Theme {
    /// CSS variables defined via @theme { ... }
    pub variables: HashMap<String, String>,
    /// Keyframes defined via @theme { @keyframes ... }
    pub keyframes: HashMap<String, Vec<AstNode>>,
    /// Whether the theme uses inline mode (--value style).
    pub is_inline: bool,
}

impl Theme {
    pub fn from_theme_block(nodes: &[AstNode]) -> Self;
    pub fn resolve(&self, key_path: &str) -> Option<String>;
}
```

### 4.7 `utilities.rs` — Built-in Utilities

Mirrors `packages/tailwindcss/src/utilities.ts`.

The built-in utility registry maps utility names (e.g., `flex`, `bg-*`, `text-*`) to CSS generation functions.

```rust
pub type UtilityFn = fn(&ParsedCandidate, &Theme) -> Vec<AstNode>;

pub struct UtilityRegistry;

impl UtilityRegistry {
    /// Return the built-in registry.
    pub fn builtin() -> Self;
    /// Look up a utility by name.
    pub fn get(&self, name: &str) -> Option<&UtilityFn>;
    /// Register a custom utility (for @utility blocks).
    pub fn register(&mut self, name: &str, func: UtilityFn);
}
```

The registry uses a trie-based lookup for matching patterns like `bg-{color}`.

### 4.8 `variants.rs` — Built-in Variants

Mirrors `packages/tailwindcss/src/variants.ts`.

```rust
pub struct VariantRegistry;

impl VariantRegistry {
    pub fn builtin(theme: &Theme) -> Self;
    pub fn resolve(&self, name: &str) -> Option<VariantFn>;
}
```

Built-in variants include: pseudo-classes (`hover`, `focus`, `active`, `disabled`, …), pseudo-elements (`before`, `after`, …), breakpoints (`sm`, `md`, `lg`, `xl`, `2xl`), container queries (`@lg`, `@min-lg`), attribute selectors (`aria-*`, `data-*`, `has-*`), media queries (`dark`, `motion-safe`, `print`, …), directional (`ltr`, `rtl`), and compound variants (`group-*`, `peer-*`).

### 4.9 `apply.rs` — `@apply` Substitution

Mirrors `packages/tailwindcss/src/apply.ts`.

```rust
pub fn substitute_at_apply(
    nodes: Vec<AstNode>,
    design_system: &DesignSystem,
) -> Result<Vec<AstNode>, ApplyError>;
```

Key behaviors:
- Collects `@utility` definitions from the AST
- Builds a dependency graph between `@apply` rules and utilities
- Topological sort to resolve dependencies
- Detects circular dependencies
- Replaces `@apply` nodes with compiled CSS rules

### 4.10 `functions.rs` — CSS Functions

Mirrors `packages/tailwindcss/src/css-functions.ts`.

```rust
pub fn substitute_css_functions(
    nodes: Vec<AstNode>,
    theme: &Theme,
) -> Vec<AstNode>;
```

Handles: `theme()`, `--theme()`, `--spacing()`, `--alpha()`.

### 4.11 `compiler.rs` — Top-Level API (rewrite)

```rust
pub struct Compiler {
    design_system: DesignSystem,
    pub features: Features,
    pub polyfills: Polyfills,
    dependencies: Vec<String>,
    source_maps_enabled: bool,
    config: Option<TailwindConfig>,
}

impl Compiler {
    /// Build final CSS from the given candidate list.
    /// This is the core method that was previously a passthrough.
    pub fn build(&mut self, candidates: &[String]) -> String {
        let nodes = self.design_system.compile_candidates(candidates);
        let optimized = optimize_ast(nodes);
        to_css(&optimized)
    }

    pub fn build_source_map(&self) -> Option<String>;
    pub fn dependencies(&self) -> &[String];
    pub fn config(&self) -> Option<&TailwindConfig>;
}

/// Parse CSS, extract @theme/@utility/@custom-variant, detect Features,
/// and return a Compiler ready for build().
///
/// The AST parse is fallible; returns `Err` on malformed CSS.
pub fn compile(css: &str, options: CompilerOptions) -> Result<Compiler, CompileError>;
```

Features are detected during `compile()` by walking the parsed AST:
- `@apply` → `Features::AT_APPLY`
- `@import "tailwindcss"` → `Features::UTILITIES`
- `theme()` → `Features::THEME_FUNCTION`

## 5. Test Migration Strategy (TDD)

### 5.1 Test Infrastructure

Farm uses `insta` for snapshot testing via `farmfe_testing_helpers`. The mapping:

| Upstream (Vitest) | Farm Rust |
|---|---|
| `expect(x).toMatchInlineSnapshot()` | `assert_snapshot!(output_string)` |
| `expect(x).toEqual(y)` | `assert_eq!(x, y)` |
| `test("name", () => {…})` | `#[test] fn name() {…}` |
| `it.each([…])("…", (arg) => {…})` | `#[test]` per case or manual loop |
| `test-utils/run.ts::run(candidates)` | Direct `compiler.build(&cands)` → optimize |

A local test helper (not exported) will provide:

```rust
#[cfg(test)]
fn run(candidates: &[&str]) -> String {
    let mut compiler = super::compile("@tailwind utilities;", CompilerOptions::default());
    let output = compiler.build(candidates);
    // Run through lightningcss optimize for comparison parity
    farmfe_ecosystem_tailwindcss_node::optimize::optimize(
        &output,
        OptimizeOptions::default(),
    ).unwrap().code
}
```

### 5.2 Test Migration Order (TDD)

Each phase writes tests FIRST (from upstream test cases), then implements the module.

#### Phase 0: Scaffold tests (infrastructure)

Delete `scanner.rs` tests. Add `tailwindcss-oxide` dependency. Verify compilation.

#### Phase 1: AST + Parser + Walk (P0)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `ast.test.ts` | `tests/ast.rs` | toCss, optimizeAst, walk actions, context propagation |
| `walk.test.ts` | `tests/walk.rs` | WalkAction variants, Replace, Stop, Skip |
| `css-parser.test.ts` | `tests/parser.rs` | Parse: rules, at-rules, declarations, comments, error cases |

**Implementation then:** `ast.rs`, `parser.rs`, `walk.rs`

#### Phase 2: Candidate Parsing (P0)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `candidate.test.ts` | `tests/candidate.rs` | static utils, functional utils, stacked variants, arbitrary variants/properties, importance modifiers, prefix handling, normalized candidate round-trips |

**Implementation then:** `candidate.rs`

#### Phase 3: Theme + Design System (P0)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `index.test.ts` (theme sections) | `tests/theme.rs` | @theme block parsing, CSS variable generation, inline theme |
| `css-functions.test.ts` | `tests/functions.rs` | `--alpha()`, `--spacing()`, `--theme()`, `theme()` |

**Implementation then:** `theme.rs`, `functions.rs`, design system scaffolding

#### Phase 4: Utilities (P1)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `utilities.test.ts` | `tests/utilities.rs` | Each utility family: display, position, flexbox, grid, spacing, sizing, typography, backgrounds, borders, effects, filters, transitions, transforms |

**Implementation then:** `utilities.rs` built-in registry

#### Phase 5: Variants (P1)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `variants.test.ts` | `tests/variants.rs` | All built-in variants, ordering, stacking, compound variants, negative cases |

**Implementation then:** `variants.rs`

#### Phase 6: Compiler Integration (P0)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `index.test.ts` (compiling CSS) | `tests/compiler.rs` | @tailwind utilities replacement, default theme snapshot, Features detection |
| `index.test.ts` (features) | `tests/features.rs` | Features::{AtApply, ThemeFunction} detection |

**Implementation then:** Full `compiler.rs` rewrite — `Compiler::build()`

#### Phase 7: @apply (P1)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `index.test.ts` (@apply sections) | `tests/apply.rs` | @apply substitution, recursive utilities, circular dependency detection, error messages |

**Implementation then:** `apply.rs`

#### Phase 8: @import (P2 — refinement)

**Tests written first (from upstream):**
| Upstream test file | → Rust test | Content |
|---|---|---|
| `at-import.test.ts` | `tests/at_import.rs` | @import resolution, @reference, cycle detection |

**Implementation then:** Update `tailwindcss-node/compile.rs` to integrate with the new compiler's import resolution.

#### Phase 9: Plugin Integration

Update `rust-plugins/tailwindcss` to use `tailwindcss-oxide::Scanner` instead of the old `scanner::extract_candidates`. Verify end-to-end flow.

### 5.3 Snapshot Handling

Upstream tests use `toMatchInlineSnapshot()`. In Rust we use `insta::assert_snapshot!`. On first run:
```bash
INSTA_UPDATE=always cargo test -p farmfe_ecosystem_tailwindcss
```
This generates `.snap` files. Subsequent runs verify against them.

The output format from Rust tests may differ slightly from JS due to:
- Property ordering (sort alphabetically to match upstream behavior)
- Whitespace formatting (normalize via `optimize` + `pretty`)
- Color function precision (upstream uses custom serializer; we accept slightly different float output for `oklab`/`oklch`)

## 6. Features Bitflags

Existing `Features` and `Polyfills` bitflag types are preserved with no changes to their bit assignments. All four flags (AT_APPLY, JS_PLUGIN_COMPAT, THEME_FUNCTION, UTILITIES) retain their current values.

Features are detected during `compile()` by walking the parsed AST — not by regex on raw CSS.

## 7. Error Handling

Core compilation errors use `Result<T, CompileError>`:

```rust
pub enum CompileError {
    ParseError { message: String, location: SourceLocation },
    CircularDependency { chain: Vec<String> },
    UnknownUtility { name: String },
    MissingVariant { variant: String, utility: String },
    InvalidCandidate { candidate: String, reason: String },
    ThemeError { message: String },
}
```

The Farm plugin shell converts errors to log messages (non-fatal for transform hooks).

## 8. Implementation Order

```
Phase 0: Scaffold (delete scanner.rs, add oxide dep)
Phase 1: ast.rs + parser.rs + walk.rs   [TDD: ast tests, parser tests]
Phase 2: candidate.rs                    [TDD: candidate tests]
Phase 3: theme.rs + functions.rs         [TDD: theme tests, css-functions tests]
Phase 4: utilities.rs                    [TDD: utilities tests]
Phase 5: variants.rs                     [TDD: variants tests]
Phase 6: compiler.rs (build)             [TDD: compiler integration tests]
Phase 7: apply.rs                        [TDD: apply tests]
Phase 8: Import refinement in node crate [TDD: at-import tests]
Phase 9: Plugin integration
```

Each phase is a vertical slice: write tests → implement → verify snapshots → move on.

## 9. Out of Scope (explicitly)

- JS plugin compat (`@plugin`, `@config "file.js"`)
- v3 configuration file compat (`compat/`)
- IntelliSense (`getClassList`, `getVariants`, `getClassOrder`)
- Class sorting (`sort.rs`)
- Source map generation (existing stub is kept)
- Config file loading (config is data-in, not file-in)
- `candidate.bench.ts` and all benchmark files
