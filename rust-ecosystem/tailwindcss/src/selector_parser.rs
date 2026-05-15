//! Selector parser, ported from
//! `packages/tailwindcss/src/selector-parser.ts`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorAstNode {
  Combinator(String),
  Function { value: String, nodes: Vec<SelectorAstNode> },
  Selector(String),
  Separator(String),
  Value(String),
}

impl SelectorAstNode {
  pub fn combinator(v: impl Into<String>) -> Self {
    Self::Combinator(v.into())
  }
  pub fn function(v: impl Into<String>, nodes: Vec<SelectorAstNode>) -> Self {
    Self::Function { value: v.into(), nodes }
  }
  pub fn selector(v: impl Into<String>) -> Self {
    Self::Selector(v.into())
  }
  pub fn separator(v: impl Into<String>) -> Self {
    Self::Separator(v.into())
  }
  pub fn value(v: impl Into<String>) -> Self {
    Self::Value(v.into())
  }
}

/// Render an AST back to its CSS source form.
pub fn to_css(ast: &[SelectorAstNode]) -> String {
  let mut out = String::new();
  append_css(ast, &mut out);
  out
}

fn append_css(ast: &[SelectorAstNode], out: &mut String) {
  for node in ast {
    match node {
      SelectorAstNode::Combinator(v)
      | SelectorAstNode::Selector(v)
      | SelectorAstNode::Separator(v)
      | SelectorAstNode::Value(v) => out.push_str(v),
      SelectorAstNode::Function { value, nodes } => {
        out.push_str(value);
        out.push('(');
        append_css(nodes, out);
        out.push(')');
      }
    }
  }
}

const BACKSLASH: u8 = 0x5c;
const CLOSE_BRACKET: u8 = 0x5d;
const CLOSE_PAREN: u8 = 0x29;
const COLON: u8 = 0x3a;
const COMMA: u8 = 0x2c;
const DOUBLE_QUOTE: u8 = 0x22;
const FULL_STOP: u8 = 0x2e;
const GREATER_THAN: u8 = 0x3e;
const NEWLINE: u8 = 0x0a;
const NUMBER_SIGN: u8 = 0x23;
const OPEN_BRACKET: u8 = 0x5b;
const OPEN_PAREN: u8 = 0x28;
const PLUS: u8 = 0x2b;
const SINGLE_QUOTE: u8 = 0x27;
const SPACE: u8 = 0x20;
const TAB: u8 = 0x09;
const TILDE: u8 = 0x7e;
const AMPERSAND: u8 = 0x26;
const ASTERISK: u8 = 0x2a;

#[inline]
fn is_combinator_byte(b: u8) -> bool {
  matches!(b, COMMA | GREATER_THAN | NEWLINE | SPACE | PLUS | TAB | TILDE)
}

/// Parse a CSS selector list into an AST.
pub fn parse(input: &str) -> Vec<SelectorAstNode> {
  let input = input.replace("\r\n", "\n");
  let bytes = input.as_bytes();
  let len = bytes.len();

  let mut ast: Vec<SelectorAstNode> = Vec::new();
  // Stack of (parent_function_value, accumulated_children).
  let mut stack: Vec<(String, Vec<SelectorAstNode>)> = Vec::new();
  let mut buffer = String::new();

  let mut i = 0usize;
  while i < len {
    let c = bytes[i];
    match c {
      _ if is_combinator_byte(c) => {
        if !buffer.is_empty() {
          let node = SelectorAstNode::selector(std::mem::take(&mut buffer));
          push_into(&mut stack, &mut ast, node);
        }
        // Greedy expand the combinator run.
        let start = i;
        let mut end = i + 1;
        while end < len && is_combinator_byte(bytes[end]) {
          end += 1;
        }
        i = end - 1;
        let contents = std::str::from_utf8(&bytes[start..end]).unwrap().to_string();
        let node = if contents.trim() == "," {
          SelectorAstNode::separator(contents)
        } else {
          SelectorAstNode::combinator(contents)
        };
        push_into(&mut stack, &mut ast, node);
      }
      OPEN_PAREN => {
        let name = std::mem::take(&mut buffer);
        let recurses =
          matches!(name.as_str(), ":not" | ":where" | ":has" | ":is");
        if !recurses {
          // Capture everything up to matching ')' as a single Value child.
          let start = i + 1;
          let mut nesting = 0i32;
          let mut j = i + 1;
          while j < len {
            let p = bytes[j];
            if p == OPEN_PAREN {
              nesting += 1;
            } else if p == CLOSE_PAREN {
              if nesting == 0 {
                i = j;
                break;
              }
              nesting -= 1;
            }
            j += 1;
          }
          let value_str = std::str::from_utf8(&bytes[start..i]).unwrap().to_string();
          let node = SelectorAstNode::function(
            name,
            vec![SelectorAstNode::value(value_str)],
          );
          push_into(&mut stack, &mut ast, node);
        } else {
          // Push function onto stack to receive children.
          stack.push((name, Vec::new()));
        }
      }
      CLOSE_PAREN => {
        if let Some((name, mut nodes)) = stack.pop() {
          if !buffer.is_empty() {
            nodes.push(SelectorAstNode::selector(std::mem::take(&mut buffer)));
          }
          let node = SelectorAstNode::function(name, nodes);
          push_into(&mut stack, &mut ast, node);
        }
      }
      FULL_STOP | COLON | NUMBER_SIGN => {
        if !buffer.is_empty() {
          let node = SelectorAstNode::selector(std::mem::take(&mut buffer));
          push_into(&mut stack, &mut ast, node);
        }
        buffer.push(c as char);
      }
      OPEN_BRACKET => {
        if !buffer.is_empty() {
          let node = SelectorAstNode::selector(std::mem::take(&mut buffer));
          push_into(&mut stack, &mut ast, node);
        }
        let start = i;
        let mut nesting = 0i32;
        let mut j = i + 1;
        while j < len {
          let p = bytes[j];
          if p == OPEN_BRACKET {
            nesting += 1;
          } else if p == CLOSE_BRACKET {
            if nesting == 0 {
              i = j;
              break;
            }
            nesting -= 1;
          }
          j += 1;
        }
        buffer.push_str(std::str::from_utf8(&bytes[start..=i]).unwrap());
      }
      SINGLE_QUOTE | DOUBLE_QUOTE => {
        let start = i;
        let mut j = i + 1;
        while j < len {
          let p = bytes[j];
          if p == BACKSLASH {
            j += 2;
            continue;
          }
          if p == c {
            i = j;
            break;
          }
          j += 1;
        }
        buffer.push_str(std::str::from_utf8(&bytes[start..=i]).unwrap());
      }
      AMPERSAND | ASTERISK => {
        if !buffer.is_empty() {
          let node = SelectorAstNode::selector(std::mem::take(&mut buffer));
          push_into(&mut stack, &mut ast, node);
        }
        let node = SelectorAstNode::selector((c as char).to_string());
        push_into(&mut stack, &mut ast, node);
      }
      BACKSLASH => {
        buffer.push(c as char);
        if i + 1 < len {
          buffer.push(bytes[i + 1] as char);
          i += 1;
        }
      }
      _ => {
        buffer.push(c as char);
      }
    }
    i += 1;
  }

  if !buffer.is_empty() {
    ast.push(SelectorAstNode::selector(buffer));
  }

  ast
}

fn push_into(
  stack: &mut [(String, Vec<SelectorAstNode>)],
  ast: &mut Vec<SelectorAstNode>,
  node: SelectorAstNode,
) {
  if let Some(top) = stack.last_mut() {
    top.1.push(node);
  } else {
    ast.push(node);
  }
}
