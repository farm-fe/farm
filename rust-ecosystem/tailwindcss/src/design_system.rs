use crate::ast::AstNode;
use crate::candidate::parse_candidate;
use crate::theme::{Theme, ThemeOptions};
use crate::utilities::UtilityRegistry;
use crate::variants::VariantRegistry;

/// The `DesignSystem` is the central orchestrator that wires together the
/// theme, utilities registry, and variants registry to compile candidate
/// strings into CSS AST nodes.
pub struct DesignSystem {
  pub theme: Theme,
  pub utilities: UtilityRegistry,
  pub variants: VariantRegistry,
  /// CSS custom-property keys (`--color-brand`, …) registered via user
  /// `@theme { … }` blocks, in source order. Used to materialise a
  /// `:root { … }` declaration set so utilities that resolve to
  /// `var(--…)` references actually have a value at runtime. Entries
  /// added with the `reference` modifier are excluded.
  user_theme_keys: Vec<String>,
}

impl DesignSystem {
  /// Construct an empty `DesignSystem` with the built-in registries and an
  /// empty theme. Used by tests that need a registry without a CSS source.
  pub fn empty() -> Self {
    Self {
      theme: Theme::default(),
      utilities: UtilityRegistry::builtin(),
      variants: VariantRegistry::builtin(),
      user_theme_keys: Vec::new(),
    }
  }
}

impl DesignSystem {
  /// Build a `DesignSystem` from a parsed CSS AST and a theme.
  ///
  /// The user AST is scanned for `@utility name { … }` and
  /// `@custom-variant name (…)` rules, which are registered with the
  /// utility / variant registries respectively. Built-in registries provide
  /// the default Tailwind utility set; user registrations take precedence.
  pub fn build(ast: &[AstNode], theme: Theme) -> Self {
    let mut design_system = Self {
      theme,
      utilities: UtilityRegistry::builtin(),
      variants: VariantRegistry::builtin(),
      user_theme_keys: Vec::new(),
    };
    design_system.collect_user_definitions(ast);
    design_system
  }

  /// Walk the user AST and register every `@utility` and `@custom-variant`
  /// rule it encounters. Nested rules inside `@layer`/`@media` are also
  /// inspected so users may scope definitions within layers.
  fn collect_user_definitions(&mut self, nodes: &[AstNode]) {
    for node in nodes {
      match node {
        AstNode::AtRule(at) if at.name == "@utility" => {
          let name = at.params.trim().to_string();
          if name.is_empty() {
            continue;
          }
          let decls = extract_declarations(&at.nodes);
          if !decls.is_empty() {
            self.utilities.register_static_utility(name, decls);
          }
        }
        AstNode::AtRule(at) if at.name == "@custom-variant" => {
          if let Some((name, body)) = parse_custom_variant_params(&at.params) {
            self.variants.register_custom_variant(name, body);
          }
        }
        AstNode::AtRule(at) if at.name == "@theme" => {
          let options = parse_theme_options(&at.params);
          self.register_theme_block(&at.nodes, options);
        }
        AstNode::AtRule(at) => self.collect_user_definitions(&at.nodes),
        AstNode::Rule(rule) => self.collect_user_definitions(&rule.nodes),
        AstNode::Context(ctx) => self.collect_user_definitions(&ctx.nodes),
        AstNode::AtRoot(at_root) => self.collect_user_definitions(&at_root.nodes),
        _ => {}
      }
    }
  }

  /// Compile raw candidate strings into CSS AST nodes.
  pub fn compile_candidates(&self, candidates: &[String]) -> Vec<AstNode> {
    let mut result = Vec::new();

    for raw in candidates {
      if let Some(candidate) = parse_candidate(raw) {
        let nodes =
          self
            .utilities
            .generate_with_variants(&candidate, &self.theme, Some(&self.variants));
        result.extend(nodes);
      }
    }

    result
  }

