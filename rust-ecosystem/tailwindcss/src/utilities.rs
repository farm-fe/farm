use std::collections::HashMap;

use crate::ast::{AstNode, Declaration};
use crate::candidate::ParsedCandidate;
use crate::theme::{Theme, ThemeOptions};

/// A list of CSS property-value pairs that form a utility class.
type DeclList = Vec<(&'static str, &'static str)>;

/// Function signature for functional utilities. Receives the parsed
/// candidate plus the resolved theme and returns the declarations to emit, or
/// `None` when the candidate cannot be handled (in which case the registry
/// falls through to other lookup paths).
pub type FunctionalHandler = fn(&ParsedCandidate, &Theme) -> Option<Vec<(String, String)>>;

/// Registry of built-in utility classes (static + functional).
pub struct UtilityRegistry {
  static_utilities: HashMap<&'static str, DeclList>,
  functional: HashMap<&'static str, FunctionalHandler>,
  /// User-defined static utilities registered from `@utility` blocks in the
  /// source CSS. Looked up before the built-ins so users may override.
  user_static_utilities: HashMap<String, Vec<(String, String)>>,
}

impl UtilityRegistry {
  /// Create the built-in utility registry.
  pub fn builtin() -> Self {
    let mut m: HashMap<&'static str, DeclList> = HashMap::new();

    // Display
    m.insert("block", vec![("display", "block")]);
    m.insert("inline", vec![("display", "inline")]);
    m.insert("inline-block", vec![("display", "inline-block")]);
    m.insert("flex", vec![("display", "flex")]);
    m.insert("inline-flex", vec![("display", "inline-flex")]);
    m.insert("grid", vec![("display", "grid")]);
    m.insert("inline-grid", vec![("display", "inline-grid")]);
    m.insert("hidden", vec![("display", "none")]);
    m.insert("table", vec![("display", "table")]);
    m.insert("table-row", vec![("display", "table-row")]);
    m.insert("table-cell", vec![("display", "table-cell")]);
    m.insert("flow-root", vec![("display", "flow-root")]);
    m.insert("contents", vec![("display", "contents")]);
    m.insert("list-item", vec![("display", "list-item")]);

    // Position
    m.insert("static", vec![("position", "static")]);
    m.insert("fixed", vec![("position", "fixed")]);
    m.insert("absolute", vec![("position", "absolute")]);
    m.insert("relative", vec![("position", "relative")]);
    m.insert("sticky", vec![("position", "sticky")]);

    // Flexbox direction
    m.insert("flex-row", vec![("flex-direction", "row")]);
    m.insert("flex-row-reverse", vec![("flex-direction", "row-reverse")]);
    m.insert("flex-col", vec![("flex-direction", "column")]);
    m.insert("flex-col-reverse", vec![("flex-direction", "column-reverse")]);

    // Flex wrap
    m.insert("flex-wrap", vec![("flex-wrap", "wrap")]);
    m.insert("flex-wrap-reverse", vec![("flex-wrap", "wrap-reverse")]);
    m.insert("flex-nowrap", vec![("flex-wrap", "nowrap")]);

    // Align items
    m.insert("items-start", vec![("align-items", "flex-start")]);
    m.insert("items-center", vec![("align-items", "center")]);
    m.insert("items-end", vec![("align-items", "flex-end")]);
    m.insert("items-baseline", vec![("align-items", "baseline")]);
    m.insert("items-stretch", vec![("align-items", "stretch")]);

    // Justify content
    m.insert("justify-start", vec![("justify-content", "flex-start")]);
    m.insert("justify-center", vec![("justify-content", "center")]);
    m.insert("justify-end", vec![("justify-content", "flex-end")]);
    m.insert("justify-between", vec![("justify-content", "space-between")]);
    m.insert("justify-around", vec![("justify-content", "space-around")]);
    m.insert("justify-evenly", vec![("justify-content", "space-evenly")]);
    m.insert("justify-normal", vec![("justify-content", "normal")]);
    m.insert("justify-stretch", vec![("justify-content", "stretch")]);

    // Text alignment
    m.insert("text-left", vec![("text-align", "left")]);
    m.insert("text-center", vec![("text-align", "center")]);
    m.insert("text-right", vec![("text-align", "right")]);
    m.insert("text-justify", vec![("text-align", "justify")]);
    m.insert("text-start", vec![("text-align", "start")]);
    m.insert("text-end", vec![("text-align", "end")]);

    // Font weight
    m.insert("font-thin", vec![("font-weight", "100")]);
    m.insert("font-extralight", vec![("font-weight", "200")]);
    m.insert("font-light", vec![("font-weight", "300")]);
    m.insert("font-normal", vec![("font-weight", "400")]);
    m.insert("font-medium", vec![("font-weight", "500")]);
    m.insert("font-semibold", vec![("font-weight", "600")]);
    m.insert("font-bold", vec![("font-weight", "700")]);
    m.insert("font-extrabold", vec![("font-weight", "800")]);
    m.insert("font-black", vec![("font-weight", "900")]);

    // Overflow
    m.insert("overflow-auto", vec![("overflow", "auto")]);
    m.insert("overflow-hidden", vec![("overflow", "hidden")]);
    m.insert("overflow-clip", vec![("overflow", "clip")]);
    m.insert("overflow-visible", vec![("overflow", "visible")]);
    m.insert("overflow-scroll", vec![("overflow", "scroll")]);
    m.insert("overflow-x-auto", vec![("overflow-x", "auto")]);
    m.insert("overflow-y-auto", vec![("overflow-y", "auto")]);
    m.insert("overflow-x-hidden", vec![("overflow-x", "hidden")]);
    m.insert("overflow-y-hidden", vec![("overflow-y", "hidden")]);
    m.insert("overflow-x-scroll", vec![("overflow-x", "scroll")]);
    m.insert("overflow-y-scroll", vec![("overflow-y", "scroll")]);

    // Visibility
    m.insert("visible", vec![("visibility", "visible")]);
    m.insert("invisible", vec![("visibility", "hidden")]);
    m.insert("collapse", vec![("visibility", "collapse")]);

    // Cursor
    m.insert("cursor-auto", vec![("cursor", "auto")]);
    m.insert("cursor-default", vec![("cursor", "default")]);
    m.insert("cursor-pointer", vec![("cursor", "pointer")]);
    m.insert("cursor-wait", vec![("cursor", "wait")]);
    m.insert("cursor-text", vec![("cursor", "text")]);
    m.insert("cursor-move", vec![("cursor", "move")]);
    m.insert("cursor-not-allowed", vec![("cursor", "not-allowed")]);
    m.insert("cursor-grab", vec![("cursor", "grab")]);
    m.insert("cursor-grabbing", vec![("cursor", "grabbing")]);

    // Text decoration
    m.insert("underline", vec![("text-decoration-line", "underline")]);
    m.insert("overline", vec![("text-decoration-line", "overline")]);
    m.insert("line-through", vec![("text-decoration-line", "line-through")]);
    m.insert("no-underline", vec![("text-decoration-line", "none")]);

    // White space
    m.insert("whitespace-normal", vec![("white-space", "normal")]);
    m.insert("whitespace-nowrap", vec![("white-space", "nowrap")]);
    m.insert("whitespace-pre", vec![("white-space", "pre")]);
    m.insert("whitespace-pre-line", vec![("white-space", "pre-line")]);
    m.insert("whitespace-pre-wrap", vec![("white-space", "pre-wrap")]);
    m.insert(
      "whitespace-break-spaces",
      vec![("white-space", "break-spaces")],
    );

    // User select
    m.insert("select-none", vec![("user-select", "none")]);
    m.insert("select-text", vec![("user-select", "text")]);
    m.insert("select-all", vec![("user-select", "all")]);
    m.insert("select-auto", vec![("user-select", "auto")]);

    // Pointer events
    m.insert("pointer-events-none", vec![("pointer-events", "none")]);
    m.insert("pointer-events-auto", vec![("pointer-events", "auto")]);

    // Float / clear
    m.insert("float-start", vec![("float", "inline-start")]);
    m.insert("float-end", vec![("float", "inline-end")]);
    m.insert("float-right", vec![("float", "right")]);
    m.insert("float-left", vec![("float", "left")]);
    m.insert("float-none", vec![("float", "none")]);
    m.insert("clear-start", vec![("clear", "inline-start")]);
    m.insert("clear-end", vec![("clear", "inline-end")]);
    m.insert("clear-left", vec![("clear", "left")]);
    m.insert("clear-right", vec![("clear", "right")]);
    m.insert("clear-both", vec![("clear", "both")]);
    m.insert("clear-none", vec![("clear", "none")]);

    // Object fit
    m.insert("object-contain", vec![("object-fit", "contain")]);
    m.insert("object-cover", vec![("object-fit", "cover")]);
    m.insert("object-fill", vec![("object-fit", "fill")]);
    m.insert("object-none", vec![("object-fit", "none")]);
    m.insert("object-scale-down", vec![("object-fit", "scale-down")]);

    // Box sizing
    m.insert("box-border", vec![("box-sizing", "border-box")]);
    m.insert("box-content", vec![("box-sizing", "content-box")]);

    // Border collapse
    m.insert("border-collapse", vec![("border-collapse", "collapse")]);
    m.insert("border-separate", vec![("border-collapse", "separate")]);

    // Caption side
    m.insert("caption-top", vec![("caption-side", "top")]);
    m.insert("caption-bottom", vec![("caption-side", "bottom")]);

    // Vertical align
    m.insert("align-baseline", vec![("vertical-align", "baseline")]);
    m.insert("align-top", vec![("vertical-align", "top")]);
    m.insert("align-middle", vec![("vertical-align", "middle")]);
    m.insert("align-bottom", vec![("vertical-align", "bottom")]);
    m.insert("align-text-top", vec![("vertical-align", "text-top")]);
    m.insert("align-text-bottom", vec![("vertical-align", "text-bottom")]);
    m.insert("align-sub", vec![("vertical-align", "sub")]);
    m.insert("align-super", vec![("vertical-align", "super")]);

    // Appearance
    m.insert("appearance-none", vec![("appearance", "none")]);
    m.insert("appearance-auto", vec![("appearance", "auto")]);

    // Outline
    m.insert(
      "outline-none",
      vec![
        ("outline", "2px solid transparent"),
        ("outline-offset", "2px"),
      ],
    );

    // List style position
    m.insert("list-inside", vec![("list-style-position", "inside")]);
    m.insert("list-outside", vec![("list-style-position", "outside")]);

    // Line clamp / truncate
    m.insert(
      "truncate",
      vec![
        ("overflow", "hidden"),
        ("text-overflow", "ellipsis"),
        ("white-space", "nowrap"),
      ],
    );
    m.insert("text-ellipsis", vec![("text-overflow", "ellipsis")]);
    m.insert("text-clip", vec![("text-overflow", "clip")]);

    // Antialiasing
    m.insert(
      "antialiased",
      vec![
        ("-webkit-font-smoothing", "antialiased"),
        ("-moz-osx-font-smoothing", "grayscale"),
      ],
    );
    m.insert(
      "subpixel-antialiased",
      vec![
        ("-webkit-font-smoothing", "auto"),
        ("-moz-osx-font-smoothing", "auto"),
      ],
    );

    // Italic / not-italic
    m.insert("italic", vec![("font-style", "italic")]);
    m.insert("not-italic", vec![("font-style", "normal")]);

    // Text-transform
    m.insert("uppercase", vec![("text-transform", "uppercase")]);
    m.insert("lowercase", vec![("text-transform", "lowercase")]);
    m.insert("capitalize", vec![("text-transform", "capitalize")]);
    m.insert("normal-case", vec![("text-transform", "none")]);

    // Word/line break
    m.insert(
      "break-normal",
      vec![("overflow-wrap", "normal"), ("word-break", "normal")],
    );
    m.insert("break-words", vec![("overflow-wrap", "break-word")]);
    m.insert("break-all", vec![("word-break", "break-all")]);
    m.insert("break-keep", vec![("word-break", "keep-all")]);

    // Default border-radius / width when used bare (no value)
    m.insert("rounded-none", vec![("border-radius", "0")]);
    m.insert("rounded-full", vec![("border-radius", "9999px")]);

    // ── Functional utilities ────────────────────────────────────────────
    let mut f: HashMap<&'static str, FunctionalHandler> = HashMap::new();

    // Spacing-driven (margin / padding / gap / inset / size)
    f.insert("m", handle_m as FunctionalHandler);
    f.insert("mx", handle_mx);
    f.insert("my", handle_my);
    f.insert("mt", handle_mt);
    f.insert("mr", handle_mr);
    f.insert("mb", handle_mb);
    f.insert("ml", handle_ml);
    f.insert("ms", handle_ms);
    f.insert("me", handle_me);

    f.insert("p", handle_p);
    f.insert("px", handle_px);
    f.insert("py", handle_py);
    f.insert("pt", handle_pt);
    f.insert("pr", handle_pr);
    f.insert("pb", handle_pb);
    f.insert("pl", handle_pl);
    f.insert("ps", handle_ps);
    f.insert("pe", handle_pe);

    f.insert("gap", handle_gap);
    f.insert("gap-x", handle_gap_x);
    f.insert("gap-y", handle_gap_y);

    f.insert("inset", handle_inset);
    f.insert("inset-x", handle_inset_x);
    f.insert("inset-y", handle_inset_y);
    f.insert("top", handle_top);
    f.insert("right", handle_right);
    f.insert("bottom", handle_bottom);
    f.insert("left", handle_left);
    f.insert("start", handle_start);
    f.insert("end", handle_end);

    // Sizing
    f.insert("w", handle_w);
    f.insert("h", handle_h);
    f.insert("size", handle_size);
    f.insert("min-w", handle_min_w);
    f.insert("min-h", handle_min_h);
    f.insert("max-w", handle_max_w);
    f.insert("max-h", handle_max_h);

    // Colors
    f.insert("text", handle_text);
    f.insert("bg", handle_bg);
    f.insert("border", handle_border);
    f.insert("fill", handle_fill);
    f.insert("stroke", handle_stroke);
    f.insert("ring", handle_ring);
    f.insert("accent", handle_accent);
    f.insert("caret", handle_caret);
    f.insert("decoration", handle_decoration);
    f.insert("outline", handle_outline);
    f.insert("placeholder", handle_placeholder);

    // Border radius
    f.insert("rounded", handle_rounded);
    f.insert("rounded-t", handle_rounded_t);
    f.insert("rounded-r", handle_rounded_r);
    f.insert("rounded-b", handle_rounded_b);
    f.insert("rounded-l", handle_rounded_l);
    f.insert("rounded-tl", handle_rounded_tl);
    f.insert("rounded-tr", handle_rounded_tr);
    f.insert("rounded-bl", handle_rounded_bl);
    f.insert("rounded-br", handle_rounded_br);

    // Border width (no value = 1px)
    f.insert("border-t", handle_border_t);
    f.insert("border-r", handle_border_r);
    f.insert("border-b", handle_border_b);
    f.insert("border-l", handle_border_l);
    f.insert("border-x", handle_border_x);
    f.insert("border-y", handle_border_y);

    // Opacity
    f.insert("opacity", handle_opacity);

    // Z-index
    f.insert("z", handle_z);

    // Order / flex-grow / flex-shrink / basis
    f.insert("order", handle_order);
    f.insert("grow", handle_grow);
    f.insert("shrink", handle_shrink);
    f.insert("basis", handle_basis);

    Self {
      static_utilities: m,
      functional: f,
      user_static_utilities: HashMap::new(),
    }
  }

