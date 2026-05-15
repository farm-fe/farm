//! Variant parsing and application.
//!
//! A "variant" is a prefix on a Tailwind candidate that modifies how the
//! generated CSS is scoped — e.g. `hover:`, `sm:`, `dark:`, `data-[open]:`,
//! `group-hover:`, `[&_p]:`, `@container`, `@sm:`, `supports-[display:grid]:`.
//!
//! This module mirrors the surface of upstream `tailwindlabs/tailwindcss`'s
//! `src/variants.ts`. Given a list of variant strings (in source order — i.e.
//! left-most first), `apply_variants` walks them in reverse to compose
//! selector transforms and at-rule wrappers, producing the final CSS AST.

use std::collections::HashSet;

use crate::ast::{AstNode, AtRule, StyleRule};

// ── public registry kept for back-compat (recognition only) ─────────────────

/// Lightweight registry that answers "is this a known variant name?".
///
/// Used by the parser/diagnostics; the actual variant-application logic lives
/// in [`apply_variants`] and friends and does **not** consult this registry.
pub struct VariantRegistry {
  variants: HashSet<&'static str>,
  /// User-defined custom variants registered from `@custom-variant` rules in
  /// the source CSS. Maps the variant name to a selector template containing
  /// `&` (e.g. `&:hover`, `&[data-state="open"]`) or an `@<at-rule>(<params>)`
  /// form (e.g. `@media (pointer: coarse)`) for at-rule wrappers.
  user_variants: std::collections::HashMap<String, String>,
}

impl Default for VariantRegistry {
  fn default() -> Self {
    Self::builtin()
  }
}

impl VariantRegistry {
  /// Create the built-in variant registry.
  pub fn builtin() -> Self {
    let variants: HashSet<&'static str> = [
      // Pseudo-classes
      "hover",
      "focus",
      "active",
      "disabled",
      "enabled",
      "visited",
      "target",
      "first",
      "last",
      "only",
      "odd",
      "even",
      "first-of-type",
      "last-of-type",
      "only-of-type",
      "empty",
      "required",
      "valid",
      "invalid",
      "checked",
      "indeterminate",
      "default",
      "optional",
      "in-range",
      "out-of-range",
      "read-only",
      "read-write",
      "placeholder-shown",
      "autofill",
      "focus-within",
      "focus-visible",
      "open",
      "inert",
      // Pseudo-elements
      "before",
      "after",
      "first-letter",
      "first-line",
      "marker",
      "selection",
      "file",
      "placeholder",
      "backdrop",
      // Directional
      "ltr",
      "rtl",
      // Media / responsive
      "dark",
      "print",
      "motion-safe",
      "motion-reduce",
      "portrait",
      "landscape",
      "contrast-more",
      "contrast-less",
      "starting",
      // Breakpoints (Tailwind v4 defaults)
      "sm",
      "md",
      "lg",
      "xl",
      "2xl",
      // Max-width counterparts
      "max-sm",
      "max-md",
      "max-lg",
      "max-xl",
      "max-2xl",
      // Container queries
      "@container",
      "@sm",
      "@md",
      "@lg",
      "@xl",
      "@2xl",
      "@3xl",
      "@4xl",
      "@5xl",
      "@6xl",
      "@7xl",
      // Group / peer
      "group",
      "peer",
    ]
    .into_iter()
    .collect();

