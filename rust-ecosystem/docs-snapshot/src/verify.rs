use crate::{error, hash, read_json, verify_resource, Receipt, ResourceRecord, Target};
use farmfe_docs_plan::DocPlan;
use serde::Deserialize;
use std::{
  collections::BTreeSet,
  fs,
  path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Verification {
  pub browser_receipt: String,
  pub browser_hash: String,
  pub server_receipt: String,
  pub server_hash: String,
  pub writes_path: String,
  pub writes_hash: String,
  pub public_directory: String,
  pub server_directory: String,
  pub template_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Writes {
  schema_version: u32,
  snapshot_id: String,
  pages: Vec<WrittenPage>,
  not_found: Vec<WrittenAuxiliary>,
  report_hash: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WrittenPage {
  key: String,
  output_path: String,
  hash: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WrittenAuxiliary {
  locale: String,
  output_path: String,
  hash: String,
}

fn sealed_json<T: serde::de::DeserializeOwned>(path: &str, expected: &str) -> Result<T, String> {
  let bytes = fs::read(path).map_err(|e| error(format!("{path}: {e}")))?;
  if hash(&bytes) != expected {
    return Err(error(format!("private receipt changed: {path}")));
  }
  serde_json::from_slice(&bytes).map_err(error)
}

fn output_path(root: &Path, name: &str) -> Result<PathBuf, String> {
  if !crate::valid_resource_name(name) {
    return Err(error(format!("invalid resource path {name}")));
  }
  let path = root.join(name);
  let mut parent = path.parent();
  while let Some(directory) = parent {
    let metadata = fs::symlink_metadata(directory).map_err(error)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
      return Err(error(format!(
        "invalid output directory {}",
        directory.display()
      )));
    }
    if directory == root {
      break;
    }
    parent = directory.parent();
  }
  Ok(path)
}

fn verify_hash(path: &Path, expected: &str) -> Result<(), String> {
  let metadata = fs::symlink_metadata(path).map_err(error)?;
  if !metadata.is_file()
    || metadata.file_type().is_symlink()
    || hash(&fs::read(path).map_err(error)?) != expected
  {
    return Err(error(format!("written file changed: {}", path.display())));
  }
  Ok(())
}

fn tree(root: &Path, directory: &Path, names: &mut BTreeSet<String>) -> Result<(), String> {
  let metadata = fs::symlink_metadata(directory).map_err(error)?;
  if !metadata.is_dir() || metadata.file_type().is_symlink() {
    return Err(error(format!(
      "invalid output tree {}",
      directory.display()
    )));
  }
  for entry in fs::read_dir(directory).map_err(error)? {
    let entry = entry.map_err(error)?;
    let kind = entry.file_type().map_err(error)?;
    if kind.is_symlink() {
      return Err(error("output tree contains a symlink"));
    }
    if kind.is_dir() {
      tree(root, &entry.path(), names)?;
    } else if kind.is_file() {
      let path = entry.path();
      let name = path
        .strip_prefix(root)
        .map_err(error)?
        .to_str()
        .ok_or_else(|| error("output path must be UTF-8"))?
        .replace('\\', "/");
      names.insert(name);
    } else {
      return Err(error("output tree contains a non-file entry"));
    }
  }
  Ok(())
}

fn exact_tree(root: &Path, expected: &BTreeSet<String>) -> Result<(), String> {
  let mut actual = BTreeSet::new();
  tree(root, root, &mut actual)?;
  if actual != *expected {
    return Err(error(format!(
      "output membership changed: {}",
      root.display()
    )));
  }
  Ok(())
}

pub fn verify_build(plan: &DocPlan, root: &str, options: &Verification) -> Result<(), String> {
  farmfe_docs_plan::verify(plan)?;
  let browser: Receipt = sealed_json(&options.browser_receipt, &options.browser_hash)?;
  let server: Receipt = sealed_json(&options.server_receipt, &options.server_hash)?;
  browser.validate(&plan.snapshot_id, Target::Browser, root)?;
  server.validate(&plan.snapshot_id, Target::Server, root)?;
  let writes: Writes = sealed_json(&options.writes_path, &options.writes_hash)?;
  if writes.schema_version != 3
    || writes.snapshot_id != plan.snapshot_id
    || writes.pages.len() != plan.routes.len()
    || writes.not_found.len() != plan.not_found.len()
  {
    return Err(error("render writer receipt differs from plan"));
  }
  let public = Path::new(&options.public_directory);
  let server_directory = Path::new(&options.server_directory);
  let manifest_record = browser
    .resources
    .iter()
    .find(|record| record.name == "docs-manifest.json")
    .ok_or_else(|| error("browser manifest absent from receipt"))?;
  let manifest_path = output_path(public, &manifest_record.name)?;
  verify_resource(manifest_record, &manifest_path)?;
  let manifest: serde_json::Value = read_json(&manifest_path)?;
  if manifest.get("notFound")
    != Some(&serde_json::json!(farmfe_docs_plan::not_found_outputs(
      plan
    )))
  {
    return Err(error("manifest auxiliary outputs differ from Rust plan"));
  }
  let template = manifest
    .get("template")
    .and_then(|value| value.as_str())
    .ok_or_else(|| error("browser template absent from manifest"))?;
  if !browser
    .resources
    .iter()
    .any(|record| record.name == template)
  {
    return Err(error("template absent from resource receipt"));
  }
  let mut expected = BTreeSet::new();
  for resource in &browser.resources {
    let file = if resource.name == template {
      PathBuf::from(&options.template_path)
    } else {
      expected.insert(resource.name.clone());
      output_path(public, &resource.name)?
    };
    verify_resource(resource, &file)?;
  }
  let mut written = BTreeSet::new();
  for page in &writes.pages {
    let route = plan
      .routes
      .iter()
      .find(|route| route.page.key == page.key)
      .ok_or_else(|| error("unknown rendered page"))?;
    if route.output_path != page.output_path
      || !written.insert(&page.key)
      || !expected.insert(page.output_path.clone())
    {
      return Err(error("duplicate or conflicting render output"));
    }
    verify_hash(&output_path(public, &page.output_path)?, &page.hash)?;
  }
  for (written, planned) in writes.not_found.iter().zip(&plan.not_found) {
    if written.locale != planned.locale
      || written.output_path != planned.output_path
      || !expected.insert(written.output_path.clone())
    {
      return Err(error("NotFound writer receipt differs from Rust plan"));
    }
    verify_hash(&output_path(public, &written.output_path)?, &written.hash)?;
  }
  if !expected.insert("docs-build-report.json".into()) {
    return Err(error("report collides with compiled output"));
  }
  let report_path = output_path(public, "docs-build-report.json")?;
  verify_hash(&report_path, &writes.report_hash)?;
  let report: serde_json::Value = read_json(&report_path)?;
  let expected_pages = plan.routes.iter().map(|route| serde_json::json!({"key":route.page.key,"routePath":route.page.route_path,"outputPath":route.output_path})).collect::<Vec<_>>();
  let expected_auxiliary = farmfe_docs_plan::not_found_outputs(plan);
  if report.get("schemaVersion") != Some(&serde_json::json!(2))
    || report.get("snapshotId") != Some(&serde_json::json!(plan.snapshot_id))
    || report.get("pages") != Some(&serde_json::json!(expected_pages))
    || report.get("notFound") != Some(&serde_json::json!(expected_auxiliary))
    || report.get("preview") != Some(&serde_json::json!({"basePath":plan.site.base_path}))
    || report.get("diagnostics") != Some(&serde_json::json!(plan.public_diagnostics))
  {
    return Err(error("public report differs from Rust plan"));
  }
  exact_tree(public, &expected)?;
  let mut server_names = BTreeSet::new();
  for resource in &server.resources {
    server_names.insert(resource.name.clone());
    verify_resource(resource, &output_path(server_directory, &resource.name)?)?;
  }
  if !server_names.insert("package.json".into()) {
    return Err(error("server package scope collision"));
  }
  let package = output_path(server_directory, "package.json")?;
  verify_resource(
    &ResourceRecord {
      name: "package.json".into(),
      resource_type: "json".into(),
      length: b"{\"type\":\"commonjs\"}\n".len(),
      hash: hash(b"{\"type\":\"commonjs\"}\n"),
    },
    &package,
  )?;
  exact_tree(server_directory, &server_names)?;
  Ok(())
}