  /// Register a user-defined static utility (typically discovered from an
  /// `@utility name { … }` block in the source CSS). User utilities are
  /// looked up before the built-ins, allowing overrides.
  pub fn register_static_utility(&mut self, name: String, declarations: Vec<(String, String)>) {
    self.user_static_utilities.insert(name, declarations);
  }

  /// Returns `true` if a utility root exists in the registry.
  pub fn has(&self, name: &str) -> bool {
    self.user_static_utilities.contains_key(name)
      || self.static_utilities.contains_key(name)
      || self.functional.contains_key(name)
  }

  /// Generate CSS AST nodes for a parsed candidate.
  pub fn generate(&self, candidate: &ParsedCandidate, theme: &Theme) -> Vec<AstNode> {
    self.generate_with_variants(candidate, theme, None)
  }

  /// Generate CSS AST nodes for a parsed candidate, consulting an optional
  /// [`crate::variants::VariantRegistry`] for user-defined custom variants.
  pub fn generate_with_variants(
    &self,
    candidate: &ParsedCandidate,
    theme: &Theme,
    variant_registry: Option<&crate::variants::VariantRegistry>,
  ) -> Vec<AstNode> {
    // Arbitrary property: [color:red]
    if let Some((ref property, ref value)) = candidate.arbitrary_property {
      let class_name = format!("[{}:{}]", property, value);
      let decl = Declaration {
        property: property.clone(),
        value: Some(value.clone()),
        important: candidate.important,
      };
      return wrap_with_variants(
        &class_name,
        &candidate.variants,
        vec![AstNode::Declaration(decl)],
        variant_registry,
      );
    }

    // Static utilities — try full class name first (e.g. "items-center")
    let compound_key: String = match &candidate.arbitrary_value {
      Some(val) => {
        let type_prefix = match &candidate.type_hint {
          Some(hint) => format!("{}:", hint),
          None => String::new(),
        };
        format!("{}-[{}{}]", candidate.utility_root, type_prefix, val)
      }
      None => match &candidate.utility_value {
        Some(val) => format!("{}-{}", candidate.utility_root, val),
        None => candidate.utility_root.clone(),
      },
    };

    // User-defined static utilities take precedence over built-ins so
    // `@utility` blocks may override built-in classes.
    if let Some(declarations) = self.user_static_utilities.get(compound_key.as_str()) {
      let class_name = build_class_name(candidate);
      let decl_nodes: Vec<AstNode> = declarations
        .iter()
        .map(|(prop, val)| {
          AstNode::Declaration(Declaration {
            property: prop.clone(),
            value: Some(val.clone()),
            important: candidate.important,
          })
        })
        .collect();
      return wrap_with_variants(
        &class_name,
        &candidate.variants,
        decl_nodes,
        variant_registry,
      );
    }

    if let Some(declarations) = self.static_utilities.get(compound_key.as_str()) {
      let class_name = build_class_name(candidate);
      let decl_nodes: Vec<AstNode> = declarations
        .iter()
        .map(|(prop, val)| {
          AstNode::Declaration(Declaration {
            property: prop.to_string(),
            value: Some(val.to_string()),
            important: candidate.important,
          })
        })
        .collect();
      return wrap_with_variants(
        &class_name,
        &candidate.variants,
        decl_nodes,
        variant_registry,
      );
    }

    // Functional utilities — match the compound key first, then the root.
    // This lets `rounded-t-lg` find `rounded-t` even though `split_root`
    // tags it as root="rounded" + value="t-lg".
    let functional_key = find_functional_key(candidate, &self.functional);
    if let Some((key, leftover_value)) = functional_key {
      let handler = self.functional[key];
      // Re-shape candidate so the handler sees the leftover value.
      let mut reshaped = candidate.clone();
      reshaped.utility_root = key.to_string();
      reshaped.utility_value = leftover_value;
      if let Some(decls) = handler(&reshaped, theme) {
        if decls.is_empty() {
          return Vec::new();
        }
        let class_name = build_class_name(candidate);
        let decl_nodes: Vec<AstNode> = decls
          .into_iter()
          .map(|(prop, val)| {
            AstNode::Declaration(Declaration {
              property: prop,
              value: Some(val),
              important: candidate.important,
            })
          })
          .collect();
        return wrap_with_variants(
          &class_name,
          &candidate.variants,
          decl_nodes,
          variant_registry,
        );
      }
    }

    vec![]
  }
}

