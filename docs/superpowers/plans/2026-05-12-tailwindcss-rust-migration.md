# TailwindCSS Rust Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the JS tailwindcss compiler with a pure-Rust implementation in `farmfe_ecosystem_tailwindcss`, using upstream `tailwindcss-oxide` for scanning.

**Architecture:** Parse CSS into AST → extract `@theme`/`@utility` directives → build DesignSystem → parse candidates from oxide Scanner → compile candidates to CSS via DesignSystem → optimize via lightningcss. Each phase is TDD: write failing tests from upstream JS test cases first, then implement.

**Tech Stack:** Rust edition 2021, `insta` for snapshots, `serde_json` for config, `tailwindcss-oxide` for scanning, `lightningcss` for optimization.

---

## Implementation Status (updated 2026-05-15)

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0: Scaffold (delete scanner, add oxide) | ⏭ **Skipped** | `tailwindcss/crates/oxide` does not exist in repo; `scanner.rs` retained |
| Phase 1.1: ast.rs (AstNode, to_css, optimize_ast) | ✅ **Done** | 9 tests pass |
| Phase 1.2: walk.rs (WalkAction, walk) | ✅ **Done** | 4 tests pass |
| Phase 1.3: parser.rs (parse) | ✅ **Done** | 8 tests pass |
| Phase 2.1: candidate.rs (ParsedCandidate, parse_candidate) | ✅ **Done** | 11 tests pass |
| Phase 3.1: theme.rs (Theme, resolve, resolve_by_key_path) | ✅ **Done** | 3 tests pass |
| Phase 3.2: functions.rs (substitute_css_functions) | ✅ **Done** | 2 tests pass |
| Phase 4.1: utilities.rs (UtilityRegistry) | ✅ **Done** | 4 tests pass |
| Phase 5.1: variants.rs (VariantRegistry) | ✅ **Done** | 5 tests pass |
| Phase 6.1: design_system.rs + Compiler::build() rewrite | ✅ **Done** | 5 tests pass; compile() now returns Result |
| Phase 7.1: apply.rs (@apply substitution) | ✅ **Done** | 4 tests pass |
| Phase 8.1: tailwindcss-node compile() for new Result API | ✅ **Done** | Node crate compiles; .map_err() added |
| Phase 9: Plugin → oxide Scanner | ✅ **Done** | Rust plugin now uses `tailwindcss-oxide` Scanner from remote package source (git tag `v4.1.12`), extension-aware scanning, and JS-parity candidate file guard |
| Phase 10: Initial verification | ✅ **Done** | All tests pass; clippy clean |
| **Phase 11: Foundation utils (Plan B alignment)** | 🚧 **In progress** | 6 of N sub-modules complete |
| Phase 11.1: utils/segment.rs | ✅ **Done** | All 12 upstream `segment.test.ts` cases ported and pass |
| Phase 11.2: utils/escape.rs | ✅ **Done** | All 3 upstream `escape.test.ts` cases ported and pass |
| Phase 11.3: utils/to_key_path.rs | ✅ **Done** | Upstream `to-key-path.test.ts` ported (1 it-block, 8 assertions) |
| Phase 11.4: utils/brace_expansion.rs | ✅ **Done** | All upstream `brace-expansion.test.ts` cases ported (9 tests covering groups, ranges, padding, steps, nesting, errors) |
| Phase 11.5: utils/compare.rs | ✅ **Done** | Full upstream `compare.test.ts` ported (9 tests incl. Heap-permutation stability) |
| Phase 11.6: utils/compare_breakpoints.rs | ✅ **Done** | Doc-derived smoke tests (5 cases); upstream has no dedicated test file |
| Phase 11.7: utils/infer_data_type.rs | ⏳ **Pending** | Port `utils/infer-data-type.ts` (371 LoC); requires is-color + math-operators + value-parser first |
| Phase 11.8: utils/decode_arbitrary_value.rs | ⏳ **Pending** | Port `utils/decode-arbitrary-value.ts` (93 LoC); requires value-parser + math-operators |
| Phase 11.9: utils/math_operators.rs | ⏳ **Pending** | 205 LoC, prerequisite for 11.7/11.8 |
| Phase 11.10: utils/is_color.rs | ⏳ **Pending** | 204 LoC, prerequisite for 11.7 |
| Phase 11.11: utils/is_valid_arbitrary.rs | ⏳ **Pending** | 93 LoC |
| Phase 11.12: utils/replace_shadow_colors.rs | ⏳ **Pending** | 48 LoC + tests |
| Phase 11.13: utils/dimensions.rs | ⏳ **Pending** | 20 LoC |
| Phase 11.14: value_parser.rs | ⏳ **Pending** | Port `value-parser.ts` (279 LoC) + tests (219 LoC) |
| Phase 11.15: selector_parser.rs | ⏳ **Pending** | Port `selector-parser.ts` (421 LoC) + tests |
| Phase 11.16: attribute_selector_parser.rs | ⏳ **Pending** | Port `attribute-selector-parser.ts` (229 LoC) + tests |
| Phase 12: AST/parser parity | ⏳ **Pending** | Full `optimize_ast` rules; nested at-rules; escape handling |
| Phase 13: Candidate parser parity | ⏳ **Pending** | important / negative / modifier / arbitrary |
| Phase 14: Theme parity | ⏳ **Pending** | Namespaces / key-paths / inline / default theme |
| Phase 15: Variants parity | ⏳ **Pending** | Functional variants, media/container/supports, data/aria, peer/group |
| Phase 16: Utilities (~560 registrations) | ⏳ **Pending** | Port all of `utilities.ts` (6751 LoC) |
| Phase 17: @apply parity | ⏳ **Pending** | Recursive resolution, important propagation |
| Phase 18: CSS function parity | ⏳ **Pending** | `theme()`, `--spacing()`, etc. |
| Phase 19: property-order.rs | ⏳ **Pending** | Deterministic output ordering (440 LoC) |
| Phase 20: compat decision | ⏳ **Pending** | Mark `compat/` out of scope or open Phase 21+ |
| Phase 21: Final integration | ⏳ **Pending** | E2E snapshot tests against upstream fixtures |

### Summary
- **111 tests** pass (88 prior + 23 new across `to_key_path` / `brace_expansion` / `compare` / `compare_breakpoints`)
- Plan B continues. Phase 11 has 6 sub-modules complete; remaining sub-modules are larger and have dependency chains (e.g. `infer_data_type` needs `math_operators` + `is_color`).
- TDD discipline: tests are ported verbatim from upstream `*.test.ts` first, then implementation. Where upstream has no dedicated test file (`compare-breakpoints`), tests are derived from documented behavior in source comments.
- New runtime dep: `thiserror = "1"` for `BraceExpansionError`.

---

## File Structure

### Crate: `farmfe_ecosystem_tailwindcss` (`rust-ecosystem/tailwindcss/`)

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Crate root, public re-exports |
| `src/compiler.rs` | `Compiler`, `compile()`, `CompilerOptions`, `Features`, `Polyfills`, `TailwindConfig` |
| `src/ast.rs` | `AstNode` enum, `to_css()`, `optimize_ast()` |
| `src/parser.rs` | `parse(css: &str) -> Vec<AstNode>` |
| `src/walk.rs` | `walk()` with `WalkAction` enum |
| `src/candidate.rs` | `parse_candidate()`, `ParsedCandidate` types |
| `src/design_system.rs` | `DesignSystem` struct, `build()`, `compile_candidates()` |
| `src/theme.rs` | `Theme` struct, `from_theme_block()` |
| `src/utilities.rs` | Built-in utility definitions |
| `src/variants.rs` | Built-in variant definitions |
| `src/apply.rs` | `substitute_at_apply()` |
| `src/functions.rs` | `substitute_css_functions()` |
| `src/selector_parser.rs` | Selector parsing utilities |
| `src/value_parser.rs` | CSS value parsing utilities |

### Crate: `farmfe_ecosystem_tailwindcss_node` (`rust-ecosystem/tailwindcss-node/`)

| File | Responsibility |
|------|----------------|
| `src/compile.rs` | Updated to wire into new `Compiler::build()` |

### Crate: `rust-plugins/tailwindcss` (`rust-plugins/tailwindcss/`)

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | Updated to use `tailwindcss-oxide::Scanner` |

---

## Phase 0: Scaffold

### Task 0.1: Delete scanner.rs and update dependencies

**Files:**
- Delete: `rust-ecosystem/tailwindcss/src/scanner.rs`
- Modify: `rust-ecosystem/tailwindcss/src/lib.rs`
- Modify: `rust-ecosystem/tailwindcss/Cargo.toml`

- [ ] **Step 1: Delete scanner.rs**

```bash
rm rust-ecosystem/tailwindcss/src/scanner.rs
```

- [ ] **Step 2: Update Cargo.toml to add tailwindcss-oxide and remove regex**

Edit `rust-ecosystem/tailwindcss/Cargo.toml`:

```toml
[dependencies]
tailwindcss-oxide = { path = "../../tailwindcss/crates/oxide" }
serde_json = "1"

[dev-dependencies]
farmfe_testing_helpers = { path = "../../crates/testing_helpers" }
```

- [ ] **Step 3: Update lib.rs to remove scanner module**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod compiler;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
```

- [ ] **Step 4: Remove scanner-related tests**

Delete the scanner test file:

```bash
rm rust-ecosystem/tailwindcss/tests/scanner.rs
```

Remove scanner test references from `tests/compiler.rs` if any.

- [ ] **Step 5: Verify compilation**

```bash
cargo check -p farmfe_ecosystem_tailwindcss
```

Expected: Compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "chore: delete scanner.rs, add tailwindcss-oxide dependency"
```

---

## Phase 1: AST + Parser + Walk

### Task 1.1: Define AST types (`src/ast.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/ast.rs`
- Create: `rust-ecosystem/tailwindcss/tests/ast.rs`

- [ ] **Step 1: Write failing tests for AST types**

Create `rust-ecosystem/tailwindcss/tests/ast.rs`:

```rust
use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule, AtRule, to_css};

#[test]
fn test_style_rule_to_css() {
    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "color".to_string(),
            value: Some("red".to_string()),
            important: false,
        })],
    };
    let css = to_css(&[AstNode::Rule(rule)]);
    assert_eq!(css, ".foo {\n  color: red;\n}\n");
}

#[test]
fn test_at_rule_without_block_to_css() {
    let at_rule = AtRule {
        name: "@import".to_string(),
        params: "\"tailwindcss\"".to_string(),
        nodes: vec![],
    };
    let css = to_css(&[AstNode::AtRule(at_rule)]);
    assert_eq!(css, "@import \"tailwindcss\";\n");
}

#[test]
fn test_at_rule_with_block_to_css() {
    let at_rule = AtRule {
        name: "@media".to_string(),
        params: "screen".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "color".to_string(),
            value: Some("red".to_string()),
            important: false,
        })],
    };
    let css = to_css(&[AstNode::AtRule(at_rule)]);
    assert_eq!(css, "@media screen {\n  color: red;\n}\n");
}

#[test]
fn test_declaration_important_to_css() {
    let decl = Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: true,
    };
    let css = to_css(&[AstNode::Declaration(decl)]);
    assert_eq!(css, "color: red !important;\n");
}

#[test]
fn test_comment_to_css() {
    let css = to_css(&[AstNode::Comment(" license ".to_string())]);
    assert_eq!(css, "/* license */\n");
}

#[test]
fn test_nested_rules_to_css() {
    let inner = StyleRule {
        selector: "& .bar".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "color".to_string(),
            value: Some("blue".to_string()),
            important: false,
        })],
    };
    let outer = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Rule(inner)],
    };
    let css = to_css(&[AstNode::Rule(outer)]);
    assert_eq!(css, ".foo {\n  & .bar {\n    color: blue;\n  }\n}\n");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests ast
```

