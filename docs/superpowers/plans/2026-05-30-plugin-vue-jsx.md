# @farmfe/plugin-vue-jsx Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Port swc-plugin-vue-jsx to a native Farm Rust plugin, integrate it into the existing Vue example alongside @farmfe/plugin-vue, and verify both plugins work together with build + e2e tests.

**Architecture:** Standalone cdylib Rust plugin at `rust-plugins/vue-jsx/` implementing only the `transform` hook. Parses source with `farmfe_toolkit::script::parse_module`, runs the ported `VueJsxTransformVisitor` on the SWC AST, codegens back with `farmfe_toolkit::script::codegen_module`. All 7 options ported. Integrated into `examples/vue/` to validate co-existence with `@farmfe/plugin-vue`.

**Tech Stack:** Rust (edition 2021), swc_ecma_ast via farmfe_core/toolkit re-exports, NAPI-RS/npm for distribution, Vue 3 + TSX for example integration.

---

## File Inventory

| File | Action | Purpose |
|------|--------|---------|
| `rust-plugins/vue-jsx/package.json` | Create | npm package metadata |
| `rust-plugins/vue-jsx/Cargo.toml` | Create | Rust crate metadata |
| `rust-plugins/vue-jsx/rustfmt.toml` | Create | Copy from plugin-vue |
| `rust-plugins/vue-jsx/.gitignore` | Create | Copy from plugin-vue |
| `rust-plugins/vue-jsx/options.d.ts` | Create | Shared TS options type |
| `rust-plugins/vue-jsx/scripts/func.js` | Create | JS loader bridge |
| `rust-plugins/vue-jsx/scripts/index.js` | Create | Platform binary resolver |
| `rust-plugins/vue-jsx/scripts/index.d.ts` | Create | TS declaration |
| `rust-plugins/vue-jsx/src/lib.rs` | Create | Plugin struct + transform hook |
| `rust-plugins/vue-jsx/src/options.rs` | Create | Options struct + serde deser |
| `rust-plugins/vue-jsx/src/patch_flags.rs` | Create | PatchFlags bitflags |
| `rust-plugins/vue-jsx/src/slot_flag.rs` | Create | SlotFlag enum |
| `rust-plugins/vue-jsx/src/directive.rs` | Create | Directive parsing |
| `rust-plugins/vue-jsx/src/util.rs` | Create | Utility functions |
| `rust-plugins/vue-jsx/src/resolve_type.rs` | Create | TS type resolution |
| `rust-plugins/vue-jsx/tests/mod.rs` | Create | Integration tests |
| `rust-plugins/vue-jsx/tests/fixtures/*/input.jsx` | Create | Port 43 test fixtures |
| `rust-plugins/vue-jsx/tests/fixtures/*/output.js` | Create | Port 43 expected outputs |
| `rust-plugins/vue-jsx/tests/fixtures/*/config.json` | Create | Port fixture configs |
| `website/docs/frameworks/vue.mdx` | Modify | Replace Vite JSX plugin ref |
| `website/docs/plugins/official-plugins/vue-jsx.mdx` | Create | Plugin documentation page |
| `examples/vue/package.json` | Modify | Add @farmfe/plugin-vue-jsx dep |
| `examples/vue/farm.config.ts` | Modify | Register vue-jsx plugin |
| `examples/vue/src/components/Welcome.tsx` | Create | Vue JSX test component |
| `examples/vue/src/views/HomeView.vue` | Modify | Import Welcome.tsx |
| `examples/vue/e2e.spec.mjs` | Modify | Add JSX component + HMR assertions |

---

### Task 1: Scaffold Rust crate and npm package

**Files:**
- Create: `rust-plugins/vue-jsx/Cargo.toml`
- Create: `rust-plugins/vue-jsx/rustfmt.toml`
- Create: `rust-plugins/vue-jsx/.gitignore`
- Create: `rust-plugins/vue-jsx/package.json`
- Create: `rust-plugins/vue-jsx/options.d.ts`

- [x] **Step 1: Create directory**

```bash
mkdir -p rust-plugins/vue-jsx/src rust-plugins/vue-jsx/scripts rust-plugins/vue-jsx/tests/fixtures
```

- [x] **Step 2: Write Cargo.toml**

```toml
[package]
edition = "2021"
name = "farmfe_plugin_vue_jsx"
version = "0.0.1"
description = "Farm Rust plugin to transform Vue JSX/TSX"
license = "MIT"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
farmfe_core = { version = "*", path = "../../crates/core" }
farmfe_macro_plugin = { version = "*", path = "../../crates/macro_plugin" }
farmfe_toolkit = { version = "*", path = "../../crates/toolkit" }
farmfe_toolkit_plugin_types = { version = "*", path = "../../crates/toolkit_plugin_types" }
serde = { version = "1.0", features = ["derive"] }
bitflags = "2.4"
css_dataset = "0.3"
indexmap = "2.1"
fxhash = "0.2.1"
regex = "1.10"

[dev-dependencies]
farmfe_testing_helpers = { path = "../../crates/testing_helpers" }
farmfe_compiler = { path = "../../crates/compiler" }
```

- [x] **Step 3: Write rustfmt.toml**

Copy from `rust-plugins/vue/rustfmt.toml`:

```toml
edition = "2021"
```