/// Locate the longest functional key (multi-segment) that matches the
/// candidate, then return that key together with the *leftover* value.
fn find_functional_key<'a>(
  candidate: &ParsedCandidate,
  functional: &'a HashMap<&'static str, FunctionalHandler>,
) -> Option<(&'a &'static str, Option<String>)> {
  // Build full prefix string (root + value before any arbitrary).
  let full_root = match &candidate.utility_value {
    Some(v) => format!("{}-{}", candidate.utility_root, v),
    None => candidate.utility_root.clone(),
  };

  // Try the full root (e.g. "rounded-tl"), then progressively trim from the
  // right at each `-` boundary until we find a matching registered key.
  let mut cursor = full_root.as_str();
  loop {
    if let Some((k, _)) = functional.get_key_value(cursor) {
      let leftover = if cursor.len() == full_root.len() {
        None
      } else {
        Some(full_root[cursor.len() + 1..].to_string())
      };
      return Some((k, leftover));
    }
    match cursor.rfind('-') {
      Some(idx) => cursor = &cursor[..idx],
      None => break,
    }
  }
  None
}

/// Build the AST for `class_name + variants[]` wrapping `decl_nodes`. When
/// there are no variants this produces a single rule; otherwise the variant
/// application pipeline composes the selector and at-rule chain. Returns an
/// empty `Vec` when any variant in the stack is unrecognised.
fn wrap_with_variants(
  class_name: &str,
  variants: &[String],
  decl_nodes: Vec<AstNode>,
  variant_registry: Option<&crate::variants::VariantRegistry>,
) -> Vec<AstNode> {
  let escaped = escape_class_name(class_name);
  let base_selector = build_base_selector_with_pseudo_classes(&escaped, variants);
  if base_selector.is_none() {
    return Vec::new();
  }
  let (base_selector, remaining) = base_selector.unwrap();
  match crate::variants::apply_variants(base_selector, &remaining, decl_nodes, variant_registry) {
    Some(node) => vec![node],
    None => Vec::new(),
  }
}

