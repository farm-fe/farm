use crate::ast::AstNode;

/// Action returned from a walk visitor.
pub enum WalkAction {
  /// Continue normal traversal (recurse into children).
  Continue,
  /// Keep this node but skip its children.
  Skip,
  /// Stop traversal entirely.
  Stop,
  /// Replace this node with new node(s) and walk them.
  Replace(Vec<AstNode>),
  /// Replace this node with new node(s) without walking them.
  ReplaceSkip(Vec<AstNode>),
}

/// Depth-first walk of an AST, returning the potentially modified nodes.
///
/// The visitor receives `(&AstNode, path: &[&AstNode], depth: usize)` and
/// returns a [`WalkAction`].
pub fn walk<F>(nodes: Vec<AstNode>, visitor: &mut F) -> Vec<AstNode>
where
  F: FnMut(&AstNode, &[&AstNode], usize) -> WalkAction,
{
  let mut stopped = false;
  walk_nodes(nodes, 0, &mut stopped, visitor)
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Extract children from a container node, returning the shell and the children.
fn take_children(node: AstNode) -> (AstNode, Vec<AstNode>) {
  match node {
    AstNode::Rule(mut rule) => {
      let children = std::mem::take(&mut rule.nodes);
      (AstNode::Rule(rule), children)
    }
    AstNode::AtRule(mut at_rule) => {
      let children = std::mem::take(&mut at_rule.nodes);
      (AstNode::AtRule(at_rule), children)
    }
    AstNode::Context(mut ctx) => {
      let children = std::mem::take(&mut ctx.nodes);
      (AstNode::Context(ctx), children)
    }
    AstNode::AtRoot(mut at_root) => {
      let children = std::mem::take(&mut at_root.nodes);
      (AstNode::AtRoot(at_root), children)
    }
    other => (other, vec![]),
  }
}

/// Put children back into a container node shell.
fn put_children(node: AstNode, children: Vec<AstNode>) -> AstNode {
  match node {
    AstNode::Rule(mut rule) => {
      rule.nodes = children;
      AstNode::Rule(rule)
    }
    AstNode::AtRule(mut at_rule) => {
      at_rule.nodes = children;
      AstNode::AtRule(at_rule)
    }
    AstNode::Context(mut ctx) => {
      ctx.nodes = children;
      AstNode::Context(ctx)
    }
    AstNode::AtRoot(mut at_root) => {
      at_root.nodes = children;
      AstNode::AtRoot(at_root)
    }
    other => other,
  }
}

fn walk_nodes<F>(
  nodes: Vec<AstNode>,
  depth: usize,
  stopped: &mut bool,
  visitor: &mut F,
) -> Vec<AstNode>
where
  F: FnMut(&AstNode, &[&AstNode], usize) -> WalkAction,
{
  let mut result = Vec::new();

  for node in nodes {
    if *stopped {
      result.push(node);
      continue;
    }

    let action = visitor(&node, &[], depth);

    match action {
      WalkAction::Continue => {
        let (shell, children) = take_children(node);
        if children.is_empty() {
          result.push(shell);
        } else {
          let new_children = walk_nodes(children, depth + 1, stopped, visitor);
          result.push(put_children(shell, new_children));
        }
      }
      WalkAction::Skip => {
        result.push(node);
      }
      WalkAction::Stop => {
        result.push(node);
        *stopped = true;
      }
      WalkAction::Replace(new_nodes) => {
        let walked = walk_nodes(new_nodes, depth, stopped, visitor);
        result.extend(walked);
      }
      WalkAction::ReplaceSkip(new_nodes) => {
        result.extend(new_nodes);
      }
    }
  }

  result
}