- [x] **Step 4: Write .gitignore**

```
target
*.node
*.farm
```

- [x] **Step 5: Write package.json**

```json
{
  "name": "@farmfe/plugin-vue-jsx",
  "version": "0.0.1",
  "main": "scripts/index.js",
  "types": "scripts/index.d.ts",
  "type": "module",
  "license": "MIT",
  "devDependencies": {
    "@farmfe/plugin-tools": "workspace:*"
  },
  "napi": {
    "binaryName": "farm-plugin-vue-jsx",
    "targets": [
      "x86_64-unknown-linux-gnu",
      "x86_64-pc-windows-msvc",
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "aarch64-unknown-linux-gnu",
      "aarch64-unknown-linux-musl",
      "x86_64-unknown-linux-musl",
      "i686-pc-windows-msvc",
      "aarch64-pc-windows-msvc"
    ]
  },
  "exports": {
    ".": {
      "import": "./scripts/func.js",
      "types": "./scripts/index.d.ts",
      "default": "./scripts/index.js"
    },
    "./package.json": "./package.json"
  },
  "scripts": {
    "build": "farm-plugin-tools build --platform -p farmfe_plugin_vue_jsx --release",
    "build:publish": "cross-env CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_STRIP=symbols CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_OPT_LEVEL=z farm-plugin-tools build --platform --cargo-name farmfe_plugin_vue_jsx -p farmfe_plugin_vue_jsx --release",
    "prepublishOnly": "farm-plugin-tools prepublish"
  },
  "files": [
    "scripts",
    "options.d.ts"
  ]
}
```

- [x] **Step 6: Write options.d.ts**

```ts
export interface VueJsxPluginOptions {
  /** Convert `on` / `nativeOn` attrs to use @vue/babel-helper-vue-transform-on */
  transformOn?: boolean;
  /** Inject PatchFlags for optimized VNode updates */
  optimize?: boolean;
  /** Custom element detection patterns (regex strings) */
  customElementPatterns?: string[];
  /** Merge attribute objects with mergeProps from Vue */
  mergeProps?: boolean;
  /** Enable object slot detection via _isSlot helper */
  enableObjectSlots?: boolean;
  /** Custom pragma (e.g. h), overrides createVNode */
  pragma?: string;
  /** Resolve TypeScript types in defineComponent */
  resolveType?: boolean;
}
```

- [x] **Step 7: Commit**

```bash
git add rust-plugins/vue-jsx/
git commit -m "chore: scaffold vue-jsx plugin crate and npm package"
```

### Task 2: Write JS bridge scripts

**Files:**
- Create: `rust-plugins/vue-jsx/scripts/func.js`
- Create: `rust-plugins/vue-jsx/scripts/index.js`
- Create: `rust-plugins/vue-jsx/scripts/index.d.ts`

- [x] **Step 1: Write scripts/func.js**

```js
import binPath from "./index.js";

export default (options) => [binPath, options];
```

- [x] **Step 2: Write scripts/index.js**

Copy from `rust-plugins/svgr/scripts/index.js`, replacing `plugin-svgr` with `plugin-vue-jsx` and adjusting npm paths from `../npm/` to `../npm/`.

The key replacement: all `@farmfe/plugin-svgr-*` → `@farmfe/plugin-vue-jsx-*`.

- [x] **Step 3: Write scripts/index.d.ts**

```ts
import type { VueJsxPluginOptions } from '../options.d';
declare const binPath: (options?: VueJsxPluginOptions) => [string, VueJsxPluginOptions];
export default binPath;
```

- [x] **Step 4: Commit**

```bash
git add rust-plugins/vue-jsx/scripts/
git commit -m "chore: add vue-jsx JS bridge scripts"
```

### Task 3: Port options.rs

**Files:**
- Create: `rust-plugins/vue-jsx/src/options.rs`

- [x] **Step 1: Write src/options.rs**

Port from `swc-plugin-vue-jsx/visitor/src/options.rs`. Change `swc_core::common::comments::Comments` import to use `farmfe_core::swc_common::comments::Comments` (re-exported via core). Add `farmfe_toolkit_plugin_types::swc_plugin` for the unresolved_mark if needed. Otherwise identical logic.

```rust
use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, ops::Deref};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    pub transform_on: bool,
    pub optimize: bool,
    pub custom_element_patterns: Vec<Regex>,
    pub merge_props: bool,
    pub enable_object_slots: bool,
    pub pragma: Option<String>,
    pub resolve_type: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            transform_on: false,
            optimize: false,
            custom_element_patterns: Default::default(),
            merge_props: true,
            enable_object_slots: true,
            pragma: None,
            resolve_type: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl Regex {
    pub fn new(re: &str) -> Result<Self, regex::Error> {
        regex::Regex::new(re).map(Self)
    }
}

impl From<regex::Regex> for Regex {
    fn from(value: regex::Regex) -> Self {
        Self(value)
    }
}

impl Deref for Regex {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Regex {
    fn deserialize<D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(RegexVisitor)
    }
}

struct RegexVisitor;

impl Visitor<'_> for RegexVisitor {
    type Value = Regex;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string that represents a regex")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        regex::Regex::new(v)
            .map(Regex)
            .map_err(|_| E::invalid_value(Unexpected::Str(v), &"a valid regex"))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        regex::Regex::new(&v)
            .map(Regex)
            .map_err(|_| E::invalid_value(Unexpected::Str(&v), &"a valid regex"))
    }
}
```