fn build_base_selector_with_pseudo_classes(
  escaped_class: &str,
  variants: &[String],
) -> Option<(String, Vec<String>)> {
  Some((format!(".{}", escaped_class), variants.to_vec()))
}

// ── helpers ────────────────────────────────────────────────────────────────

pub(crate) fn build_class_name(candidate: &ParsedCandidate) -> String {
  let base = match &candidate.arbitrary_value {
    Some(val) => {
      let type_prefix = match &candidate.type_hint {
        Some(hint) => format!("{}:", hint),
        None => String::new(),
      };
      format!("{}-[{}{}]", candidate.utility_root, type_prefix, val)
    }
    None => match &candidate.utility_value {
      Some(val) => format!("{}-{}", candidate.utility_root, val),
      None => candidate.utility_root.clone(),
    },
  };
  let with_modifier = match &candidate.modifier {
    Some(m) if candidate.modifier_is_arbitrary => format!("{}/{}", base, m),
    Some(m) => format!("{}/{}", base, m),
    None => base,
  };
  let with_negative = if candidate.negative {
    format!("-{}", with_modifier)
  } else {
    with_modifier
  };
  if candidate.important {
    format!("{}!", with_negative)
  } else {
    with_negative
  }
}

pub(crate) fn escape_class_name(name: &str) -> String {
  name
    .replace(':', "\\:")
    .replace('/', "\\/")
    .replace('.', "\\.")
    .replace('[', "\\[")
    .replace(']', "\\]")
    .replace('(', "\\(")
    .replace(')', "\\)")
    .replace('#', "\\#")
    .replace('!', "\\!")
}

