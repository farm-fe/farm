//! Versioned plan metadata and immutable content leases supplied by one host.
//!
//! The wire plan deliberately omits payloads and is never verified as a plan.
//! Consumers first hydrate every payload, then use the ordinary full verifier.
use std::{
  collections::BTreeMap,
  fs,
  io::{Read, Write},
  path::Path,
  sync::Arc,
};

use farmfe_core::context::shared_resources::SharedResourceProvider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::DocPlan;

/// Explicit namespace; unrelated providers cannot accidentally supply a Docs plan.
pub const NAMESPACE: &str = "farm:docs-prepared-plan:v1";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModulePayloads {
  key: String,
  code: String,
  source_map: Option<String>,
  machine_markdown: Option<String>,
  generated_source: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanWire {
  wire_schema_version: u32,
  plan_path: String,
  plan_file_hash: String,
  plan: DocPlan,
  payloads: Vec<ModulePayloads>,
}

struct DigestWriter(Sha256);
impl Write for DigestWriter {
  fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
    self.0.update(bytes);
    Ok(bytes.len())
  }
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

fn plan_bytes_hash(plan: &DocPlan) -> Result<String, String> {
  let mut digest = DigestWriter(Sha256::new());
  {
    let mut writer = std::io::BufWriter::with_capacity(65536, &mut digest);
    serde_json::to_writer(&mut writer, plan).map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())?;
  }
  Ok(format!("{:x}", digest.0.finalize()))
}

fn file_hash(path: &Path) -> Result<String, String> {
  let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
  let mut digest = Sha256::new();
  let mut buffer = [0; 65536];
  loop {
    let length = file.read(&mut buffer).map_err(|error| error.to_string())?;
    if length == 0 {
      break;
    }
    digest.update(&buffer[..length]);
  }
  Ok(format!("{:x}", digest.finalize()))
}

fn canonical_key(path: &Path) -> Result<String, String> {
  let canonical =
    fs::canonicalize(path).map_err(|error| format!("[DOCS_SHARED_PLAN_PATH] {error}"))?;
  let value = canonical
    .to_str()
    .ok_or("[DOCS_SHARED_PLAN_PATH] non UTF-8 path")?
    .replace('\\', "/");
  if let Some(unc) = value.strip_prefix("//?/UNC/") {
    return Ok(format!("//{unc}"));
  }
  Ok(value.strip_prefix("//?/").unwrap_or(&value).to_owned())
}

fn manifest_key(path: &str) -> String {
  format!("plan:{path}")
}

/// Canonical host plan and its immutable payloads. Target views only lease Arcs.
pub struct PlanResources {
  plan: Arc<DocPlan>,
  path: String,
  file_hash: String,
  wire: Arc<str>,
  blobs: BTreeMap<String, Arc<str>>,
}

impl PlanResources {
  /// Bind a plan to the already-written private artifact, failing on drift.
  pub fn new(plan: DocPlan, path: &Path) -> Result<Self, String> {
    let path_key = canonical_key(path)?;
    let expected = plan_bytes_hash(&plan)?;
    if file_hash(path)? != expected {
      return Err("[DOCS_SHARED_PLAN_FILE] private plan differs".into());
    }
    let mut view = plan.clone();
    let mut blobs = BTreeMap::new();
    let mut payloads = Vec::with_capacity(plan.modules.len());
    for module in &mut view.modules {
      let prefix = format!("{}:{}", plan.snapshot_id, module.key);
      let code = format!("{prefix}:code");
      blobs.insert(code.clone(), module.code.clone());
      module.code = Arc::from("");
      let mut put = |name: &str, field: &mut Option<Arc<str>>| {
        field.take().map(|value| {
          let key = format!("{prefix}:{name}");
          blobs.insert(key.clone(), value);
          key
        })
      };
      payloads.push(ModulePayloads {
        key: module.key.clone(),
        code,
        source_map: put("map", &mut module.source_map),
        machine_markdown: put("markdown", &mut module.machine_markdown),
        generated_source: put("generated", &mut module.generated_source),
      });
    }
    let wire = serde_json::to_string(&PlanWire {
      wire_schema_version: 1,
      plan_path: path_key.clone(),
      plan_file_hash: expected.clone(),
      plan: view,
      payloads,
    })
    .map_err(|error| error.to_string())?
    .into();
    Ok(Self {
      plan: Arc::new(plan),
      path: path_key,
      file_hash: expected,
      wire,
      blobs,
    })
  }

