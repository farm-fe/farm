use farmfe_docs_config::parse_config;
use farmfe_docs_plan::prepare;
use serde_json::json;
use std::fs;
fn fixture() -> tempfile::TempDir {
  let root = tempfile::tempdir().unwrap();
  fs::create_dir(root.path().join("pages")).unwrap();
  fs::write(root.path().join("pages/index.tsx"),"export const metadata = { title: 'Home', description: 'Standalone page' } as const;\ntype Name = string;\nexport default function Home(){ const name: Name = 'Home'; return <h1 id='home'>{name}</h1>; }\n").unwrap();
  fs::write(
    root.path().join("pages/guide.md"),
    "# Guide\n\n[Home](./index.tsx#home)\n",
  )
  .unwrap();
  fs::write(
    root.path().join("pages/data.js"),
    "export const members = ['Alice'];\n",
  )
  .unwrap();
  root
}
fn plan(root: &std::path::Path) -> Result<farmfe_docs_plan::DocPlan, String> {
  let config=parse_config(&json!({"title":"Pages","url":"https://example.com","basePath":"/guide/","locales":{"default":"en","items":["en","zh"]},"pages":{"dir":"pages"}}).to_string(),root).map_err(|error|error.to_string())?;
  prepare(&config)
}

#[test]
fn page_titles_are_selected_in_rust_from_declared_locale_overrides() {
  let root = fixture();
  fs::write(root.path().join("pages/index.tsx"), "export const metadata={title:'Search the documentation',titleByLocale:{zh:'在文档中搜索'}};export default function Page(){return <h1 id='home'>Search</h1>;}\n").unwrap();
  let prepared = plan(root.path()).unwrap();
  for route in prepared
    .routes
    .iter()
    .filter(|route| route.page.id == "index")
  {
    let title = if route.page.locale == "zh" {
      "在文档中搜索"
    } else {
      "Search the documentation"
    };
    assert_eq!(route.page.title, title);
    assert_eq!(
      prepared.seo[&route.page.key].title.as_deref(),
      Some(format!("{title} | Pages").as_str())
    );
  }
  for source in [
    "export const metadata={titleByLocale:{fr:'Recherche'}};export default ()=>null;",
    "export const metadata={titleByLocale:{zh:readTitle()}};export default ()=>null;",
    "export const metadata={titleByLocale:{zh:'A',zh:'B'}};export default ()=>null;",
    "export const metadata={titleByLocale:{zh:''}};export default ()=>null;",
    "export const metadata={titleByLocale:null};export default ()=>null;",
    "export const metadata={titleByLocale:{...titles}};export default ()=>null;",
    "export const metadata={titleByLocale:{['zh']:'A'}};export default ()=>null;",
    "export const metadata={titleByLocale:{zh:'A\\nB'}};export default ()=>null;",
  ] {
    fs::write(root.path().join("pages/index.tsx"), source).unwrap();
    assert!(
      plan(root.path())
        .unwrap_err()
        .contains("DOCS_PAGE_METADATA")
    );
  }
}

