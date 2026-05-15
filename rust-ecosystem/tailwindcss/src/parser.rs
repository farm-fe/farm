use crate::ast::{AstNode, AtRule, Declaration, StyleRule};

/// Parse a CSS string into an AST.
pub fn parse(input: &str) -> Vec<AstNode> {
  Parser::new(input).parse()
}

// ── byte constants ────────────────────────────────────────────────────────────
const BACKSLASH: u8 = b'\\';
const SLASH: u8 = b'/';
const ASTERISK: u8 = b'*';
const DOUBLE_QUOTE: u8 = b'"';
const SINGLE_QUOTE: u8 = b'\'';
const SEMICOLON: u8 = b';';
const LINE_BREAK: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const OPEN_CURLY: u8 = b'{';
const CLOSE_CURLY: u8 = b'}';
const OPEN_PAREN: u8 = b'(';
const CLOSE_PAREN: u8 = b')';
const AT_SIGN: u8 = b'@';
const EXCLAMATION: u8 = b'!';

// ── internal parent enum ──────────────────────────────────────────────────────

#[derive(Debug)]
enum Parent {
  Rule(StyleRule),
  AtRule(AtRule),
}

impl Parent {
  fn push_node(&mut self, node: AstNode) {
    match self {
      Parent::Rule(r) => r.nodes.push(node),
      Parent::AtRule(a) => a.nodes.push(node),
    }
  }

  fn into_ast_node(self) -> AstNode {
    match self {
      Parent::Rule(r) => AstNode::Rule(r),
      Parent::AtRule(a) => AstNode::AtRule(a),
    }
  }
}

// ── parser ────────────────────────────────────────────────────────────────────

struct Parser<'a> {
  bytes: &'a [u8],
  input: &'a str,
  len: usize,
  i: usize,
}

impl<'a> Parser<'a> {
  fn new(input: &'a str) -> Self {
    Self {
      bytes: input.as_bytes(),
      input,
      len: input.len(),
      i: 0,
    }
  }

  fn byte(&self) -> u8 {
    self.bytes[self.i]
  }

  fn peek(&self, offset: usize) -> u8 {
    let pos = self.i + offset;
    if pos < self.len {
      self.bytes[pos]
    } else {
      0
    }
  }

  /// Skip over a CSS string literal, leaving `self.i` at the closing quote.
  fn skip_string(&mut self, quote: u8) {
    self.i += 1; // skip opening quote
    while self.i < self.len {
      let ch = self.bytes[self.i];
      if ch == BACKSLASH {
        self.i += 2; // skip escaped character
        continue;
      }
      if ch == quote {
        // leave on the closing quote; caller will advance past it
        return;
      }
      self.i += 1;
    }
  }

  /// Skip a block comment `/* ... */`, leaving `self.i` past the closing `*/`.
  fn skip_comment(&mut self) -> Option<String> {
    let start = self.i;
    self.i += 2; // skip /*
    while self.i + 1 < self.len {
      if self.bytes[self.i] == ASTERISK && self.bytes[self.i + 1] == SLASH {
        self.i += 2; // skip */
        let text = &self.input[start..self.i];
        return Some(text.to_string());
      }
      self.i += 1;
    }
    // Unterminated comment
    self.i = self.len;
    None
  }

