use farmfe_ecosystem_tailwindcss::ast::to_css;
use farmfe_ecosystem_tailwindcss::candidate::{parse_candidate, ParsedCandidate};
use farmfe_ecosystem_tailwindcss::theme::Theme;
use farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry;

fn make_static(name: &str) -> ParsedCandidate {
  ParsedCandidate {
    utility_root: name.to_string(),
    utility_value: None,
    arbitrary_property: None,
    arbitrary_value: None,
    type_hint: None,
    variants: vec![],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    negative: false,
    is_static: true,
    raw: name.to_string(),
  }
}

fn render(input: &str, theme: &Theme) -> String {
  let registry = UtilityRegistry::builtin();
  let c = parse_candidate(input).expect("parse");
  let result = registry.generate(&c, theme);
  to_css(&result)
}

#[test]
fn test_flex_utility() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("flex");
  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert!(!output.is_empty());
  assert!(output.contains("display: flex"));
}

#[test]
fn test_display_block_utility() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("block");
  let result = registry.generate(&candidate, &theme);
  let output = to_css(&result);
  assert_eq!(output.trim(), ".block {\n  display: block;\n}");
}

#[test]
fn test_unknown_utility_returns_empty() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = make_static("nonexistent-utility-xyz");
  let result = registry.generate(&candidate, &theme);
  assert!(result.is_empty());
}

#[test]
fn test_arbitrary_property() {
  let registry = UtilityRegistry::builtin();
  let theme = Theme::default();
  let candidate = ParsedCandidate {
    utility_root: String::new(),
    utility_value: None,
    arbitrary_property: Some(("color".to_string(), "red".to_string())),
    arbitrary_value: None,
    type_hint: None,
    variants: vec![],
    modifier: None,
    modifier_is_arbitrary: false,
    important: false,
    negative: false,
    is_static: false,
    raw: "[color:red]".to_string(),
  };
  let result = registry.generate(&candidate, &theme);
  assert!(!result.is_empty());
  let output = to_css(&result);
  assert!(output.contains("color: red"));
}

// ── Phase 16: functional utilities ────────────────────────────────────────

#[test]
fn margin_basic() {
  let t = Theme::with_defaults();
  let css = render("m-4", &t);
  assert!(css.contains("margin: calc(var(--spacing) * 4)"));
}

#[test]
fn margin_auto() {
  let t = Theme::with_defaults();
  let css = render("m-auto", &t);
  assert!(css.contains("margin: auto"));
}

#[test]
fn margin_px() {
  let t = Theme::with_defaults();
  let css = render("m-px", &t);
  assert!(css.contains("margin: 1px"));
}

#[test]
fn margin_top_negative() {
  let t = Theme::with_defaults();
  let css = render("-mt-4", &t);
  assert!(css.contains("margin-top: calc(var(--spacing) * -4)"));
}

#[test]
fn margin_x_emits_left_and_right() {
  let t = Theme::with_defaults();
  let css = render("mx-2", &t);
  assert!(css.contains("margin-left: calc(var(--spacing) * 2)"));
  assert!(css.contains("margin-right: calc(var(--spacing) * 2)"));
}

#[test]
fn padding_y() {
  let t = Theme::with_defaults();
  let css = render("py-3", &t);
  assert!(css.contains("padding-top: calc(var(--spacing) * 3)"));
  assert!(css.contains("padding-bottom: calc(var(--spacing) * 3)"));
}

#[test]
fn gap_arbitrary() {
  let t = Theme::with_defaults();
  let css = render("gap-[10px]", &t);
  assert!(css.contains("gap: 10px"), "got: {}", css);
}

#[test]
fn width_full() {
  let t = Theme::with_defaults();
  let css = render("w-full", &t);
  assert!(css.contains("width: 100%"));
}

#[test]
fn width_fraction() {
  let t = Theme::with_defaults();
  let css = render("w-1/2", &t);
  assert!(css.contains("width: 50%"));
}

#[test]
fn height_screen() {
  let t = Theme::with_defaults();
  let css = render("h-screen", &t);
  assert!(css.contains("height: 100vh"), "got: {}", css);
}

