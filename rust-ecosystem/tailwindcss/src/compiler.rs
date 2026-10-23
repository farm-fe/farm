use serde_json::Value;

use crate::apply::substitute_at_apply;
use crate::ast::{self, AstNode};
use crate::design_system::DesignSystem;
use crate::parser::parse;
use crate::theme::{Theme, ThemeOptions};

/// Bitflags that describe which Tailwind CSS features are used in the compiled
/// CSS.
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

/// Polyfill flags that control which CSS compatibility transforms are applied.
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

/// Externally supplied Tailwind config payload.
///
/// This crate accepts the config as data and does not load JS config files.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TailwindConfig {
  pub data: Value,
}

impl TailwindConfig {
  pub fn new(data: Value) -> Self {
    Self { data }
  }
}

/// Options passed to [`compile`].
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

/// Compiled core state.
pub struct Compiler {
  design_system: DesignSystem,
  ast: Vec<AstNode>,
  pub features: Features,
  pub polyfills: Polyfills,
  dependencies: Vec<String>,
  source_maps_enabled: bool,
  original_css: String,
  source_map: Option<String>,
  input_source_map: Option<sourcemap::SourceMap>,
  color_variables: bool,
  config: Option<TailwindConfig>,
}

impl Compiler {
  fn new(
    design_system: DesignSystem,
    ast: Vec<AstNode>,
    features: Features,
    options: CompilerOptions,
  ) -> Self {
    Self {
      design_system,
      ast,
      features,
      polyfills: options.polyfills,
      dependencies: options.dependencies,
      source_maps_enabled: options.source_maps_enabled,
      original_css: String::new(),
      source_map: None,
      input_source_map: None,
      color_variables: false,
      config: options.config,
    }
  }