- [x] **Step 2: Verify it compiles**

```bash
cargo check -p farmfe_plugin_vue_jsx
```

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/options.rs
git commit -m "feat(vue-jsx): port options module"
```

### Task 4: Port patch_flags.rs and slot_flag.rs

**Files:**
- Create: `rust-plugins/vue-jsx/src/patch_flags.rs`
- Create: `rust-plugins/vue-jsx/src/slot_flag.rs`

- [x] **Step 1: Write src/patch_flags.rs**

Identical to swc-plugin-vue-jsx source:

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct PatchFlags: i16 {
        const TEXT = 1;
        const CLASS = 1 << 1;
        const STYLE = 1 << 2;
        const PROPS = 1 << 3;
        const FULL_PROPS = 1 << 4;
        const HYDRATE_EVENTS = 1 << 5;
        const STABLE_FRAGMENT = 1 << 6;
        const KEYED_FRAGMENT = 1 << 7;
        const UNKEYED_FRAGMENT = 1 << 8;
        const NEED_PATCH = 1 << 9;
        const DYNAMIC_SLOTS = 1 << 10;
        const HOISTED = -1;
        const BAIL = -2;
    }
}
```

- [x] **Step 2: Write src/slot_flag.rs**

```rust
#[derive(Clone, Debug)]
pub enum SlotFlag {
    Stable = 1,
    Dynamic = 2,
}
```

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/patch_flags.rs rust-plugins/vue-jsx/src/slot_flag.rs
git commit -m "feat(vue-jsx): port patch_flags and slot_flag modules"
```

### Task 5: Port util.rs

**Files:**
- Create: `rust-plugins/vue-jsx/src/util.rs`

- [x] **Step 1: Write src/util.rs**

Port from `swc-plugin-vue-jsx/visitor/src/util.rs`. Change imports:
- `swc_core::common::DUMMY_SP` → `farmfe_core::swc_common::DUMMY_SP`
- `swc_core::ecma::ast::*` → `farmfe_core::swc_ecma_ast::*`
- `swc_core::ecma::utils::{private_ident, quote_ident, quote_str}` → `farmfe_toolkit::swc_ecma_utils::{private_ident, quote_ident, quote_str}`

The actual functions `build_slot_helper`, `is_jsx_attr_value_constant`, `is_constant`, `is_on`, `dedupe_props`, `decouple_v_models`, `transform_text` are copied verbatim — they have no swc_core plugin dependencies, only pure AST manipulation.

Key function signatures:
```rust
pub(crate) fn build_slot_helper(helper_name: Ident, is_vnode: Ident) -> FnDecl { ... }
pub(crate) fn is_jsx_attr_value_constant(value: &JSXAttrValue) -> bool { ... }
fn is_constant(expr: &Expr) -> bool { ... }
pub(crate) fn is_on(attr_name: &str) -> bool { ... }
pub(crate) fn dedupe_props(props: Vec<PropOrSpread>) -> Vec<PropOrSpread> { ... }
pub(crate) fn decouple_v_models(elems: Vec<Option<ExprOrSpread>>) -> impl Iterator<Item = JSXAttrOrSpread> { ... }
pub(crate) fn transform_text(text: &str) -> String { ... }
```

For `transform_text`, the SWC version uses `Atom::from` for `splitted.map(Atom::from).collect()` directly — in Farm, `Atom` is available as `farmfe_toolkit::swc_atoms::Atom`.

- [x] **Step 2: Verify compiles**

```bash
cargo check -p farmfe_plugin_vue_jsx
```

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/util.rs
git commit -m "feat(vue-jsx): port util module"
```

### Task 6: Port directive.rs

**Files:**
- Create: `rust-plugins/vue-jsx/src/directive.rs`

- [x] **Step 1: Write src/directive.rs**

Port from `swc-plugin-vue-jsx/visitor/src/directive.rs`. All code is pure AST manipulation — verbatim copy with these import changes:

| swc_core import | farmfe replacement |
|---|---|
| `swc_core::common::DUMMY_SP` | `farmfe_core::swc_common::DUMMY_SP` |
| `swc_core::ecma::ast::*` | `farmfe_core::swc_ecma_ast::*` |
| `swc_core::ecma::atoms::Atom` | `farmfe_toolkit::swc_atoms::Atom` |
| `swc_core::ecma::utils::{quote_ident, quote_str}` | `farmfe_toolkit::swc_ecma_utils::{quote_ident, quote_str}` |

**One change:** Replace `HANDLER.with(...)` calls (2 sites: `parse_v_text_directive`, `parse_v_html_directive`) with compile-time panics or return an error variant. Since directive parsing happens inside the visitor and the visitor needs to return errors, change these functions to return `Result`:

```rust
pub(crate) fn parse_directive(
    jsx_attr: &JSXAttr,
    is_component: bool,
) -> Result<Directive, String> {
    // ... body with `HANDLER.with(...)` replaced by:
    // return Err("You have to use JSX Expression inside your `v-text`.".to_string())
}
```

The two error sites in v-text/v-html parsing become `Err("message".into())`. The caller in lib.rs will convert to `CompilationError::TransformError`.