#[test]
fn size_emits_both_dims() {
  let t = Theme::with_defaults();
  let css = render("size-8", &t);
  assert!(css.contains("width: calc(var(--spacing) * 8)"));
  assert!(css.contains("height: calc(var(--spacing) * 8)"));
}

#[test]
fn bg_color_red_500_uses_var() {
  let t = Theme::with_defaults();
  let css = render("bg-red-500", &t);
  assert!(
    css.contains("background-color: var(--color-red-500)"),
    "got: {}",
    css
  );
}

#[test]
fn text_color_blue_600() {
  let t = Theme::with_defaults();
  let css = render("text-blue-600", &t);
  assert!(css.contains("color: var(--color-blue-600)"), "got: {}", css);
}

#[test]
fn text_font_size_takes_priority_over_color() {
  let t = Theme::with_defaults();
  let css = render("text-lg", &t);
  assert!(css.contains("font-size: var(--text-lg)"), "got: {}", css);
  assert!(
    css.contains("line-height: var(--text-lg--line-height)"),
    "got: {}",
    css
  );
}

#[test]
fn color_with_opacity_modifier() {
  let t = Theme::with_defaults();
  let css = render("bg-red-500/50", &t);
  assert!(
    css.contains("color-mix(in oklab, var(--color-red-500) 50%, transparent)"),
    "got: {}",
    css
  );
}

#[test]
fn border_default_is_1px() {
  let t = Theme::with_defaults();
  let css = render("border", &t);
  assert!(css.contains("border-width: 1px"));
  assert!(css.contains("border-style: solid"));
}

#[test]
fn border_numeric_width() {
  let t = Theme::with_defaults();
  let css = render("border-2", &t);
  assert!(css.contains("border-width: 2px"), "got: {}", css);
}

#[test]
fn border_color() {
  let t = Theme::with_defaults();
  let css = render("border-red-500", &t);
  assert!(
    css.contains("border-color: var(--color-red-500)"),
    "got: {}",
    css
  );
}

#[test]
fn rounded_default() {
  let t = Theme::with_defaults();
  let css = render("rounded", &t);
  assert!(css.contains("border-radius: 0.25rem"), "got: {}", css);
}

#[test]
fn rounded_lg() {
  let t = Theme::with_defaults();
  let css = render("rounded-lg", &t);
  assert!(
    css.contains("border-radius: var(--radius-lg)"),
    "got: {}",
    css
  );
}

#[test]
fn rounded_t_lg_emits_top_corners() {
  let t = Theme::with_defaults();
  let css = render("rounded-t-lg", &t);
  assert!(css.contains("border-top-left-radius: var(--radius-lg)"));
  assert!(css.contains("border-top-right-radius: var(--radius-lg)"));
}

#[test]
fn rounded_full() {
  let t = Theme::with_defaults();
  let css = render("rounded-full", &t);
  assert!(css.contains("border-radius: 9999px"), "got: {}", css);
}

#[test]
fn list_style_type_disc() {
  let t = Theme::with_defaults();
  let css = render("list-disc", &t);
  assert!(css.contains("list-style-type: disc"), "got: {}", css);
}

#[test]
fn line_clamp_numeric() {
  let t = Theme::with_defaults();
  let css = render("line-clamp-3", &t);
  assert!(css.contains("overflow: hidden"), "got: {}", css);
  assert!(css.contains("display: -webkit-box"), "got: {}", css);
  assert!(css.contains("-webkit-box-orient: vertical"), "got: {}", css);
  assert!(css.contains("-webkit-line-clamp: 3"), "got: {}", css);
}

#[test]
fn columns_numeric() {
  let t = Theme::with_defaults();
  let css = render("columns-3", &t);
  assert!(css.contains("columns: 3"), "got: {}", css);
}

#[test]
fn aspect_video_uses_default_theme_token() {
  let t = Theme::with_defaults();
  let css = render("aspect-video", &t);
  assert!(
    css.contains("aspect-ratio: var(--aspect-video)"),
    "got: {}",
    css
  );
}

#[test]
fn opacity_numeric() {
  let t = Theme::with_defaults();
  let css = render("opacity-50", &t);
  assert!(css.contains("opacity: 50%"), "got: {}", css);
}