  /// Borrow the canonical complete plan for native prepare/verification services.
  pub fn plan(&self) -> &DocPlan {
    &self.plan
  }

  /// Recheck the private artifact rather than silently ignoring tampering.
  pub fn verify_private_file(&self) -> Result<(), String> {
    if file_hash(Path::new(&self.path))? != self.file_hash {
      return Err("[DOCS_SHARED_PLAN_FILE] private plan changed".into());
    }
    Ok(())
  }

  /// Small workflow-facing metadata, without compiled content or private inputs.
  pub fn metadata(&self) -> Result<String, String> {
    let p = &self.plan;
    serde_json::to_string(&serde_json::json!({
      "snapshotId": p.snapshot_id, "site": p.site, "seo": p.seo,
      "routes": p.routes, "notFound": p.not_found, "ssg": p.ssg,
      "publicDiagnostics": p.public_diagnostics
    }))
    .map_err(|error| error.to_string())
  }
}

impl SharedResourceProvider for PlanResources {
  fn get(&self, namespace: &str, key: &str) -> farmfe_core::error::Result<Option<Arc<str>>> {
    if namespace != NAMESPACE {
      return Ok(None);
    }
    if key == manifest_key(&self.path) {
      return Ok(Some(self.wire.clone()));
    }
    Ok(self.blobs.get(key).cloned())
  }
}

/// Read a legacy plan or hydrate a host view. Callers retain ordinary `verify`.
/// An explicitly supplied but wrong provider is an error, never a silent fallback.
pub fn read_plan(
  path: &Path,
  provider: Option<&dyn SharedResourceProvider>,
) -> Result<DocPlan, String> {
  let Some(provider) = provider else {
    let bytes = fs::read(path).map_err(|error| format!("[DOCS_PLAN_READ] {error}"))?;
    return serde_json::from_slice(&bytes).map_err(|error| format!("[DOCS_PLAN_PARSE] {error}"));
  };
  let path_key = canonical_key(path)?;
  let wire = provider
    .get(NAMESPACE, &manifest_key(&path_key))
    .map_err(|error| error.to_string())?
    .ok_or("[DOCS_SHARED_PLAN_PROVIDER] snapshot does not own this plan")?;
  let mut frame: PlanWire =
    serde_json::from_str(&wire).map_err(|error| format!("[DOCS_SHARED_PLAN_WIRE] {error}"))?;
  if frame.wire_schema_version != 1
    || frame.plan_path != path_key
    || frame.payloads.len() != frame.plan.modules.len()
  {
    return Err("[DOCS_SHARED_PLAN_WIRE] invalid identity or shape".into());
  }
  if file_hash(path)? != frame.plan_file_hash {
    return Err("[DOCS_SHARED_PLAN_FILE] private plan changed".into());
  }
  for (module, refs) in frame.plan.modules.iter_mut().zip(frame.payloads) {
    if module.key != refs.key
      || !module.code.is_empty()
      || module.source_map.is_some()
      || module.machine_markdown.is_some()
      || module.generated_source.is_some()
    {
      return Err("[DOCS_SHARED_PLAN_WIRE] payload fields must be references only".into());
    }
    let prefix = format!("{}:{}", frame.plan.snapshot_id, module.key);
    let required = |key: &str, field: &str| -> Result<Arc<str>, String> {
      if key != format!("{prefix}:{field}") {
        return Err("[DOCS_SHARED_PLAN_WIRE] invalid blob identity".into());
      }
      provider
        .get(NAMESPACE, key)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("[DOCS_SHARED_PLAN_BLOB] missing {field}"))
    };
    module.code = required(&refs.code, "code")?;
    module.source_map = refs
      .source_map
      .as_deref()
      .map(|key| required(key, "map"))
      .transpose()?;
    module.machine_markdown = refs
      .machine_markdown
      .as_deref()
      .map(|key| required(key, "markdown"))
      .transpose()?;
    module.generated_source = refs
      .generated_source
      .as_deref()
      .map(|key| required(key, "generated"))
      .transpose()?;
  }
  Ok(frame.plan)
}