    Self {
      variants,
      user_variants: std::collections::HashMap::new(),
    }
  }

  /// Register a user-defined custom variant (typically from a
  /// `@custom-variant name (selector);` rule in the source CSS).
  ///
  /// The `body` is either a selector template containing `&` (e.g.
  /// `&:hover`) which replaces the current selector, or an at-rule form
  /// `@media (...)` / `@supports (...)` which wraps the rule.
  pub fn register_custom_variant(&mut self, name: String, body: String) {
    self.user_variants.insert(name, body);
  }

  /// Look up a registered custom variant by name.
  pub fn custom_variant(&self, name: &str) -> Option<&str> {
    self.user_variants.get(name).map(|s| s.as_str())
  }

  /// Returns `true` if the variant name is registered as a built-in.
  ///
  /// Functional variants (e.g. `data-[…]`, `not-…`, `group-…`, `[&_p]`,
  /// `min-[…]`, `@min-[…]`) are recognised by [`apply_variants`] structurally,
  /// not by name, so they return `false` here unless they happen to be a
  /// registered alias.
  pub fn has(&self, name: &str) -> bool {
    self.variants.contains(name) || self.user_variants.contains_key(name)
  }
}

// ── variant application ─────────────────────────────────────────────────────

/// One step of variant application.
#[derive(Debug, Clone, PartialEq)]
enum VariantStep {
  /// Replace the current selector with a new one.
  Selector(String),
  /// Wrap the entire rule in an at-rule.
  AtRule { name: String, params: String },
}

/// Apply a list of variants (source order, left-most first) to a base class
/// selector + inner CSS nodes, producing the resulting AST node.
///
/// Returns `None` if any variant in the stack is not recognised — callers
/// typically drop such candidates.
pub fn apply_variants(
  class_selector: String,
  variants: &[String],
  nodes: Vec<AstNode>,
  registry: Option<&VariantRegistry>,
) -> Option<AstNode> {
  // Start with a single-rule node carrying the base selector + declarations.
  let mut current_selector = class_selector;
  let mut at_rules: Vec<(String, String)> = Vec::new();

  // Walk variants in reverse so that the right-most variant in the source
  // (closest to the utility) is applied first. This is the same order as
  // upstream's variant application.
  for variant in variants.iter().rev() {
    let step = resolve_variant(&current_selector, variant, registry)?;
    match step {
      VariantStep::Selector(new_sel) => current_selector = new_sel,
      VariantStep::AtRule { name, params } => at_rules.push((name, params)),
    }
  }

  let mut node: AstNode = AstNode::Rule(StyleRule {
    selector: current_selector,
    nodes,
  });

  // Wrap from innermost to outermost: the first at-rule encountered (which
  // was the right-most variant in source) becomes the innermost wrapper.
  for (name, params) in at_rules {
    node = AstNode::AtRule(AtRule {
      name,
      params,
      nodes: vec![node],
    });
  }

  Some(node)
}

/// Resolve a single variant against the current selector, producing a
/// [`VariantStep`]. Returns `None` for unknown variants.
fn resolve_variant(
  selector: &str,
  variant: &str,
  registry: Option<&VariantRegistry>,
) -> Option<VariantStep> {
  // 0. User-defined custom variants (registered via `@custom-variant`) take
  //    precedence over built-ins so users can override.
  if let Some(reg) = registry {
    if let Some(body) = reg.custom_variant(variant) {
      return Some(resolve_custom_variant_body(selector, body));
    }
  }

  // 1. Arbitrary selector variant: `[&_p]`, `[&:hover]`, `[&>div]`, …
  if let Some(body) = variant.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
    return Some(VariantStep::Selector(apply_arbitrary_selector(
      selector, body,
    )));
  }

  // 2. Container-query variants (start with `@`).
  if variant.starts_with('@') {
    return resolve_container_query(variant);
  }

  // 3. Functional variants of the form `prefix-[…]` or `prefix-name`.
  if let Some(step) = resolve_functional(selector, variant) {
    return Some(step);
  }

  // 4. Plain pseudo / at-rule alias.
  resolve_named(selector, variant)
}

