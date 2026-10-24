use crate::{Receipt, ResourceRecord, Target, error, hash, read_json, verify_resource};
use farmfe_docs_plan::DocPlan;
use serde::Deserialize;
use std::{
  collections::BTreeSet,
  fs,
  path::{Path, PathBuf},
  time::{Duration, Instant},
};

#[derive(Default)]
struct HtmlTimings {
  lookup: Duration,
  parents: Duration,
  leaf: Duration,
  read: Duration,
  sha: Duration,
  bytes: usize,
  files: usize,
  parent_stats: usize,
  minimum: Option<Duration>,
  maximum: Duration,
}

impl HtmlTimings {
  fn merge(&mut self, other: Self) {
    self.lookup += other.lookup;
    self.parents += other.parents;
    self.leaf += other.leaf;
    self.read += other.read;
    self.sha += other.sha;
    self.bytes += other.bytes;
    self.files += other.files;
    self.parent_stats += other.parent_stats;
    if let Some(minimum) = other.minimum {
      self.minimum = Some(self.minimum.map_or(minimum, |old| old.min(minimum)));
    }
    self.maximum = self.maximum.max(other.maximum);
  }
  fn enabled() -> Option<Self> {
    std::env::var("FARM_DOCS_VERIFY_PROFILE")
      .is_ok_and(|value| value == "1")
      .then(Self::default)
  }

  fn verify_hash(&mut self, path: &Path, expected: &str) -> Result<(), String> {
    let started = Instant::now();
    let metadata = fs::symlink_metadata(path).map_err(error)?;
    let leaf_done = Instant::now();
    self.leaf += leaf_done.duration_since(started);
    if !metadata.is_file() || metadata.file_type().is_symlink() {
      return Err(error(format!("written file changed: {}", path.display())));
    }
    let bytes = fs::read(path).map_err(error)?;
    let read_done = Instant::now();
    self.read += read_done.duration_since(leaf_done);
    let matches = hash(&bytes) == expected;
    let sha_done = Instant::now();
    self.sha += sha_done.duration_since(read_done);
    self.bytes += bytes.len();
    drop(bytes);
    if !matches {
      return Err(error(format!("written file changed: {}", path.display())));
    }
    Ok(())
  }

  fn report(&self) {
    let ms = |duration: Duration| duration.as_secs_f64() * 1000.0;
    eprintln!(
      "[FarmDocsVerifyProfile] {{\"component\":\"snapshot.verify_build\",\"stage\":\"rendered_html_breakdown\",\"count\":{},\"bytes\":{},\"parentMetadataCalls\":{},\"leafMetadataCalls\":{},\"readCalls\":{},\"hashCalls\":{},\"lookupSetMs\":{:.3},\"parentMetadataMs\":{:.3},\"leafMetadataMs\":{:.3},\"readMs\":{:.3},\"sha256Ms\":{:.3},\"minimumPageMs\":{:.3},\"maximumPageMs\":{:.3}}}",
      self.files,
      self.bytes,
      self.parent_stats,
      self.files,
      self.files,
      self.files,
      ms(self.lookup),
      ms(self.parents),
      ms(self.leaf),
      ms(self.read),
      ms(self.sha),
      ms(self.minimum.unwrap_or_default()),
      ms(self.maximum)
    );
  }
}

fn html_concurrency() -> Result<usize, String> {
  match std::env::var("FARM_DOCS_VERIFY_CONCURRENCY") {
    Err(std::env::VarError::NotPresent) => Ok(4),
    Ok(value) => match value.as_str() {
      "1" => Ok(1),
      "2" => Ok(2),
      "3" => Ok(3),
      "4" => Ok(4),
      _ => Err(error(
        "FARM_DOCS_VERIFY_CONCURRENCY must be exactly 1, 2, 3 or 4",
      )),
    },
    Err(_) => Err(error("FARM_DOCS_VERIFY_CONCURRENCY must be valid UTF-8")),
  }
}