Expected: FAIL (module `ast` not found)

- [ ] **Step 3: Implement AST types**

Create `rust-ecosystem/tailwindcss/src/ast.rs`:

```rust
/// A single CSS AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Rule(StyleRule),
    AtRule(AtRule),
    Declaration(Declaration),
    Comment(String),
    Context(Context),
    AtRoot(AtRoot),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub selector: String,
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    pub name: String,
    pub params: String,
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: Option<String>,
    pub important: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub context: std::collections::HashMap<String, String>,
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtRoot {
    pub nodes: Vec<AstNode>,
}

/// Serialise AST nodes to a CSS string.
pub fn to_css(nodes: &[AstNode]) -> String {
    let mut css = String::new();
    for node in nodes {
        stringify(node, 0, &mut css);
    }
    css
}

fn stringify(node: &AstNode, depth: usize, css: &mut String) {
    let indent = "  ".repeat(depth);

    match node {
        AstNode::Declaration(decl) => {
            let important = if decl.important { " !important" } else { "" };
            css.push_str(&format!(
                "{}{}: {}{};\n",
                indent,
                decl.property,
                decl.value.as_deref().unwrap_or(""),
                important
            ));
        }
        AstNode::Rule(rule) => {
            css.push_str(&format!("{}{} {{\n", indent, rule.selector));
            for child in &rule.nodes {
                stringify(child, depth + 1, css);
            }
            css.push_str(&format!("{}}}\n", indent));
        }
        AstNode::AtRule(at_rule) => {
            if at_rule.nodes.is_empty() {
                css.push_str(&format!("{}{} {};\n", indent, at_rule.name, at_rule.params));
            } else {
                let sep = if at_rule.params.is_empty() { "" } else { " " };
                css.push_str(&format!(
                    "{}{}{}{}{{\n",
                    indent, at_rule.name, sep, at_rule.params, sep.trim_end()
                ));
                for child in &at_rule.nodes {
                    stringify(child, depth + 1, css);
                }
                css.push_str(&format!("{}}}\n", indent));
            }
        }
        AstNode::Comment(value) => {
            css.push_str(&format!("{}/*{}*/\n", indent, value));
        }
        AstNode::Context(ctx) => {
            for child in &ctx.nodes {
                stringify(child, depth, css);
            }
        }
        AstNode::AtRoot(at_root) => {
            for child in &at_root.nodes {
                stringify(child, depth, css);
            }
        }
    }
}
```

- [ ] **Step 4: Update lib.rs to export ast module**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod compiler;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests ast
```

Expected: All PASS

- [ ] **Step 6: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add CSS AST types with to_css serialization"
```

### Task 1.2: Implement Walk (`src/walk.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/walk.rs`
- Create: `rust-ecosystem/tailwindcss/tests/walk.rs`

- [ ] **Step 1: Write failing tests for walk**

Create `rust-ecosystem/tailwindcss/tests/walk.rs`:

```rust
use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration};
use farmfe_ecosystem_tailwindcss::walk::{walk, WalkAction};

#[test]
fn test_walk_visits_all_nodes() {
    let decl = AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
    });
    let mut visited = 0;
    walk(vec![decl.clone(), decl.clone()], &mut |node: &AstNode, _path, _depth| {
        visited += 1;
        WalkAction::Continue
    });
    assert_eq!(visited, 2);
}

#[test]
fn test_walk_skip_children() {
    use farmfe_ecosystem_tailwindcss::ast::StyleRule;

    let inner = AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
    });
    let rule = AstNode::Rule(StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![inner],
    });
    let mut visited_selectors = Vec::new();
    walk(vec![rule], &mut |node: &AstNode, _path, _depth| {
        if let AstNode::Rule(r) = node {
            visited_selectors.push(r.selector.clone());
            return WalkAction::Skip;
        }
        visited_selectors.push("other".to_string());
        WalkAction::Continue
    });
    // Only the rule was visited, not its children
    assert_eq!(visited_selectors, vec![".foo"]);
}

#[test]
fn test_walk_replace_node() {
    use farmfe_ecosystem_tailwindcss::ast::StyleRule;

    let decl = AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
    });
    let replacement = AstNode::Declaration(Declaration {
        property: "color".to_string(),
        value: Some("blue".to_string()),
        important: false,
    });
    let rule = AstNode::Rule(StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![decl],
    });
    let result = walk(rule.nodes, &mut |node: &AstNode, _path, _depth| {
        if let AstNode::Declaration(d) = node {
            if d.value.as_deref() == Some("red") {
                return WalkAction::Replace(vec![replacement.clone()]);
            }
        }
        WalkAction::Continue
    });
    assert_eq!(result.len(), 1);
    if let AstNode::Declaration(d) = &result[0] {
        assert_eq!(d.value.as_deref(), Some("blue"));
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn test_walk_stop() {
    let d1 = AstNode::Declaration(Declaration {
        property: "a".to_string(),
        value: Some("1".to_string()),
        important: false,
    });
    let d2 = AstNode::Declaration(Declaration {
        property: "b".to_string(),
        value: Some("2".to_string()),
        important: false,
    });
    let mut visited = Vec::new();
    walk(vec![d1, d2], &mut |node: &AstNode, _path, _depth| {
        if let AstNode::Declaration(d) = node {
            visited.push(d.property.clone());
            if d.property == "a" {
                return WalkAction::Stop;
            }
        }
        WalkAction::Continue
    });
    assert_eq!(visited, vec!["a"]);
}
```

- [ ] **Step 2: Implement walk**

Create `rust-ecosystem/tailwindcss/src/walk.rs`:

```rust
use crate::ast::AstNode;

/// Action returned from a walk visitor.
pub enum WalkAction {
    /// Continue normal traversal.
    Continue,
    /// Skip children of this node.
    Skip,
    /// Stop traversal entirely.
    Stop,
    /// Replace this node with new node(s).
    Replace(Vec<AstNode>),
    /// Replace and skip children of the replacement(s).
    ReplaceSkip(Vec<AstNode>),
}

/// Depth-first walk of an AST, returning the potentially modified nodes.
/// The visitor receives `(&AstNode, path: &[&AstNode], depth: usize)` and
/// returns a `WalkAction`.
pub fn walk<F>(nodes: Vec<AstNode>, visitor: &mut F) -> Vec<AstNode>
where
    F: FnMut(&AstNode, &[&AstNode], usize) -> WalkAction,
{
    let mut result = Vec::new();
    let path: Vec<&AstNode> = Vec::new();

    for node in nodes {
        walk_node(node, &path, 0, visitor, &mut result);
    }

    result
}

fn walk_node<F>(
    node: AstNode,
    path: &[&AstNode],
    depth: usize,
    visitor: &mut F,
    result: &mut Vec<AstNode>,
) where
    F: FnMut(&AstNode, &[&AstNode], usize) -> WalkAction,
{
    let action = visitor(&node, path, depth);

    match action {
        WalkAction::Continue => {
            match &node {
                AstNode::Rule(rule) if !rule.nodes.is_empty() => {
                    let mut new_path: Vec<&AstNode> = path.to_vec();
                    new_path.push(&node);
                    let children = std::mem::take(
                        &mut *unsafe { &mut *(&rule.nodes as *const _ as *mut Vec<AstNode>) },
                    );
                    let new_children = walk_inner(children, &new_path, depth + 1, visitor);
                    // Create a new rule with walked children
                    let new_node = AstNode::Rule(super::ast::StyleRule {
                        selector: rule.selector.clone(),
                        nodes: new_children,
                    });
                    result.push(new_node);
                }
                AstNode::AtRule(at_rule) if !at_rule.nodes.is_empty() => {
                    let mut new_path: Vec<&AstNode> = path.to_vec();
                    new_path.push(&node);
                    let children = std::mem::take(
                        &mut *unsafe { &mut *(&at_rule.nodes as *const _ as *mut Vec<AstNode>) },
                    );
                    let new_children = walk_inner(children, &new_path, depth + 1, visitor);
                    let new_node = AstNode::AtRule(super::ast::AtRule {
                        name: at_rule.name.clone(),
                        params: at_rule.params.clone(),
                        nodes: new_children,
                    });
                    result.push(new_node);
                }
                AstNode::Context(ctx) if !ctx.nodes.is_empty() => {
                    let mut new_path: Vec<&AstNode> = path.to_vec();
                    new_path.push(&node);
                    let children = std::mem::take(
                        &mut *unsafe { &mut *(&ctx.nodes as *const _ as *mut Vec<AstNode>) },
                    );
                    let new_children = walk_inner(children, &new_path, depth + 1, visitor);
                    let new_node = AstNode::Context(super::ast::Context {
                        context: ctx.context.clone(),
                        nodes: new_children,
                    });
                    result.push(new_node);
                }
                AstNode::AtRoot(at_root) if !at_root.nodes.is_empty() => {
                    let children = std::mem::take(
                        &mut *unsafe {
                            &mut *(&at_root.nodes as *const _ as *mut Vec<AstNode>)
                        },
                    );
                    let new_children = walk_inner(children, path, depth, visitor);
                    let new_node = AstNode::AtRoot(super::ast::AtRoot {
                        nodes: new_children,
                    });
                    result.push(new_node);
                }
                _ => {
                    result.push(node);
                }
            }
        }
        WalkAction::Skip => {
            result.push(node);
        }
        WalkAction::Stop => {
            result.push(node);
        }
        WalkAction::Replace(nodes) => {
            let mut new_path: Vec<&AstNode> = path.to_vec();
            new_path.push(&node);
            let walked = walk_inner(nodes, &new_path, depth, visitor);
            result.extend(walked);
        }
        WalkAction::ReplaceSkip(nodes) => {
            result.extend(nodes);
        }
    }
}

fn walk_inner<F>(
    nodes: Vec<AstNode>,
    path: &[&AstNode],
    depth: usize,
    visitor: &mut F,
) -> Vec<AstNode>
where
    F: FnMut(&AstNode, &[&AstNode], usize) -> WalkAction,
{
    let mut result = Vec::new();
    for node in nodes {
        walk_node(node, path, depth, visitor, &mut result);
    }
    result
}
```

Note: The above uses `unsafe` to work around borrow-checker issues when modifying children. A cleaner approach will be to use indices or `RefCell` during the actual implementation. This is intentionally kept as the initial simplest-possible approach per TDD.

- [ ] **Step 3: Update lib.rs to export walk module**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod compiler;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests walk
```

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add AST walker with WalkAction support"
```

### Task 1.3: Implement CSS Parser (`src/parser.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/parser.rs`
- Create: `rust-ecosystem/tailwindcss/tests/parser.rs`

- [ ] **Step 1: Write failing tests for CSS parser**

Create `rust-ecosystem/tailwindcss/tests/parser.rs`:

```rust
use farmfe_ecosystem_tailwindcss::ast::AstNode;
use farmfe_ecosystem_tailwindcss::parser::parse;

#[test]
fn test_parse_simple_rule() {
    let css = ".foo { color: red; }";
    let ast = parse(css);
    assert!(!ast.is_empty());
}

#[test]
fn test_parse_at_rule_semicolon() {
    let css = r#"@import "tailwindcss";"#;
    let ast = parse(css);
    assert!(!ast.is_empty());
}

#[test]
fn test_parse_at_rule_block() {
    let css = "@media screen { .foo { color: red; } }";
    let ast = parse(css);
    assert!(!ast.is_empty());
}

#[test]
fn test_parse_declaration_with_important() {
    let css = ".foo { color: red !important; }";
    let ast = parse(css);
    assert!(!ast.is_empty());
}

#[test]
fn test_parse_comment() {
    let css = "/* license */\n.foo { color: red; }";
    let ast = parse(css);
    assert!(!ast.is_empty());
}

#[test]
fn test_parse_empty_input() {
    let css = "";
    let ast = parse(css);
    assert!(ast.is_empty());
}

#[test]
fn test_parse_multiple_rules() {
    let css = ".foo { color: red; }\n.bar { color: blue; }";
    let ast = parse(css);
    assert_eq!(ast.len(), 2);
}

#[test]
fn test_parse_nested_at_rule() {
    let css = "@layer base {\n  .foo { color: red; }\n}";
    let ast = parse(css);
    assert!(!ast.is_empty());
}
```

- [ ] **Step 2: Implement CSS parser**

Create `rust-ecosystem/tailwindcss/src/parser.rs`:

```rust
use crate::ast::{AstNode, AtRule, Declaration, StyleRule};

const BACKSLASH: u8 = b'\\';
const SLASH: u8 = b'/';
const ASTERISK: u8 = b'*';
const DOUBLE_QUOTE: u8 = b'"';
const SINGLE_QUOTE: u8 = b'\'';
const COLON: u8 = b':';
const SEMICOLON: u8 = b';';
const LINE_BREAK: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const OPEN_CURLY: u8 = b'{';
const CLOSE_CURLY: u8 = b'}';
const OPEN_PAREN: u8 = b'(';
const CLOSE_PAREN: u8 = b')';
const AT_SIGN: u8 = b'@';
const EXCLAMATION: u8 = b'!';

/// Parse a CSS string into an AST.
pub fn parse(input: &str) -> Vec<AstNode> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut ast: Vec<AstNode> = Vec::new();
    let mut license_comments: Vec<AstNode> = Vec::new();
    let mut stack: Vec<Option<Box<dyn AnyParent>>> = Vec::new();

    // Type-erased parent handle (StyleRule or AtRule)
    // Simplified: we use enum instead.
    #[derive(Debug, Clone)]
    enum Parent {
        Rule(StyleRule),
        AtRule(AtRule),
    }

    impl Parent {
        fn push_node(&mut self, node: AstNode) {
            match self {
                Parent::Rule(r) => r.nodes.push(node),
                Parent::AtRule(a) => a.nodes.push(node),
            }
        }
        fn into_ast_node(self) -> AstNode {
            match self {
                Parent::Rule(r) => AstNode::Rule(r),
                Parent::AtRule(a) => AstNode::AtRule(a),
            }
        }
    }

    let mut parent: Option<Parent> = None;
    let mut buffer = String::new();
    let mut closing_bracket_stack = String::new();
    let mut buffer_start: usize = 0;
    let mut i = 0;

    while i < len {
        let current = bytes[i];

        // Skip CR in CRLF
        if current == CARRIAGE_RETURN {
            let peek = if i + 1 < len { bytes[i + 1] } else { 0 };
            if peek == LINE_BREAK {
                i += 1;
                continue;
            }
        }

        // Backslash escape
        if current == BACKSLASH {
            if buffer.is_empty() {
                buffer_start = i;
            }
            if i + 1 < len {
                buffer.push(input.as_bytes()[i] as char);
                buffer.push(input.as_bytes()[i + 1] as char);
                i += 2;
            } else {
                buffer.push('\\');
                i += 1;
            }
            continue;
        }

        // Start of comment
        if current == SLASH && i + 1 < len && bytes[i + 1] == ASTERISK {
            let start = i;
            for j in (i + 2)..len {
                let peek = bytes[j];
                if peek == BACKSLASH {
                    continue;
                }
                if peek == ASTERISK && j + 1 < len && bytes[j + 1] == SLASH {
                    i = j + 1;
                    break;
                }
            }
            let comment_str = &input[start..=i];
            // License comments (/*! ... */) hoisted to top
            if comment_str.as_bytes().len() > 2 && comment_str.as_bytes()[2] == EXCLAMATION {
                let inner = &comment_str[3..comment_str.len() - 2];
                license_comments.push(AstNode::Comment(inner.to_string()));
            }
            continue;
        }

        // Start of string
        if current == SINGLE_QUOTE || current == DOUBLE_QUOTE {
            let end = parse_string(input, i, current);
            buffer.push_str(&input[i..=end]);
            i = end;
            continue;
        }

        // Whitespace collapsing
        if (current == SPACE || current == LINE_BREAK || current == TAB)
            && i + 1 < len
        {
            let peek = bytes[i + 1];
            if peek == SPACE
                || peek == LINE_BREAK
                || peek == TAB
                || (peek == CARRIAGE_RETURN
                    && i + 2 < len
                    && bytes[i + 2] == LINE_BREAK)
            {
                i += 1;
                continue;
            }
        }

        // Newline -> space
        if current == LINE_BREAK {
            if buffer.is_empty() {
                i += 1;
                continue;
            }
            let last = buffer.as_bytes()[buffer.len() - 1];
            if last != SPACE && last != LINE_BREAK && last != TAB {
                buffer.push(' ');
            }
            i += 1;
            continue;
        }

        // End of body-less at-rule
        if current == SEMICOLON && !buffer.is_empty() && buffer.as_bytes()[0] == AT_SIGN {
            let at_rule = parse_at_rule(&buffer);
            if let Some(ref mut p) = parent {
                p.push_node(at_rule);
            } else {
                ast.push(at_rule);
            }
            buffer.clear();
            i += 1;
            continue;
        }

        // End of declaration
        if current == SEMICOLON
            && closing_bracket_stack
                .chars()
                .last()
                .map_or(true, |c| c != ')')
        {
            if !buffer.is_empty() {
                let decl = parse_declaration(&buffer);
                match decl {
                    Some(d) => {
                        if let Some(ref mut p) = parent {
                            p.push_node(AstNode::Declaration(d));
                        } else {
                            ast.push(AstNode::Declaration(d));
                        }
                    }
                    None => {
                        if !buffer.trim().is_empty() {
                            // Invalid declaration, skip for now
                        }
                    }
                }
                buffer.clear();
            }
            i += 1;
            continue;
        }

        // Start of block
        if current == OPEN_CURLY
            && closing_bracket_stack
                .chars()
                .last()
                .map_or(true, |c| c != ')')
        {
            closing_bracket_stack.push('}');
            let trimmed = buffer.trim().to_string();
            buffer.clear();

            let new_parent = if !trimmed.is_empty() && trimmed.as_bytes()[0] == AT_SIGN {
                let at_rule = parse_at_rule_raw(&trimmed);
                Parent::AtRule(at_rule)
            } else {
                Parent::Rule(StyleRule {
                    selector: trimmed,
                    nodes: vec![],
                })
            };

            if let Some(p) = parent.take() {
                let node = p.into_ast_node();
                stack.push(Some(node));
            } else {
                stack.push(None);
            }
            parent = Some(new_parent);
            i += 1;
            continue;
        }

        // End of block
        if current == CLOSE_CURLY
            && closing_bracket_stack
                .chars()
                .last()
                .map_or(true, |c| c != ')')
        {
            if closing_bracket_stack.is_empty() {
                // Error: missing opening {
                i += 1;
                continue;
            }
            closing_bracket_stack.pop();

            // Handle buffer with trailing declaration without semicolon
            if !buffer.is_empty() {
                if !buffer.is_empty() && buffer.as_bytes()[0] == AT_SIGN {
                    let at_rule = parse_at_rule(&buffer);
                    if let Some(ref mut p) = parent {
                        p.push_node(at_rule);
                    }
                } else {
                    let decl = parse_declaration(&buffer);
                    if let Some(d) = decl {
                        if let Some(ref mut p) = parent {
                            p.push_node(AstNode::Declaration(d));
                        }
                    }
                }
                buffer.clear();
            }

            // Pop stack
            let finished = parent.take().unwrap();
            let finished_node = finished.into_ast_node();

            if let Some(saved) = stack.pop() {
                match saved {
                    Some(existing) => {
                        // There was a previously saved parent — this shouldn't happen
                        // in our simplified model since we only stack None.
                        ast.push(existing);
                    }
                    None => {}
                }
                parent = None; // We're back at root
                ast.push(finished_node);
            }
            i += 1;
            continue;
        }

        // ( and )
        if current == OPEN_PAREN {
            closing_bracket_stack.push(')');
            buffer.push('(');
            i += 1;
            continue;
        }
        if current == CLOSE_PAREN {
            if closing_bracket_stack.ends_with(')') {
                closing_bracket_stack.pop();
            }
            buffer.push(')');
            i += 1;
            continue;
        }

        // Any other character
        if buffer.is_empty()
            && (current == SPACE || current == LINE_BREAK || current == TAB)
        {
            i += 1;
            continue;
        }

        if buffer.is_empty() {
            buffer_start = i;
        }
        buffer.push(input[i..].chars().next().unwrap());
        i += 1;
    }

    // Handle leftover buffer at end (e.g. @tailwind utilities without semicolon)
    if !buffer.is_empty() && buffer.as_bytes()[0] == AT_SIGN {
        let at_rule = parse_at_rule(&buffer);
        ast.push(at_rule);
    }

    // Prepend license comments
    if !license_comments.is_empty() {
        let mut result = license_comments;
        result.extend(ast);
        return result;
    }

    ast
}

fn parse_at_rule(buffer: &str) -> AstNode {
    AstNode::AtRule(parse_at_rule_raw(buffer))
}

fn parse_at_rule_raw(buffer: &str) -> AtRule {
    let trimmed = buffer.trim();
    let mut name = trimmed.to_string();
    let mut params = String::new();

    // Find the first space or ( after the at-rule name
    // Skip the first character (@) and look from there
    // Minimum at-rule length: @x (2 chars), but we start scanning from index 5 to optimize
    let scan_start = if trimmed.len() > 5 { 5 } else { 2 };
    for (i, ch) in trimmed.char_indices().skip(scan_start) {
        if ch == ' ' || ch == '\t' || ch == '(' {
            name = trimmed[..i].to_string();
            params = trimmed[i..].trim().to_string();
            break;
        }
    }

    AtRule {
        name: name.trim().to_string(),
        params: params.trim().to_string(),
        nodes: vec![],
    }
}

fn parse_declaration(buffer: &str) -> Option<Declaration> {
    let colon_idx = buffer.find(':')?;
    let property = buffer[..colon_idx].trim().to_string();
    if property.is_empty() {
        return None;
    }
    let rest = &buffer[colon_idx + 1..];
    let important_idx = rest.find("!important");
    let (value, important) = match important_idx {
        Some(idx) => (rest[..idx].trim().to_string(), true),
        None => (rest.trim().to_string(), false),
    };
    Some(Declaration {
        property,
        value: if value.is_empty() { None } else { Some(value) },
        important,
    })
}

fn parse_string(input: &str, start: usize, quote_char: u8) -> usize {
    let bytes = input.as_bytes();
    for i in (start + 1)..bytes.len() {
        let ch = bytes[i];
        if ch == BACKSLASH {
            continue;
        }
        if ch == quote_char {
            return i;
        }
    }
    start // Unterminated string, return start (error handled upstream)
}
```

- [ ] **Step 3: Update lib.rs to export parser module**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod compiler;
pub mod parser;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests parser
```

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add CSS parser (recursive descent)"
```