#[test]
fn z_index_positive_and_negative() {
  let t = Theme::with_defaults();
  let css = render("z-10", &t);
  assert!(css.contains("z-index: 10"));
  let css = render("-z-10", &t);
  assert!(css.contains("z-index: -10"), "got: {}", css);
}

#[test]
fn z_index_auto() {
  let t = Theme::with_defaults();
  let css = render("z-auto", &t);
  assert!(css.contains("z-index: auto"));
}

#[test]
fn inset_x_emits_inset_inline() {
  let t = Theme::with_defaults();
  let css = render("inset-x-0", &t);
  assert!(css.contains("inset-inline: calc(var(--spacing) * 0)"));
}

#[test]
fn order_first_last_none() {
  let t = Theme::with_defaults();
  assert!(render("order-first", &t).contains("order: -9999"));
  assert!(render("order-last", &t).contains("order: 9999"));
  assert!(render("order-none", &t).contains("order: 0"));
  assert!(render("order-3", &t).contains("order: 3"));
}

#[test]
fn flex_grow_shrink_defaults() {
  let t = Theme::with_defaults();
  assert!(render("grow", &t).contains("flex-grow: 1"));
  assert!(render("grow-0", &t).contains("flex-grow: 0"));
  assert!(render("shrink", &t).contains("flex-shrink: 1"));
  assert!(render("shrink-0", &t).contains("flex-shrink: 0"));
}

#[test]
fn fill_and_stroke() {
  let t = Theme::with_defaults();
  assert!(render("fill-red-500", &t).contains("fill: var(--color-red-500)"));
  assert!(render("stroke-2", &t).contains("stroke-width: 2"));
  assert!(render("stroke-blue-500", &t).contains("stroke: var(--color-blue-500)"));
}

#[test]
fn ring_default_and_color() {
  let t = Theme::with_defaults();
  assert!(render("ring", &t).contains("--tw-ring-width: 3px"));
  assert!(render("ring-2", &t).contains("--tw-ring-width: 2px"));
  assert!(render("ring-red-500", &t).contains("--tw-ring-color: var(--color-red-500)"));
}

#[test]
fn decoration_numeric_and_color() {
  let t = Theme::with_defaults();
  assert!(render("decoration-2", &t).contains("text-decoration-thickness: 2px"));
  assert!(
    render("decoration-blue-500", &t).contains("text-decoration-color: var(--color-blue-500)")
  );
}

#[test]
fn arbitrary_bg_color() {
  let t = Theme::with_defaults();
  let css = render("bg-[#0088cc]", &t);
  assert!(css.contains("background-color: #0088cc"), "got: {}", css);
}

#[test]
fn unknown_root_returns_empty() {
  let t = Theme::with_defaults();
  let css = render("totally-unknown-utility-xyz", &t);
  assert!(css.is_empty(), "got: {}", css);
}

#[test]
fn spacing_decimal() {
  let t = Theme::with_defaults();
  let css = render("m-1.5", &t);
  assert!(
    css.contains("margin: calc(var(--spacing) * 1.5)"),
    "got: {}",
    css
  );
}

#[test]
fn padding_inline_start_end() {
  let t = Theme::with_defaults();
  assert!(render("ps-4", &t).contains("padding-inline-start: calc(var(--spacing) * 4)"));
  assert!(render("pe-4", &t).contains("padding-inline-end: calc(var(--spacing) * 4)"));
}

#[test]
fn rounded_corner_individual() {
  let t = Theme::with_defaults();
  assert!(render("rounded-tl-lg", &t).contains("border-top-left-radius: var(--radius-lg)"));
  assert!(render("rounded-br-md", &t).contains("border-bottom-right-radius: var(--radius-md)"));
}

#[test]
fn border_side_widths() {
  let t = Theme::with_defaults();
  assert!(render("border-t", &t).contains("border-top-width: 1px"));
  assert!(render("border-t-2", &t).contains("border-top-width: 2px"));
  assert!(render("border-x-4", &t).contains("border-left-width: 4px"));
  assert!(render("border-x-4", &t).contains("border-right-width: 4px"));
}