  /// Build final CSS for the given candidate list.
  ///
  /// The compiler weaves the user CSS AST together with generated utility
  /// declarations:
  ///   1. `@apply` at-rules in the user AST are substituted against the
  ///      design system (Phase 17).
  ///   2. `@tailwind utilities` / `@tailwind components` / `@tailwind base`
  ///      and `@import "tailwindcss"`-style markers are replaced inline with
  ///      the CSS generated from the supplied candidate list.
  ///   3. The combined AST is optimised (adjacent at-rule merging, ...) and
  ///      serialised to CSS.
  pub fn build(&mut self, candidates: &[String]) -> String {
    let mut ast = self.ast.clone();

    // 1. Substitute @apply against the design system. If substitution fails
    //    (unknown utility / @apply inside @keyframes), fall back to the
    //    unmodified AST so the compiler is still infallible at this stage —
    //    upstream surfaces these errors separately.
    let pre_apply = ast.clone();
    if let Ok(replaced) = substitute_at_apply(ast, &self.design_system) {
      ast = replaced;
    } else {
      ast = pre_apply;
    }

    // 2. Inline generated utilities at every Tailwind marker. A `:root`
    //    block materialising the user `@theme { … }` custom properties is
    //    prepended so utilities that resolve to `var(--…)` references have
    //    a runtime value.
    let mut utilities = Vec::new();
    if let Some(root) = self.design_system.user_theme_root_rule() {
      utilities.push(root);
    }
    let candidate_nodes = self.design_system.compile_candidates(candidates);

    // Collect every `var(--*)` reference made by the generated utilities so
    // the matching theme defaults (`--color-blue-500`, `--spacing`, …) can
    // be materialised on `:root`. Without this, utilities like
    // `bg-blue-500` would compute to `background-color: var(--color-blue-500)`
    // with no value bound at runtime.
    let referenced = collect_var_references(&candidate_nodes);
    if let Some(root) = self.design_system.theme_root_rule_for(&referenced) {
      utilities.insert(0, root);
    }

    // Keep package theme variables in the theme cascade layer. Putting them
    // inside utilities changes their priority relative to business CSS.
    if insert_theme_definition_roots(&mut ast, &utilities) {
      utilities.clear();
    }
    utilities.extend(candidate_nodes);
    utilities.extend(self.design_system.theme.get_keyframes().iter().cloned());
    if referenced.contains("--tw-blur") || referenced.contains("--tw-invert") {
      // Reset per element and pseudo-element even when preflight is disabled;
      // otherwise a child with invert inherits its parent's blur variable.
      utilities.push(AstNode::AtRule(ast::AtRule {
        name: "@layer".into(),
        params: "base".into(),
        nodes: vec![AstNode::Rule(ast::StyleRule {
          selector: "*, ::before, ::after".into(),
          nodes: ["--tw-blur", "--tw-invert"]
            .map(|property| {
              AstNode::Declaration(ast::Declaration {
                property: property.into(),
                value: Some(String::new()),
                important: false,
              })
            })
            .into(),
        })],
      }));
    }
    if referenced.contains("--tw-gradient-stops") {
      // Stops must not inherit from a parent gradient (including pseudo-elements).
      utilities.push(AstNode::AtRule(ast::AtRule {
        name: "@layer".into(),
        params: "base".into(),
        nodes: vec![AstNode::Rule(ast::StyleRule {
          selector: "*, ::before, ::after".into(),
          nodes: [
            "--tw-gradient-stops",
            "--tw-gradient-from",
            "--tw-gradient-via",
            "--tw-gradient-to",
            "--tw-gradient-via-stops",
            "--tw-gradient-from-position",
            "--tw-gradient-via-position",
            "--tw-gradient-to-position",
          ]
          .map(|property| {
            AstNode::Declaration(ast::Declaration {
              property: property.into(),
              value: Some("initial".into()),
              important: false,
            })
          })
          .into(),
        })],
      }));
    }
    if referenced.iter().any(|name| {
      [
        "--tw-backdrop-blur",
        "--tw-translate-x",
        "--tw-translate-y",
        "--tw-duration",
        "--tw-ease",
        "--tw-shadow",
      ]
      .contains(&name.as_str())
    }) {
      utilities.push(AstNode::AtRule(ast::AtRule {
        name: "@layer".into(),
        params: "base".into(),
        nodes: vec![AstNode::Rule(ast::StyleRule {
          selector: "*, ::before, ::after".into(),
          nodes: [
            "--tw-backdrop-blur",
            "--tw-transform-gpu",
            "--tw-translate-x",
            "--tw-translate-y",
            "--tw-duration",
            "--tw-ease",
            "--tw-shadow",
          ]
          .map(|property| {
            AstNode::Declaration(ast::Declaration {
              property: property.into(),
              value: Some("initial".into()),
              important: false,
            })
          })
          .into(),
        })],
      }));
    }
    if self.color_variables {
      let mut colors: Vec<_> = self
        .design_system
        .theme
        .namespace("--color")
        .into_iter()
        .filter_map(|(name, value)| name.map(|name| (name, value)))
        .collect();
      colors.sort_by(|left, right| left.0.cmp(&right.0));
      let declarations = colors
        .into_iter()
        .map(|(name, value)| {
          AstNode::Declaration(ast::Declaration {
            property: self.design_system.theme.prefix_key(&format!("--{name}")),
            value: Some(value),
            important: false,
          })
        })
        .collect();
      utilities.push(AstNode::AtRule(ast::AtRule {
        name: "@layer".into(),
        params: "base".into(),
        nodes: vec![AstNode::Rule(ast::StyleRule {
          selector: ":root".into(),
          nodes: declarations,
        })],
      }));
    }
    ast = strip_user_definitions(ast);
    ast = inline_tailwind_markers(ast, &utilities);
    ast = crate::functions::substitute_css_functions(ast, &self.design_system.theme);

    // 3. Optimise + serialise.
    let optimized = ast::optimize_ast(ast);
    if self.source_maps_enabled {
      let (css, map) = ast::to_css_with_source_map(
        &optimized,
        &self.original_css,
        self.input_source_map.as_ref(),
      );
      self.source_map = Some(map);
      css
    } else {
      ast::to_css(&optimized)
    }
  }

  pub fn build_source_map(&self) -> Option<String> {
    self.source_map.clone()
  }

  /// Expose flattened color aliases in the base layer (disabled by default).
  pub fn set_color_variables(&mut self, enabled: bool) {
    self.color_variables = enabled;
    self.source_map = None;
  }