### Task 1.4: Implement optimize_ast

**Files:**
- Modify: `rust-ecosystem/tailwindcss/src/ast.rs`
- Modify: `rust-ecosystem/tailwindcss/tests/ast.rs`

- [ ] **Step 1: Write failing tests for optimize_ast**

Add to `rust-ecosystem/tailwindcss/tests/ast.rs`:

```rust
use farmfe_ecosystem_tailwindcss::ast::optimize_ast;

#[test]
fn test_optimize_ast_removes_empty_rules() {
    let rule = StyleRule {
        selector: ".empty".to_string(),
        nodes: vec![],
    };
    let result = optimize_ast(vec![AstNode::Rule(rule)]);
    assert!(result.is_empty());
}

#[test]
fn test_optimize_ast_preserves_empty_at_rules() {
    // @layer, @charset, @custom-media, @namespace, @import are preserved
    let at_rule = AtRule {
        name: "@layer".to_string(),
        params: "base".to_string(),
        nodes: vec![],
    };
    let result = optimize_ast(vec![AstNode::AtRule(at_rule)]);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_optimize_ast_deduplicates_declarations() {
    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![
            AstNode::Declaration(Declaration {
                property: "color".to_string(),
                value: Some("red".to_string()),
                important: false,
            }),
            AstNode::Declaration(Declaration {
                property: "color".to_string(),
                value: Some("red".to_string()),
                important: false,
            }),
            AstNode::Declaration(Declaration {
                property: "margin".to_string(),
                value: Some("0".to_string()),
                important: false,
            }),
        ],
    };
    let result = optimize_ast(vec![AstNode::Rule(rule)]);
    // Should have 2 declarations (deduplicated + margin kept)
    if let AstNode::Rule(r) = &result[0] {
        assert_eq!(r.nodes.len(), 2);
    } else {
        panic!("expected rule");
    }
}
```

- [ ] **Step 2: Implement optimize_ast**

Add to `rust-ecosystem/tailwindcss/src/ast.rs`:

```rust
use std::collections::HashSet;

/// Optimize the AST: remove empty rules, deduplicate declarations.
/// Preserves empty at-rules that are semantically meaningful.
pub fn optimize_ast(nodes: Vec<AstNode>) -> Vec<AstNode> {
    let preservable_at_rules: HashSet<&str> = [
        "@layer", "@charset", "@custom-media", "@namespace", "@import",
    ]
    .iter()
    .cloned()
    .collect();

    nodes
        .into_iter()
        .filter_map(|node| match node {
            AstNode::Rule(mut rule) => {
                if rule.nodes.is_empty() {
                    return None;
                }
                // Sort and deduplicate declarations
                let mut seen: HashSet<String> = HashSet::new();
                let mut deduped: Vec<AstNode> = Vec::new();
                for child in rule.nodes.into_iter().rev() {
                    if let AstNode::Declaration(ref d) = child {
                        let key = format!(
                            "{}:{}:{}",
                            d.property,
                            d.value.as_deref().unwrap_or(""),
                            d.important
                        );
                        if seen.insert(key) {
                            deduped.push(child);
                        }
                    } else {
                        deduped.push(child);
                    }
                }
                deduped.reverse();
                if deduped.is_empty() {
                    return None;
                }
                // Recursively optimize children
                rule.nodes = optimize_ast(deduped);
                if rule.nodes.is_empty() {
                    None
                } else {
                    Some(AstNode::Rule(rule))
                }
            }
            AstNode::AtRule(mut at_rule) => {
                // Preserve empty at-rules that are semantically meaningful
                if at_rule.nodes.is_empty() {
                    if preservable_at_rules.contains(at_rule.name.as_str()) {
                        return Some(AstNode::AtRule(at_rule));
                    }
                    return None;
                }
                at_rule.nodes = optimize_ast(at_rule.nodes);
                if at_rule.nodes.is_empty()
                    && !preservable_at_rules.contains(at_rule.name.as_str())
                {
                    return None;
                }
                Some(AstNode::AtRule(at_rule))
            }
            AstNode::Context(ctx) => {
                let nodes = optimize_ast(ctx.nodes);
                if nodes.is_empty() {
                    None
                } else {
                    Some(AstNode::Context(Context {
                        context: ctx.context,
                        nodes,
                    }))
                }
            }
            AstNode::AtRoot(at_root) => {
                let nodes = optimize_ast(at_root.nodes);
                if nodes.is_empty() {
                    None
                } else {
                    Some(AstNode::AtRoot(AtRoot { nodes }))
                }
            }
            other => Some(other),
        })
        .collect()
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests ast
```

Expected: All PASS

- [ ] **Step 4: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add optimize_ast for dedup and empty rule removal"
```

---

## Phase 2: Candidate Parsing

### Task 2.1: Implement candidate types and parsing (`src/candidate.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/candidate.rs`
- Create: `rust-ecosystem/tailwindcss/tests/candidate.rs`

- [ ] **Step 1: Write failing tests for candidate parsing**

Create `rust-ecosystem/tailwindcss/tests/candidate.rs`:

```rust
use farmfe_ecosystem_tailwindcss::candidate::parse_candidate;

#[test]
fn test_parse_static_utility() {
    let result = parse_candidate("flex");
    assert!(result.is_some());
    let c = result.unwrap();
    assert_eq!(c.utility_root, "flex");
    assert!(c.variants.is_empty());
    assert!(!c.important);
}

#[test]
fn test_parse_functional_utility() {
    let result = parse_candidate("bg-red-500");
    assert!(result.is_some());
}

#[test]
fn test_parse_utility_with_variant() {
    let result = parse_candidate("hover:bg-red-500");
    assert!(result.is_some());
    let c = result.unwrap();
    assert!(!c.variants.is_empty());
}

#[test]
fn test_parse_utility_with_important_prefix() {
    let result = parse_candidate("!flex");
    assert!(result.is_some());
    let c = result.unwrap();
    assert!(c.important);
}

#[test]
fn test_parse_utility_with_important_suffix() {
    let result = parse_candidate("flex!");
    assert!(result.is_some());
    let c = result.unwrap();
    assert!(c.important);
}

#[test]
fn test_parse_stacked_variants() {
    let result = parse_candidate("focus:hover:flex");
    assert!(result.is_some());
    let c = result.unwrap();
    assert_eq!(c.variants.len(), 2);
}

#[test]
fn test_parse_arbitrary_value() {
    let result = parse_candidate("bg-[#0088cc]");
    assert!(result.is_some());
}

#[test]
fn test_parse_with_modifier() {
    let result = parse_candidate("bg-red-500/50");
    assert!(result.is_some());
}

#[test]
fn test_parse_arbitrary_property() {
    let result = parse_candidate("[color:red]");
    assert!(result.is_some());
}

#[test]
fn test_parse_invalid_candidate_returns_none() {
    assert!(parse_candidate("").is_none());
    assert!(parse_candidate("not:a:valid:candidate:at:all").is_none());
}

#[test]
fn test_parse_arbitrary_variant() {
    let result = parse_candidate("[&_p]:flex");
    assert!(result.is_some());
}
```

- [ ] **Step 2: Implement candidate parsing**

Create `rust-ecosystem/tailwindcss/src/candidate.rs`:

```rust
/// Parsed candidate from a raw string like "hover:bg-red-500/50".
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCandidate {
    /// The utility root, e.g. "flex", "bg", "text"
    pub utility_root: String,
    /// The utility value, e.g. "red-500" for "bg-red-500"
    pub utility_value: Option<String>,
    /// Whether this is an arbitrary property like "[color:red]"
    pub arbitrary_property: Option<(String, String)>, // (property, value)
    /// Whether this is an arbitrary value like "bg-[#0088cc]"
    pub arbitrary_value: Option<String>,
    /// Optional type hint for arbitrary values, e.g. "color" in "bg-[color:var(--x)]"
    pub type_hint: Option<String>,
    /// Variant stack, e.g. ["hover", "focus"] for "focus:hover:bg-red-500"
    pub variants: Vec<String>,
    /// Modifier, e.g. "50" for "bg-red-500/50"
    pub modifier: Option<String>,
    /// Whether the modifier is arbitrary (e.g., /[50%])
    pub modifier_is_arbitrary: bool,
    /// Whether the utility is important (!)
    pub important: bool,
    /// Whether this is a static utility (exact match lookup)
    pub is_static: bool,
    /// The raw input string
    pub raw: String,
}

/// Parse a raw candidate string into structured components.
/// Returns None if the candidate is invalid.
pub fn parse_candidate(input: &str) -> Option<ParsedCandidate> {
    if input.is_empty() {
        return None;
    }

    let raw = input.to_string();

    // Split by ':' to separate variants from base
    let segments: Vec<&str> = raw.split(':').collect();

    if segments.is_empty() {
        return None;
    }

    // The last segment is the base utility
    let base = segments.last().unwrap();
    let variants: Vec<String> = segments[..segments.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Variant count sanity check
    if variants.len() > 10 {
        return None;
    }

    // Check for important
    let (base, important) = if base.ends_with('!') {
        (&base[..base.len() - 1], true)
    } else if base.starts_with('!') {
        (&base[1..], true)
    } else {
        (base, false)
    };

    // Split base into utility + modifier by '/'
    let slash_parts: Vec<&str> = base.split('/').collect();
    if slash_parts.len() > 2 {
        return None; // Multiple slashes is invalid
    }
    let base_no_modifier = slash_parts[0];
    let modifier = if slash_parts.len() == 2 {
        Some(slash_parts[1].to_string())
    } else {
        None
    };

    let modifier_is_arbitrary = modifier
        .as_ref()
        .map_or(false, |m| m.starts_with('[') && m.ends_with(']'));

    // Check for arbitrary property: [property:value]
    if base_no_modifier.starts_with('[') && base_no_modifier.ends_with(']') {
        let inner = &base_no_modifier[1..base_no_modifier.len() - 1];
        let colon_idx = inner.find(':')?;
        if colon_idx == 0 || colon_idx == inner.len() - 1 {
            return None;
        }
        let property = inner[..colon_idx].to_string();
        let value = inner[colon_idx + 1..].to_string();

        // Property must start with a-z or -
        let first = property.as_bytes()[0];
        if !first.is_ascii_lowercase() && first != b'-' {
            return None;
        }

        return Some(ParsedCandidate {
            utility_root: String::new(),
            utility_value: None,
            arbitrary_property: Some((property, value)),
            arbitrary_value: None,
            type_hint: None,
            variants,
            modifier,
            modifier_is_arbitrary,
            important,
            is_static: false,
            raw,
        });
    }

    // Check for arbitrary value: bg-[#0088cc] or bg-[color:var(--x)]
    if base_no_modifier.ends_with(']') {
        let bracket_start = base_no_modifier.find("-{")?;
        let root = base_no_modifier[..bracket_start].to_string();
        let raw_value = &base_no_modifier[bracket_start + 2..base_no_modifier.len() - 1];

        // Split type hint if present
        let colon_idx = raw_value.find(':');
        let (type_hint, value) = match colon_idx {
            Some(idx) if idx > 0
                && raw_value[..idx]
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b == b'-') =>
            {
                (Some(raw_value[..idx].to_string()), raw_value[idx + 1..].to_string())
            }
            _ => (None, raw_value.to_string()),
        };

        if value.is_empty() || value.trim().is_empty() {
            return None;
        }

        return Some(ParsedCandidate {
            utility_root: root,
            utility_value: None,
            arbitrary_property: None,
            arbitrary_value: Some(value),
            type_hint,
            variants,
            modifier,
            modifier_is_arbitrary,
            important,
            is_static: false,
            raw,
        });
    }

    // Named/static utility: try to find root by splitting at dashes
    // e.g., "bg-red-500" -> root="bg", value="red-500"
    // e.g., "flex" -> root="flex", value=None

    // Check for static utility first (no dash after root)
    if base_no_modifier.chars().all(|c| c.is_alphanumeric() || c == '-') {
        // Try splitting from the right
        let root = find_root(base_no_modifier);
        let value = if root == base_no_modifier {
            None
        } else {
            Some(base_no_modifier[root.len() + 1..].to_string())
        };

        Some(ParsedCandidate {
            utility_root: root,
            utility_value: value,
            arbitrary_property: None,
            arbitrary_value: None,
            type_hint: None,
            variants,
            modifier,
            modifier_is_arbitrary,
            important,
            is_static: root == base_no_modifier,
            raw,
        })
    } else {
        None
    }
}