// ═══ Functional utility helpers ════════════════════════════════════════════

/// Resolve a spacing value. Supports:
/// - `auto` / `full` / `screen` / `min` / `max` / `fit` / `px`
/// - integer multipliers like `4` → `calc(var(--spacing) * 4)`
/// - decimal/fraction multipliers like `2.5` → `calc(var(--spacing) * 2.5)`
/// - simple fractions `1/2` → `50%`
/// - arbitrary values (via `arbitrary_value`)
fn resolve_spacing(c: &ParsedCandidate, _theme: &Theme) -> Option<String> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(maybe_neg(arb.clone(), c.negative));
  }
  let v = c.utility_value.as_deref()?;

  // Fractional values are parsed by the candidate parser as
  // `value="num"` + `modifier="den"`. Reconstruct the fraction here.
  if let Some(m) = c.modifier.as_deref() {
    if !c.modifier_is_arbitrary && is_numeric(v) && is_numeric(m) {
      let num: f64 = v.parse().ok()?;
      let den: f64 = m.parse().ok()?;
      if den != 0.0 {
        let pct = (num / den) * 100.0;
        return Some(maybe_neg(format!("{}%", trim_float(pct)), c.negative));
      }
    }
  }

  match v {
    "auto" => Some("auto".into()),
    "full" => Some("100%".into()),
    "screen" => Some("100vh".into()),
    "min" => Some("min-content".into()),
    "max" => Some("max-content".into()),
    "fit" => Some("fit-content".into()),
    "px" => Some(maybe_neg("1px".into(), c.negative)),
    _ => {
      // Fractions encoded as `1/2` (rare path; the parser strips them as
      // modifiers but keep this branch for direct-call callers).
      if let Some((num, den)) = parse_fraction(v) {
        let pct = (num / den) * 100.0;
        return Some(maybe_neg(format!("{}%", trim_float(pct)), c.negative));
      }
      // Numeric multiplier
      if is_numeric(v) {
        let sign = if c.negative { "-" } else { "" };
        return Some(format!("calc(var(--spacing) * {}{})", sign, v));
      }
      None
    }
  }
}

