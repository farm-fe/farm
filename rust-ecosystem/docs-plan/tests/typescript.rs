#[test]
fn generates_plan_and_config_types_without_duplicate_definitions() {
  let output = farmfe_docs_config::typescript_with_schemas(vec![(
    "DocPlan".into(),
    farmfe_docs_plan::schema(),
  )])
  .unwrap();
  assert!(output.contains("export type DocPlan ="));
  assert!(output.contains("export type PublicPage ="));
  assert_eq!(output.matches("export type PublicSiteConfig =").count(), 1);
  assert!(output.contains("\"outputPath\": string;"));
}