/// Apply a registered `@custom-variant` body. The body is either:
///   - a selector template containing `&`, e.g. `&:hover` →
///     [`VariantStep::Selector`]
///   - an at-rule form `@<name> (<params>)`, e.g. `@media (pointer:coarse)` →
///     [`VariantStep::AtRule`]
fn resolve_custom_variant_body(selector: &str, body: &str) -> VariantStep {
  let trimmed = body.trim();
  if let Some(rest) = trimmed.strip_prefix('@') {
    // Split into `<name> <params>` at the first whitespace or `(`.
    let mut split_at = rest.len();
    for (i, ch) in rest.char_indices() {
      if ch == ' ' || ch == '\t' || ch == '(' {
        split_at = i;
        break;
      }
    }
    let name = format!("@{}", &rest[..split_at]);
    let params = rest[split_at..].trim().to_string();
    return VariantStep::AtRule { name, params };
  }
  // Selector template — substitute `&` with the current selector.
  VariantStep::Selector(trimmed.replace('&', selector))
}

// ── named (alias) variants ─────────────────────────────────────────────────

fn resolve_named(selector: &str, name: &str) -> Option<VariantStep> {
  // Pseudo-classes — appended after the current selector.
  let pseudo = match name {
    // States
    "hover" => ":hover",
    "focus" => ":focus",
    "focus-within" => ":focus-within",
    "focus-visible" => ":focus-visible",
    "active" => ":active",
    "disabled" => ":disabled",
    "enabled" => ":enabled",
    "visited" => ":visited",
    "target" => ":target",
    "checked" => ":checked",
    "indeterminate" => ":indeterminate",
    "default" => ":default",
    "required" => ":required",
    "optional" => ":optional",
    "valid" => ":valid",
    "invalid" => ":invalid",
    "in-range" => ":in-range",
    "out-of-range" => ":out-of-range",
    "placeholder-shown" => ":placeholder-shown",
    "autofill" => ":autofill",
    "read-only" => ":read-only",
    "read-write" => ":read-write",
    "open" => ":open",
    "inert" => ":inert",
    "empty" => ":empty",
    // Position
    "first" => ":first-child",
    "last" => ":last-child",
    "only" => ":only-child",
    "odd" => ":nth-child(odd)",
    "even" => ":nth-child(2n)",
    "first-of-type" => ":first-of-type",
    "last-of-type" => ":last-of-type",
    "only-of-type" => ":only-of-type",
    _ => "",
  };
  if !pseudo.is_empty() {
    return Some(VariantStep::Selector(format!("{}{}", selector, pseudo)));
  }

  // Pseudo-elements.
  let element = match name {
    "before" => "::before",
    "after" => "::after",
    "first-letter" => "::first-letter",
    "first-line" => "::first-line",
    "marker" => "::marker",
    "selection" => "::selection",
    "file" => "::file-selector-button",
    "placeholder" => "::placeholder",
    "backdrop" => "::backdrop",
    _ => "",
  };
  if !element.is_empty() {
    return Some(VariantStep::Selector(format!("{}{}", selector, element)));
  }

  // Direction.
  if name == "ltr" {
    return Some(VariantStep::Selector(format!(
      ":where([dir=\"ltr\"], [dir=\"ltr\"] *) {}",
      selector
    )));
  }
  if name == "rtl" {
    return Some(VariantStep::Selector(format!(
      ":where([dir=\"rtl\"], [dir=\"rtl\"] *) {}",
      selector
    )));
  }

  // Media / responsive.
  if let Some(media) = media_for_named(name) {
    return Some(VariantStep::AtRule {
      name: "@media".to_string(),
      params: media,
    });
  }

  // Container-query named breakpoint (handled in `@`-prefixed branch already
  // for `@sm` etc. — left here for completeness).
  None
}

fn media_for_named(name: &str) -> Option<String> {
  // Breakpoints — Tailwind v4 defaults.
  if let Some(min) = breakpoint_default(name) {
    return Some(format!("(min-width: {})", min));
  }
  if let Some(stripped) = name.strip_prefix("max-") {
    if let Some(min) = breakpoint_default(stripped) {
      return Some(format!("(max-width: calc({} - 1px))", min));
    }
  }
  Some(
    match name {
      "dark" => "(prefers-color-scheme: dark)",
      "print" => "print",
      "motion-safe" => "(prefers-reduced-motion: no-preference)",
      "motion-reduce" => "(prefers-reduced-motion: reduce)",
      "portrait" => "(orientation: portrait)",
      "landscape" => "(orientation: landscape)",
      "contrast-more" => "(prefers-contrast: more)",
      "contrast-less" => "(prefers-contrast: less)",
      _ => return None,
    }
    .to_string(),
  )
}

