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
  /// `@source` directives collected from the user AST, in source order.
  /// Consumed by the host (Node scanner / Farm plugin) — the core crate
  /// only parses and exposes them via [`DesignSystem::sources`].
  sources: Vec<SourceDirective>,
}

/// A parsed `@source` rule. Tailwind v4 supports four shapes:
///
/// ```css
/// @source "glob";                /* Include  */
/// @source not "glob";            /* Exclude  */
/// @source inline("a b !c");      /* Inline   — literal candidates */
/// @source not inline("foo*");    /* NotInline — class-name patterns */
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDirective {
  /// `@source "glob"` — include files matching the glob in scanning.
  Include(String),
  /// `@source not "glob"` — exclude files matching the glob from scanning.
  Exclude(String),
  /// `@source inline("…")` — treat the contents as a literal candidate
  /// list to be included unconditionally.
  Inline(String),
  /// `@source not inline("…")` — class-name pattern list of candidates
  /// to remove from the build.
  NotInline(String),
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
      sources: Vec::new(),
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
      sources: Vec::new(),
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
        AstNode::AtRule(at) if at.name == "@source" => {
          if let Some(directive) = parse_source_params(&at.params) {
            self.sources.push(directive);
          }
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

  /// Materialise a `:root { … }` rule containing the **referenced subset**
  /// of theme tokens.
  ///
  /// Built-in v4 utilities resolve to `var(--color-…)`, `var(--spacing)`,
  /// `var(--radius-…)`, … expressions, so the compiler must emit the
  /// matching theme defaults to `:root` for those references to bind at
  /// runtime. Only keys present in `referenced` are emitted to keep the
  /// output close to upstream tree-shaken behaviour. Keys already emitted
  /// via [`user_theme_root_rule`] are skipped to avoid duplicate rules.
  pub fn theme_root_rule_for(
    &self,
    referenced: &std::collections::HashSet<String>,
  ) -> Option<AstNode> {
    if referenced.is_empty() {
      return None;
    }
    let user_keys: std::collections::HashSet<&String> = self.user_theme_keys.iter().collect();

    // Stable, deterministic output.
    let mut sorted: Vec<&String> = referenced.iter().collect();
    sorted.sort();

    let mut decls = Vec::new();
    for key in sorted {
      if user_keys.contains(key) {
        continue;
      }
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

  /// Every `@source` directive collected from the user AST, in source order.
  ///
  /// The Rust core does not perform filesystem scanning itself — these
  /// entries are exposed so a host (Node bridge, Farm plugin) can extend or
  /// constrain its candidate scan. See [`SourceDirective`] for the
  /// supported shapes.
  pub fn sources(&self) -> &[SourceDirective] {
    &self.sources
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

/// Parse the params of an `@source` at-rule into a [`SourceDirective`].
///
/// Supported syntaxes (matching upstream Tailwind v4):
///
/// ```text
///   @source "path/glob"            → Include
///   @source not "path/glob"        → Exclude
///   @source inline("a b !c")       → Inline
///   @source not inline("foo*")     → NotInline
/// ```
///
/// The quoted argument may use single or double quotes. Surrounding
/// whitespace is trimmed. Malformed directives return `None` and are
/// silently dropped — this mirrors the upstream behaviour of ignoring
/// unrecognised forms so users can author new variants without breakage.
fn parse_source_params(params: &str) -> Option<SourceDirective> {
  let trimmed = params.trim();
  if trimmed.is_empty() {
    return None;
  }

  // Detect the optional `not` prefix.
  let (negated, rest) = match trimmed.strip_prefix("not") {
    Some(after) if after.starts_with(|c: char| c.is_ascii_whitespace()) => {
      (true, after.trim_start())
    }
    _ => (false, trimmed),
  };

  // Detect the optional `inline(…)` form.
  if let Some(after) = rest.strip_prefix("inline") {
    let inner = after.trim_start();
    let inside = inner
      .strip_prefix('(')
      .and_then(|s| s.strip_suffix(')'))?
      .trim();
    let unquoted = strip_quotes(inside).unwrap_or(inside).to_string();
    return Some(if negated {
      SourceDirective::NotInline(unquoted)
    } else {
      SourceDirective::Inline(unquoted)
    });
  }

  // Otherwise expect a quoted glob.
  let glob = strip_quotes(rest)?.to_string();
  Some(if negated {
    SourceDirective::Exclude(glob)
  } else {
    SourceDirective::Include(glob)
  })
}

/// Strip a matching pair of leading/trailing `"` or `'` quotes from `s`.
/// Returns `None` if `s` is not quoted.
fn strip_quotes(s: &str) -> Option<&str> {
  let s = s.trim();
  if s.len() >= 2 {
    let bytes = s.as_bytes();
    let first = bytes[0];
    let last = bytes[s.len() - 1];
    if (first == b'"' || first == b'\'') && first == last {
      return Some(&s[1..s.len() - 1]);
    }
  }
  None
}
