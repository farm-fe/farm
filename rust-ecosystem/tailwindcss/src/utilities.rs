use std::collections::HashMap;

use crate::ast::{AstNode, Declaration, StyleRule};
use crate::candidate::ParsedCandidate;
use crate::theme::Theme;

/// A list of CSS property-value pairs that form a utility class.
type DeclList = Vec<(&'static str, &'static str)>;

/// Registry of built-in static utility classes.
pub struct UtilityRegistry {
  static_utilities: HashMap<&'static str, DeclList>,
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
    m.insert(
      "flex-col-reverse",
      vec![("flex-direction", "column-reverse")],
    );

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
    m.insert(
      "justify-between",
      vec![("justify-content", "space-between")],
    );
    m.insert(
      "justify-around",
      vec![("justify-content", "space-around")],
    );
    m.insert(
      "justify-evenly",
      vec![("justify-content", "space-evenly")],
    );
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
    m.insert(
      "underline",
      vec![("text-decoration-line", "underline")],
    );
    m.insert(
      "overline",
      vec![("text-decoration-line", "overline")],
    );
    m.insert(
      "line-through",
      vec![("text-decoration-line", "line-through")],
    );
    m.insert(
      "no-underline",
      vec![("text-decoration-line", "none")],
    );

    // White space
    m.insert("whitespace-normal", vec![("white-space", "normal")]);
    m.insert("whitespace-nowrap", vec![("white-space", "nowrap")]);
    m.insert("whitespace-pre", vec![("white-space", "pre")]);
    m.insert("whitespace-pre-line", vec![("white-space", "pre-line")]);
    m.insert("whitespace-pre-wrap", vec![("white-space", "pre-wrap")]);
    m.insert("whitespace-break-spaces", vec![("white-space", "break-spaces")]);

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
    m.insert(
      "border-collapse",
      vec![("border-collapse", "collapse")],
    );
    m.insert(
      "border-separate",
      vec![("border-collapse", "separate")],
    );

    // Caption side
    m.insert("caption-top", vec![("caption-side", "top")]);
    m.insert("caption-bottom", vec![("caption-side", "bottom")]);

    // Vertical align
    m.insert("align-baseline", vec![("vertical-align", "baseline")]);
    m.insert("align-top", vec![("vertical-align", "top")]);
    m.insert("align-middle", vec![("vertical-align", "middle")]);
    m.insert("align-bottom", vec![("vertical-align", "bottom")]);
    m.insert(
      "align-text-top",
      vec![("vertical-align", "text-top")],
    );
    m.insert(
      "align-text-bottom",
      vec![("vertical-align", "text-bottom")],
    );
    m.insert("align-sub", vec![("vertical-align", "sub")]);
    m.insert("align-super", vec![("vertical-align", "super")]);

    // Appearance
    m.insert("appearance-none", vec![("appearance", "none")]);
    m.insert("appearance-auto", vec![("appearance", "auto")]);

    // Outline
    m.insert("outline-none", vec![("outline", "2px solid transparent"), ("outline-offset", "2px")]);

    // List style position
    m.insert("list-inside", vec![("list-style-position", "inside")]);
    m.insert("list-outside", vec![("list-style-position", "outside")]);

    // Line clamp
    m.insert(
      "truncate",
      vec![
        ("overflow", "hidden"),
        ("text-overflow", "ellipsis"),
        ("white-space", "nowrap"),
      ],
    );
    m.insert(
      "text-ellipsis",
      vec![("text-overflow", "ellipsis")],
    );
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

    // Uppercase / lowercase / capitalize / normal-case
    m.insert(
      "uppercase",
      vec![("text-transform", "uppercase")],
    );
    m.insert(
      "lowercase",
      vec![("text-transform", "lowercase")],
    );
    m.insert(
      "capitalize",
      vec![("text-transform", "capitalize")],
    );
    m.insert(
      "normal-case",
      vec![("text-transform", "none")],
    );

    // Break
    m.insert("break-normal", vec![("overflow-wrap", "normal"), ("word-break", "normal")]);
    m.insert("break-words", vec![("overflow-wrap", "break-word")]);
    m.insert("break-all", vec![("word-break", "break-all")]);
    m.insert("break-keep", vec![("word-break", "keep-all")]);

