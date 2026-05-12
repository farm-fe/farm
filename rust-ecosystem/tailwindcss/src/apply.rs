use crate::ast::AstNode;
use crate::candidate::parse_candidate;
use crate::design_system::DesignSystem;

/// Error returned from [`substitute_at_apply`].
#[derive(Debug, Clone)]
pub enum ApplyError {
  /// An unrecognised utility was used in `@apply`.
  UnknownUtility(String),
  /// `@apply` inside `@keyframes` is not allowed.
  KeyframesNotAllowed,
}

impl std::fmt::Display for ApplyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ApplyError::UnknownUtility(name) => {
        write!(f, "Unknown utility in @apply: {}", name)
      }
      ApplyError::KeyframesNotAllowed => {
        write!(f, "@apply is not allowed inside @keyframes")
      }
    }
  }
}

/// Substitute all `@apply` at-rules in the given AST with the declarations of
/// the referenced utility classes.
///
/// Returns `Err` if an unknown utility is referenced or if `@apply` is used
/// inside `@keyframes`.
pub fn substitute_at_apply(
  nodes: Vec<AstNode>,
  design_system: &DesignSystem,
) -> Result<Vec<AstNode>, ApplyError> {
  process_nodes(nodes, design_system, false)
}

fn process_nodes(
  nodes: Vec<AstNode>,
  ds: &DesignSystem,
  inside_keyframes: bool,
) -> Result<Vec<AstNode>, ApplyError> {
  let mut result = Vec::new();

  for node in nodes {
    match node {
      // @keyframes block: recurse but mark as inside_keyframes = true
      AstNode::AtRule(ref at_rule) if at_rule.name == "@keyframes" => {
        // Check if any direct child is @apply
        for child in &at_rule.nodes {
          if let AstNode::AtRule(a) = child {
            if a.name == "@apply" {
              return Err(ApplyError::KeyframesNotAllowed);
            }
          }
        }
        // Recurse (error if deeper @apply found)
        if let AstNode::AtRule(mut at) = node {
          at.nodes = process_nodes(at.nodes, ds, true)?;
          result.push(AstNode::AtRule(at));
        }
      }

      // @apply inside @keyframes — error
      AstNode::AtRule(ref at_rule) if at_rule.name == "@apply" && inside_keyframes => {
        return Err(ApplyError::KeyframesNotAllowed);
      }

      // @apply at-rule — expand
      AstNode::AtRule(ref at_rule) if at_rule.name == "@apply" => {
        let params = at_rule.params.clone();
        let names: Vec<&str> = params.split_whitespace().collect();

        for name in names {
          if let Some(candidate) = parse_candidate(name) {
            let generated = ds.utilities.generate(&candidate, &ds.theme);
            if generated.is_empty() {
              return Err(ApplyError::UnknownUtility(name.to_string()));
            }
            // Extract declarations from the generated rules and inline them
            for g_node in generated {
              match g_node {
                AstNode::Rule(rule) => result.extend(rule.nodes),
                other => result.push(other),
              }
            }
          } else {
            return Err(ApplyError::UnknownUtility(name.to_string()));
          }
        }
      }

      // Rule: recurse into children
      AstNode::Rule(mut rule) => {
        rule.nodes = process_nodes(rule.nodes, ds, inside_keyframes)?;
        result.push(AstNode::Rule(rule));
      }

      // Other at-rule: recurse
      AstNode::AtRule(mut at_rule) => {
        at_rule.nodes = process_nodes(at_rule.nodes, ds, inside_keyframes)?;
        result.push(AstNode::AtRule(at_rule));
      }

      // Leaf nodes
      other => result.push(other),
    }
  }

  Ok(result)
}