/// Resolve a sizing value (w/h/min-*/max-*/size). Reuses spacing rules,
/// adds the sizing-specific tokens.
fn resolve_sizing(c: &ParsedCandidate, theme: &Theme) -> Option<String> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(arb.clone());
  }
  let v = c.utility_value.as_deref()?;
  match v {
    "screen" if c.utility_root.starts_with('h') => Some("100vh".into()),
    "screen" => Some("100vw".into()),
    "dvw" => Some("100dvw".into()),
    "dvh" => Some("100dvh".into()),
    "svw" => Some("100svw".into()),
    "svh" => Some("100svh".into()),
    "lvw" => Some("100lvw".into()),
    "lvh" => Some("100lvh".into()),
    _ => resolve_spacing(c, theme),
  }
}

fn parse_fraction(v: &str) -> Option<(f64, f64)> {
  let (a, b) = v.split_once('/')?;
  let num = a.parse::<f64>().ok()?;
  let den = b.parse::<f64>().ok()?;
  if den == 0.0 {
    return None;
  }
  Some((num, den))
}

fn is_numeric(v: &str) -> bool {
  !v.is_empty() && v.parse::<f64>().is_ok()
}

fn trim_float(n: f64) -> String {
  if n.fract() == 0.0 {
    format!("{}", n as i64)
  } else {
    // Trim trailing zeros without losing precision for normal values.
    let s = format!("{:.6}", n);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    s
  }
}

fn maybe_neg(value: String, negative: bool) -> String {
  if !negative {
    return value;
  }
  // Prefer wrapping in calc(-1 * …) only for non-numeric/non-percent values.
  // For simple numbers/lengths/percents, prepend `-`.
  if let Some(stripped) = value.strip_prefix('-') {
    stripped.to_string()
  } else {
    format!("calc({} * -1)", value)
  }
}

/// Resolve a colour value via the theme `--color-*` namespace, honouring an
/// optional `/<opacity>` modifier.
fn resolve_color(c: &ParsedCandidate, theme: &Theme) -> Option<String> {
  let color_value = if let Some(arb) = &c.arbitrary_value {
    arb.clone()
  } else {
    let v = c.utility_value.as_deref()?;
    theme.resolve(Some(v), &["--color"], ThemeOptions::NONE)?
  };

  match c.modifier.as_deref() {
    None => Some(color_value),
    Some(m) => {
      let alpha = if c.modifier_is_arbitrary {
        // Strip the surrounding `[...]`
        m.trim_start_matches('[').trim_end_matches(']').to_string()
      } else if is_numeric(m) {
        let n: f64 = m.parse().ok()?;
        format!("{}%", trim_float(n))
      } else {
        m.to_string()
      };
      // color-mix is the v4-canonical opacity form.
      Some(format!(
        "color-mix(in oklab, {} {}, transparent)",
        color_value, alpha
      ))
    }
  }
}

/// Resolve a radius value. Supports theme `--radius-*` + arbitrary + the
/// special values used by upstream defaults.
fn resolve_radius(c: &ParsedCandidate, theme: &Theme) -> Option<String> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(arb.clone());
  }
  let v = match c.utility_value.as_deref() {
    None => return Some("0.25rem".into()),
    Some(v) => v,
  };
  match v {
    "none" => Some("0".into()),
    "full" => Some("9999px".into()),
    _ => theme.resolve(Some(v), &["--radius"], ThemeOptions::NONE),
  }
}

// ── spacing-style helpers (just shape declarations from resolve_spacing) ──

fn spacing_decl(c: &ParsedCandidate, theme: &Theme, props: &[&str]) -> Option<Vec<(String, String)>> {
  let v = resolve_spacing(c, theme)?;
  Some(props.iter().map(|p| (p.to_string(), v.clone())).collect())
}

fn sizing_decl(c: &ParsedCandidate, theme: &Theme, props: &[&str]) -> Option<Vec<(String, String)>> {
  let v = resolve_sizing(c, theme)?;
  Some(props.iter().map(|p| (p.to_string(), v.clone())).collect())
}