/// Every started worker is explicitly joined, including partial spawn failure.
/// Job failures do not cancel other jobs; the lowest-index error wins only once
/// all workers have drained. At most `limit` file buffers can be live at once.
fn bounded_html_jobs(
  count: usize,
  limit: usize,
  operation: &(impl Fn(usize) -> Result<Option<HtmlTimings>, String> + Sync),
) -> Result<Vec<Option<HtmlTimings>>, String> {
  std::thread::scope(|scope| {
    let mut handles = vec![];
    let mut spawn_failure = None;
    for worker in 0..limit.min(count) {
      match std::thread::Builder::new()
        .name(format!("docs-html-{worker}"))
        .spawn_scoped(scope, move || {
          (worker..count)
            .step_by(limit)
            .map(|index| {
              let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| operation(index)))
                  .unwrap_or_else(|_| Err(error("HTML verification worker panicked")));
              (index, result)
            })
            .collect::<Vec<_>>()
        }) {
        Ok(handle) => handles.push(handle),
        Err(failure) => {
          spawn_failure = Some(error(format!("start HTML verification worker: {failure}")));
          break;
        }
      }
    }
    let mut results = vec![];
    let mut join_failure = None;
    for handle in handles {
      match handle.join() {
        Ok(rows) => results.extend(rows),
        Err(_) => {
          join_failure = Some(error("HTML verification worker panicked outside job"));
        }
      }
    }
    results.sort_by_key(|(index, _)| *index);
    // All I/O and joins have finished before collecting can return an error.
    let completed = results
      .into_iter()
      .map(|(_, result)| result)
      .collect::<Result<Vec<_>, _>>()?;
    if let Some(failure) = spawn_failure.or(join_failure) {
      return Err(failure);
    }
    Ok(completed)
  })
}

fn verify_parallel_html(
  public: &Path,
  page: &WrittenPage,
  profiled: bool,
) -> Result<Option<HtmlTimings>, String> {
  if !profiled {
    verify_hash(&output_path(public, &page.output_path)?, &page.hash)?;
    return Ok(None);
  }
  let mut timings = HtmlTimings::default();
  let started = Instant::now();
  let file = output_path(public, &page.output_path)?;
  timings.parents = started.elapsed();
  timings.verify_hash(&file, &page.hash)?;
  let total = started.elapsed();
  timings.minimum = Some(total);
  timings.maximum = total;
  timings.files = 1;
  let mut parent = file.parent();
  while let Some(directory) = parent {
    timings.parent_stats += 1;
    if directory == public {
      break;
    }
    parent = directory.parent();
  }
  Ok(Some(timings))
}