/// Find the utility root by splitting at the first dash.
/// "bg-red-500" -> "bg"
/// "flex" -> "flex"
fn find_root(input: &str) -> String {
    if let Some(idx) = input.find('-') {
        input[..idx].to_string()
    } else {
        input.to_string()
    }
}
```

- [ ] **Step 3: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod parser;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
pub use candidate::parse_candidate;
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests candidate
```

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add candidate parser for tailwind utility classes"
```

---

## Phase 3: Theme + CSS Functions

### Task 3.1: Implement Theme (`src/theme.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/theme.rs`
- Create: `rust-ecosystem/tailwindcss/tests/theme.rs`

- [ ] **Step 1: Write failing tests for Theme**

Create `rust-ecosystem/tailwindcss/tests/theme.rs`:

```rust
use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn test_theme_parse_simple_variables() {
    let mut variables = HashMap::new();
    variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
    variables.insert("--color-blue-500".to_string(), "#3b82f6".to_string());
    let theme = Theme {
        variables,
        keyframes: HashMap::new(),
    };

    assert_eq!(theme.resolve("--color-red-500"), Some("#ef4444".to_string()));
    assert_eq!(theme.resolve("--color-blue-500"), Some("#3b82f6".to_string()));
    assert_eq!(theme.resolve("--color-green-500"), None);
}

#[test]
fn test_theme_resolve_by_key_path() {
    let mut variables = HashMap::new();
    variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
    let theme = Theme {
        variables,
        keyframes: HashMap::new(),
    };

    // Dot-path resolution: colors.red.500 -> --color-red-500
    assert_eq!(
        theme.resolve_by_key_path("colors.red.500"),
        Some("#ef4444".to_string())
    );
}

#[test]
fn test_theme_empty() {
    let theme = Theme {
        variables: HashMap::new(),
        keyframes: HashMap::new(),
    };
    assert_eq!(theme.resolve("--anything"), None);
}
```

- [ ] **Step 2: Implement Theme**

Create `rust-ecosystem/tailwindcss/src/theme.rs`:

```rust
use std::collections::HashMap;

/// Resolved Tailwind theme with CSS variables.
#[derive(Debug, Clone, Default)]
pub struct Theme {
    /// CSS custom properties defined in @theme { ... }
    pub variables: HashMap<String, String>,
    /// Keyframe definitions
    pub keyframes: HashMap<String, Vec<crate::ast::AstNode>>,
}

impl Theme {
    /// Resolve a CSS variable name to its value.
    pub fn resolve(&self, name: &str) -> Option<String> {
        self.variables.get(name).cloned()
    }

    /// Resolve a dot-path key like "colors.red.500" to a CSS variable value.
    /// Converts dot-path to --kebab-case and looks it up.
    pub fn resolve_by_key_path(&self, key_path: &str) -> Option<String> {
        let var_name = key_path_to_var(key_path);
        self.variables.get(&var_name).cloned()
    }
}

/// Convert a dot-path theme key to a CSS variable name.
/// E.g., "colors.red.500" -> "--color-red-500"
fn key_path_to_var(key_path: &str) -> String {
    let parts: Vec<&str> = key_path.split('.').collect();
    let mut result = String::from("--");
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push('-');
        }
        result.push_str(part);
    }
    result
}
```

- [ ] **Step 3: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod parser;
pub mod theme;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
pub use candidate::parse_candidate;
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests theme
```

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add Theme struct with CSS variable resolution"
```

### Task 3.2: Implement CSS Functions (`src/functions.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/functions.rs`
- Create: `rust-ecosystem/tailwindcss/tests/functions.rs`

- [ ] **Step 1: Write failing tests for CSS functions**

Create `rust-ecosystem/tailwindcss/tests/functions.rs`:

```rust
use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule};
use farmfe_ecosystem_tailwindcss::functions::substitute_css_functions;
use farmfe_ecosystem_tailwindcss::theme::Theme;

#[test]
fn test_substitute_theme_function() {
    let mut variables = HashMap::new();
    variables.insert("--color-red-500".to_string(), "#ef4444".to_string());
    let theme = Theme {
        variables,
        keyframes: HashMap::new(),
    };

    let decl = Declaration {
        property: "color".to_string(),
        value: Some("theme(colors.red.500)".to_string()),
        important: false,
    };
    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(decl)],
    };

    let result = substitute_css_functions(vec![AstNode::Rule(rule)], &theme);

    // Should have replaced theme() with the resolved value
    if let AstNode::Rule(r) = &result[0] {
        if let AstNode::Declaration(d) = &r.nodes[0] {
            assert_eq!(d.value.as_deref(), Some("#ef4444"));
        } else {
            panic!("expected declaration");
        }
    } else {
        panic!("expected rule");
    }
}

#[test]
fn test_no_substitution_without_theme_function() {
    let theme = Theme::default();
    let decl = Declaration {
        property: "color".to_string(),
        value: Some("red".to_string()),
        important: false,
    };
    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(decl)],
    };
    let result = substitute_css_functions(vec![AstNode::Rule(rule)], &theme);

    if let AstNode::Rule(r) = &result[0] {
        if let AstNode::Declaration(d) = &r.nodes[0] {
            assert_eq!(d.value.as_deref(), Some("red"));
        } else {
            panic!("expected declaration");
        }
    } else {
        panic!("expected rule");
    }
}
```

- [ ] **Step 2: Implement CSS functions substitution**

Create `rust-ecosystem/tailwindcss/src/functions.rs`:

```rust
use crate::ast::AstNode;
use crate::theme::Theme;
use crate::walk::{walk, WalkAction};

/// Substitute `theme()` calls in CSS values with resolved theme values.
pub fn substitute_css_functions(nodes: Vec<AstNode>, theme: &Theme) -> Vec<AstNode> {
    walk(nodes, &mut |node: &AstNode, _path, _depth| {
        match node {
            AstNode::Declaration(decl) => {
                if let Some(ref value) = decl.value {
                    if let Some(new_value) = replace_theme_functions(value, theme) {
                        if &new_value != value {
                            let mut new_decl = decl.clone();
                            new_decl.value = Some(new_value);
                            return WalkAction::Replace(vec![AstNode::Declaration(new_decl)]);
                        }
                    }
                }
                WalkAction::Continue
            }
            _ => WalkAction::Continue,
        }
    })
}

/// Replace all `theme(...)` occurrences in a value string with resolved values.
fn replace_theme_functions(value: &str, theme: &Theme) -> Option<String> {
    if !value.contains("theme(") {
        return Some(value.to_string());
    }

    let mut result = value.to_string();

    // Find and replace all theme() calls
    while let Some(start) = result.find("theme(") {
        let after_name = &result[start + 6..];
        let mut depth = 1;
        let mut end = 0;
        for (i, ch) in after_name.char_indices() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
        }

        if end == 0 {
            break;
        }

        let key_path = after_name[..end].trim().to_string();
        let replacement = theme
            .resolve_by_key_path(&key_path)
            .unwrap_or_else(|| format!("theme({})", key_path));

        result.replace_range(start..start + 6 + end + 1, &replacement);
    }

    Some(result)
}
```

- [ ] **Step 4: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod functions;
pub mod parser;
pub mod theme;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
pub use candidate::parse_candidate;
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests functions
```

Expected: All PASS

- [ ] **Step 6: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add theme() CSS function substitution"
```

---

## Phase 4: Utilities Registry

### Task 4.1: Implement built-in utilities (`src/utilities.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/utilities.rs`
- Create: `rust-ecosystem/tailwindcss/tests/utilities.rs`

- [ ] **Step 1: Write failing tests for utilities**

Create `rust-ecosystem/tailwindcss/tests/utilities.rs`:

```rust
use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::candidate::ParsedCandidate;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_testing_helpers::assert_snapshot;

#[test]
fn test_flex_utility() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "flex".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec![],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "flex".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    assert_snapshot!(output);
}

#[test]
fn test_display_block_utility() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "block".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec![],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "block".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    assert_eq!(
        output.trim(),
        ".block {\n  display: block;\n}"
    );
}

#[test]
fn test_unknown_utility_returns_empty() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "nonexistent-utility-xyz".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec![],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "nonexistent-utility-xyz".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    assert!(result.is_empty());
}
```

- [ ] **Step 2: Update snapshot**

```bash
INSTA_UPDATE=always cargo test -p farmfe_ecosystem_tailwindcss --tests utilities
```

- [ ] **Step 3: Implement utility registry**

Create `rust-ecosystem/tailwindcss/src/utilities.rs`. This provides the built-in utility definitions. For the initial implementation, support the most common static utilities:

```rust
use std::collections::HashMap;
use crate::ast::{AstNode, Declaration, StyleRule};
use crate::candidate::ParsedCandidate;
use crate::theme::Theme;

/// Registry of built-in utility classes.
pub struct UtilityRegistry {
    static_utilities: HashMap<String, Vec<(String, String)>>,
}

