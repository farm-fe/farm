//! Brace-expansion ported from
//! `packages/tailwindcss/src/utils/brace-expansion.ts`.

use super::segment::segment;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BraceExpansionError {
  #[error("The pattern `{pattern}` is not balanced.")]
  Unbalanced { pattern: String },
  #[error("Step cannot be zero in sequence expansion.")]
  ZeroStep,
}

/// Expand a brace pattern into all concrete strings.
///
/// Supports comma alternation (`a/{x,y,z}/b`), numeric ranges
/// (`a/{0..5}/b`, with optional step `..2`), and nested braces.
pub fn expand(pattern: &str) -> Result<Vec<String>, BraceExpansionError> {
  expand_inner(pattern)
}

fn expand_inner(pattern: &str) -> Result<Vec<String>, BraceExpansionError> {
  let Some(open_idx) = pattern.find('{') else {
    return Ok(vec![pattern.to_string()]);
  };

  let pre = &pattern[..open_idx];
  let rest = &pattern[open_idx..];
  let rest_bytes = rest.as_bytes();

  // Find the matching closing brace.
  let mut end_index: Option<usize> = rest.rfind('}');
  let mut depth: i32 = 0;
  for (i, &b) in rest_bytes.iter().enumerate() {
    if b == b'{' {
      depth += 1;
    } else if b == b'}' {
      depth -= 1;
      if depth == 0 {
        end_index = Some(i);
        break;
      }
    }
  }

  let Some(end_index) = end_index else {
    return Err(BraceExpansionError::Unbalanced {
      pattern: pattern.to_string(),
    });
  };

  if depth != 0 {
    // Mirrors upstream behavior: rfind('}') may have returned a brace at
    // a non-matching position, but if the loop never reached depth==0 we
    // still treat as unbalanced.
    return Err(BraceExpansionError::Unbalanced {
      pattern: pattern.to_string(),
    });
  }

  let inside = &rest[1..end_index];
  let post = &rest[end_index + 1..];

  let mut parts: Vec<String> = if is_sequence(inside) {
    expand_sequence(inside)?
  } else {
    segment(inside, ',')
  };

  // Recurse on each part (parts may themselves contain braces).
  let mut expanded_parts: Vec<String> = Vec::new();
  for p in parts.drain(..) {
    expanded_parts.extend(expand_inner(&p)?);
  }

  let expanded_tail = expand_inner(post)?;

  let mut result = Vec::new();
  for tail in &expanded_tail {
    for part in &expanded_parts {
      result.push(format!("{}{}{}", pre, part, tail));
    }
  }
  Ok(result)
}

fn is_sequence(s: &str) -> bool {
  parse_sequence(s).is_some()
}

/// Parse `start..end[..step]` where each is an optionally-signed integer.
fn parse_sequence(s: &str) -> Option<(i64, i64, Option<i64>)> {
  let mut iter = s.split("..");
  let start = iter.next()?;
  let end = iter.next()?;
  let step = iter.next();
  if iter.next().is_some() {
    return None;
  }
  let start_n = parse_signed_int(start)?;
  let end_n = parse_signed_int(end)?;
  let step_n = match step {
    Some(s) => Some(parse_signed_int(s)?),
    None => None,
  };
  Some((start_n, end_n, step_n))
}

fn parse_signed_int(s: &str) -> Option<i64> {
  if s.is_empty() {
    return None;
  }
  let bytes = s.as_bytes();
  let start = if bytes[0] == b'-' { 1 } else { 0 };
  if start >= bytes.len() {
    return None;
  }
  for &b in &bytes[start..] {
    if !b.is_ascii_digit() {
      return None;
    }
  }
  s.parse::<i64>().ok()
}

fn expand_sequence(seq: &str) -> Result<Vec<String>, BraceExpansionError> {
  let Some((start, end, step_opt)) = parse_sequence(seq) else {
    return Ok(vec![seq.to_string()]);
  };

  let mut step = step_opt.unwrap_or(if start <= end { 1 } else { -1 });
  if step == 0 {
    return Err(BraceExpansionError::ZeroStep);
  }

  let increasing = start < end;
  // Force the step's sign to match the direction of iteration.
  if (increasing && step < 0) || (!increasing && step > 0) {
    step = -step;
  }

  let mut result = Vec::new();
  let mut i = start;
  loop {
    if increasing {
      if i > end {
        break;
      }
    } else if i < end {
      break;
    }
    result.push(i.to_string());
    if !increasing && i == end {
      break;
    }
    if increasing && i == end {
      break;
    }
    i += step;
  }
  Ok(result)
}