// Margin
fn handle_m(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin"])
}
fn handle_mx(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-left", "margin-right"])
}
fn handle_my(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-top", "margin-bottom"])
}
fn handle_mt(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-top"])
}
fn handle_mr(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-right"])
}
fn handle_mb(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-bottom"])
}
fn handle_ml(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-left"])
}
fn handle_ms(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-inline-start"])
}
fn handle_me(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["margin-inline-end"])
}

// Padding (no negatives in upstream but we permit the negative flag silently)
fn handle_p(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding"])
}
fn handle_px(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-left", "padding-right"])
}
fn handle_py(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-top", "padding-bottom"])
}
fn handle_pt(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-top"])
}
fn handle_pr(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-right"])
}
fn handle_pb(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-bottom"])
}
fn handle_pl(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-left"])
}
fn handle_ps(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-inline-start"])
}
fn handle_pe(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["padding-inline-end"])
}

// Gap
fn handle_gap(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["gap"])
}
fn handle_gap_x(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["column-gap"])
}
fn handle_gap_y(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["row-gap"])
}

// Inset / top / right / bottom / left
fn handle_inset(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["inset"])
}
fn handle_inset_x(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["inset-inline"])
}
fn handle_inset_y(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["inset-block"])
}
fn handle_top(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["top"])
}
fn handle_right(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["right"])
}
fn handle_bottom(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["bottom"])
}
fn handle_left(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["left"])
}
fn handle_start(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["inset-inline-start"])
}
fn handle_end(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  spacing_decl(c, t, &["inset-inline-end"])
}

// Sizing
fn handle_w(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["width"])
}
fn handle_h(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["height"])
}
fn handle_size(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["width", "height"])
}
fn handle_min_w(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["min-width"])
}
fn handle_min_h(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["min-height"])
}
fn handle_max_w(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["max-width"])
}
fn handle_max_h(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["max-height"])
}

// Colors
fn handle_text(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // `text-<size>` (font-size) takes precedence over color when the value
  // matches a `--text-*` key.
  if let Some(v) = c.utility_value.as_deref() {
    if let Some(size) = t.resolve(Some(v), &["--text"], ThemeOptions::NONE) {
      let mut out = vec![("font-size".to_string(), size)];
      // Pull nested line-height when present.
      if let Some(lh) = t.resolve(
        Some(&format!("{}--line-height", v)),
        &["--text"],
        ThemeOptions::NONE,
      ) {
        out.push(("line-height".to_string(), lh));
      }
      return Some(out);
    }
  }
  let v = resolve_color(c, t)?;
  Some(vec![("color".into(), v)])
}
fn handle_bg(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_color(c, t)?;
  Some(vec![("background-color".into(), v)])
}
fn handle_border(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // `border` (no value) → border-width: 1px
  if c.utility_value.is_none() && c.arbitrary_value.is_none() {
    return Some(vec![
      ("border-width".into(), "1px".into()),
      ("border-style".into(), "solid".into()),
    ]);
  }
  // `border-<n>` → border-width: <n>px when numeric
  if let Some(v) = c.utility_value.as_deref() {
    if is_numeric(v) {
      return Some(vec![
        ("border-width".into(), format!("{}px", v)),
        ("border-style".into(), "solid".into()),
      ]);
    }
  }
  // Otherwise treat as a color.
  let v = resolve_color(c, t)?;
  Some(vec![("border-color".into(), v)])
}
fn handle_fill(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_color(c, t)?;
  Some(vec![("fill".into(), v)])
}
fn handle_stroke(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // numeric → stroke-width
  if let Some(v) = c.utility_value.as_deref() {
    if is_numeric(v) {
      return Some(vec![("stroke-width".into(), v.to_string())]);
    }
  }
  let v = resolve_color(c, t)?;
  Some(vec![("stroke".into(), v)])
}
fn handle_ring(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // numeric / none → ring-width
  if c.utility_value.is_none() && c.arbitrary_value.is_none() {
    return Some(vec![("--tw-ring-width".into(), "3px".into())]);
  }
  if let Some(v) = c.utility_value.as_deref() {
    if is_numeric(v) {
      return Some(vec![("--tw-ring-width".into(), format!("{}px", v))]);
    }
  }
  let v = resolve_color(c, t)?;
  Some(vec![("--tw-ring-color".into(), v)])
}
fn handle_accent(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_color(c, t)?;
  Some(vec![("accent-color".into(), v)])
}
fn handle_caret(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_color(c, t)?;
  Some(vec![("caret-color".into(), v)])
}
fn handle_decoration(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // numeric → text-decoration-thickness
  if let Some(v) = c.utility_value.as_deref() {
    if is_numeric(v) {
      return Some(vec![(
        "text-decoration-thickness".into(),
        format!("{}px", v),
      )]);
    }
  }
  let v = resolve_color(c, t)?;
  Some(vec![("text-decoration-color".into(), v)])
}
fn handle_outline(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  // numeric → outline-width
  if let Some(v) = c.utility_value.as_deref() {
    if is_numeric(v) {
      return Some(vec![("outline-width".into(), format!("{}px", v))]);
    }
  }
  let v = resolve_color(c, t)?;
  Some(vec![("outline-color".into(), v)])
}
fn handle_placeholder(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_color(c, t)?;
  Some(vec![("color".into(), v)])
}