fn breakpoint_default(name: &str) -> Option<&'static str> {
  Some(match name {
    "sm" => "640px",
    "md" => "768px",
    "lg" => "1024px",
    "xl" => "1280px",
    "2xl" => "1536px",
    _ => return None,
  })
}

fn container_default(name: &str) -> Option<&'static str> {
  Some(match name {
    "sm" => "24rem",
    "md" => "28rem",
    "lg" => "32rem",
    "xl" => "36rem",
    "2xl" => "42rem",
    "3xl" => "48rem",
    "4xl" => "56rem",
    "5xl" => "64rem",
    "6xl" => "72rem",
    "7xl" => "80rem",
    _ => return None,
  })
}

// ── container queries ──────────────────────────────────────────────────────

fn resolve_container_query(variant: &str) -> Option<VariantStep> {
  // Bare `@container` — wraps in `@container` with no condition.
  if variant == "@container" {
    return Some(VariantStep::AtRule {
      name: "@container".to_string(),
      params: String::new(),
    });
  }

  // Named container w/ optional `/ident` suffix: `@sm/sidebar`.
  let (head, container_ident) = match variant.split_once('/') {
    Some((h, t)) => (h, Some(t)),
    None => (variant, None),
  };

  // Arbitrary: `@min-[20rem]`, `@max-[40rem]`, `@[24rem]`.
  if let Some(arg) = head.strip_prefix("@min-").and_then(arbitrary_body) {
    return Some(make_container_at_rule(
      container_ident,
      format!("(min-width: {})", arg),
    ));
  }
  if let Some(arg) = head.strip_prefix("@max-").and_then(arbitrary_body) {
    return Some(make_container_at_rule(
      container_ident,
      format!("(max-width: {})", arg),
    ));
  }
  if let Some(arg) = head.strip_prefix('@').and_then(arbitrary_body) {
    return Some(make_container_at_rule(
      container_ident,
      format!("(min-width: {})", arg),
    ));
  }

  // Named breakpoint: `@sm`, `@md`, …
  if let Some(name) = head.strip_prefix('@') {
    if let Some(size) = container_default(name) {
      return Some(make_container_at_rule(
        container_ident,
        format!("(min-width: {})", size),
      ));
    }
  }

  None
}

fn make_container_at_rule(ident: Option<&str>, condition: String) -> VariantStep {
  let params = match ident {
    Some(name) if !name.is_empty() => format!("{} {}", name, condition),
    _ => condition,
  };
  VariantStep::AtRule {
    name: "@container".to_string(),
    params,
  }
}

// ── functional variants (`prefix-…` or `prefix-[…]`) ───────────────────────

