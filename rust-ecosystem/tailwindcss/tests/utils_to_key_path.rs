//! Ported from upstream `packages/tailwindcss/src/utils/to-key-path.test.ts`.

use farmfe_ecosystem_tailwindcss::utils::to_key_path::to_key_path;

#[test]
fn can_convert_key_paths_to_arrays() {
  assert_eq!(to_key_path("fontSize.xs"), vec!["fontSize", "xs"]);
  assert_eq!(
    to_key_path("fontSize.xs[1].lineHeight"),
    vec!["fontSize", "xs", "1", "lineHeight"]
  );
  assert_eq!(to_key_path("colors.red.500"), vec!["colors", "red", "500"]);
  assert_eq!(
    to_key_path("colors[red].500"),
    vec!["colors", "red", "500"]
  );
  assert_eq!(
    to_key_path("colors[red].[500]"),
    vec!["colors", "red", "500"]
  );
  assert_eq!(
    to_key_path("colors[red]500"),
    vec!["colors", "red", "500"]
  );
  assert_eq!(
    to_key_path("colors[red][500]"),
    vec!["colors", "red", "500"]
  );
  assert_eq!(
    to_key_path("colors[red]500[50]5"),
    vec!["colors", "red", "500", "50", "5"]
  );
}
