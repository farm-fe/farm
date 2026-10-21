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
  let mut origins = Vec::new();
  for node in nodes {
    stringify(node, 0, &mut css, &mut origins, None);
  }
  css
}

/// Serialize the same AST and its retained parser origins in a single walk.
pub fn to_css_with_source_map(
  nodes: &[AstNode],
  source: &str,
  input_map: Option<&sourcemap::SourceMap>,
) -> (String, String) {
  let mut css = String::new();
  let mut origins = Vec::new();
  for node in nodes {
    stringify(node, 0, &mut css, &mut origins, None);
  }
  let mut builder = sourcemap::SourceMapBuilder::new(None);
  if input_map.is_none() {
    let source_id = builder.add_source("input.css");
    builder.set_source_contents(source_id, Some(source));
  }
  let generated_lines = LineIndex::new(&css);
  let source_lines = LineIndex::new(source);
  for (generated, original) in origins {
    let (line, column) = generated_lines.coordinates(generated);
    if let Some(original) = original {
      let (source_line, source_column) = source_lines.coordinates(original);
      if let Some(input_map) = input_map {
        if let Some(token) = input_map
          .lookup_token(source_line, source_column)
          .filter(|token| token.has_source())
        {
          let added = builder.add(
            line,
            column,
            token.get_src_line(),
            token.get_src_col(),
            token.get_source(),
            None,
            false,
          );
          builder.set_source_contents(
            added.src_id,
            input_map.get_source_contents(token.get_src_id()),
          );
        } else {
          builder.add(line, column, 0, 0, None, None, false);
        }
        continue;
      }
      builder.add(
        line,
        column,
        source_line,
        source_column,
        Some("input.css"),
        None,
        false,
      );
    } else {
      builder.add(line, column, 0, 0, None, None, false);
    }
  }
  let mut bytes = Vec::new();
  // SourceMap's writer only fails on I/O; writing to Vec cannot fail.
  builder
    .into_sourcemap()
    .to_writer(&mut bytes)
    .expect("writing map to memory");
  (
    css,
    String::from_utf8(bytes).expect("source map JSON is UTF-8"),
  )
}

struct LineIndex<'a> {
  text: &'a str,
  starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
  fn new(text: &'a str) -> Self {
    let starts = std::iter::once(0)
      .chain(
        text
          .bytes()
          .enumerate()
          .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
      )
      .collect();
    Self { text, starts }
  }

  fn coordinates(&self, offset: usize) -> (u32, u32) {
    let line = self.starts.partition_point(|start| *start <= offset) - 1;
    let column = self.text[self.starts[line]..offset].encode_utf16().count();
    (line as u32, column as u32)
  }
}

fn stringify(
  node: &AstNode,
  depth: usize,
  css: &mut String,
  origins: &mut Vec<(usize, Option<usize>)>,
  inherited: Option<usize>,
) {
  let indent = "  ".repeat(depth);
  if !matches!(node, AstNode::Context(_) | AstNode::AtRoot(_)) {
    origins.push((css.len() + indent.len(), inherited));
  }

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
        stringify(child, depth + 1, css, origins, inherited);
      }
      css.push_str(&format!("{}}}\n", indent));
    }
    AstNode::AtRule(at_rule) => {
      if at_rule.nodes.is_empty() {
        css.push_str(&format!("{}{} {};\n", indent, at_rule.name, at_rule.params));
      } else if at_rule.params.is_empty() {
        css.push_str(&format!("{}{} {{\n", indent, at_rule.name));
        for child in &at_rule.nodes {
          stringify(child, depth + 1, css, origins, inherited);
        }
        css.push_str(&format!("{}}}\n", indent));
      } else {
        css.push_str(&format!(
          "{}{} {} {{\n",
          indent, at_rule.name, at_rule.params
        ));
        for child in &at_rule.nodes {
          stringify(child, depth + 1, css, origins, inherited);
        }
        css.push_str(&format!("{}}}\n", indent));
      }
    }
    AstNode::Comment(value) => {
      css.push_str(&format!("{}/*{}*/\n", indent, value));
    }
    AstNode::Context(ctx) => {
      let origin = ctx
        .context
        .get("farm:source-offset")
        .and_then(|value| value.parse().ok())
        .or(inherited);
      for child in &ctx.nodes {
        stringify(child, depth, css, origins, origin);
      }
    }
    AstNode::AtRoot(at_root) => {
      for child in &at_root.nodes {
        stringify(child, depth, css, origins, inherited);
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
          if let AstNode::Declaration(d) = origin_node(&child) {
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
        if at_rule.nodes.is_empty() && !PRESERVABLE_AT_RULES.contains(&at_rule.name.as_str()) {
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
  for mut node in nodes {
    let mut merged = false;
    if let (Some(prev), Some(curr)) = (
      out.last_mut().and_then(origin_at_rule_mut),
      origin_at_rule_mut(&mut node),
    ) {
      if prev.name == curr.name
        && prev.params == curr.params
        && !prev.nodes.is_empty()
        && !curr.nodes.is_empty()
      {
        prev.nodes.append(&mut curr.nodes);
        // Re-merge children since the appended nodes may now be adjacent
        // mergeable at-rules themselves.
        let merged_children = std::mem::take(&mut prev.nodes);
        prev.nodes = merge_adjacent_at_rules(merged_children);
        merged = true;
      }
    }
    if !merged {
      out.push(node);
    }
  }
  out
}

fn origin_node(node: &AstNode) -> &AstNode {
  match node {
    AstNode::Context(ctx)
      if ctx.context.contains_key("farm:source-offset") && ctx.nodes.len() == 1 =>
    {
      origin_node(&ctx.nodes[0])
    }
    other => other,
  }
}

fn origin_at_rule_mut(node: &mut AstNode) -> Option<&mut AtRule> {
  match node {
    AstNode::AtRule(at) => Some(at),
    AstNode::Context(ctx)
      if ctx.context.contains_key("farm:source-offset") && ctx.nodes.len() == 1 =>
    {
      origin_at_rule_mut(&mut ctx.nodes[0])
    }
    _ => None,
  }
}