#[cfg(test)]
mod tests {
  use super::*;
  fn fixture() -> DocPlan {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site");
    let config = farmfe_docs_config::parse_config(r#"{"title":"Fixture","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/"}]}}"#, &root).unwrap();
    crate::prepare(&config).unwrap()
  }
  #[test]
  fn hydrated_views_share_payloads_and_preserve_exact_plan_bytes() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("plan.json");
    let plan = fixture();
    fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let provider = PlanResources::new(plan, &path).unwrap();
    let first = read_plan(&path, Some(&provider)).unwrap();
    let second = read_plan(&path, Some(&provider)).unwrap();
    assert!(Arc::ptr_eq(&first.modules[0].code, &second.modules[0].code));
    assert!(Arc::ptr_eq(
      &first.modules[0].code,
      &provider.plan().modules[0].code
    ));
    assert_eq!(
      serde_json::to_vec(&first).unwrap(),
      serde_json::to_vec(provider.plan()).unwrap()
    );
    crate::verify(&first).unwrap();
  }
  #[test]
  fn streaming_identity_matches_the_original_clone_and_vec_algorithm() {
    let plan = fixture();
    let mut legacy = plan.clone();
    legacy.snapshot_id.clear();
    assert_eq!(
      crate::identity(&plan).unwrap(),
      crate::hash(&serde_json::to_vec(&legacy).unwrap())
    );
  }

  #[test]
  fn buffered_hashes_match_legacy_for_escaped_payloads_crossing_multiple_buffers() {
    let mut plan = fixture();
    let payload = "\"quoted\"\\path\n\t中文🙂\u{0001}".repeat(8000);
    plan.modules[0].code = payload.clone().into();
    plan.modules[0].source_map = Some(payload.clone().into());
    plan.modules[0].machine_markdown = Some(payload.into());
    assert_eq!(
      plan_bytes_hash(&plan).unwrap(),
      crate::hash(&serde_json::to_vec(&plan).unwrap())
    );
    let mut legacy = plan.clone();
    legacy.snapshot_id.clear();
    assert_eq!(
      crate::identity(&plan).unwrap(),
      crate::hash(&serde_json::to_vec(&legacy).unwrap())
    );
  }
  #[test]
  fn private_plan_tampering_is_rejected_even_with_a_live_provider() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("plan.json");
    let plan = fixture();
    fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let provider = PlanResources::new(plan, &path).unwrap();
    fs::write(&path, b"{}").unwrap();
    assert!(
      read_plan(&path, Some(&provider))
        .unwrap_err()
        .contains("private plan")
    );
  }

  #[test]
  fn explicit_provider_never_falls_back_to_private_file() {
    struct Empty;
    impl SharedResourceProvider for Empty {
      fn get(&self, _: &str, _: &str) -> farmfe_core::error::Result<Option<Arc<str>>> {
        Ok(None)
      }
    }
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("plan.json");
    fs::write(&path, serde_json::to_vec(&fixture()).unwrap()).unwrap();
    assert!(
      read_plan(&path, Some(&Empty))
        .unwrap_err()
        .contains("does not own")
    );
    crate::verify(&read_plan(&path, None).unwrap()).unwrap();
  }
}