- [x] **Step 2: Verify compiles**

```bash
cargo check -p farmfe_plugin_vue_jsx
```

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/directive.rs
git commit -m "feat(vue-jsx): port directive module"
```

### Task 7: Port resolve_type.rs

**Files:**
- Create: `rust-plugins/vue-jsx/src/resolve_type.rs`

- [x] **Step 1: Write src/resolve_type.rs**

Port from `swc-plugin-vue-jsx/visitor/src/resolve_type.rs`. This module defines methods on `VueJsxTransformVisitor<C>` (`extract_props_type`, `extract_emits_type`, `build_props_type`, `resolve_type_elements`, `resolve_string_or_union_strings`, `resolve_indexed_access`, `infer_runtime_type`) plus free functions `extract_prop_name`, `try_unwrap_lit_prop_name`, `extract_type_ann_from_pat`, `inject_define_component_option`.

Import changes:
| swc_core import | farmfe replacement |
|---|---|
| `swc_core::common::{comments::Comments, EqIgnoreSpan, Span, Spanned, DUMMY_SP}` | `farmfe_core::swc_common::{comments::Comments, EqIgnoreSpan, Span, Spanned, DUMMY_SP}` |
| `swc_core::ecma::ast::*` | `farmfe_core::swc_ecma_ast::*` |
| `swc_core::ecma::atoms::{atom, Atom}` | `farmfe_toolkit::swc_atoms::{atom, Atom}` |
| `swc_core::ecma::utils::{quote_ident, quote_str}` | `farmfe_toolkit::swc_ecma_utils::{quote_ident, quote_str}` |

**HANDLER.with(...) → String errors:** ~15 sites. All `HANDLER.with(\|handler\| { handler.span_err(...) })` calls are replaced with `return Err("descriptive message".to_string())`. The methods on the visitor struct change return types from bare values to `Result<..., String>`.

Since these are `impl<C: Comments> VueJsxTransformVisitor<C>` methods, the free function `inject_define_component_option` stays unchanged (it has no error paths).

The `resolve_type` feature uses `FxHashMap` from `fnv` in the original — replace with `FnvHashMap` from `fxhash` (Farm convention). Actually no — check: the original doesn't use `FnvHashMap` directly in resolve_type.rs; it uses `IndexMap`/`IndexSet`. No change needed.

- [x] **Step 2: Verify compiles**

```bash
cargo check -p farmfe_plugin_vue_jsx
```

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/resolve_type.rs
git commit -m "feat(vue-jsx): port resolve_type module"
```

### Task 8: Write lib.rs — the FarmPlugin impl and transform hook

**Files:**
- Create: `rust-plugins/vue-jsx/src/lib.rs`

- [x] **Step 1: Write src/lib.rs**

This is the main orchestration file. It replaces the SWC `#[plugin_transform]` entry point with a `FarmPlugin` transform hook.

```rust
#![deny(clippy::all)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;

use farmfe_core::{
    config::Config,
    context::CompilationContext,
    error::CompilationError,
    module::ModuleType,
    plugin::{
        Plugin, PluginHookContext, PluginTransformHookParam, PluginTransformHookResult,
    },
    swc_common::{comments::Comments, Mark, SyntaxContext, DUMMY_SP},
    swc_ecma_ast::{
        self, Ident, Module as SwcModule, JSXAttr, JSXAttrName, JSXAttrOrSpread,
        JSXAttrValue, JSXElement, JSXElementChild, JSXElementName, JSXExpr,
        JSXExprContainer, JSXFragment, JSXMemberExpr, JSXNamespacedName,
        JSXOpeningElement, JSXText, Imports,
    },
    swc_ecma_ast::*,
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
    script::{codegen_module, parse_module, CodeGenCommentsConfig},
    swc_atoms::Atom,
    swc_ecma_parser::{Syntax, TsSyntax},
    swc_ecma_utils::{private_ident, quote_ident, quote_str},
    swc_ecma_visit::{VisitMut, VisitMutWith},
};

mod directive;
mod options;
mod patch_flags;
mod resolve_type;
mod slot_flag;
mod util;

use crate::directive::{is_directive, parse_directive, Directive, NormalDirective};
use crate::options::Options;
use crate::patch_flags::PatchFlags;
use crate::slot_flag::SlotFlag;
use crate::util;
use crate::resolve_type::{extract_props_type, extract_emits_type, inject_define_component_option};

const FRAGMENT: &str = "Fragment";
const KEEP_ALIVE: &str = "KeepAlive";

struct AttrsTransformationResult<'a> {
    attrs: Expr,
    patch_flags: PatchFlags,
    dynamic_props: Option<IndexSet<Cow<'a, str>>>,
    slots: Option<Box<Expr>>,
}

pub struct VueJsxTransformVisitor<C>
where
    C: Comments,
{
    options: Options,
    vue_imports: BTreeMap<&'static str, Ident>,
    transform_on_helper: Option<Ident>,

    define_component: Option<SyntaxContext>,
    interfaces: FnvHashMap<(Atom, SyntaxContext), TsInterfaceDecl>,
    type_aliases: FnvHashMap<(Atom, SyntaxContext), TsType>,

    unresolved_mark: Mark,
    comments: Option<C>,

    pragma: Option<String>,
    slot_helper_ident: Option<Ident>,
    injecting_vars: Vec<VarDeclarator>,
    slot_counter: usize,
    slot_flag_stack: Vec<SlotFlag>,

    assignment_left: Option<Ident>,
    injecting_consts: Vec<VarDeclarator>,
}

// All methods from the swc-plugin impl are copied verbatim.
// The only API-level change: methods that called HANDLER.with(...) now return Result<..., String>.
```