// Border radius
fn handle_rounded(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![("border-radius".into(), v)])
}
fn handle_rounded_t(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![
    ("border-top-left-radius".into(), v.clone()),
    ("border-top-right-radius".into(), v),
  ])
}
fn handle_rounded_r(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![
    ("border-top-right-radius".into(), v.clone()),
    ("border-bottom-right-radius".into(), v),
  ])
}
fn handle_rounded_b(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![
    ("border-bottom-left-radius".into(), v.clone()),
    ("border-bottom-right-radius".into(), v),
  ])
}
fn handle_rounded_l(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![
    ("border-top-left-radius".into(), v.clone()),
    ("border-bottom-left-radius".into(), v),
  ])
}
fn handle_rounded_tl(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![("border-top-left-radius".into(), v)])
}
fn handle_rounded_tr(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![("border-top-right-radius".into(), v)])
}
fn handle_rounded_bl(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![("border-bottom-left-radius".into(), v)])
}
fn handle_rounded_br(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  let v = resolve_radius(c, t)?;
  Some(vec![("border-bottom-right-radius".into(), v)])
}

// Border width side variants
fn border_side_width(c: &ParsedCandidate) -> Option<String> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(arb.clone());
  }
  if c.utility_value.is_none() {
    return Some("1px".into());
  }
  let v = c.utility_value.as_deref()?;
  if is_numeric(v) {
    return Some(format!("{}px", v));
  }
  None
}
fn handle_border_t(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-top-width".into(), w),
    ("border-top-style".into(), "solid".into()),
  ])
}
fn handle_border_r(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-right-width".into(), w),
    ("border-right-style".into(), "solid".into()),
  ])
}
fn handle_border_b(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-bottom-width".into(), w),
    ("border-bottom-style".into(), "solid".into()),
  ])
}
fn handle_border_l(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-left-width".into(), w),
    ("border-left-style".into(), "solid".into()),
  ])
}
fn handle_border_x(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-left-width".into(), w.clone()),
    ("border-right-width".into(), w),
  ])
}
fn handle_border_y(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  let w = border_side_width(c)?;
  Some(vec![
    ("border-top-width".into(), w.clone()),
    ("border-bottom-width".into(), w),
  ])
}

// Opacity
fn handle_opacity(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(vec![("opacity".into(), arb.clone())]);
  }
  let v = c.utility_value.as_deref()?;
  if !is_numeric(v) {
    return None;
  }
  Some(vec![("opacity".into(), format!("{}%", v))])
}

// Z-index
fn handle_z(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(vec![("z-index".into(), arb.clone())]);
  }
  let v = c.utility_value.as_deref()?;
  match v {
    "auto" => Some(vec![("z-index".into(), "auto".into())]),
    _ if is_numeric(v) => {
      let val = if c.negative {
        format!("-{}", v)
      } else {
        v.to_string()
      };
      Some(vec![("z-index".into(), val)])
    }
    _ => None,
  }
}

// Order
fn handle_order(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(vec![("order".into(), arb.clone())]);
  }
  let v = c.utility_value.as_deref()?;
  match v {
    "first" => Some(vec![("order".into(), "-9999".into())]),
    "last" => Some(vec![("order".into(), "9999".into())]),
    "none" => Some(vec![("order".into(), "0".into())]),
    _ if is_numeric(v) => {
      let val = if c.negative {
        format!("-{}", v)
      } else {
        v.to_string()
      };
      Some(vec![("order".into(), val)])
    }
    _ => None,
  }
}

// Flex grow / shrink
fn handle_grow(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(vec![("flex-grow".into(), arb.clone())]);
  }
  let v = c.utility_value.as_deref();
  match v {
    None => Some(vec![("flex-grow".into(), "1".into())]),
    Some(s) if is_numeric(s) => Some(vec![("flex-grow".into(), s.to_string())]),
    _ => None,
  }
}
fn handle_shrink(c: &ParsedCandidate, _t: &Theme) -> Option<Vec<(String, String)>> {
  if let Some(arb) = &c.arbitrary_value {
    return Some(vec![("flex-shrink".into(), arb.clone())]);
  }
  let v = c.utility_value.as_deref();
  match v {
    None => Some(vec![("flex-shrink".into(), "1".into())]),
    Some(s) if is_numeric(s) => Some(vec![("flex-shrink".into(), s.to_string())]),
    _ => None,
  }
}

// Flex basis (sizing scale)
fn handle_basis(c: &ParsedCandidate, t: &Theme) -> Option<Vec<(String, String)>> {
  sizing_decl(c, t, &["flex-basis"])
}