impl UtilityRegistry {
    /// Create the built-in utility registry with all standard utilities.
    pub fn builtin() -> Self {
        let mut static_utilities = HashMap::new();

        // Display utilities
        static_utilities.insert("block".into(), vec![("display".into(), "block".into())]);
        static_utilities.insert("inline".into(), vec![("display".into(), "inline".into())]);
        static_utilities.insert("inline-block".into(), vec![("display".into(), "inline-block".into())]);
        static_utilities.insert("flex".into(), vec![("display".into(), "flex".into())]);
        static_utilities.insert("inline-flex".into(), vec![("display".into(), "inline-flex".into())]);
        static_utilities.insert("grid".into(), vec![("display".into(), "grid".into())]);
        static_utilities.insert("inline-grid".into(), vec![("display".into(), "inline-grid".into())]);
        static_utilities.insert("hidden".into(), vec![("display".into(), "none".into())]);
        static_utilities.insert("table".into(), vec![("display".into(), "table".into())]);
        static_utilities.insert("flow-root".into(), vec![("display".into(), "flow-root".into())]);
        static_utilities.insert("contents".into(), vec![("display".into(), "contents".into())]);
        static_utilities.insert("list-item".into(), vec![("display".into(), "list-item".into())]);

        // Position
        static_utilities.insert("static".into(), vec![("position".into(), "static".into())]);
        static_utilities.insert("fixed".into(), vec![("position".into(), "fixed".into())]);
        static_utilities.insert("absolute".into(), vec![("position".into(), "absolute".into())]);
        static_utilities.insert("relative".into(), vec![("position".into(), "relative".into())]);
        static_utilities.insert("sticky".into(), vec![("position".into(), "sticky".into())]);

        // Flexbox
        static_utilities.insert("flex-row".into(), vec![("flex-direction".into(), "row".into())]);
        static_utilities.insert("flex-col".into(), vec![("flex-direction".into(), "column".into())]);
        static_utilities.insert("flex-wrap".into(), vec![("flex-wrap".into(), "wrap".into())]);
        static_utilities.insert("flex-nowrap".into(), vec![("flex-wrap".into(), "nowrap".into())]);
        static_utilities.insert("items-start".into(), vec![("align-items".into(), "flex-start".into())]);
        static_utilities.insert("items-center".into(), vec![("align-items".into(), "center".into())]);
        static_utilities.insert("items-end".into(), vec![("align-items".into(), "flex-end".into())]);
        static_utilities.insert("justify-start".into(), vec![("justify-content".into(), "flex-start".into())]);
        static_utilities.insert("justify-center".into(), vec![("justify-content".into(), "center".into())]);
        static_utilities.insert("justify-end".into(), vec![("justify-content".into(), "flex-end".into())]);
        static_utilities.insert("justify-between".into(), vec![("justify-content".into(), "space-between".into())]);

        // Text alignment
        static_utilities.insert("text-left".into(), vec![("text-align".into(), "left".into())]);
        static_utilities.insert("text-center".into(), vec![("text-align".into(), "center".into())]);
        static_utilities.insert("text-right".into(), vec![("text-align".into(), "right".into())]);

        // Font weight
        static_utilities.insert("font-bold".into(), vec![("font-weight".into(), "700".into())]);
        static_utilities.insert("font-normal".into(), vec![("font-weight".into(), "400".into())]);

        // Overflow
        static_utilities.insert("overflow-auto".into(), vec![("overflow".into(), "auto".into())]);
        static_utilities.insert("overflow-hidden".into(), vec![("overflow".into(), "hidden".into())]);
        static_utilities.insert("overflow-visible".into(), vec![("overflow".into(), "visible".into())]);

        // Visibility
        static_utilities.insert("visible".into(), vec![("visibility".into(), "visible".into())]);
        static_utilities.insert("invisible".into(), vec![("visibility".into(), "hidden".into())]);

        // Cursor
        static_utilities.insert("cursor-pointer".into(), vec![("cursor".into(), "pointer".into())]);

        // Text decoration
        static_utilities.insert("underline".into(), vec![("text-decoration-line".into(), "underline".into())]);

        // White space
        static_utilities.insert("whitespace-nowrap".into(), vec![("white-space".into(), "nowrap".into())]);

        Self { static_utilities }
    }

    /// Check if a utility root exists in the registry.
    pub fn has(&self, name: &str) -> bool {
        self.static_utilities.contains_key(name)
    }

    /// Generate CSS AST nodes for a parsed candidate.
    pub fn generate(&self, candidate: &ParsedCandidate, _theme: &Theme) -> Vec<AstNode> {
        // Handle arbitrary properties
        if let Some((ref property, ref value)) = candidate.arbitrary_property {
            let class_name = format!("[{}:{}]", property, value);
            let selector = build_selector(&class_name, &candidate.variants);
            let decl = Declaration {
                property: property.clone(),
                value: Some(value.clone()),
                important: candidate.important,
            };
            return vec![AstNode::Rule(StyleRule {
                selector,
                nodes: vec![AstNode::Declaration(decl)],
            })];
        }

        // Handle static utilities
        if let Some(declarations) = self.static_utilities.get(&candidate.utility_root) {
            let class_name = build_class_name(candidate);
            let selector = build_selector(&class_name, &candidate.variants);

            let nodes: Vec<AstNode> = declarations
                .iter()
                .map(|(prop, val)| {
                    AstNode::Declaration(Declaration {
                        property: prop.clone(),
                        value: Some(val.clone()),
                        important: candidate.important,
                    })
                })
                .collect();

            return vec![AstNode::Rule(StyleRule { selector, nodes })];
        }

        // Unknown utility
        vec![]
    }
}

/// Build a CSS class name from a candidate.
fn build_class_name(candidate: &ParsedCandidate) -> String {
    let root = &candidate.utility_root;
    match &candidate.arbitrary_value {
        Some(val) => {
            let type_prefix = match &candidate.type_hint {
                Some(hint) => format!("{}:", hint),
                None => String::new(),
            };
            format!("{}-[{}{}]", root, type_prefix, val)
        }
        None => match &candidate.utility_value {
            Some(val) => format!("{}-{}", root, val),
            None => root.clone(),
        },
    }
}

/// Build a CSS selector including variant prefixes.
fn build_selector(class_name: &str, variants: &[String]) -> String {
    if variants.is_empty() {
        return format!(".{}", escape_class_name(class_name));
    }

    // For simplicity in this phase, chain variants as pseudo-classes
    let escaped = escape_class_name(class_name);
    let variant_str = variants
        .iter()
        .rev()
        .map(|v| match v.as_str() {
            "hover" => ".\\:hover:hover",
            "focus" => ".\\:focus:focus",
            "active" => ".\\:active:active",
            "disabled" => ".\\:disabled:disabled",
            "first" => ".\\:first:first-child",
            "last" => ".\\:last:last-child",
            "odd" => ".\\:odd:nth-child(odd)",
            "even" => ".\\:even:nth-child(2n)",
            _ => return format!("{} .{}", variant_selector(v), escaped),
        })
        .collect::<Vec<_>>()
        .join("");

    // For pseudo-class variants that don't use compound selector syntax
    if variant_str.contains(":hover")
        || variant_str.contains(":focus")
        || variant_str.contains(":active")
    {
        // Remove leading dot from variant class and use regular variant naming
        let mut selector = format!(".{}", escaped);
        for v in variants.iter().rev() {
            selector = match v.as_str() {
                "hover" => format!(".\\:{}\\:{}{}", "hover", v, &selector[1..]),
                "focus" => format!(".\\:{}\\:{}{}", "focus", v, &selector[1..]),
                "active" => format!(".\\:{}\\:{}{}", "active", v, &selector[1..]),
                v => format!(".\\:{}\\:{}", v, &selector[1..]),
            };
        }
        return selector;
    }

    format!(".{}", escaped)
}

fn variant_selector(variant: &str) -> String {
    format!(".\\:{}", variant)
}

fn escape_class_name(name: &str) -> String {
    name.replace(':', "\\:")
        .replace('/', "\\/")
        .replace('.', "\\.")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('#', "\\#")
        .replace('!', "\\!")
}
```

- [ ] **Step 4: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod functions;
pub mod parser;
pub mod theme;
pub mod utilities;
pub mod walk;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
pub use candidate::parse_candidate;
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss --tests utilities
```

Expected: All PASS

- [ ] **Step 6: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add built-in utility registry (display, position, flexbox, ...)"
```

---

## Phase 5: Variants Registry

### Task 5.1: Implement built-in variants (`src/variants.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/variants.rs`
- Create: `rust-ecosystem/tailwindcss/tests/variants.rs`

- [ ] **Step 1: Write failing tests for variants**

Create `rust-ecosystem/tailwindcss/tests/variants.rs`:

```rust
use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::variants::VariantRegistry;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::candidate::ParsedCandidate;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_testing_helpers::assert_snapshot;

#[test]
fn test_hover_variant_generates_hover_selector() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "flex".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec!["hover".to_string()],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "hover:flex".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    // Output should include :hover pseudo-class
    assert!(output.contains(":hover"));
    assert_snapshot!(output);
}

#[test]
fn test_focus_variant() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "block".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec!["focus".to_string()],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "focus:block".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    assert!(output.contains(":focus"));
}

#[test]
fn test_stacked_variants() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "flex".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec!["hover".to_string(), "focus".to_string()],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "focus:hover:flex".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    assert!(output.contains(":focus"));
    assert!(output.contains(":hover"));
}

#[test]
fn test_no_variant_returns_plain_selector() {
    let registry = UtilityRegistry::builtin();
    let theme = Theme::default();
    let candidate = ParsedCandidate {
        utility_root: "flex".to_string(),
        utility_value: None,
        arbitrary_property: None,
        arbitrary_value: None,
        type_hint: None,
        variants: vec![],
        modifier: None,
        modifier_is_arbitrary: false,
        important: false,
        is_static: true,
        raw: "flex".to_string(),
    };

    let result = registry.generate(&candidate, &theme);
    let output = to_css(&result);
    assert!(output.starts_with(".flex"));
}

#[test]
fn test_variant_registry_has_builtins() {
    let registry = VariantRegistry::builtin();
    assert!(registry.has("hover"));
    assert!(registry.has("focus"));
    assert!(registry.has("active"));
    assert!(registry.has("disabled"));
    assert!(!registry.has("nonexistent-variant"));
}
```

- [ ] **Step 2: Implement variant registry and update utility selector generation**

Create `rust-ecosystem/tailwindcss/src/variants.rs`:

```rust
use std::collections::HashSet;

/// Registry of built-in variants.
pub struct VariantRegistry {
    variants: HashSet<String>,
}

impl VariantRegistry {
    /// Create the built-in variant registry.
    pub fn builtin() -> Self {
        let mut variants = HashSet::new();

        // Pseudo-classes
        variants.insert("hover".to_string());
        variants.insert("focus".to_string());
        variants.insert("active".to_string());
        variants.insert("disabled".to_string());
        variants.insert("enabled".to_string());
        variants.insert("visited".to_string());
        variants.insert("target".to_string());
        variants.insert("first".to_string());
        variants.insert("last".to_string());
        variants.insert("only".to_string());
        variants.insert("odd".to_string());
        variants.insert("even".to_string());
        variants.insert("first-of-type".to_string());
        variants.insert("last-of-type".to_string());
        variants.insert("only-of-type".to_string());
        variants.insert("empty".to_string());
        variants.insert("required".to_string());
        variants.insert("valid".to_string());
        variants.insert("invalid".to_string());
        variants.insert("checked".to_string());
        variants.insert("indeterminate".to_string());
        variants.insert("default".to_string());
        variants.insert("optional".to_string());
        variants.insert("in-range".to_string());
        variants.insert("out-of-range".to_string());
        variants.insert("read-only".to_string());
        variants.insert("read-write".to_string());
        variants.insert("placeholder-shown".to_string());
        variants.insert("autofill".to_string());
        variants.insert("focus-within".to_string());
        variants.insert("focus-visible".to_string());
        variants.insert("open".to_string());
        variants.insert("inert".to_string());

        // Pseudo-elements
        variants.insert("before".to_string());
        variants.insert("after".to_string());
        variants.insert("first-letter".to_string());
        variants.insert("first-line".to_string());
        variants.insert("marker".to_string());
        variants.insert("selection".to_string());
        variants.insert("file".to_string());
        variants.insert("placeholder".to_string());
        variants.insert("backdrop".to_string());

        // Directional
        variants.insert("ltr".to_string());
        variants.insert("rtl".to_string());

        // Media
        variants.insert("dark".to_string());
        variants.insert("print".to_string());
        variants.insert("motion-safe".to_string());
        variants.insert("motion-reduce".to_string());
        variants.insert("portrait".to_string());
        variants.insert("landscape".to_string());
        variants.insert("contrast-more".to_string());
        variants.insert("contrast-less".to_string());
        variants.insert("starting".to_string());

        // Breakpoints (default values from tailwind v4)
        variants.insert("sm".to_string());
        variants.insert("md".to_string());
        variants.insert("lg".to_string());
        variants.insert("xl".to_string());
        variants.insert("2xl".to_string());

        // Container queries
        variants.insert("@lg".to_string());
        variants.insert("@md".to_string());
        variants.insert("@sm".to_string());
        variants.insert("@xl".to_string());

        // Group and peer
        variants.insert("group".to_string());
        variants.insert("peer".to_string());
        variants.insert("group-hover".to_string());
        variants.insert("group-focus".to_string());
        variants.insert("peer-hover".to_string());
        variants.insert("peer-focus".to_string());

        Self { variants }
    }

