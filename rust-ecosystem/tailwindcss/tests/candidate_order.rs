use farmfe_ecosystem_tailwindcss::{CompilerOptions, compile};

fn render(candidates: &[&str]) -> String {
  compile("@tailwind utilities;", CompilerOptions::default())
    .expect("valid stylesheet")
    .build(
      &candidates
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>(),
    )
}

#[test]
fn base_rule_always_precedes_its_responsive_override() {
  let css = render(&["md:grid-cols-2", "grid-cols-1"]);
  assert!(
    css.find(".grid-cols-1 {").unwrap() < css.find(".md\\:grid-cols-2").unwrap(),
    "{css}"
  );
}

#[test]
fn default_breakpoints_are_ordered_by_increasing_minimum_width() {
  let css = render(&[
    "2xl:grid-cols-6",
    "lg:grid-cols-4",
    "sm:grid-cols-2",
    "xl:grid-cols-5",
    "md:grid-cols-3",
    "grid-cols-1",
  ]);
  let positions: Vec<_> = [640, 768, 1024, 1280, 1536]
    .iter()
    .map(|width| {
      css
        .find(&format!("@media (min-width: {width}px)"))
        .expect("breakpoint rule")
    })
    .collect();
  assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{css}");
}

#[test]
fn maximum_width_variants_are_ordered_from_broad_to_narrow() {
  let css = render(&[
    "max-sm:grid-cols-1",
    "max-lg:grid-cols-3",
    "max-2xl:grid-cols-5",
    "max-md:grid-cols-2",
    "max-xl:grid-cols-4",
  ]);
  let positions: Vec<_> = [
    "max-2xl:grid-cols-5",
    "max-xl:grid-cols-4",
    "max-lg:grid-cols-3",
    "max-md:grid-cols-2",
    "max-sm:grid-cols-1",
  ]
  .iter()
  .map(|class| {
    css
      .find(&class.replace(':', "\\:"))
      .expect("breakpoint selector")
  })
  .collect();
  assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{css}");
}

#[test]
fn custom_variants_follow_registration_order() {
  let mut compiler = compile(
    "@custom-variant zeta (&:hover); @custom-variant alpha (&:focus); @tailwind utilities;",
    CompilerOptions::default(),
  )
  .expect("custom variants");
  let css = compiler.build(&["alpha:flex".into(), "zeta:flex".into()]);
  assert!(
    css.find(".zeta\\:flex").unwrap() < css.find(".alpha\\:flex").unwrap(),
    "{css}"
  );
}

#[test]
fn duplicate_candidates_do_not_emit_duplicate_rules() {
  assert_eq!(
    render(&["grid", "md:grid-cols-2", "grid", "md:grid-cols-2"]),
    render(&["grid", "md:grid-cols-2"])
  );
}

#[test]
fn candidate_set_permutations_produce_identical_css() {
  let mut candidates = vec![
    "md:hover:block",
    "grid-cols-1",
    "md:grid-cols-2",
    "flex",
    "sm:flex",
    "hover:flex",
  ];
  let expected = render(&candidates);
  for _ in 0..candidates.len() {
    candidates.rotate_left(1);
    assert_eq!(render(&candidates), expected);
  }
  candidates.reverse();
  assert_eq!(render(&candidates), expected);
}