  /// Register every CSS custom-property declaration in an `@theme { … }`
  /// block into [`Self::theme`], applying any modifier flags parsed from the
  /// at-rule params (e.g. `@theme reference`, `@theme inline default`).
  ///
  /// Declarations whose property does not begin with `--` are ignored — the
  /// `@theme` block in Tailwind v4 only stores CSS custom properties. The
  /// upstream parser additionally accepts namespace-reset forms like
  /// `--color-*: initial;` and `--*: initial;`, both of which are forwarded
  /// to [`Theme::add`] which already knows how to handle them.
  fn register_theme_block(&mut self, nodes: &[AstNode], options: ThemeOptions) {
    for node in nodes {
      if let AstNode::Declaration(decl) = node {
        if !decl.property.starts_with("--") {
          continue;
        }
        if let Some(value) = &decl.value {
          self.theme.add(&decl.property, value, options);

          // Namespace resets (`--color-*: initial`) and per-key removals
          // (`--color-foo: initial`) must not contribute a `:root` entry.
          if value == "initial" || decl.property.ends_with("-*") {
            continue;
          }
          // Reference-only entries are not emitted to `:root` — they exist
          // purely for utility resolution.
          if options.contains(ThemeOptions::REFERENCE) {
            continue;
          }
          // Preserve insertion order, dedup.
          if !self.user_theme_keys.iter().any(|k| k == &decl.property) {
            self.user_theme_keys.push(decl.property.clone());
          }
        }
      }
    }
  }

  /// Materialise a `:root { … }` rule containing every CSS custom property
  /// registered via user `@theme { … }` blocks that survived
  /// `Theme::add`. Returns `None` when no such entries exist so the compiler
  /// can skip emission entirely.
  pub fn user_theme_root_rule(&self) -> Option<AstNode> {
    let mut decls = Vec::new();
    for key in &self.user_theme_keys {
      // Honour potential later removals (e.g. namespace reset that ran after
      // the key was inserted): only emit keys still present in the theme.
      if let Some(value) = self.theme.get(&[key.as_str()]) {
        decls.push(AstNode::Declaration(crate::ast::Declaration {
          property: key.clone(),
          value: Some(value.to_string()),
          important: false,
        }));
      }
    }
    if decls.is_empty() {
      return None;
    }
    Some(AstNode::Rule(crate::ast::StyleRule {
      selector: ":root, :host".to_string(),
      nodes: decls,
    }))
  }
}

/// Extract `(property, value)` pairs from a node list, skipping any nested
/// rules or at-rules. Important flag is currently ignored — `@utility` blocks
/// declare normal-priority CSS.
fn extract_declarations(nodes: &[AstNode]) -> Vec<(String, String)> {
  let mut out = Vec::new();
  for node in nodes {
    if let AstNode::Declaration(decl) = node {
      if let Some(value) = &decl.value {
        out.push((decl.property.clone(), value.clone()));
      }
    }
  }
  out
}

/// Parse the params of a `@custom-variant` rule into `(name, body)`.
///
/// Accepts the v4 shorthand forms:
///   - `name (selector)` — body is the parenthesised selector (which may
///     contain `&`) or an at-rule expression like `@media (...)`.
///   - `name selector`   — body is everything after the first whitespace.
fn parse_custom_variant_params(params: &str) -> Option<(String, String)> {
  let trimmed = params.trim();
  if trimmed.is_empty() {
    return None;
  }

  // Find the first whitespace or `(` that ends the variant name.
  let mut split_at = trimmed.len();
  for (i, ch) in trimmed.char_indices() {
    if ch == ' ' || ch == '\t' || ch == '(' {
      split_at = i;
      break;
    }
  }
  if split_at == 0 {
    return None;
  }
  let name = trimmed[..split_at].to_string();
  let rest = trimmed[split_at..].trim();
  if rest.is_empty() {
    return None;
  }

  // Strip a single pair of outer parentheses if present.
  let body = if let Some(inner) = rest.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
    inner.trim().to_string()
  } else {
    rest.to_string()
  };

  if body.is_empty() {
    return None;
  }
  Some((name, body))
}

/// Parse the params of an `@theme` at-rule into a [`ThemeOptions`] bit-set.
///
/// Recognised whitespace-separated tokens (matching upstream Tailwind v4):
///   - `reference` → [`ThemeOptions::REFERENCE`]
///   - `inline`    → [`ThemeOptions::INLINE`]
///   - `static`    → [`ThemeOptions::STATIC`]
///   - `default`   → [`ThemeOptions::DEFAULT`]
///
/// Unknown tokens are ignored so users may evolve the directive without us
/// breaking their build.
fn parse_theme_options(params: &str) -> ThemeOptions {
  let mut options = ThemeOptions::NONE;
  for token in params.split_ascii_whitespace() {
    match token {
      "reference" => options |= ThemeOptions::REFERENCE,
      "inline" => options |= ThemeOptions::INLINE,
      "static" => options |= ThemeOptions::STATIC,
      "default" => options |= ThemeOptions::DEFAULT,
      _ => {}
    }
  }
  options
}
