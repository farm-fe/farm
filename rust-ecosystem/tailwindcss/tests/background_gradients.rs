use farmfe_ecosystem_tailwindcss::scanner::extract_candidates;
use farmfe_ecosystem_tailwindcss::{compile, parse_candidate, CompilerOptions};

#[test]
fn original_homepage_gradient_survives_scanning_parsing_and_generation() {
  let classes = [
    "bg-gradient-to-r",
    "from-[#ffaa40]",
    "via-[#9c40ff]",
    "to-[#ffaa40]",
  ];
  let source = format!("<span class=\"{}\">Star</span>", classes.join(" "));
  let scanned = extract_candidates(&source);
  for class in classes {
    assert!(
      scanned.contains(&class.to_owned()),
      "scanner lost {class}: {scanned:?}"
    );
    assert!(parse_candidate(class).is_some(), "parser lost {class}");
  }
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(&classes.map(str::to_owned));
  assert!(
    css.contains("background-image: linear-gradient(to right, var(--tw-gradient-stops))"),
    "missing gradient: {css}"
  );
  for color in ["#ffaa40", "#9c40ff"] {
    assert!(css.contains(color), "missing {color}: {css}");
  }
}

#[test]
fn directional_spellings_keep_their_color_interpolation_contract() {
  let directions = [
    ("t", "top"),
    ("tr", "top right"),
    ("r", "right"),
    ("br", "bottom right"),
    ("b", "bottom"),
    ("bl", "bottom left"),
    ("l", "left"),
    ("tl", "top left"),
  ];
  for (suffix, direction) in directions {
    for (prefix, interpolation) in [("bg-gradient-to", ""), ("bg-linear-to", " in oklab")] {
      let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
      let css = compiler.build(&[
        format!("{prefix}-{suffix}"),
        "from-red-500/50".into(),
        "to-blue-500".into(),
      ]);
      assert!(
        css.contains(&format!(
          "linear-gradient(to {direction}{interpolation}, var(--tw-gradient-stops))"
        )),
        "{css}"
      );
      assert!(
        css.contains("color-mix(in oklab,"),
        "stop opacity lost: {css}"
      );
      assert!(
        css.contains("--tw-gradient-via-stops: initial"),
        "parent via stop must be reset: {css}"
      );
      assert!(css.contains("*, ::before, ::after"), "{css}");
      assert!(
        css.contains("--tw-gradient-stops: initial"),
        "composed stops must not inherit: {css}"
      );
    }
  }
}

#[test]
fn arbitrary_backgrounds_use_image_size_position_or_color_properties() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "bg-[linear-gradient(to_bottom,var(--tw-gradient-stops))]",
      "bg-[url(/img/my_image.svg)]",
      "bg-[image:var(--hero)]",
      "bg-[length:var(--bg-size)_100%]",
      "bg-[position:center_top]",
      "bg-[#ff00aa]/50",
      "hover:from-[#112233]",
      "from-10%",
      "via-40%",
      "to-90%",
    ]
    .map(str::to_owned),
  );
  for required in [
    "background-image: linear-gradient(to bottom,var(--tw-gradient-stops))",
    "background-image: url(/img/my_image.svg)",
    "background-image: var(--hero)",
    "background-size: var(--bg-size) 100%",
    "background-position: center top",
    "background-color: color-mix(in oklab, #ff00aa 50%, transparent)",
    "--tw-gradient-from: #112233",
    "--tw-gradient-from-position: 10%",
    "--tw-gradient-via-position: 40%",
    "--tw-gradient-to-position: 90%",
  ] {
    assert!(css.contains(required), "missing {required}: {css}");
  }
  assert!(!css.contains("background-color: linear-gradient"), "{css}");
  assert!(!css.contains("background-color: var(--bg-size)"), "{css}");
}

#[test]
fn unsupported_gradient_modifiers_are_not_silently_approximated() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "bg-linear-to-r/srgb",
      "bg-linear-45",
      "bg-gradient-to-nope",
      "-from-red-500",
      "bg-[image:var(--hero)]/50",
    ]
    .map(str::to_owned),
  );
  assert!(!css.contains("background-image:"), "{css}");
  assert!(!css.contains("--tw-gradient-from:"), "{css}");
}

#[test]
fn arbitrary_gradient_images_compose_with_stop_utilities_without_direction_class() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "bg-[linear-gradient(to_bottom,var(--tw-gradient-stops))]",
      "from-[#ffaa40]",
      "via-[#9c40ff]",
      "to-[#ffaa40]",
    ]
    .map(str::to_owned),
  );
  assert!(
    css.contains("background-image: linear-gradient(to bottom,var(--tw-gradient-stops))"),
    "{css}"
  );
  assert!(
    css.contains("--tw-gradient-stops: var(--tw-gradient-from, transparent)"),
    "arbitrary image needs composed stops too: {css}"
  );
  assert!(
    css.contains("--tw-gradient-via-stops: var(--tw-gradient-via)"),
    "{css}"
  );
}

#[test]
fn gradient_text_emits_clipping_instead_of_only_transparent_text() {
  let clip = parse_candidate("bg-clip-text").unwrap();
  let nodes = farmfe_ecosystem_tailwindcss::utilities::UtilityRegistry::builtin().generate(
    &clip,
    &farmfe_ecosystem_tailwindcss::theme::Theme::with_defaults(),
  );
  assert!(
    !nodes.is_empty(),
    "bg-clip-text must generate clipping declarations"
  );

  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "bg-gradient-to-r",
      "from-[#ffaa40]",
      "to-[#9c40ff]",
      "bg-clip-text",
      "text-transparent",
    ]
    .map(str::to_owned),
  );
  assert!(css.contains("background-image: linear-gradient"), "{css}");
  assert!(
    css.contains("--color-transparent: transparent")
      && css.contains("color: var(--color-transparent)"),
    "{css}"
  );
  assert!(
    css.contains(".bg-clip-text {"),
    "clip utility must have its own rule: {css}"
  );
  assert!(
    css.contains("-webkit-background-clip: text"),
    "WebKit text clipping: {css}"
  );
  assert!(
    css.contains("background-clip: text"),
    "standard text clipping: {css}"
  );
}

#[test]
fn background_clip_box_utilities_and_variants_preserve_their_own_rules() {
  let mut compiler = compile("@tailwind utilities;", CompilerOptions::default()).unwrap();
  let css = compiler.build(
    &[
      "bg-clip-border",
      "bg-clip-padding",
      "bg-clip-content",
      "hover:bg-clip-text!",
    ]
    .map(str::to_owned),
  );
  for value in ["border-box", "padding-box", "content-box"] {
    assert!(css.contains(&format!("background-clip: {value}")), "{css}");
  }
  assert!(css.contains(":hover"), "{css}");
  assert!(
    css.contains("-webkit-background-clip: text !important"),
    "{css}"
  );
  assert!(css.contains("background-clip: text !important"), "{css}");
  assert!(
    !css.contains("background-color:"),
    "clipping must not become color: {css}"
  );
}
