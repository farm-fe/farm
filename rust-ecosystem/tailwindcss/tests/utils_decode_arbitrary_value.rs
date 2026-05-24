//! Verbatim port of upstream
//! `packages/tailwindcss/src/utils/decode-arbitrary-value.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::decode_arbitrary_value::decode_arbitrary_value;

// ----- "decoding arbitrary values" describe -----------------------------------

#[test]
fn replace_an_underscore_with_a_space() {
  assert_eq!(decode_arbitrary_value("foo_bar"), "foo bar");
}

#[test]
fn replace_multiple_underscores_with_spaces() {
  assert_eq!(decode_arbitrary_value("__foo__bar__"), "  foo  bar  ");
}

#[test]
fn replace_escaped_underscores_with_a_normal_underscore() {
  assert_eq!(decode_arbitrary_value("foo\\_bar"), "foo_bar");
}

#[test]
fn does_not_replace_underscores_in_url() {
  assert_eq!(
    decode_arbitrary_value("url(./my_file.jpg)"),
    "url(./my_file.jpg)"
  );
  assert_eq!(
    decode_arbitrary_value("no-repeat_url(./my_file.jpg)"),
    "no-repeat url(./my_file.jpg)"
  );
}

#[test]
fn does_not_replace_underscores_in_first_var_arg() {
  assert_eq!(
    decode_arbitrary_value("var(--spacing-1_5)"),
    "var(--spacing-1_5)"
  );
  assert_eq!(
    decode_arbitrary_value("var(--spacing-1_5,_1rem)"),
    "var(--spacing-1_5, 1rem)"
  );
  assert_eq!(
    decode_arbitrary_value("var(--spacing-1_5,_var(--spacing-2_5,_1rem))"),
    "var(--spacing-1_5, var(--spacing-2_5, 1rem))"
  );
}

#[test]
fn does_not_replace_underscores_in_first_theme_arg() {
  assert_eq!(
    decode_arbitrary_value("theme(--spacing-1_5)"),
    "theme(--spacing-1_5)"
  );
  assert_eq!(
    decode_arbitrary_value("theme(--spacing-1_5,_1rem)"),
    "theme(--spacing-1_5, 1rem)"
  );
  assert_eq!(
    decode_arbitrary_value("theme(--spacing-1_5,_theme(--spacing-2_5,_1rem))"),
    "theme(--spacing-1_5, theme(--spacing-2_5, 1rem))"
  );
}

#[test]
fn leaves_var_as_is() {
  assert_eq!(decode_arbitrary_value("var(--foo)"), "var(--foo)");
  assert_eq!(
    decode_arbitrary_value("var(--headings-h1-size)"),
    "var(--headings-h1-size)"
  );
}

// ----- "adds spaces around math operators" describe ---------------------------

macro_rules! math_case {
  ($name:ident, $input:expr, $output:expr) => {
    #[test]
    fn $name() {
      assert_eq!(decode_arbitrary_value($input), $output);
    }
  };
}

math_case!(math_calc_1p2, "calc(1+2)", "calc(1 + 2)");
math_case!(math_calc_pct_rem, "calc(100%+1rem)", "calc(100% + 1rem)");
math_case!(
  math_nested_calc_pct_20px,
  "calc(1+calc(100%-20px))",
  "calc(1 + calc(100% - 20px))"
);
math_case!(
  math_calc_var_times_100,
  "calc(var(--headings-h1-size)*100)",
  "calc(var(--headings-h1-size) * 100)"
);
math_case!(
  math_calc_var_times_calc,
  "calc(var(--headings-h1-size)*calc(100%+50%))",
  "calc(var(--headings-h1-size) * calc(100% + 50%))"
);
math_case!(math_min, "min(1+2)", "min(1 + 2)");
math_case!(math_max, "max(1+2)", "max(1 + 2)");
math_case!(
  math_clamp,
  "clamp(1+2,1+3,1+4)",
  "clamp(1 + 2, 1 + 3, 1 + 4)"
);
math_case!(
  math_var_with_calc,
  "var(--width, calc(100%+1rem))",
  "var(--width, calc(100% + 1rem))"
);
math_case!(
  math_calc_1px_times_grouping,
  "calc(1px*(7--12/24))",
  "calc(1px * (7 - -12 / 24))"
);
math_case!(
  math_grouping_division_7_32,
  "calc((7-32)/(1400-782))",
  "calc((7 - 32) / (1400 - 782))"
);
math_case!(
  math_grouping_division_7_3,
  "calc((7-3)/(1400-782))",
  "calc((7 - 3) / (1400 - 782))"
);
math_case!(
  math_grouping_division_70_3,
  "calc((70-3)/(1400-782))",
  "calc((70 - 3) / (1400 - 782))"
);
math_case!(
  math_grouping_division_70_32,
  "calc((70-32)/(1400-782))",
  "calc((70 - 32) / (1400 - 782))"
);
math_case!(
  math_grouping_division_704_3,
  "calc((704-3)/(1400-782))",
  "calc((704 - 3) / (1400 - 782))"
);
math_case!(math_grouping_simple, "calc((704-320))", "calc((704 - 320))");
math_case!(
  math_grouping_div_1,
  "calc((704-320)/1)",
  "calc((704 - 320) / 1)"
);
math_case!(
  math_grouping_division_full,
  "calc((704-320)/(1400-782))",
  "calc((704 - 320) / (1400 - 782))"
);
math_case!(math_calc_neg_rem, "calc(24px+-1rem)", "calc(24px + -1rem)");
math_case!(
  math_calc_neg_paren,
  "calc(24px+(-1rem))",
  "calc(24px + (-1rem))"
);
math_case!(
  math_calc_neg_rem_underscored,
  "calc(24px_+_-1rem)",
  "calc(24px + -1rem)"
);
math_case!(
  math_calc_neg_paren_underscored,
  "calc(24px_+_(-1rem))",
  "calc(24px + (-1rem))"
);
math_case!(
  math_calc_nested_var_calc,
  "calc(var(--10-10px,calc(-20px-(-30px--40px)-50px)))",
  "calc(var(--10-10px,calc(-20px - (-30px - -40px) - 50px)))"
);
math_case!(
  math_calc_theme_bar,
  "calc(theme(spacing.1-bar))",
  "calc(theme(spacing.1-bar))"
);
math_case!(
  theme_bar_top,
  "theme(spacing.1-bar)",
  "theme(spacing.1-bar)"
);
math_case!(
  math_calc_1rem_minus_theme,
  "calc(1rem-theme(spacing.1-bar))",
  "calc(1rem - theme(spacing.1-bar))"
);
math_case!(
  math_calc_theme_foo_2,
  "calc(theme(spacing.foo-2))",
  "calc(theme(spacing.foo-2))"
);
math_case!(
  math_calc_theme_foo_bar,
  "calc(theme(spacing.foo-bar))",
  "calc(theme(spacing.foo-bar))"
);