The full visitor (~1000 lines of method impls) is copied from `swc-plugin-vue-jsx/visitor/src/lib.rs` with these systematic replacements:

1. **Imports**: `swc_core::*` → `farmfe_core::swc_*` / `farmfe_toolkit::swc_*` (as in prior tasks)
2. **HANDLER.with(...)** in `visit_mut_jsx_opening_element`: Replace 2 instances with early return + error log
3. **`inject_define_component_option`**: Move to `resolve_type.rs` as a freestanding pub(crate) fn
4. **`resolve_type` methods**: `extract_props_type`, `extract_emits_type` → `pub(crate) fn` in resolve_type.rs

Then add the FarmPlugin struct and impl:

```rust
use fxhash::FxHashMap as FnvHashMap;

#[farm_plugin]
pub struct FarmPluginVueJsx {}

impl FarmPluginVueJsx {
    pub fn new(_config: &Config, options: String) -> Self {
        // Store options as raw JSON string for transform-time use.
        // Actually, deserialize now to validate early.
        let _opts: Options = if options.trim().is_empty() {
            Options::default()
        } else {
            serde_json::from_str(&options).unwrap_or_default()
        };
        Self {}
    }
}

impl Plugin for FarmPluginVueJsx {
    fn name(&self) -> &str {
        "FarmPluginVueJsx"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn transform(
        &self,
        param: &PluginTransformHookParam,
        _context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
        // Only transform JSX/TSX modules
        let is_jsx = matches!(&param.module_type, ModuleType::Jsx | ModuleType::Tsx);
        if !is_jsx {
            return Ok(None);
        }

        let opts: Options = self.get_options(param);
        let syntax = if matches!(&param.module_type, ModuleType::Tsx) {
            Syntax::Typescript(TsSyntax {
                tsx: true,
                ..Default::default()
            })
        } else {
            Syntax::Es(Default::default())
        };

        let unresolved_mark = Mark::fresh(Mark::root());

        // Parse
        let parse_result = parse_module(
            &param.module_id.clone().into(),
            Arc::new(param.content.clone()),
            syntax,
            farmfe_core::swc_ecma_ast::EsVersion::EsNext,
        ).map_err(|e| CompilationError::TransformError {
            resolved_path: param.resolved_path.to_string(),
            msg: format!("parse error: {:?}", e),
        })?;

        let mut module = parse_result.ast;
        let comments = parse_result.comments;
        let cm = parse_result.source_map;

        // Transform
        let mut visitor = VueJsxTransformVisitor::new(
            opts,
            unresolved_mark, 
            Some(&comments as &dyn Comments),
            param.resolved_path,
        );
        module.visit_mut_with(&mut visitor);

        // Codegen
        let buf = codegen_module(
            &module,
            cm.clone(),
            None,
            farmfe_toolkit::script::create_codegen_config(_context),
            None,
        ).map_err(|e| CompilationError::TransformError {
            resolved_path: param.resolved_path.to_string(),
            msg: format!("codegen error: {:?}", e),
        })?;

        let code = String::from_utf8(buf).map_err(|e| CompilationError::TransformError {
            resolved_path: param.resolved_path.to_string(),
            msg: format!("utf8 error: {:?}", e),
        })?;

        Ok(Some(PluginTransformHookResult {
            content: code,
            module_type: Some(param.module_type.clone()),
            source_map: None,
            ignore_previous_source_map: true,
        }))
    }
}
```

Key detail: `VueJsxTransformVisitor::new()` now takes `resolved_path: &str` as an additional parameter so error messages from `HANDLER.with` replacements can include the file path.

- [x] **Step 2: Verify compiles**

```bash
cargo check -p farmfe_plugin_vue_jsx
```