  /// Compose parser origins with the host's import/URL preprocessing map.
  /// Must be supplied before build; it describes `compile`'s exact CSS input.
  pub fn set_input_source_map(&mut self, map: sourcemap::SourceMap) {
    self.input_source_map = Some(map);
    self.source_map = None;
  }

  pub fn dependencies(&self) -> &[String] {
    &self.dependencies
  }

  pub fn config(&self) -> Option<&TailwindConfig> {
    self.config.as_ref()
  }

  /// Every `@source` directive parsed from the user CSS, in source order.
  ///
  /// The Rust core does not perform filesystem scanning — the host (Node
  /// bridge / Farm plugin) consumes these to extend or constrain its
  /// candidate scan. See [`crate::design_system::SourceDirective`] for
  /// the supported shapes.
  pub fn sources(&self) -> &[crate::design_system::SourceDirective] {
    self.design_system.sources()
  }
}

fn insert_theme_definition_roots(nodes: &mut Vec<AstNode>, roots: &[AstNode]) -> bool {
  for index in 0..nodes.len() {
    if matches!(&nodes[index], AstNode::AtRule(at) if at.name == "@theme"
      && !at.params.split_whitespace().any(|option| option == "reference"))
    {
      nodes.splice(index..=index, roots.iter().cloned());
      return true;
    }
    let inserted = match &mut nodes[index] {
      AstNode::AtRule(at) => insert_theme_definition_roots(&mut at.nodes, roots),
      AstNode::Rule(rule) => insert_theme_definition_roots(&mut rule.nodes, roots),
      AstNode::Context(ctx) => insert_theme_definition_roots(&mut ctx.nodes, roots),
      _ => false,
    };
    if inserted {
      return true;
    }
  }
  false
}

/// Return `true` if the given at-rule is a Tailwind injection marker — i.e.
/// `@tailwind utilities|components|base|screens` or
/// `@import "tailwindcss"…`.
fn is_tailwind_marker(name: &str, params: &str) -> bool {
  match name {
    "@tailwind" => true,
    "@import" => params.contains("tailwindcss"),
    _ => false,
  }
}

/// Walk the AST and replace every Tailwind marker at-rule (see
/// [`is_tailwind_marker`]) with a clone of `utilities`. Nested rules are
/// recursed into so markers inside `@layer` / `@media` are also handled.
fn inline_tailwind_markers(nodes: Vec<AstNode>, utilities: &[AstNode]) -> Vec<AstNode> {
  let mut out = Vec::with_capacity(nodes.len());
  for node in nodes {
    match node {
      AstNode::AtRule(at) if is_tailwind_marker(&at.name, &at.params) => {
        out.extend(utilities.iter().cloned());
      }
      AstNode::AtRule(mut at) => {
        at.nodes = inline_tailwind_markers(at.nodes, utilities);
        out.push(AstNode::AtRule(at));
      }
      AstNode::Rule(mut rule) => {
        rule.nodes = inline_tailwind_markers(rule.nodes, utilities);
        out.push(AstNode::Rule(rule));
      }
      AstNode::Context(mut ctx) => {
        ctx.nodes = inline_tailwind_markers(ctx.nodes, utilities);
        out.push(AstNode::Context(ctx));
      }
      other => out.push(other),
    }
  }
  out
}

/// Walk `nodes` recursively and collect every CSS custom property
/// referenced via `var(--name, …)`. The leading `--` is preserved so the
/// result can be looked up directly in the theme.
fn collect_var_references(nodes: &[AstNode]) -> std::collections::HashSet<String> {
  let mut out = std::collections::HashSet::new();
  for node in nodes {
    collect_var_references_in(node, &mut out);
  }
  out
}

fn collect_var_references_in(node: &AstNode, out: &mut std::collections::HashSet<String>) {
  match node {
    AstNode::Declaration(decl) => {
      if let Some(value) = &decl.value {
        scan_var_calls(value, out);
      }
    }
    AstNode::AtRule(at) => {
      scan_var_calls(&at.params, out);
      for child in &at.nodes {
        collect_var_references_in(child, out);
      }
    }
    AstNode::Rule(rule) => {
      for child in &rule.nodes {
        collect_var_references_in(child, out);
      }
    }
    AstNode::Context(ctx) => {
      for child in &ctx.nodes {
        collect_var_references_in(child, out);
      }
    }
    AstNode::AtRoot(at_root) => {
      for child in &at_root.nodes {
        collect_var_references_in(child, out);
      }
    }
    AstNode::Comment(_) => {}
  }
}