  pub fn parse(mut self) -> Vec<AstNode> {
    let mut ast: Vec<AstNode> = Vec::new();
    let mut license_comments: Vec<AstNode> = Vec::new();
    let mut stack: Vec<(Option<AstNode>, Parent)> = Vec::new();
    let mut current_parent: Option<Parent> = None;
    let mut buffer = String::new();
    let mut paren_depth: usize = 0;

    macro_rules! push_node {
      ($node:expr) => {
        match current_parent.as_mut() {
          Some(p) => p.push_node($node),
          None => ast.push($node),
        }
      };
    }

    while self.i < self.len {
      let ch = self.byte();

      // CRLF: skip CR
      if ch == CARRIAGE_RETURN {
        if self.peek(1) == LINE_BREAK {
          self.i += 1;
          continue;
        }
      }

      // Backslash escape
      if ch == BACKSLASH && self.i + 1 < self.len {
        buffer.push(self.byte() as char);
        self.i += 1;
        buffer.push(self.byte() as char);
        self.i += 1;
        continue;
      }

      // Block comment
      if ch == SLASH && self.peek(1) == ASTERISK {
        if let Some(text) = self.skip_comment() {
          // license comment (/*! ... */)
          if text.len() > 3 && text.as_bytes()[2] == EXCLAMATION {
            let inner = &text[3..text.len() - 2];
            license_comments.push(AstNode::Comment(inner.to_string()));
          }
          // regular comments are discarded (not added to AST)
        }
        continue;
      }

      // String literal
      if ch == SINGLE_QUOTE || ch == DOUBLE_QUOTE {
        let start = self.i;
        self.skip_string(ch);
        let end = self.i + 1; // include closing quote
        let s = &self.input[start..end.min(self.len)];
        buffer.push_str(s);
        self.i = end.min(self.len);
        continue;
      }

      // Track paren depth
      if ch == OPEN_PAREN {
        paren_depth += 1;
        buffer.push('(');
        self.i += 1;
        continue;
      }
      if ch == CLOSE_PAREN {
        if paren_depth > 0 {
          paren_depth -= 1;
        }
        buffer.push(')');
        self.i += 1;
        continue;
      }

      // When inside parens, treat { } ; as regular characters
      let in_paren = paren_depth > 0;

      // Collapse whitespace
      if !in_paren && (ch == SPACE || ch == TAB || ch == LINE_BREAK) {
        let next = self.peek(1);
        if next == SPACE
          || next == TAB
          || next == LINE_BREAK
          || (next == CARRIAGE_RETURN && self.peek(2) == LINE_BREAK)
        {
          self.i += 1;
          continue;
        }
        // Convert newline to space
        if ch == LINE_BREAK {
          if !buffer.is_empty() {
            let last = *buffer.as_bytes().last().unwrap();
            if last != SPACE && last != TAB {
              buffer.push(' ');
            }
          }
          self.i += 1;
          continue;
        }
        // Skip leading whitespace in buffer
        if buffer.is_empty() {
          self.i += 1;
          continue;
        }
        buffer.push(ch as char);
        self.i += 1;
        continue;
      }

      // Body-less at-rule ending with `;`
      if !in_paren && ch == SEMICOLON && !buffer.is_empty() && buffer.as_bytes()[0] == AT_SIGN {
        let node = self.parse_at_rule(&buffer);
        push_node!(node);
        buffer.clear();
        self.i += 1;
        continue;
      }

      // End of declaration
      if !in_paren && ch == SEMICOLON {
        if !buffer.is_empty() {
          if let Some(d) = self.parse_declaration(&buffer) {
            push_node!(AstNode::Declaration(d));
          }
          buffer.clear();
        }
        self.i += 1;
        continue;
      }

      // Start of block
      if !in_paren && ch == OPEN_CURLY {
        let trimmed = buffer.trim().to_string();
        buffer.clear();

        let new_parent = if !trimmed.is_empty() && trimmed.as_bytes()[0] == AT_SIGN {
          Parent::AtRule(self.parse_at_rule_raw(&trimmed))
        } else {
          Parent::Rule(StyleRule {
            selector: trimmed,
            nodes: vec![],
          })
        };

        // Save current context
        let saved_ast_node: Option<AstNode> = None;
        let old_parent = current_parent.take();
        if let Some(p) = old_parent {
          stack.push((saved_ast_node, p));
        }

        current_parent = Some(new_parent);
        self.i += 1;
        continue;
      }

      // End of block
      if !in_paren && ch == CLOSE_CURLY {
        // Flush any trailing declaration without semicolon
        if !buffer.is_empty() {
          let trimmed = buffer.trim().to_string();
          buffer.clear();
          if !trimmed.is_empty() {
            if trimmed.as_bytes()[0] == AT_SIGN {
              let node = self.parse_at_rule(&trimmed);
              if let Some(ref mut p) = current_parent {
                p.push_node(node);
              }
            } else if let Some(d) = self.parse_declaration(&trimmed) {
              if let Some(ref mut p) = current_parent {
                p.push_node(AstNode::Declaration(d));
              }
            }
          }
        }

        // Pop the finished node
        let finished = current_parent.take();
        if let Some(f) = finished {
          let finished_node = f.into_ast_node();
          if let Some((_saved, mut outer)) = stack.pop() {
            outer.push_node(finished_node);
            current_parent = Some(outer);
          } else {
            ast.push(finished_node);
          }
        }

        self.i += 1;
        continue;
      }

      // Skip leading whitespace
      if buffer.is_empty() && (ch == SPACE || ch == TAB) {
        self.i += 1;
        continue;
      }

      // Collect character
      // Handle multi-byte UTF-8 gracefully
      let ch_start = self.i;
      let ch_char = self.input[self.i..].chars().next().unwrap_or('\0');
      let ch_len = ch_char.len_utf8();
      buffer.push_str(&self.input[ch_start..ch_start + ch_len]);
      self.i += ch_len;
    }

    // Handle leftover at-rule at end of file (no semicolon)
    if !buffer.is_empty() && buffer.as_bytes()[0] == AT_SIGN {
      let node = self.parse_at_rule(&buffer);
      match current_parent.as_mut() {
        Some(p) => p.push_node(node),
        None => ast.push(node),
      }
    }

    // Prepend license comments
    if !license_comments.is_empty() {
      let mut result = license_comments;
      result.extend(ast);
      return result;
    }

    ast
  }

  fn parse_at_rule(&self, buffer: &str) -> AstNode {
    AstNode::AtRule(self.parse_at_rule_raw(buffer))
  }

  fn parse_at_rule_raw(&self, buffer: &str) -> AtRule {
    let trimmed = buffer.trim();

    // Find end of at-rule name (first whitespace or '(' after @word)
    let mut name_end = trimmed.len();
    let mut found_space = false;
    for (i, ch) in trimmed.char_indices() {
      if i == 0 {
        // '@' itself
        continue;
      }
      if ch == ' ' || ch == '\t' || ch == '(' {
        name_end = i;
        found_space = true;
        break;
      }
    }

    let name = trimmed[..name_end].to_string();
    let params = if found_space {
      trimmed[name_end..].trim().to_string()
    } else {
      String::new()
    };

    AtRule {
      name,
      params,
      nodes: vec![],
    }
  }

  fn parse_declaration(&self, buffer: &str) -> Option<Declaration> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
      return None;
    }
    let colon_idx = trimmed.find(':')?;
    let property = trimmed[..colon_idx].trim().to_string();
    if property.is_empty() {
      return None;
    }
    let rest = trimmed[colon_idx + 1..].trim();
    let (value, important) = if let Some(pos) = rest.find("!important") {
      let v = rest[..pos].trim().to_string();
      (v, true)
    } else {
      (rest.to_string(), false)
    };
    Some(Declaration {
      property,
      value: if value.is_empty() { None } else { Some(value) },
      important,
    })
  }
}