    Self {
      static_utilities: m,
    }
  }

  /// Returns `true` if a utility root exists in the registry.
  pub fn has(&self, name: &str) -> bool {
    self.static_utilities.contains_key(name)
  }

  /// Generate CSS AST nodes for a parsed candidate.
  pub fn generate(&self, candidate: &ParsedCandidate, _theme: &Theme) -> Vec<AstNode> {
    // Arbitrary property: [color:red]
    if let Some((ref property, ref value)) = candidate.arbitrary_property {
      let class_name = format!("[{}:{}]", property, value);
      let selector = build_selector(&class_name, &candidate.variants);
      let decl = Declaration {
        property: property.clone(),
        value: Some(value.clone()),
        important: candidate.important,
      };
      return vec![AstNode::Rule(StyleRule {
        selector,
        nodes: vec![AstNode::Declaration(decl)],
      })];
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

    if let Some(declarations) = self
      .static_utilities
      .get(compound_key.as_str())
      .or_else(|| self.static_utilities.get(candidate.utility_root.as_str()))
    {
      let class_name = build_class_name(candidate);
      let selector = build_selector(&class_name, &candidate.variants);

      let nodes: Vec<AstNode> = declarations
        .iter()
        .map(|(prop, val)| {
          AstNode::Declaration(Declaration {
            property: prop.to_string(),
            value: Some(val.to_string()),
            important: candidate.important,
          })
        })
        .collect();

      return vec![AstNode::Rule(StyleRule { selector, nodes })];
    }

    vec![]
  }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn build_class_name(candidate: &ParsedCandidate) -> String {
  match &candidate.arbitrary_value {
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
  }
}

fn build_selector(class_name: &str, variants: &[String]) -> String {
  let escaped = escape_class_name(class_name);

  if variants.is_empty() {
    return format!(".{}", escaped);
  }

  // Build selector with pseudo-class variants
  let mut selector = format!(".{}", escaped);
  for variant in variants.iter().rev() {
    selector = apply_variant(selector, variant);
  }
  selector
}

fn apply_variant(selector: String, variant: &str) -> String {
  match variant {
    "hover" => format!("{}:hover", selector),
    "focus" => format!("{}:focus", selector),
    "active" => format!("{}:active", selector),
    "disabled" => format!("{}:disabled", selector),
    "enabled" => format!("{}:enabled", selector),
    "visited" => format!("{}:visited", selector),
    "checked" => format!("{}:checked", selector),
    "focus-visible" => format!("{}:focus-visible", selector),
    "focus-within" => format!("{}:focus-within", selector),
    "first" => format!("{}:first-child", selector),
    "last" => format!("{}:last-child", selector),
    "odd" => format!("{}:nth-child(odd)", selector),
    "even" => format!("{}:nth-child(2n)", selector),
    "first-of-type" => format!("{}:first-of-type", selector),
    "last-of-type" => format!("{}:last-of-type", selector),
    "only" => format!("{}:only-child", selector),
    "only-of-type" => format!("{}:only-of-type", selector),
    "empty" => format!("{}:empty", selector),
    "required" => format!("{}:required", selector),
    "valid" => format!("{}:valid", selector),
    "invalid" => format!("{}:invalid", selector),
    "placeholder-shown" => format!("{}:placeholder-shown", selector),
    "autofill" => format!("{}:autofill", selector),
    "read-only" => format!("{}:read-only", selector),
    "read-write" => format!("{}:read-write", selector),
    "in-range" => format!("{}:in-range", selector),
    "out-of-range" => format!("{}:out-of-range", selector),
    "indeterminate" => format!("{}:indeterminate", selector),
    "default" => format!("{}:default", selector),
    "optional" => format!("{}:optional", selector),
    "target" => format!("{}:target", selector),
    "open" => format!("{}:open", selector),
    "inert" => format!("{}:inert", selector),
    "before" => format!("{}::before", selector),
    "after" => format!("{}::after", selector),
    "first-letter" => format!("{}::first-letter", selector),
    "first-line" => format!("{}::first-line", selector),
    "marker" => format!("{}::marker", selector),
    "selection" => format!("{}::selection", selector),
    "file" => format!("{}::file-selector-button", selector),
    "placeholder" => format!("{}::placeholder", selector),
    "backdrop" => format!("{}::backdrop", selector),
    _ => selector,
  }
}

fn escape_class_name(name: &str) -> String {
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
