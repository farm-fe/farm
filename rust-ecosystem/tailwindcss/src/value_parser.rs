//! Value parser, ported from `packages/tailwindcss/src/value-parser.ts`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueAstNode {
  Word(String),
  Separator(String),
  Function {
    value: String,
    nodes: Vec<ValueAstNode>,
  },
}

impl ValueAstNode {
  pub fn word(value: impl Into<String>) -> Self {
    ValueAstNode::Word(value.into())
  }
  pub fn separator(value: impl Into<String>) -> Self {
    ValueAstNode::Separator(value.into())
  }
  pub fn function(value: impl Into<String>, nodes: Vec<ValueAstNode>) -> Self {
    ValueAstNode::Function {
      value: value.into(),
      nodes,
    }
  }
}

/// Convert a value AST back to its CSS source form.
pub fn to_css(ast: &[ValueAstNode]) -> String {
  let mut out = String::new();
  append_css(ast, &mut out);
  out
}

fn append_css(ast: &[ValueAstNode], out: &mut String) {
  for node in ast {
    match node {
      ValueAstNode::Word(v) | ValueAstNode::Separator(v) => out.push_str(v),
      ValueAstNode::Function { value, nodes } => {
        out.push_str(value);
        out.push('(');
        append_css(nodes, out);
        out.push(')');
      }
    }
  }
}

const BACKSLASH: u8 = 0x5c;
const CLOSE_PAREN: u8 = 0x29;
const COLON: u8 = 0x3a;
const COMMA: u8 = 0x2c;
const DOUBLE_QUOTE: u8 = 0x22;
const EQUALS: u8 = 0x3d;
const GREATER_THAN: u8 = 0x3e;
const LESS_THAN: u8 = 0x3c;
const NEWLINE: u8 = 0x0a;
const OPEN_PAREN: u8 = 0x28;
const SINGLE_QUOTE: u8 = 0x27;
const SLASH: u8 = 0x2f;
const SPACE: u8 = 0x20;
const TAB: u8 = 0x09;

#[inline]
fn is_separator_byte(b: u8) -> bool {
  matches!(
    b,
    COLON | COMMA | EQUALS | GREATER_THAN | LESS_THAN | NEWLINE | SPACE | TAB
  )
}

/// Parse a CSS value string into an AST.
pub fn parse(input: &str) -> Vec<ValueAstNode> {
  let input = input.replace("\r\n", "\n");
  let bytes = input.as_bytes();
  let len = bytes.len();

  // We carry the ast as a Vec, with a stack of "parent function indices".
  // To keep ownership simple, we use a stack of owned child Vecs and at
  // function-close we move the children into the parent.
  let mut stack: Vec<(String, Vec<ValueAstNode>)> = Vec::new();
  let mut ast: Vec<ValueAstNode> = Vec::new();
  let mut buffer = String::new();

  let mut i = 0usize;
  while i < len {
    let c = bytes[i];
    match c {
      BACKSLASH => {
        // Consume the next character literally.
        buffer.push(bytes[i] as char);
        if i + 1 < len {
          buffer.push(bytes[i + 1] as char);
          i += 2;
        } else {
          i += 1;
        }
        continue;
      }
      SLASH => {
        // Flush buffer, push the `/` as its own word.
        if !buffer.is_empty() {
          push_node(
            &mut stack,
            &mut ast,
            ValueAstNode::word(std::mem::take(&mut buffer)),
          );
        }
        push_node(&mut stack, &mut ast, ValueAstNode::word("/"));
        i += 1;
        continue;
      }
      c if is_separator_byte(c) => {
        if !buffer.is_empty() {
          push_node(
            &mut stack,
            &mut ast,
            ValueAstNode::word(std::mem::take(&mut buffer)),
          );
        }
        let start = i;
        let mut end = i + 1;
        while end < len && is_separator_byte(bytes[end]) {
          end += 1;
        }
        let sep = std::str::from_utf8(&bytes[start..end]).unwrap().to_string();
        push_node(&mut stack, &mut ast, ValueAstNode::separator(sep));
        i = end;
        continue;
      }
      SINGLE_QUOTE | DOUBLE_QUOTE => {
        let quote = c;
        let start = i;
        let mut j = i + 1;
        while j < len {
          let p = bytes[j];
          if p == BACKSLASH {
            j += 2;
            continue;
          }
          if p == quote {
            break;
          }
          j += 1;
        }
        // Include the closing quote (or run to end if missing).
        let end = (j + 1).min(len);
        buffer.push_str(std::str::from_utf8(&bytes[start..end]).unwrap());
        i = end;
        continue;
      }
      OPEN_PAREN => {
        let name = std::mem::take(&mut buffer);
        stack.push((name, Vec::new()));
        i += 1;
        continue;
      }
      CLOSE_PAREN => {
        if let Some((name, mut nodes)) = stack.pop() {
          if !buffer.is_empty() {
            nodes.push(ValueAstNode::word(std::mem::take(&mut buffer)));
          }
          let fun = ValueAstNode::function(name, nodes);
          push_node(&mut stack, &mut ast, fun);
        } else {
          // Extra ')' — discard buffer flush per upstream behaviour (it does
          // not flush in this branch either).
        }
        i += 1;
        continue;
      }
      other => {
        buffer.push(other as char);
        i += 1;
      }
    }
  }

  if !buffer.is_empty() {
    ast.push(ValueAstNode::word(buffer));
  }

  ast
}

fn push_node(
  stack: &mut [(String, Vec<ValueAstNode>)],
  ast: &mut Vec<ValueAstNode>,
  node: ValueAstNode,
) {
  if let Some(top) = stack.last_mut() {
    top.1.push(node);
  } else {
    ast.push(node);
  }
}