fn resolve_functional(selector: &str, variant: &str) -> Option<VariantStep> {
  // `not-…` → `:not(…)`
  if let Some(rest) = variant.strip_prefix("not-") {
    let inner = lookup_pseudo_body(rest)?;
    return Some(VariantStep::Selector(format!(
      "{}:not({})",
      selector, inner
    )));
  }

  // `has-…` → `:has(…)`
  if let Some(rest) = variant.strip_prefix("has-") {
    // Arbitrary form `has-[input]` keeps the body as-is.
    if let Some(body) = arbitrary_body(rest) {
      return Some(VariantStep::Selector(format!("{}:has({})", selector, body)));
    }
    let inner = lookup_pseudo_body(rest)?;
    return Some(VariantStep::Selector(format!(
      "{}:has({})",
      selector, inner
    )));
  }

  // `data-…` → `[data-…]` (arbitrary or named alias).
  if let Some(rest) = variant.strip_prefix("data-") {
    if let Some(body) = arbitrary_body(rest) {
      let attr = format_data_aria_attr("data", body);
      return Some(VariantStep::Selector(format!("{}{}", selector, attr)));
    }
    // Named: `data-open` → `[data-open]`.
    return Some(VariantStep::Selector(format!(
      "{}[data-{}]",
      selector, rest
    )));
  }

  // `aria-…` → `[aria-…]`.
  if let Some(rest) = variant.strip_prefix("aria-") {
    if let Some(body) = arbitrary_body(rest) {
      let attr = format_data_aria_attr("aria", body);
      return Some(VariantStep::Selector(format!("{}{}", selector, attr)));
    }
    // Named: `aria-checked` → `[aria-checked="true"]`.
    return Some(VariantStep::Selector(format!(
      "{}[aria-{}=\"true\"]",
      selector, rest
    )));
  }

  // `supports-[…]` → `@supports (…)`.
  if let Some(rest) = variant.strip_prefix("supports-") {
    if let Some(body) = arbitrary_body(rest) {
      // If the body looks like `display:grid` (no parens), wrap in `(…)`.
      let params = if body.starts_with('(') {
        body.to_string()
      } else {
        format!("({})", body)
      };
      return Some(VariantStep::AtRule {
        name: "@supports".to_string(),
        params,
      });
    }
    return None;
  }

  // `min-[…]` / `max-[…]` arbitrary media breakpoints.
  if let Some(rest) = variant.strip_prefix("min-") {
    if let Some(body) = arbitrary_body(rest) {
      return Some(VariantStep::AtRule {
        name: "@media".to_string(),
        params: format!("(min-width: {})", body),
      });
    }
  }
  if let Some(rest) = variant.strip_prefix("max-") {
    if let Some(body) = arbitrary_body(rest) {
      return Some(VariantStep::AtRule {
        name: "@media".to_string(),
        params: format!("(max-width: {})", body),
      });
    }
  }

  // `group-…` and `peer-…` (with optional `/name` modifier, ignored for now).
  if let Some(rest) = variant.strip_prefix("group-") {
    return group_peer_step(selector, "group", rest);
  }
  if let Some(rest) = variant.strip_prefix("peer-") {
    return group_peer_step(selector, "peer", rest);
  }

  None
}

fn group_peer_step(selector: &str, kind: &str, rest: &str) -> Option<VariantStep> {
  // Strip optional `/name` modifier — `group-hover/sidebar` targets a named
  // group. We represent it as `.group\/sidebar` per upstream.
  let (rest, name) = match rest.split_once('/') {
    Some((head, tag)) => (head, Some(tag)),
    None => (rest, None),
  };

  let scope_class = match name {
    Some(tag) => format!(".{}\\/{}", kind, tag),
    None => format!(".{}", kind),
  };

  // Arbitrary: `group-[.is-open]` → `:where(.group.is-open) <selector>`.
  if let Some(body) = arbitrary_body(rest) {
    // `body` may start with `&` to reference the group element itself.
    let group_selector = if body.contains('&') {
      body.replace('&', &scope_class)
    } else {
      format!("{}{}", scope_class, body)
    };
    let combinator = if kind == "peer" { " ~ " } else { " " };
    return Some(VariantStep::Selector(format!(
      ":where({}){}{}",
      group_selector, combinator, selector
    )));
  }

  // Named: `group-hover` → `:where(.group):hover <selector>`.
  let inner = lookup_pseudo_body(rest)?;
  let combinator = if kind == "peer" { " ~ " } else { " " };
  Some(VariantStep::Selector(format!(
    ":where({}){}{}{}",
    scope_class, inner, combinator, selector
  )))
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Returns `Some(body)` if `s` is of the form `[body]`, else `None`.
fn arbitrary_body(s: &str) -> Option<&str> {
  s.strip_prefix('[').and_then(|t| t.strip_suffix(']'))
}

/// Replace `&` in an arbitrary selector body with the current selector. If no
/// `&` is present, append the body as a descendant (mirrors upstream).
fn apply_arbitrary_selector(selector: &str, body: &str) -> String {
  if body.contains('&') {
    // Underscores inside arbitrary variants are spaces (just like arbitrary
    // values), but `\_` should be preserved as a literal `_`.
    let body = decode_underscores(body);
    body.replace('&', selector)
  } else {
    format!("{} {}", selector, body)
  }
}

fn decode_underscores(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  let bytes = input.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let c = bytes[i];
    if c == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'_' {
      out.push('_');
      i += 2;
      continue;
    }
    if c == b'_' {
      out.push(' ');
      i += 1;
      continue;
    }
    out.push(c as char);
    i += 1;
  }
  out
}

