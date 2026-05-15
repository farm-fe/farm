use crate::ast::{AstNode, StyleRule};
use crate::candidate::parse_candidate;
use crate::design_system::DesignSystem;
use crate::utilities::{build_class_name, escape_class_name};

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
/// Variant-bearing candidates (e.g. `@apply hover:bg-red-500`) are inlined as
/// nested `&`-prefixed rules and/or wrapping at-rules, matching upstream
/// tailwindcss v4 behaviour which relies on native CSS nesting.
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
      // @keyframes block: recurse but mark as inside_keyframes = true.
      AstNode::AtRule(ref at_rule) if at_rule.name == "@keyframes" => {
        for child in &at_rule.nodes {
          if let AstNode::AtRule(a) = child {
            if a.name == "@apply" {
              return Err(ApplyError::KeyframesNotAllowed);
            }
          }
        }
        if let AstNode::AtRule(mut at) = node {
          at.nodes = process_nodes(at.nodes, ds, true)?;
          result.push(AstNode::AtRule(at));
        }
      }

      // @apply inside @keyframes — error.
      AstNode::AtRule(ref at_rule) if at_rule.name == "@apply" && inside_keyframes => {
        return Err(ApplyError::KeyframesNotAllowed);
      }

      // @apply at-rule — expand.
      AstNode::AtRule(ref at_rule) if at_rule.name == "@apply" => {
        let params = at_rule.params.clone();
        let names: Vec<&str> = params.split_whitespace().collect();

        for name in names {
          let candidate =
            parse_candidate(name).ok_or_else(|| ApplyError::UnknownUtility(name.to_string()))?;

          let generated =
            ds.utilities
              .generate_with_variants(&candidate, &ds.theme, Some(&ds.variants));
          if generated.is_empty() {
            return Err(ApplyError::UnknownUtility(name.to_string()));
          }

          let class_name = build_class_name(&candidate);
          let escaped = escape_class_name(&class_name);
          let prefix = format!(".{}", escaped);

          for g_node in generated {
            let replaced = replace_class_with_ampersand(g_node, &prefix);
            // If the outermost replaced node is a rule whose selector is
            // exactly `&`, unwrap it and inline the declarations directly
            // into the parent so plain `@apply flex` keeps the prior
            // (decl-only) shape.
            match replaced {
              AstNode::Rule(rule) if rule.selector == "&" => {
                result.extend(rule.nodes);
              }
              other => result.push(other),
            }
          }
        }
      }

      // Style rule: recurse into children.
      AstNode::Rule(mut rule) => {
        rule.nodes = process_nodes(rule.nodes, ds, inside_keyframes)?;
        result.push(AstNode::Rule(rule));
      }

      // Other at-rule: recurse.
      AstNode::AtRule(mut at_rule) => {
        at_rule.nodes = process_nodes(at_rule.nodes, ds, inside_keyframes)?;
        result.push(AstNode::AtRule(at_rule));
      }

      // Leaf nodes.
      other => result.push(other),
    }
  }

  Ok(result)
}

/// Walk a generated utility AST and rewrite the class selector prefix (e.g.
/// `.hover\:bg-red-500`) into `&` so the result becomes a nested rule that
/// inherits the parent's selector.
fn replace_class_with_ampersand(node: AstNode, class_prefix: &str) -> AstNode {
  match node {
    AstNode::Rule(rule) => {
      let new_selector = rewrite_selector(&rule.selector, class_prefix);
      let new_nodes = rule
        .nodes
        .into_iter()
        .map(|n| replace_class_with_ampersand(n, class_prefix))
        .collect();
      AstNode::Rule(StyleRule {
        selector: new_selector,
        nodes: new_nodes,
      })
    }
    AstNode::AtRule(mut at) => {
      at.nodes = at
        .nodes
        .into_iter()
        .map(|n| replace_class_with_ampersand(n, class_prefix))
        .collect();
      AstNode::AtRule(at)
    }
    other => other,
  }
}

/// Replace every occurrence of `class_prefix` (already including the leading
/// `.`) in `selector` with `&`. This handles compound selectors produced by
/// variants such as `.hover\:bg-red:hover`, `.peer-hover\:foo:is(...):hover`,
/// and selector lists.
fn rewrite_selector(selector: &str, class_prefix: &str) -> String {
  if class_prefix.is_empty() {
    return selector.to_string();
  }
  // Replace all occurrences; class-prefix is unique enough thanks to
  // escaping.
  selector.replace(class_prefix, "&")
}