    /// Check if a variant exists in the registry.
    pub fn has(&self, name: &str) -> bool {
        self.variants.contains(name)
    }
}
```

- [ ] **Step 3: Run tests and update snapshots**

```bash
INSTA_UPDATE=always cargo test -p farmfe_ecosystem_tailwindcss --tests variants
```

Expected: All PASS

- [ ] **Step 4: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add built-in variant registry"
```

---

## Phase 6: Compiler Integration (rewrite `Compiler::build()`)

### Task 6.1: Implement DesignSystem and Compiler::build()

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/design_system.rs`
- Modify: `rust-ecosystem/tailwindcss/src/compiler.rs`
- Modify: `rust-ecosystem/tailwindcss/tests/compiler.rs`

- [ ] **Step 1: Write integration tests for Compiler::build()**

Rewrite `rust-ecosystem/tailwindcss/tests/compiler.rs`:

```rust
use farmfe_ecosystem_tailwindcss::{compile, CompilerOptions, Features, Polyfills};
use farmfe_testing_helpers::assert_snapshot;

#[test]
fn test_compiler_build_processes_candidates() {
    let mut compiler = compile(
        "@tailwind utilities;",
        CompilerOptions::default(),
    ).unwrap();

    let output = compiler.build(&["flex".to_string(), "block".to_string()]);
    // Should not be a passthrough — should contain generated CSS
    assert!(!output.is_empty());
    assert_snapshot!(output);
}

#[test]
fn test_compiler_detects_utilities_feature() {
    let compiler = compile(
        "@import \"tailwindcss\";\n@tailwind utilities;",
        CompilerOptions::default(),
    ).unwrap();

    assert!(compiler.features.contains(Features::UTILITIES));
}

#[test]
fn test_compiler_detects_apply_feature() {
    let compiler = compile(
        ".foo { @apply flex; }",
        CompilerOptions::default(),
    ).unwrap();

    assert!(compiler.features.contains(Features::AT_APPLY));
}

#[test]
fn test_compiler_detects_theme_function_feature() {
    let compiler = compile(
        ".foo { color: theme(colors.red.500); }",
        CompilerOptions::default(),
    ).unwrap();

    assert!(compiler.features.contains(Features::THEME_FUNCTION));
}

#[test]
fn test_compiler_build_empty_candidates() {
    let mut compiler = compile(
        "@tailwind utilities;",
        CompilerOptions::default(),
    ).unwrap();

    let output = compiler.build(&[]);
    // Should produce valid (possibly empty) CSS
    insta::assert_snapshot!(output);
}

#[test]
fn test_compiler_with_config() {
    use serde_json::json;
    use farmfe_ecosystem_tailwindcss::TailwindConfig;

    let compiler = compile(
        ".foo { color: red; }",
        CompilerOptions {
            features: Features::AT_APPLY | Features::THEME_FUNCTION,
            polyfills: Polyfills::AT_MEDIA_HOVER,
            dependencies: vec!["/tmp/input.css".to_string()],
            source_maps_enabled: true,
            config: Some(TailwindConfig::new(json!({
                "theme": {
                    "extend": {
                        "colors": { "brand": "#123456" }
                    }
                }
            }))),
        },
    ).unwrap();

    let output = format!(
        "css: {}\nmap: {}\ndeps: {}\nhas_config: {}\nfeatures: {}\npolyfills: {}",
        compiler.build(&[]),
        compiler.build_source_map().unwrap_or_default(),
        compiler.dependencies().join(","),
        compiler.config().is_some(),
        compiler.features.contains(Features::AT_APPLY),
        compiler.polyfills.contains(Polyfills::AT_MEDIA_HOVER),
    );

    assert_snapshot!(output);
}
```

- [ ] **Step 2: Implement DesignSystem**

Create `rust-ecosystem/tailwindcss/src/design_system.rs`:

```rust
use crate::ast::AstNode;
use crate::candidate::ParsedCandidate;
use crate::theme::Theme;
use crate::utilities::UtilityRegistry;
use crate::variants::VariantRegistry;

/// The DesignSystem is the central orchestrator that wires together
/// theme, utilities, and variants to compile candidates into CSS AST.
pub struct DesignSystem {
    pub theme: Theme,
    pub utilities: UtilityRegistry,
    pub variants: VariantRegistry,
}

impl DesignSystem {
    /// Build a DesignSystem from parsed CSS AST and external config.
    pub fn build(_ast: &[AstNode], _theme: Theme) -> Self {
        // For now, use built-in registries
        // In future phases, extract @utility and @custom-variant from AST
        Self {
            theme: _theme,
            utilities: UtilityRegistry::builtin(),
            variants: VariantRegistry::builtin(),
        }
    }

    /// Compile candidate strings into CSS AST nodes.
    pub fn compile_candidates(&self, candidates: &[String]) -> Vec<AstNode> {
        let mut result = Vec::new();

        for raw in candidates {
            if let Some(candidate) = crate::candidate::parse_candidate(raw) {
                let nodes = self.utilities.generate(&candidate, &self.theme);
                result.extend(nodes);
            }
        }

        result
    }
}
```

- [ ] **Step 3: Rewrite compiler.rs with real build()**

Rewrite `rust-ecosystem/tailwindcss/src/compiler.rs`:

```rust
use crate::ast::{self, AstNode};
use crate::design_system::DesignSystem;
use crate::parser::parse;
use crate::theme::Theme;
use serde_json::Value;

// ── Features ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features(u32);

impl Features {
    pub const NONE: Self = Self(0);
    pub const AT_APPLY: Self = Self(1 << 0);
    pub const JS_PLUGIN_COMPAT: Self = Self(1 << 1);
    pub const THEME_FUNCTION: Self = Self(1 << 2);
    pub const UTILITIES: Self = Self(1 << 3);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn has_any_output_feature(self) -> bool {
        self.contains(Self::AT_APPLY)
            || self.contains(Self::JS_PLUGIN_COMPAT)
            || self.contains(Self::THEME_FUNCTION)
            || self.contains(Self::UTILITIES)
    }
}

impl std::ops::BitOr for Features {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Features {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Features {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

// ── Polyfills ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Polyfills(u32);

impl Polyfills {
    pub const NONE: Self = Self(0);
    pub const AT_MEDIA_HOVER: Self = Self(1 << 0);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for Polyfills {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Polyfills {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TailwindConfig {
    pub data: Value,
}

impl TailwindConfig {
    pub fn new(data: Value) -> Self {
        Self { data }
    }
}

// ── CompilerOptions ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub features: Features,
    pub polyfills: Polyfills,
    pub dependencies: Vec<String>,
    pub source_maps_enabled: bool,
    pub config: Option<TailwindConfig>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            features: Features::NONE,
            polyfills: Polyfills::NONE,
            dependencies: Vec::new(),
            source_maps_enabled: false,
            config: None,
        }
    }
}

// ── Compiler ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Compiler {
    design_system: DesignSystem,
    pub features: Features,
    pub polyfills: Polyfills,
    dependencies: Vec<String>,
    source_maps_enabled: bool,
    config: Option<TailwindConfig>,
}

impl Compiler {
    pub fn new(
        design_system: DesignSystem,
        features: Features,
        options: CompilerOptions,
    ) -> Self {
        Self {
            design_system,
            features,
            polyfills: options.polyfills,
            dependencies: options.dependencies,
            source_maps_enabled: options.source_maps_enabled,
            config: options.config,
        }
    }

    /// Build final CSS from the given candidate list.
    pub fn build(&mut self, candidates: &[String]) -> String {
        let nodes = self.design_system.compile_candidates(candidates);
        let optimized = ast::optimize_ast(nodes);
        ast::to_css(&optimized)
    }

    pub fn build_source_map(&self) -> Option<String> {
        if !self.source_maps_enabled {
            return None;
        }
        Some(r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string())
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub fn config(&self) -> Option<&TailwindConfig> {
        self.config.as_ref()
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CompileError {
    ParseError(String),
}

// ── compile() ─────────────────────────────────────────────────────────────────

/// Parse CSS, detect features, build DesignSystem, and return a Compiler.
pub fn compile(css: &str, options: CompilerOptions) -> Result<Compiler, CompileError> {
    let ast = parse(css);

    // Detect features from AST
    let features = detect_features(&ast);

    // Build theme from @theme blocks (for now, empty theme)
    let theme = Theme::default();

    // Build design system
    let design_system = DesignSystem::build(&ast, theme);

    Ok(Compiler::new(design_system, features, options))
}

/// Walk the AST to detect which Tailwind features are used.
fn detect_features(ast: &[AstNode]) -> Features {
    let mut features = Features::NONE;

    fn walk_features(nodes: &[AstNode], features: &mut Features) {
        for node in nodes {
            match node {
                AstNode::AtRule(at_rule) => {
                    // @import "tailwindcss" → utilities
                    if at_rule.name == "@import" {
                        if at_rule.params.contains("tailwindcss") {
                            *features |= Features::UTILITIES;
                        }
                    }
                    // @tailwind utilities → utilities
                    if at_rule.name == "@tailwind" {
                        *features |= Features::UTILITIES;
                    }
                    walk_features(&at_rule.nodes, features);
                }
                AstNode::Rule(rule) => {
                    walk_features(&rule.nodes, features);
                }
                AstNode::Declaration(decl) => {
                    if decl.property == "@apply" {
                        *features |= Features::AT_APPLY;
                    }
                    if let Some(ref value) = decl.value {
                        if value.contains("theme(") {
                            *features |= Features::THEME_FUNCTION;
                        }
                    }
                }
                AstNode::Context(ctx) => {
                    walk_features(&ctx.nodes, features);
                }
                AstNode::AtRoot(at_root) => {
                    walk_features(&at_root.nodes, features);
                }
                _ => {}
            }
        }
    }

    walk_features(ast, &mut features);

    // Also check raw CSS text for @apply and @import that might not be
    // captured by the parser yet (inline @apply in properties)
    // This is handled by the Declaration check above.

    features
}
```

- [ ] **Step 4: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod design_system;
pub mod functions;
pub mod parser;
pub mod theme;
pub mod utilities;
pub mod variants;
pub mod walk;

pub use compiler::{
    compile, CompileError, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig,
};
pub use candidate::parse_candidate;
```

- [ ] **Step 5: Run tests and update snapshots**

```bash
INSTA_UPDATE=always cargo test -p farmfe_ecosystem_tailwindcss --tests compiler
```

Expected: All PASS with generated snapshots

- [ ] **Step 6: Run all existing tests**

```bash
cargo test -p farmfe_ecosystem_tailwindcss
```

Expected: All tests pass (including features tests that were existing)

- [ ] **Step 7: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: implement real Compiler::build() with design system integration"
```

---

## Phase 7: @apply Support

### Task 7.1: Implement `@apply` substitution (`src/apply.rs`)

**Files:**
- Create: `rust-ecosystem/tailwindcss/src/apply.rs`
- Create: `rust-ecosystem/tailwindcss/tests/apply.rs`

- [ ] **Step 1: Write failing tests for @apply**

Create `rust-ecosystem/tailwindcss/tests/apply.rs`:

```rust
use std::collections::HashMap;
use farmfe_ecosystem_tailwindcss::apply::substitute_at_apply;
use farmfe_ecosystem_tailwindcss::ast::{AstNode, Declaration, StyleRule, to_css};
use farmfe_ecosystem_tailwindcss::design_system::DesignSystem;
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;
use farmfe_ecosystem_tailwindcss::variants::VariantRegistry;
use farmfe_testing_helpers::assert_snapshot;

fn make_design_system() -> DesignSystem {
    DesignSystem {
        theme: Theme::default(),
        utilities: UtilityRegistry::builtin(),
        variants: VariantRegistry::builtin(),
    }
}

#[test]
fn test_substitute_simple_apply() {
    let ds = make_design_system();

    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "@apply".to_string(),
            value: Some("flex".to_string()),
            important: false,
        })],
    };