math_case!(
  math_pct_with_var,
  "calc(100%-var(--foo))",
  "calc(100% - var(--foo))"
);

math_case!(
  math_uppercase_units,
  "calc(100PX-theme(spacing.1))",
  "calc(100PX - theme(spacing.1))"
);

math_case!(
  math_min_fitcontent_calc,
  "min(fit-content,calc(100dvh-4rem))",
  "min(fit-content, calc(100dvh - 4rem))"
);
math_case!(
  math_min_theme_fitcontent_calc,
  "min(theme(spacing.foo-bar),fit-content,calc(20*calc(40-30)))",
  "min(theme(spacing.foo-bar), fit-content, calc(20 * calc(40 - 30)))"
);
math_case!(
  math_min_complex,
  "min(fit-content,calc(100dvh-4rem)-calc(50dvh--2px))",
  "min(fit-content, calc(100dvh - 4rem) - calc(50dvh - -2px))"
);
math_case!(
  math_min_scientific,
  "min(-3.4e-2-var(--foo),calc-size(auto))",
  "min(-3.4e-2 - var(--foo), calc-size(auto))"
);
math_case!(
  math_clamp_scientific,
  "clamp(-10e3-var(--foo),calc-size(max-content),var(--foo)+-10e3)",
  "clamp(-10e3 - var(--foo), calc-size(max-content), var(--foo) + -10e3)"
);

math_case!(
  math_clamp_neg_no_space,
  "clamp(-3px+4px,-3px+4px,-3px+4px)",
  "clamp(-3px + 4px, -3px + 4px, -3px + 4px)"
);

math_case!(
  math_no_format_in_var,
  "calc(var(--foo-bar-bar)*2)",
  "calc(var(--foo-bar-bar) * 2)"
);

math_case!(
  math_no_format_in_env,
  "calc(env(safe-area-inset-bottom)*2)",
  "calc(env(safe-area-inset-bottom) * 2)"
);

math_case!(
  math_dashed_idents,
  "fit-content(min(max-content,max(min-content,calc(20px+1em))))",
  "fit-content(min(max-content, max(min-content, calc(20px + 1em))))"
);

math_case!(
  math_env_with_calc_arg,
  "env(safe-area-inset-bottom,calc(10px+20px))",
  "env(safe-area-inset-bottom,calc(10px + 20px))"
);

math_case!(
  math_calc_env_with_calc_arg,
  "calc(env(safe-area-inset-bottom,calc(10px+20px))+5px)",
  "calc(env(safe-area-inset-bottom,calc(10px + 20px)) + 5px)"
);

math_case!(
  math_minmax,
  "minmax(min-content,25%)",
  "minmax(min-content,25%)"
);

math_case!(
  math_radial_gradients,
  "radial-gradient(calc(1+2)),radial-gradient(calc(1+2))",
  "radial-gradient(calc(1 + 2)),radial-gradient(calc(1 + 2))"
);
math_case!(
  math_anchor_size,
  "w-[calc(anchor-size(width)+8px)]",
  "w-[calc(anchor-size(width) + 8px)]"
);
math_case!(
  math_anchor_size_nested,
  "w-[calc(anchor-size(foo(bar))+8px)]",
  "w-[calc(anchor-size(foo(bar)) + 8px)]"
);
math_case!(
  math_content_start_minmax,
  "[content-start]_calc(100%-1px)_[content-end]_minmax(1rem,1fr)",
  "[content-start] calc(100% - 1px) [content-end] minmax(1rem,1fr)"
);

math_case!(math_round_1, "round(1+2,1+3)", "round(1 + 2, 1 + 3)");
math_case!(
  math_round_2,
  "round(to-zero,1+2,1+3)",
  "round(to-zero, 1 + 2, 1 + 3)"
);

math_case!(
  math_env_nested_parens,
  "env((safe-area-inset-bottom))",
  "env((safe-area-inset-bottom))"
);

math_case!(
  math_neg_infinity,
  "atan(1 + -infinity)",
  "atan(1 + -infinity)"
);
