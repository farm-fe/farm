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
  /// Build a `DesignSystem` from a parsed CSS AST and an optional theme.
  ///
  /// In Phase 6 the AST is not yet analysed for `@utility` / `@custom-variant`
  /// blocks; built-in registries are always used.
  pub fn build(_ast: &[AstNode], theme: Theme) -> Self {
    Self {
      theme,
      utilities: UtilityRegistry::builtin(),
      variants: VariantRegistry::builtin(),
    }
  }

  /// Compile raw candidate strings into CSS AST nodes.
  pub fn compile_candidates(&self, candidates: &[String]) -> Vec<AstNode> {
    let mut result = Vec::new();

    for raw in candidates {
      if let Some(candidate) = parse_candidate(raw) {
        let nodes = self.utilities.generate(&candidate, &self.theme);
        result.extend(nodes);
      }
    }

    result
  }
}