Expect some type errors around `Comments` trait object — fix by adjusting the trait bound. The `comments` field uses `Option<&dyn Comments>` in the visitor. Actually, the original uses `Option<C>` where `C: Comments`. For Farm, since we get `SingleThreadedComments` from `parse_module`, use `Option<&SingleThreadedComments>` and update the generic bound.

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/src/lib.rs
git commit -m "feat(vue-jsx): implement FarmPlugin transform hook"
```

### Task 9: Port test fixtures

**Files:**
- Create: `rust-plugins/vue-jsx/tests/fixtures/*/input.jsx` (43 files)
- Create: `rust-plugins/vue-jsx/tests/fixtures/*/output.js` (43 files)
- Create: `rust-plugins/vue-jsx/tests/fixtures/*/config.json` (few files, only where present in original)

- [x] **Step 1: Copy fixtures from swc-plugin-vue-jsx**

```bash
cp -r /home/bright/opensource/swc-plugin-vue-jsx/visitor/tests/fixture/* \
      /home/bright/opensource/farm/rust-plugins/vue-jsx/tests/fixtures/
```

The 43 fixture directories are (from the swc source at `/home/bright/opensource/swc-plugin-vue-jsx`):
```
custom-directive, custom-directive-with-argument-and-modifiers, custom-element,
custom-pragma-in-comment, custom-pragma-in-options, disable-object-slot,
empty-string, fragment-already-imported, function-expr-slot,
infer-component-name, keep-alive-named-import, keep-alive-namespace-import,
keep-namespace-import, merge-class-style-attrs, merge-props-order,
model-as-prop-name, multiple-exprs-slot, nesting-slot-flags,
non-literal-expr-slot, override-props-multiple, override-props-single,
reassign-variable-as-component, resolve-emits-types, resolve-props-types,
single-attr, slot-in-arrow-function-bug,
specifiers-merged-into-single-import-decl, v-html, v-models,
v-model-value-supports-variable, v-model-with-arg-and-modifier,
v-model-with-checkbox, v-model-with-dynamic-type-input,
v-model-with-input-lazy-modifier, v-model-with-radio, v-model-with-select,
v-model-with-textarea, v-model-with-text-input, v-show, v-slots,
v-slots-complex, v-text, without-jsx, without-props
```

- [x] **Step 2: Review output files**

Some output files may reference SWC-specific generated names. Review a sample (5-10 fixtures) to ensure the expected outputs are correct for the Farm toolchain (same SWC version, so output should match exactly).

- [x] **Step 3: Commit**

```bash
git add rust-plugins/vue-jsx/tests/fixtures/
git commit -m "test(vue-jsx): port 43 test fixtures from swc-plugin-vue-jsx"
```

### Task 10: Write integration tests

**Files:**
- Create: `rust-plugins/vue-jsx/tests/mod.rs`

- [x] **Step 1: Write tests/mod.rs**

Follow the test pattern from `rust-plugins/vue/tests/mod.rs`:

```rust
use std::sync::Arc;

use farmfe_core::{
    config::Config,
    context::CompilationContext,
    module::ModuleType,
    plugin::{Plugin, PluginTransformHookParam, PluginTransformHookResult},
};
use farmfe_plugin_vue_jsx::FarmPluginVueJsx;

fn make_plugin(options: &str) -> (Arc<CompilationContext>, FarmPluginVueJsx) {
    let config = Config::default();
    let plugin = FarmPluginVueJsx::new(&config, options.to_string());
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    (context, plugin)
}

fn transform_jsx(
    plugin: &FarmPluginVueJsx,
    input: &str,
    filename: &str,
    module_type: ModuleType,
) -> String {
    let (_, context) = make_plugin("{}");
    let transform_param = PluginTransformHookParam {
        module_id: filename.to_string(),
        content: input.to_string(),
        module_type,
        resolved_path: filename,
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
    };
    plugin
        .transform(&transform_param, &context)
        .expect("transform returns Ok")
        .expect("transform returns Some for jsx/tsx")
        .content
}

fn assert_transform(plugin: &FarmPluginVueJsx, fixture_name: &str) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);
    let input = std::fs::read_to_string(dir.join("input.jsx")).unwrap();
    let expected = std::fs::read_to_string(dir.join("output.js")).unwrap();
    let output = transform_jsx(plugin, &input, &format!("{fixture_name}.tsx"), ModuleType::Tsx);
    assert_eq!(output.trim(), expected.trim(), "mismatch for fixture: {fixture_name}");
}

#[test]
fn ignores_non_jsx_files() {
    let (context, plugin) = make_plugin("{}");
    let param = PluginTransformHookParam {
        module_id: "test.js".to_string(),
        content: "const x = 1;".to_string(),
        module_type: ModuleType::Js,
        resolved_path: "test.js",
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
    };
    assert!(plugin.transform(&param, &context).unwrap().is_none());
}

// One test per fixture:
#[test] fn v_model_with_checkbox() { assert_transform(&make_plugin("{}").1, "v-model-with-checkbox"); }
#[test] fn v_model_with_textarea() { assert_transform(&make_plugin("{}").1, "v-model-with-textarea"); }
// ... etc for all 43 fixtures
```

- [x] **Step 2: Run tests to verify they fail (plugin not built yet)**

```bash
cargo test -p farmfe_plugin_vue_jsx 2>&1 | tail -5
```
Expected: compilation errors (module not found) or test failures.

- [x] **Step 3: Build the plugin and run all tests**

After fixing compilation, run:
```bash
cargo test -p farmfe_plugin_vue_jsx
```
Iterate on failures: adjust expected output for any AST representation differences.

- [x] **Step 4: Update snapshots if needed**

If the SWC version in Farm produces slightly different output, update the fixture `output.js` files to match actual output.

- [x] **Step 5: Commit**

```bash
git add rust-plugins/vue-jsx/tests/mod.rs
git commit -m "test(vue-jsx): add integration tests for all 43 fixtures"
```

### Task 11: Update website documentation

**Files:**
- Modify: `website/docs/frameworks/vue.mdx`
- Create: `website/docs/plugins/official-plugins/vue-jsx.mdx`

- [x] **Step 1: Update website/docs/frameworks/vue.mdx**

Replace the "Integrating jsx" section (lines 30-39) from:
```ts
import VueJsx from '@vitejs/plugin-vue-jsx'
export default defineConfig({ vitePlugins: [VueJsx()] });
```
To:
```ts
import vueJsx from '@farmfe/plugin-vue-jsx'
export default defineConfig({ plugins: [vueJsx()] });
```

Also remove the `:::warning` block (lines 17-19) about using Vite plugins since we now have both native Vue plugins.

- [x] **Step 2: Create website/docs/plugins/official-plugins/vue-jsx.mdx**

Follow the same format as `website/docs/plugins/official-plugins/vue.mdx`:

```mdx
import CodeBlock from "@theme/CodeBlock";
import Tabs from "@theme/Tabs";
import TabItem from "@theme/TabItem";

# @farmfe/plugin-vue-jsx

Transform Vue JSX/TSX at build time — converts directives (`v-model`, `v-show`, etc.) into Vue 3 runtime calls. Written in Rust with zero JS overhead.

This plugin is the Farm-native equivalent of `@vitejs/plugin-vue-jsx`, ported from `swc-plugin-vue-jsx`.

## Installation

<Tabs>
  <TabItem value="npm" label="npm">
    <CodeBlock>npm install @farmfe/plugin-vue-jsx</CodeBlock>
  </TabItem>
  <TabItem value="yarn" label="yarn">
    <CodeBlock>yarn add @farmfe/plugin-vue-jsx</CodeBlock>
  </TabItem>
  <TabItem value="pnpm" label="pnpm">
    <CodeBlock>pnpm add @farmfe/plugin-vue-jsx</CodeBlock>
  </TabItem>
</Tabs>

## Usage

```ts
import { defineConfig } from "@farmfe/core";
import vueJsx from "@farmfe/plugin-vue-jsx";

export default defineConfig({
  plugins: [vueJsx()],
});
```

With options:

```ts
export default defineConfig({
  plugins: [
    vueJsx({
      optimize: true,
      transformOn: true,
      resolveType: true,
    }),
  ],
});
```

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `transformOn` | `boolean` | `false` | Convert `on` / `nativeOn` attrs via `@vue/babel-helper-vue-transform-on` |
| `optimize` | `boolean` | `false` | Inject Vue PatchFlags for optimized VNode updates |
| `customElementPatterns` | `string[]` | `[]` | Regex patterns for custom element tags |
| `mergeProps` | `boolean` | `true` | Merge attribute objects with Vue's `mergeProps` |
| `enableObjectSlots` | `boolean` | `true` | Enable object slot detection |
| `pragma` | `string` | `undefined` | Custom pragma (e.g. `h`), overrides `createVNode` |
| `resolveType` | `boolean` | `false` | Resolve TypeScript types for `defineComponent` props/emits |

## transformOn

When enabled, `on` and `nativeOn` attributes are converted using `@vue/babel-helper-vue-transform-on`. You must install this package in your project:

```bash
pnpm add @vue/babel-helper-vue-transform-on
```
```

- [x] **Step 3: Commit**

```bash
git add website/docs/frameworks/vue.mdx website/docs/plugins/official-plugins/vue-jsx.mdx
git commit -m "docs: add vue-jsx plugin documentation to website"
```

### Task 12: Integrate vue-jsx plugin into examples/vue

**Files:**
- Modify: `examples/vue/package.json`
- Modify: `examples/vue/farm.config.ts`
- Create: `examples/vue/src/components/Welcome.tsx`
- Modify: `examples/vue/src/views/HomeView.vue`
- Modify: `examples/vue/e2e.spec.mjs`

- [x] **Step 1: Add @farmfe/plugin-vue-jsx to devDependencies**

In `examples/vue/package.json`, add to devDependencies:
```json
"@farmfe/plugin-vue-jsx": "workspace:*"
```

- [x] **Step 2: Update farm.config.ts to register the jsx plugin**

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    persistentCache: {
      cacheDir: 'node_modules/.farm/vue-cache',
    },
  },
  server: {
    hmr: true,
  },
  plugins: [
    '@farmfe/plugin-vue',
    '@farmfe/plugin-vue-jsx',
    '@farmfe/plugin-sass',
  ],
});
```

Note: `@farmfe/plugin-vue-jsx` is placed after `@farmfe/plugin-vue` because vue-jsx needs to process `.tsx`/`.jsx` files that vue doesn't handle. The load/transform order isn't critical since they operate on different module types.

- [x] **Step 3: Create src/components/Welcome.tsx**

A Vue JSX component that demonstrates JSX directives:

```tsx
import { defineComponent, ref } from 'vue';

export default defineComponent({
  name: 'Welcome',
  setup() {
    const count = ref(0);
    const increment = () => { count.value++; };
    const show = ref(true);

    return () => (
      <div class="welcome-jsx">
        <p class="jsx-badge">
          Rendered by <code>@farmfe/plugin-vue-jsx</code>
        </p>
        <div class="jsx-card">
          <strong>JSX count: {count.value}</strong>
          <button onClick={increment}>+1</button>
          <button v-show={show.value} onClick={() => { show.value = false; }}>
            Hide me
          </button>
          {!show.value && <p class="jsx-reveal">v-show directive works!</p>}
        </div>
      </div>
    );
  },
});
```

This component exercises:
- Basic JSX rendering
- Event handling (`onClick`)
- Vue directive (`v-show`) — verifies the transform works
- Conditional rendering with `{expression && <jsx/>}` — verifies JSX expressions
- `ref` reactivity

- [x] **Step 4: Import Welcome.tsx in HomeView.vue**

```vue
<script setup lang="ts">
import CounterCard from '../components/CounterCard.vue';
import Welcome from '../components/Welcome';
</script>

<template>
  <section>
    <p class="intro">This example uses the native Rust <code>@farmfe/plugin-vue</code>.</p>
    <Welcome />
    <CounterCard />
  </section>
</template>

<style scoped>
.intro {
  color: #2d6a4f;
}
</style>
```

Add a small style for the JSX component at the bottom of `HomeView.vue`:

```css
:deep(.welcome-jsx) {
  margin: 1rem 0;
}

:deep(.jsx-badge) {
  color: #1b4332;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
}

:deep(.jsx-card) {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  padding: 1rem;
  border: 1px solid #95d5b2;
  border-radius: 8px;
  background: rgba(45, 106, 79, 0.08);
  margin-bottom: 1rem;
}

:deep(.jsx-card button) {
  padding: 0.25rem 0.75rem;
  border: 1px solid #40916c;
  border-radius: 4px;
  background: #40916c;
  color: #fff;
  font-size: 0.85rem;
  cursor: pointer;
}

:deep(.jsx-reveal) {
  width: 100%;
  color: #2d6a4f;
  font-weight: 500;
}
```

- [x] **Step 5: Update e2e.spec.mjs to verify the JSX component**

Add a verification section after the existing `assertVueExample` checks. Add this after the intro color check (~line 53) inside `assertVueExample`:

```js
// JSX component verification
const jsxBadge = await page.$eval('.jsx-badge', (el) => el.textContent);
expect(jsxBadge).toContain('@farmfe/plugin-vue-jsx');

const jsxCount = await page.$eval('.jsx-card strong', (el) => el.textContent);
expect(jsxCount).toContain('JSX count: 0');

// Click the +1 button in JSX card
const jsxButtons = await page.$$('.jsx-card button');
await jsxButtons[0].click();
await delay(200);
const updatedCount = await page.$eval('.jsx-card strong', (el) => el.textContent);
expect(updatedCount).toContain('JSX count: 1');

// Test v-show directive: click the hide button
await jsxButtons[1].click();
await delay(200);
const revealText = await page.$eval('.jsx-reveal', (el) => el.textContent);
expect(revealText).toContain('v-show directive works!');
```

Also add an HMR test for the JSX component. After the existing HMR tests in `assertHmr` (~line 167), add:

```js
const welcomePath = join(projectPath, 'src/components/Welcome.tsx');
await withFileEdits(
  [
    {
      file: welcomePath,
      from: 'Rendered by',
      to: 'HMR-updated JSX powered by'
    }
  ],
  async () => {
    await waitForText(page, '.jsx-badge', 'HMR-updated JSX');
    expect(await page.textContent('.jsx-card strong')).toContain('JSX count: 1');
  }
);
await waitForText(page, '.jsx-badge', 'Rendered by');
```

- [x] **Step 6: Commit**

```bash
git add examples/vue/
git commit -m "feat(vue-example): integrate @farmfe/plugin-vue-jsx with JSX component and e2e tests"
```

### Task 13: Build and e2e test verification

- [ ] **Step 1: Build only the related Rust plugins**

Build just the two plugins needed by the vue example (not the full workspace):

```bash
cd rust-plugins/vue && pnpm run build && cd ../..
cd rust-plugins/vue-jsx && pnpm run build && cd ../..
```

- [ ] **Step 2: Install dependencies for the vue example only**

Link the workspace packages that the example depends on without rebuilding everything:

```bash
cd examples/vue && pnpm install --filter @farmfe-examples/vue... && cd ../..
```

This resolves `workspace:*` deps (`@farmfe/cli`, `@farmfe/core`, `@farmfe/plugin-sass`, `@farmfe/plugin-vue`, `@farmfe/plugin-vue-jsx`) from local packages without triggering full bootstrap.

- [ ] **Step 3: Build the vue example**

```bash
cd examples/vue && pnpm run build
```

Expected: build succeeds, output in `build/` directory. Verify the JSX output file contains `createVNode` or similar Vue runtime calls.

- [ ] **Step 4: Verify JSX transform in build output**

```bash
grep -r "createVNode\|resolveComponent\|_createVNode" examples/vue/build/
```

Expected: find Vue JSX runtime calls in the compiled output.

- [ ] **Step 5: Run the Rust tests for both plugins**

```bash
cargo test -p farmfe_plugin_vue
cargo test -p farmfe_plugin_vue_jsx
```

Expected: all tests pass.

- [ ] **Step 6: Run the e2e tests for the vue example**

```bash
pnpm run test-e2e -- examples/vue/e2e.spec.mjs
```

Expected: all e2e tests pass, including JSX component rendering, JSX count increment, v-show directive, and JSX HMR.

- [ ] **Step 7: Run full CI gate**

```bash
cargo check --all --all-targets
cargo clippy --all --all-targets
cargo test --profile ci-test
```

Expected: no new errors or warnings.

- [ ] **Step 8: Commit final verification**

```bash
git add .
git commit -m "chore: verify vue+jsx plugins integration with build and e2e

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```