    let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds).unwrap();
    let output = to_css(&result);

    // The @apply should be replaced with the actual flex declarations
    assert!(!output.contains("@apply"));
    assert!(output.contains("display: flex"));
    assert_snapshot!(output);
}

#[test]
fn test_substitute_apply_with_multiple_utilities() {
    let ds = make_design_system();

    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "@apply".to_string(),
            value: Some("flex items-center".to_string()),
            important: false,
        })],
    };

    let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds).unwrap();
    let output = to_css(&result);

    assert!(!output.contains("@apply"));
    assert_snapshot!(output);
}

#[test]
fn test_apply_with_unknown_utility_errors() {
    let ds = make_design_system();

    let rule = StyleRule {
        selector: ".foo".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "@apply".to_string(),
            value: Some("nonexistent-utility-xyz".to_string()),
            important: false,
        })],
    };

    let result = substitute_at_apply(vec![AstNode::Rule(rule)], &ds);
    assert!(result.is_err());
}

#[test]
fn test_apply_inside_keyframes_is_rejected() {
    let ds = make_design_system();

    let at_rule = farmfe_ecosystem_tailwindcss::ast::AtRule {
        name: "@keyframes".to_string(),
        params: "spin".to_string(),
        nodes: vec![AstNode::Declaration(Declaration {
            property: "@apply".to_string(),
            value: Some("flex".to_string()),
            important: false,
        })],
    };

    let result = substitute_at_apply(vec![AstNode::AtRule(at_rule)], &ds);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Implement @apply substitution**

Create `rust-ecosystem/tailwindcss/src/apply.rs`:

```rust
use crate::ast::AstNode;
use crate::candidate::parse_candidate;
use crate::design_system::DesignSystem;
use crate::walk::{walk, WalkAction};

#[derive(Debug, Clone)]
pub enum ApplyError {
    UnknownUtility(String),
    KeyframesNotAllowed,
    CircularDependency(String),
}

/// Substitute `@apply` declarations with the corresponding utility CSS rules.
pub fn substitute_at_apply(
    nodes: Vec<AstNode>,
    design_system: &DesignSystem,
) -> Result<Vec<AstNode>, ApplyError> {
    let mut inside_keyframes = false;

    let result = walk(nodes, &mut |node: &AstNode, _path, _depth| {
        match node {
            AstNode::AtRule(at_rule) if at_rule.name == "@keyframes" => {
                // Check if any child is @apply
                for child in &at_rule.nodes {
                    if let AstNode::Declaration(decl) = child {
                        if decl.property == "@apply" {
                            // Collect error to return later
                            return WalkAction::Continue; // handled by the keyframe check below
                        }
                    }
                }
                inside_keyframes = true;
                WalkAction::Continue
            }
            AstNode::Declaration(decl) if decl.property == "@apply" => {
                if inside_keyframes {
                    // Error: can't @apply in @keyframes
                    return WalkAction::Continue;
                }

                if let Some(ref value) = decl.value {
                    let utility_names: Vec<&str> = value.split_whitespace().collect();
                    let mut replacement_nodes: Vec<AstNode> = Vec::new();

                    for name in utility_names {
                        if let Some(candidate) = parse_candidate(name) {
                            let generated = design_system.utilities.generate(
                                &candidate,
                                &design_system.theme,
                            );
                            if generated.is_empty() {
                                // Unknown utility
                                return WalkAction::Continue; // will be filtered after
                            } else {
                                replacement_nodes.extend(generated);
                            }
                        } else {
                            // Could not parse — unknown utility
                            return WalkAction::Continue;
                        }
                    }

                    return WalkAction::Replace(replacement_nodes);
                }
                WalkAction::Continue
            }
            _ => WalkAction::Continue,
        }
    });

    // Walk again to check for @apply inside @keyframes
    let mut has_keyframe_error = false;
    walk(result.clone(), &mut |node: &AstNode, path, _depth| {
        if let AstNode::Declaration(decl) = node {
            if decl.property == "@apply" {
                // Check if any parent is @keyframes
                for ancestor in path {
                    if let AstNode::AtRule(at) = ancestor {
                        if at.name == "@keyframes" {
                            has_keyframe_error = true;
                            return WalkAction::Stop;
                        }
                    }
                }
                // If we still have @apply after substitution, it's an error
                has_keyframe_error = true;
                return WalkAction::Stop;
            }
        }
        WalkAction::Continue
    });

    if has_keyframe_error {
        return Err(ApplyError::KeyframesNotAllowed);
    }

    Ok(result)
}
```

- [ ] **Step 3: Update lib.rs**

Edit `rust-ecosystem/tailwindcss/src/lib.rs`:

```rust
pub mod apply;
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod design_system;
pub mod functions;
pub mod parser;
pub mod theme;
pub mod utilities;
pub mod variants;
pub mod walk;

pub use compiler::{
    compile, CompileError, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig,
};
pub use candidate::parse_candidate;
```

- [ ] **Step 4: Run tests and update snapshots**

```bash
INSTA_UPDATE=always cargo test -p farmfe_ecosystem_tailwindcss --tests apply
```

Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add rust-ecosystem/tailwindcss/
git commit -m "feat: add @apply substitution with error handling"
```

---

## Phase 8: Node Crate Integration

### Task 8.1: Update tailwindcss-node compile.rs

**Files:**
- Modify: `rust-ecosystem/tailwindcss-node/src/compile.rs`

- [ ] **Step 1: Update compile() in tailwindcss-node**

Edit `rust-ecosystem/tailwindcss-node/src/compile.rs` to handle the new `Result` return type from `farmfe_ecosystem_tailwindcss::compile()`:

In the `compile()` function (public API), change the call site from:

```rust
let compiler = farmfe_ecosystem_tailwindcss::compiler::compile(
    &processed_css,
    CompilerOptions { ... },
);
```

To:

```rust
let compiler = farmfe_ecosystem_tailwindcss::compiler::compile(
    &processed_css,
    CompilerOptions {
        features,
        polyfills: options.polyfills,
        dependencies: seen.keys().cloned().collect(),
        source_maps_enabled: options.from.is_some(),
        config: options.config.clone(),
    },
)
.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Compile error: {:?}", e)))?;
```

- [ ] **Step 2: Verify compilation of the node crate**

```bash
cargo check -p farmfe_ecosystem_tailwindcss_node
```

Expected: Compiles without errors

- [ ] **Step 3: Run all tests in node crate**

```bash
cargo test -p farmfe_ecosystem_tailwindcss_node
```

Expected: Existing tests pass (or fix any failures from the Result change)

- [ ] **Step 4: Commit**

```bash
git add rust-ecosystem/tailwindcss-node/
git commit -m "chore: update node crate compile() for new Result-returning core API"
```

---

## Phase 9: Plugin Integration

### Task 9.1: Update rust-plugins/tailwindcss to use oxide Scanner

**Files:**
- Modify: `rust-plugins/tailwindcss/Cargo.toml`
- Modify: `rust-plugins/tailwindcss/src/lib.rs`

- [x] **Step 1: Add tailwindcss-oxide to plugin dependencies**

Edit `rust-plugins/tailwindcss/Cargo.toml`:

```toml
[dependencies]
# ...existing dependencies...
tailwindcss-oxide = { path = "../../tailwindcss/crates/oxide" }
```

- [x] **Step 2: Update plugin lib.rs scanner usage**

Edit `rust-plugins/tailwindcss/src/lib.rs`:

Replace:
```rust
use farmfe_ecosystem_tailwindcss::scanner::extract_candidates;
```

With:
```rust
use tailwindcss_oxide::Scanner;
use tailwindcss_oxide::ChangedContent;
```

Replace the `scan_candidates()` method:
```rust
fn scan_candidates(&self, content: &str, extension: &str) -> Vec<String> {
    let mut scanner = Scanner::new(vec![]);
    scanner.scan_content(vec![ChangedContent::Content(
        content.to_string(),
        extension.to_string(),
    )]);
    let result: Vec<String> = scanner // scanner.scan_content returns String directly
        .scan(vec![ChangedContent::Content(content.to_string(), extension.to_string())])
        .into_iter()
        .collect();
    result
}
```

Note: The exact Scanner API call will be adjusted based on the upstream `tailwindcss-oxide` crate's actual API. The key methods are:
- `Scanner::new(sources: Vec<PublicSourceEntry>)` — create a new scanner
- `Scanner::scan_content(changed_content: Vec<ChangedContent>) -> Vec<String>` — scan content and return candidates

- [x] **Step 3: Update the transform method**

In the `transform()` method, update the scan call:

```rust
if CANDIDATE_EXT_RE.is_match(resolved_path) {
    let extension = Self::get_extension(resolved_path);
    let new_candidates = self.scan_candidates(&param.content, extension);
    // ... rest unchanged
}
```

Remove `get_extension` `#[cfg(test)]` attribute so it's available at runtime.

- [x] **Step 4: Verify compilation**

```bash
cargo check -p farmfe_plugin_tailwindcss
```

Expected: Compiles without errors

- [x] **Step 5: Run plugin tests**

```bash
cargo test -p farmfe_plugin_tailwindcss
```

Expected: Tests pass

- [ ] **Step 6: Commit**

```bash
git add rust-plugins/tailwindcss/
git commit -m "feat: use upstream tailwindcss-oxide Scanner in plugin"
```

---

## Phase 10: Final Verification

### Task 10.1: Full workspace check

- [ ] **Step 1: Run full Rust checks**

```bash
cargo check --all --all-targets
```

Expected: No errors

- [ ] **Step 2: Run full Rust test suite**

```bash
cargo test -p farmfe_ecosystem_tailwindcss
cargo test -p farmfe_ecosystem_tailwindcss_node
cargo test -p farmfe_plugin_tailwindcss
```

Expected: All tests pass

- [ ] **Step 3: Run clippy**

```bash
cargo clippy --all --all-targets
```

Expected: No new warnings

- [ ] **Step 4: Run farm-ready-gate**

```bash
# Run the full quality gate
pnpm run check
```

Expected: All checks pass

- [ ] **Step 5: Commit final state**

```bash
git add -A
git commit -m "chore: final verification and cleanup for tailwindcss Rust migration"
```

---

## Summary

**Total phases:** 10 (0-9)
**Total tasks:** ~15
**Total steps:** ~60

**Key deliverables:**
1. `farmfe_ecosystem_tailwindcss` — Core compiler with real `Compiler::build()`
2. `farmfe_ecosystem_tailwindcss_node` — Updated orchestration for new Result API
3. `rust-plugins/tailwindcss` — Uses upstream `tailwindcss-oxide` Scanner
4. Comprehensive test coverage migrated from upstream JS tests

**Post-migration:**
- JS `@tailwindcss/node` dependency can be removed from `js-plugins/tailwindcss`
- The JS plugin becomes a thin compatibility wrapper (or can be deleted entirely)
- `Compiler::build()` is no longer a passthrough