/// Lookup the inner pseudo-class body for a named state used inside `not-…`,
/// `has-…`, `group-…`, `peer-…`. Returns e.g. `":hover"` for `"hover"`.
fn lookup_pseudo_body(name: &str) -> Option<String> {
  // Re-use `resolve_named` with an empty selector so we get just the pseudo.
  let step = resolve_named("", name)?;
  match step {
    VariantStep::Selector(s) => Some(s),
    VariantStep::AtRule { .. } => None,
  }
}

/// Turn an `aria` / `data` arbitrary body into an attribute selector.
/// - `data-[size=large]` → `[data-size="large"]`
/// - `data-[open]`       → `[data-open]`
/// - `data-[type~="x"]`  → `[data-type~="x"]` (already valid CSS)
fn format_data_aria_attr(prefix: &str, body: &str) -> String {
  // If body already looks like a full attribute (contains `=` with quoted value
  // or operator like ~/^/$/*), keep as-is.
  if body.starts_with('[') {
    // Defensive: unlikely double-wrapped.
    return body.to_string();
  }
  if let Some((key, value)) = body.split_once('=') {
    // Strip surrounding quotes from value, if any.
    let value = value.trim_matches(|c| c == '"' || c == '\'');
    return format!("[{}-{}=\"{}\"]", prefix, key, value);
  }
  format!("[{}-{}]", prefix, body)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hover_appends_pseudo() {
    let step = resolve_variant(".a", "hover", None).unwrap();
    assert_eq!(step, VariantStep::Selector(".a:hover".to_string()));
  }

  #[test]
  fn sm_wraps_media() {
    let step = resolve_variant(".a", "sm", None).unwrap();
    assert!(matches!(
      step,
      VariantStep::AtRule { ref name, ref params }
        if name == "@media" && params == "(min-width: 640px)"
    ));
  }

  #[test]
  fn data_arbitrary_attribute() {
    let step = resolve_variant(".a", "data-[size=large]", None).unwrap();
    assert_eq!(
      step,
      VariantStep::Selector(".a[data-size=\"large\"]".to_string())
    );
  }

  #[test]
  fn group_hover_compounds() {
    let step = resolve_variant(".a", "group-hover", None).unwrap();
    assert_eq!(
      step,
      VariantStep::Selector(":where(.group):hover .a".to_string())
    );
  }

  #[test]
  fn arbitrary_selector_replaces_ampersand() {
    let step = resolve_variant(".a", "[&_p]", None).unwrap();
    assert_eq!(step, VariantStep::Selector(".a p".to_string()));
  }

  #[test]
  fn arbitrary_min_breakpoint() {
    let step = resolve_variant(".a", "min-[640px]", None).unwrap();
    assert!(matches!(
      step,
      VariantStep::AtRule { ref name, ref params }
        if name == "@media" && params == "(min-width: 640px)"
    ));
  }

  #[test]
  fn container_query_named() {
    let step = resolve_variant(".a", "@md", None).unwrap();
    assert!(matches!(
      step,
      VariantStep::AtRule { ref name, ref params }
        if name == "@container" && params == "(min-width: 28rem)"
    ));
  }
}