#[cfg(test)]
mod bounded_tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};

  #[test]
  fn failed_and_panicking_jobs_drain_every_started_worker_before_returning_first_error() {
    let completed = AtomicUsize::new(0);
    let result = bounded_html_jobs(12, 4, &|index| {
      completed.fetch_add(1, Ordering::SeqCst);
      if index == 2 {
        panic!("test worker panic");
      }
      if index == 0 {
        return Err("first-index-error".into());
      }
      if index == 1 {
        return Err("later-index-error".into());
      }
      Ok(None)
    });
    assert_eq!(completed.load(Ordering::SeqCst), 12);
    assert_eq!(result.err().as_deref(), Some("first-index-error"));
  }
}

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
  let concurrency = html_concurrency()?;
  if std::env::var("FARM_DOCS_VERIFY_PROFILE").is_ok_and(|value| value == "1") {
    eprintln!(
      "[FarmDocsVerifyProfile] {{\"component\":\"snapshot.verify_build\",\"stage\":\"html_io_policy\",\"concurrency\":{concurrency},\"timingMode\":\"worker_intervals_sum_may_overlap_not_wall\"}}"
    );
  }
  let mut profile = farmfe_docs_plan::verify_profile::Timer::new("snapshot.verify_build");
  farmfe_docs_plan::verify(plan)?;
  profile.checkpoint("plan_verify", plan.modules.len());
  let browser: Receipt = sealed_json(&options.browser_receipt, &options.browser_hash)?;
  let server: Receipt = sealed_json(&options.server_receipt, &options.server_hash)?;
  profile.checkpoint("receipt_read_hash_parse", 2);
  browser.validate(&plan.snapshot_id, Target::Browser, root)?;
  profile.checkpoint("browser_sources_observations", browser.sources.len());
  server.validate(&plan.snapshot_id, Target::Server, root)?;
  profile.checkpoint("server_sources_observations", server.sources.len());
  let writes: Writes = sealed_json(&options.writes_path, &options.writes_hash)?;
  if writes.schema_version != 3
    || writes.snapshot_id != plan.snapshot_id
    || writes.pages.len() != plan.routes.len()
    || writes.not_found.len() != plan.not_found.len()
  {
    return Err(error("render writer receipt differs from plan"));
  }
  profile.checkpoint("writes_read_hash_parse_shape", writes.pages.len());
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
  profile.checkpoint("manifest_read_hash_parse_shape", 1);
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
  profile.checkpoint("browser_resource_hashes", browser.resources.len());
  let mut written = BTreeSet::new();
  let mut html_timings = HtmlTimings::enabled();
  if concurrency == 1 {
    for page in &writes.pages {
      let started = html_timings.as_ref().map(|_| Instant::now());
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
      if let (Some(timings), Some(started)) = (&mut html_timings, started) {
        let lookup_done = Instant::now();
        timings.lookup += lookup_done.duration_since(started);
        let file = output_path(public, &page.output_path)?;
        let parents_done = Instant::now();
        timings.parents += parents_done.duration_since(lookup_done);
        timings.verify_hash(&file, &page.hash)?;
        let total = started.elapsed();
        timings.minimum = Some(timings.minimum.map_or(total, |old| old.min(total)));
        timings.maximum = timings.maximum.max(total);
        timings.files += 1;
        // Count the successful helper's exact ancestor calls without introducing
        // extra filesystem queries or changing the helper's validation order.
        let mut parent = file.parent();
        while let Some(directory) = parent {
          timings.parent_stats += 1;
          if directory == public {
            break;
          }
          parent = directory.parent();
        }
      } else {
        verify_hash(&output_path(public, &page.output_path)?, &page.hash)?;
      }
    }
  } else {
    // Structural/namespace checks stay serial and deterministic. Only immutable
    // independent output reads are dispatched, never snapshot or receipt work.
    let started = html_timings.as_ref().map(|_| Instant::now());
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
    }
    if let (Some(timings), Some(started)) = (&mut html_timings, started) {
      timings.lookup = started.elapsed();
    }
    let profiled = html_timings.is_some();
    let rows = bounded_html_jobs(writes.pages.len(), concurrency, &|index| {
      verify_parallel_html(public, &writes.pages[index], profiled)
    })?;
    if let Some(timings) = &mut html_timings {
      for row in rows.into_iter().flatten() {
        timings.merge(row);
      }
    }
  }
  if let Some(timings) = html_timings {
    timings.report();
  }
  profile.checkpoint("rendered_html_hashes", writes.pages.len());
  for (written, planned) in writes.not_found.iter().zip(&plan.not_found) {
    if written.locale != planned.locale
      || written.output_path != planned.output_path
      || !expected.insert(written.output_path.clone())
    {
      return Err(error("NotFound writer receipt differs from Rust plan"));
    }
    verify_hash(&output_path(public, &written.output_path)?, &written.hash)?;
  }
  profile.checkpoint("not_found_html_hashes", writes.not_found.len());
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
  profile.checkpoint("report_read_hash_parse_shape", 1);
  exact_tree(public, &expected)?;
  profile.checkpoint("public_exact_tree", expected.len());
  let mut server_names = BTreeSet::new();
  for resource in &server.resources {
    server_names.insert(resource.name.clone());
    verify_resource(resource, &output_path(server_directory, &resource.name)?)?;
  }
  profile.checkpoint("server_resource_hashes", server.resources.len());
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
  profile.checkpoint("server_package_hash", 1);
  exact_tree(server_directory, &server_names)?;
  profile.checkpoint("server_exact_tree", server_names.len());
  Ok(())
}