/// Extract `--name` from every `var(--name, …)` call in `value`. Tolerates
/// nested `var()` calls (e.g. `var(--a, var(--b))`) by scanning the entire
/// string for `var(` openings rather than parsing a strict expression tree.
fn scan_var_calls(value: &str, out: &mut std::collections::HashSet<String>) {
  let bytes = value.as_bytes();
  let mut i = 0;
  while i + 4 <= bytes.len() {
    if &bytes[i..i + 4] == b"var(" {
      let mut j = i + 4;
      // Skip whitespace.
      while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
      }
      // Read the custom-property name (starts with `--`, ends at `,` or `)`
      // or whitespace).
      if j + 2 <= bytes.len() && &bytes[j..j + 2] == b"--" {
        let start = j;
        while j < bytes.len() {
          let c = bytes[j];
          if c == b',' || c == b')' || c.is_ascii_whitespace() {
            break;
          }
          j += 1;
        }
        if j > start {
          if let Ok(name) = std::str::from_utf8(&bytes[start..j]) {
            out.insert(name.to_string());
          }
        }
      }
      i = j;
      continue;
    }
    i += 1;
  }
}

/// Recursively remove `@utility`, `@custom-variant` and `@theme` rules from
/// the AST. These are consumed during design-system construction and must
/// not appear in the serialised CSS output.
fn strip_user_definitions(nodes: Vec<AstNode>) -> Vec<AstNode> {
  let mut out = Vec::with_capacity(nodes.len());
  for node in nodes {
    match node {
      AstNode::AtRule(at)
        if at.name == "@utility"
          || at.name == "@custom-variant"
          || at.name == "@theme"
          || at.name == "@source" =>
      {
        // drop
      }
      AstNode::AtRule(mut at) => {
        at.nodes = strip_user_definitions(at.nodes);
        out.push(AstNode::AtRule(at));
      }
      AstNode::Rule(mut rule) => {
        rule.nodes = strip_user_definitions(rule.nodes);
        out.push(AstNode::Rule(rule));
      }
      AstNode::Context(mut ctx) => {
        ctx.nodes = strip_user_definitions(ctx.nodes);
        out.push(AstNode::Context(ctx));
      }
      other => out.push(other),
    }
  }
  out
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CompileError {
  ParseError(String),
}

impl std::fmt::Display for CompileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CompileError::ParseError(msg) => write!(f, "CSS parse error: {}", msg),
    }
  }
}

// ── compile() ─────────────────────────────────────────────────────────────────

/// Parse CSS, detect features, build a [`DesignSystem`], and return a
/// [`Compiler`].
pub fn compile(css: &str, options: CompilerOptions) -> Result<Compiler, CompileError> {
  let ast = parse(css);

  let features = detect_features(&ast);

  let mut theme = Theme::with_defaults();
  if let Some(config) = &options.config {
    apply_external_config_theme(&mut theme, &config.data);
  }
  let mut design_system = DesignSystem::build(&ast, theme);
  // This v3 configuration key is an explicit compatibility request, not a
  // global change to the default modern border utility contract.
  // It is deliberately not a full Tailwind v3 compatibility mode.
  let preserve_border_width_style = options
    .config
    .as_ref()
    .and_then(|config| config.data.pointer("/corePlugins/preflight"))
    .and_then(Value::as_bool)
    == Some(false);
  design_system
    .utilities
    .set_preserve_border_width_style(preserve_border_width_style);

  if let Some(mode) = options
    .config
    .as_ref()
    .and_then(|config| config.data.get("darkMode"))
  {
    let name = mode.as_str().or_else(|| {
      mode
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_str)
    });
    if design_system.variants.custom_variant("dark").is_none() {
      match name {
        Some("class" | "selector") => {
          let selector = mode
            .as_array()
            .and_then(|items| items.get(1))
            .and_then(Value::as_str)
            .unwrap_or(".dark");
          design_system
            .variants
            .register_custom_variant("dark".into(), format!("&:where({selector}, {selector} *)"));
        }
        Some("media") => {}
        _ => {
          return Err(CompileError::ParseError(
            "darkMode must be media, class or selector (with an optional selector)".into(),
          ));
        }
      }
    }
  }

  let ast = if options.source_maps_enabled {
    crate::parser::parse_with_origins(css)
  } else {
    ast
  };
  let mut compiler = Compiler::new(design_system, ast, features, options);
  compiler.original_css = css.to_string();
  Ok(compiler)
}