#[test]
fn markdown_pages_use_the_same_declared_locale_title_rules() {
  let root = fixture();
  fs::write(
    root.path().join("pages/guide.md"),
    "---\ntitle: Guide\ntitleByLocale:\n  zh: 中文指南\n---\n# Guide\n",
  )
  .unwrap();
  let prepared = plan(root.path()).unwrap();
  let titles = prepared
    .routes
    .iter()
    .filter(|route| route.page.id == "guide")
    .map(|route| route.page.title.as_str())
    .collect::<Vec<_>>();
  assert_eq!(titles, vec!["Guide", "中文指南"]);
  for field in ["  fr: Other", "  zh: 42", "  zh: ''"] {
    fs::write(
      root.path().join("pages/guide.md"),
      format!("---\ntitleByLocale:\n{field}\n---\n# Guide\n"),
    )
    .unwrap();
    assert!(
      plan(root.path())
        .unwrap_err()
        .contains("DOCS_PAGE_METADATA")
    );
  }
}
#[test]
fn pages_use_native_script_types_and_distinct_locale_routes_without_helpers() {
  let root = fixture();
  let prepared = plan(root.path()).unwrap();
  let mut paths: Vec<_> = prepared
    .routes
    .iter()
    .map(|route| route.page.route_path.as_str())
    .collect();
  paths.sort();
  assert_eq!(
    paths,
    vec!["/guide/", "/guide/guide/", "/guide/zh/", "/guide/zh/guide/"]
  );
  assert!(
    prepared
      .routes
      .iter()
      .all(|route| route.page.kind == "page" && route.page.version.is_none())
  );
  let home = prepared
    .routes
    .iter()
    .position(|route| route.page.route_path == "/guide/")
    .unwrap();
  assert_eq!(prepared.routes[home].page.title, "Home");
  assert!(prepared.modules[home].code.contains("type Name = string"));
  let value = serde_json::to_value(&prepared).unwrap();
  assert_eq!(value["modules"][home]["language"], "tsx");
  assert!(
    prepared
      .modules
      .iter()
      .all(|module| !module.source_path.ends_with("/data.js"))
  );
  assert_eq!(prepared.routes[home].page.alternates.len(), 2);
}
#[test]
fn dynamic_page_metadata_is_rejected_with_original_utf16_location() {
  let root = fixture();
  fs::write(root.path().join("pages/index.tsx"),"const title = 'Home';\nexport const metadata = {title};\nexport default () => <h1>Home</h1>;\n").unwrap();
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("DOCS_PAGE_METADATA") && error.contains("index.tsx:2:"),
    "{error}"
  );
}
#[test]
fn raw_page_syntax_errors_and_ignored_helpers_do_not_execute_javascript() {
  let root = fixture();
  fs::write(
    root.path().join("pages/data.js"),
    "throw new Error('must not execute'); export const members = [];\n",
  )
  .unwrap();
  assert!(plan(root.path()).is_ok());
  fs::write(
    root.path().join("pages/index.tsx"),
    "export default function Home( {\n",
  )
  .unwrap();
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("DOCS_PAGE_PARSE") && error.contains("index.tsx:"),
    "{error}"
  );
}

#[test]
fn default_data_values_and_underscore_components_are_not_routes() {
  let root = fixture();
  fs::write(
    root.path().join("pages/members.js"),
    "const values = ['Alice']; export default values;\n",
  )
  .unwrap();
  fs::write(
    root.path().join("pages/members-named.js"),
    "const members = []; export {members as default};\n",
  )
  .unwrap();
  fs::write(
    root.path().join("pages/_component.tsx"),
    "this is intentionally not parsed\n",
  )
  .unwrap();
  let prepared = plan(root.path()).unwrap();
  assert_eq!(prepared.routes.len(), 4);
}

#[test]
fn expression_metadata_after_an_emoji_reports_utf16_column() {
  let root = fixture();
  fs::write(root.path().join("pages/index.tsx"),"export const metadata = {title: '🙂', slug: compute()};\nexport default () => <h1>Home</h1>;\n").unwrap();
  let error = plan(root.path()).unwrap_err();
  assert!(
    error.contains("index.tsx:1:45") && error.contains("DOCS_PAGE_METADATA"),
    "{error}"
  );
}

#[test]
fn page_layout_and_metadata_require_real_component_exports() {
  let root = fixture();
  fs::write(root.path().join("pages/index.tsx"),"export const metadata = {title:'Home',layout:'none'};\nexport default () => <h1 id='home'>Home</h1>;\n").unwrap();
  let prepared = plan(root.path()).unwrap();
  let value = serde_json::to_value(&prepared).unwrap();
  assert_eq!(
    value["routes"]
      .as_array()
      .unwrap()
      .iter()
      .find(|route| route["page"]["routePath"] == "/guide/")
      .unwrap()["page"]["layout"],
    "none"
  );
  fs::write(
    root.path().join("pages/index.tsx"),
    "export const metadata = {title:'Home'}; export default [];\n",
  )
  .unwrap();
  assert!(plan(root.path()).unwrap_err().contains("DOCS_PAGE_EXPORT"));
}
