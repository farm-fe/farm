use std::collections::HashMap;
use std::collections::HashSet;

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
  pub context: HashMap<String, String>,
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
        css.push_str(&format!(
          "{}{} {};\n",
          indent, at_rule.name, at_rule.params
        ));
      } else if at_rule.params.is_empty() {
        css.push_str(&format!("{}{} {{\n", indent, at_rule.name));
        for child in &at_rule.nodes {
          stringify(child, depth + 1, css);
        }
        css.push_str(&format!("{}}}\n", indent));
      } else {
        css.push_str(&format!(
          "{}{} {} {{\n",
          indent, at_rule.name, at_rule.params
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

/// At-rules that are preserved even when empty.
const PRESERVABLE_AT_RULES: &[&str] = &[
  "@layer",
  "@charset",
  "@custom-media",
  "@namespace",
  "@import",
];

/// Optimize the AST: remove empty rules, deduplicate declarations, merge
/// consecutive at-rules with identical name + params.
/// Preserves empty at-rules that are semantically meaningful.
pub fn optimize_ast(nodes: Vec<AstNode>) -> Vec<AstNode> {
  let optimised: Vec<AstNode> = nodes
    .into_iter()
    .filter_map(|node| match node {
      AstNode::Rule(mut rule) => {
        if rule.nodes.is_empty() {
          return None;
        }
        // Deduplicate declarations (keep last occurrence wins — scan in reverse)
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
        rule.nodes = optimize_ast(deduped);
        if rule.nodes.is_empty() {
          None
        } else {
          Some(AstNode::Rule(rule))
        }
      }
      AstNode::AtRule(mut at_rule) => {
        if at_rule.nodes.is_empty() {
          if PRESERVABLE_AT_RULES.contains(&at_rule.name.as_str()) {
            return Some(AstNode::AtRule(at_rule));
          }
          return None;
        }
        at_rule.nodes = optimize_ast(at_rule.nodes);
        if at_rule.nodes.is_empty()
          && !PRESERVABLE_AT_RULES.contains(&at_rule.name.as_str())
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
    .collect();

  merge_adjacent_at_rules(optimised)
}

/// Merge consecutive `@`-rules that share the same name and params into a
/// single block. This matches Tailwind's upstream `optimize_ast` behaviour
/// where, for example, multiple `@media (min-width: 640px)` blocks emitted
/// while compiling separate candidates are coalesced into one.
fn merge_adjacent_at_rules(nodes: Vec<AstNode>) -> Vec<AstNode> {
  let mut out: Vec<AstNode> = Vec::with_capacity(nodes.len());
  for node in nodes {
    match (out.last_mut(), node) {
      (Some(AstNode::AtRule(prev)), AstNode::AtRule(curr))
        if prev.name == curr.name
          && prev.params == curr.params
          && !prev.nodes.is_empty()
          && !curr.nodes.is_empty() =>
      {
        prev.nodes.extend(curr.nodes);
        // Re-merge children since the appended nodes may now be adjacent
        // mergeable at-rules themselves.
        let merged_children = std::mem::take(&mut prev.nodes);
        prev.nodes = merge_adjacent_at_rules(merged_children);
      }
      (_, node) => out.push(node),
    }
  }
  out
}
