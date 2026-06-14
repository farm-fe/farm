//! Arbitrary-value decoder, ported from
//! `packages/tailwindcss/src/utils/decode-arbitrary-value.ts`.

use super::math_operators::add_whitespace_around_math_operators;
use crate::value_parser::{self, ValueAstNode};

/// Decode a Tailwind arbitrary value:
/// - Convert underscores to spaces (with escape handling)
/// - Special-case `url()`, `var()`, `theme()` and their `*_url`/`*_var`/`*_theme` variants
/// - Add whitespace around math operators inside math functions
pub fn decode_arbitrary_value(input: &str) -> String {
  if !input.contains('(') {
    return convert_underscores_to_whitespace(input, false);
  }

  let mut ast = value_parser::parse(input);
  recursively_decode_arbitrary_values(&mut ast);
  let css = value_parser::to_css(&ast);
  add_whitespace_around_math_operators(&css)
}

fn convert_underscores_to_whitespace(input: &str, skip_underscore_to_space: bool) -> String {
  let bytes = input.as_bytes();
  let mut out = String::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    let c = bytes[i];
    if c == b'\\' && bytes.get(i + 1).copied() == Some(b'_') {
      out.push('_');
      i += 2;
      continue;
    }
    if c == b'_' && !skip_underscore_to_space {
      out.push(' ');
    } else {
      // Push the byte; arbitrary values are typically ASCII-only at this stage,
      // but to be safe we push as a char from u32.
      out.push(c as char);
    }
    i += 1;
  }
  out
}

fn recursively_decode_arbitrary_values(ast: &mut [ValueAstNode]) {
  for node in ast.iter_mut() {
    match node {
      ValueAstNode::Function { value, nodes } => {
        // url() or *_url(): only convert in the name; leave args as-is.
        if value == "url" || value.ends_with("_url") {
          *value = convert_underscores_to_whitespace(value, false);
          continue;
        }
        if value == "var"
          || value.ends_with("_var")
          || value == "theme"
          || value.ends_with("_theme")
        {
          *value = convert_underscores_to_whitespace(value, false);
          let len = nodes.len();
          for (idx, child) in nodes.iter_mut().enumerate() {
            if idx == 0 {
              if let ValueAstNode::Word(w) = child {
                *w = convert_underscores_to_whitespace(w, true);
                continue;
              }
            }
            // Recurse on a single-element slice via temporary vec swap.
            let mut tmp = std::mem::replace(child, ValueAstNode::Word(String::new()));
            let mut wrapped = vec![std::mem::replace(
              &mut tmp,
              ValueAstNode::Word(String::new()),
            )];
            recursively_decode_arbitrary_values(&mut wrapped);
            *child = wrapped.into_iter().next().unwrap();
            let _ = idx;
            let _ = len;
          }
          continue;
        }
        *value = convert_underscores_to_whitespace(value, false);
        recursively_decode_arbitrary_values(nodes);
      }
      ValueAstNode::Word(v) | ValueAstNode::Separator(v) => {
        *v = convert_underscores_to_whitespace(v, false);
      }
    }
  }
}
