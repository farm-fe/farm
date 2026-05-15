use crate::ast::AstNode;
use crate::candidate::parse_candidate;
use crate::theme::Theme;
use crate::utilities::UtilityRegistry;
use crate::variants::VariantRegistry;

/// The `DesignSystem` is the central orchestrator that wires together the
/// theme, utilities registry, and variants registry to compile candidate
/// strings into CSS AST nodes.
pub struct DesignSystem {
  pub theme: Theme,
  pub utilities: UtilityRegistry,
  pub variants: VariantRegistry,
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