fn apply_external_config_theme(theme: &mut Theme, config: &Value) {
  let Some(theme_config) = config.get("theme") else {
    return;
  };
  for section in [Some(theme_config), theme_config.get("extend")]
    .into_iter()
    .flatten()
  {
    if let Some(animations) = section.get("animation").and_then(Value::as_object) {
      for (name, value) in animations {
        if let Some(value) = value.as_str() {
          theme.add(&format!("--animate-{name}"), value, ThemeOptions::NONE);
        }
      }
    }
    if let Some(keyframes) = section.get("keyframes").and_then(Value::as_object) {
      for (name, steps) in keyframes {
        let Some(steps) = steps.as_object() else {
          continue;
        };
        let mut nodes = vec![];
        for (selector, declarations) in steps {
          let Some(declarations) = declarations.as_object() else {
            continue;
          };
          let declarations = declarations
            .iter()
            .filter_map(|(property, value)| {
              let value = match value {
                Value::String(value) => value.clone(),
                Value::Number(value) => value.to_string(),
                _ => return None,
              };
              let property = if property.starts_with("--") {
                property.clone()
              } else {
                property
                  .chars()
                  .fold(String::new(), |mut output, character| {
                    if character.is_ascii_uppercase() {
                      output.push('-');
                      output.push(character.to_ascii_lowercase());
                    } else {
                      output.push(character);
                    }
                    output
                  })
              };
              Some(AstNode::Declaration(crate::ast::Declaration {
                property,
                value: Some(value),
                important: false,
              }))
            })
            .collect();
          nodes.push(AstNode::Rule(crate::ast::StyleRule {
            selector: selector.clone(),
            nodes: declarations,
          }));
        }
        theme.add_keyframes(AstNode::AtRule(crate::ast::AtRule {
          name: "@keyframes".into(),
          params: name.clone(),
          nodes,
        }));
      }
    }
  }

  if let Some(colors) = theme_config.get("colors") {
    add_config_colors(theme, colors, &mut Vec::new());
  }

  if let Some(colors) = theme_config
    .get("extend")
    .and_then(|extend| extend.get("colors"))
  {
    add_config_colors(theme, colors, &mut Vec::new());
  }
}

fn add_config_colors(theme: &mut Theme, colors: &Value, path: &mut Vec<String>) {
  match colors {
    Value::String(value) => {
      if path.is_empty() {
        return;
      }

      let key = format!("--color-{}", path.join("-"));
      theme.add(&key, value, ThemeOptions::NONE);
    }
    Value::Object(map) => {
      for (name, value) in map {
        let path_len = path.len();
        if name != "DEFAULT" {
          path.push(name.to_string());
        }
        add_config_colors(theme, value, path);
        path.truncate(path_len);
      }
    }
    _ => {}
  }
}

// ── feature detection ─────────────────────────────────────────────────────────

/// Walk the AST and determine which Tailwind features are used.
fn detect_features(ast: &[AstNode]) -> Features {
  let mut features = Features::NONE;
  walk_features(ast, &mut features);
  features
}

fn walk_features(nodes: &[AstNode], features: &mut Features) {
  for node in nodes {
    match node {
      AstNode::AtRule(at_rule) => {
        match at_rule.name.as_str() {
          "@import" if at_rule.params.contains("tailwindcss") => {
            *features |= Features::UTILITIES;
          }
          "@tailwind" => {
            *features |= Features::UTILITIES;
          }
          "@apply" => {
            *features |= Features::AT_APPLY;
          }
          _ => {}
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
      AstNode::Context(ctx) => walk_features(&ctx.nodes, features),
      AstNode::AtRoot(at_root) => walk_features(&at_root.nodes, features),
      _ => {}
    }
  }
}
