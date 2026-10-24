use farmfe_docs_config::typescript;

#[test]
fn emits_input_and_public_types_from_the_same_rust_schemas() {
  let output = typescript().unwrap();
  assert!(output.ends_with('\n'));
  assert!(!output.ends_with("\n\n"));
  for expected in [
    "export const DOCS_SCHEMA_VERSION = 1 as const;",
    "export type SiteConfig =",
    "export type PublicSiteConfig =",
    "\"schemaVersion\"?: 1;",
    "\"schemaVersion\": 1;",
    "\"locales\": PublicLocales;",
    "export type ContentFormat = \"auto\" | \"markdown\" | \"mdx\";",
  ] {
    assert!(output.contains(expected), "missing {expected}: {output}");
  }
  let public = output
    .split("export type PublicThemeConfig =")
    .nth(1)
    .unwrap()
    .split("\n\n")
    .next()
    .unwrap();
  assert!(!public.contains("components") && !public.contains("customCss"));
  assert_eq!(
    output,
    typescript().unwrap(),
    "generation must be deterministic"
  );
}
