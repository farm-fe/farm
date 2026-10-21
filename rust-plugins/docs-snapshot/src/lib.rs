use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, Module, ModuleId},
  plugin::{
    Plugin, PluginFreezeModuleHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult,
  },
  stats::Stats,
};
use farmfe_docs_plan::DocPlan;
use farmfe_docs_snapshot::{
  error, hash, observe, read_json, verify_build, verify_source, Receipt, ResourceRecord,
  SourceRecord, Target, Verification,
};
use farmfe_macro_plugin::farm_plugin;
use serde::{Deserialize, Serialize};
use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
#[serde(
  tag = "mode",
  rename_all = "lowercase",
  rename_all_fields = "camelCase",
  deny_unknown_fields
)]
enum Options {
  Capture {
    plan_path: String,
    receipt_path: String,
    seed_path: String,
    target: Target,
    output_path: String,
  },
  Verify {
    plan_path: String,
    verification: Verification,
  },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Cache {
  schema_version: u32,
  plan_snapshot_id: String,
  target: Target,
  root: String,
  modules: BTreeMap<String, Vec<SourceRecord>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Seed {
  schema_version: u32,
  plan_snapshot_id: String,
  target: Target,
  root: String,
  sources: Vec<SourceRecord>,
  modules: BTreeSet<String>,
}

#[derive(Default)]
struct State {
  plan: Option<DocPlan>,
  cached: Option<Cache>,
  known: BTreeMap<String, SourceRecord>,
  initial_sources: BTreeSet<String>,
  own_sources: BTreeMap<String, String>,
  modules: BTreeMap<String, Vec<SourceRecord>>,
  exclusions: Vec<PathBuf>,
  hard_roots: Vec<PathBuf>,
  retry_invalidations: BTreeSet<String>,
  discovered_modules: BTreeSet<String>,
  discovery: bool,
}

fn diagnostic(message: impl std::fmt::Display) -> CompilationError {
  CompilationError::GenericError(error(message))
}
fn native(message: String) -> CompilationError {
  CompilationError::GenericError(message)
}

#[farm_plugin]
pub struct FarmPluginDocsSnapshot {
  options: std::result::Result<Options, String>,
  state: Mutex<State>,
}

impl FarmPluginDocsSnapshot {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str(&options).map_err(|e| e.to_string()),
      state: Mutex::new(State::default()),
    }
  }
  fn options(&self) -> Result<&Options> {
    self.options.as_ref().map_err(diagnostic)
  }

  fn assert_source_scope(path: &Path, state: &State) -> Result<()> {
    let within = |path: &Path, root: &Path| {
      let normalized = |path: &Path| {
        let value = path.to_string_lossy().replace('\\', "/");
        if cfg!(windows) {
          value.to_lowercase()
        } else {
          value
        }
      };
      let path = normalized(path);
      let root = normalized(root);
      path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
    };
    let canonical = fs::canonicalize(path).ok();
    for root in &state.hard_roots {
      if within(path, root)
        || canonical
          .as_ref()
          .zip(fs::canonicalize(root).ok())
          .is_some_and(|(path, root)| within(path, &root))
      {
        return Err(diagnostic(format!(
          "generated output or compiler cache cannot be a source: {}",
          path.display()
        )));
      }
    }
    Ok(())
  }

  fn observe_input(path: &Path, state: &State) -> Result<SourceRecord> {
    Self::assert_source_scope(path, state)?;
    observe(path, &state.exclusions).map_err(native)
  }

  fn verify_input(source: &SourceRecord, state: &State) -> Result<()> {
    Self::assert_source_scope(Path::new(&source.path), state)?;
    verify_source(source).map_err(native)
  }

  fn observe_watches(
    &self,
    module_id: &ModuleId,
    external: bool,
    context: &Arc<CompilationContext>,
  ) -> Result<()> {
    if external {
      return Ok(());
    }
    let id = module_id.to_string();
    let mut state = self.state.lock().map_err(diagnostic)?;
    let mut records = state
      .modules
      .get(&id)
      .cloned()
      .ok_or_else(|| diagnostic(format!("module bypassed source observation: {id}")))?;
    for record in &records {
      Self::verify_input(record, &state)?;
    }
    let watch = context.watch_graph.read();
    for dependency in watch.relation_dependencies(module_id) {
      let path = PathBuf::from(dependency.resolved_path(&context.config.root));
      if !path.is_absolute() {
        continue;
      }
      let key = path
        .to_str()
        .ok_or_else(|| diagnostic("non UTF-8 watch path"))?;
      let record = if let Some(record) = state.known.get(key) {
        Self::verify_input(record, &state)?;
        record.clone()
      } else {
        let record = Self::observe_input(&path, &state)?;
        state.known.insert(record.path.clone(), record.clone());
        record
      };
      // Only observations made before compilation (or this reader's own raw
      // load) establish that the source preceded the reader's transformation.
      // Another module may have loaded the same source after this reader ran.
      if !state.initial_sources.contains(&record.canonical_path)
        && state.own_sources.get(&id) != Some(&record.canonical_path)
      {
        state.discovery = true;
        state.discovered_modules.insert(id.clone());
      }
      if !records.iter().any(|source| source.path == record.path) {
        records.push(record);
      }
    }
    state.modules.insert(id, records);
    Ok(())
  }

  fn reject_unobserved_inputs(&self, context: &Arc<CompilationContext>) -> Result<()> {
    let Options::Capture {
      seed_path, target, ..
    } = self.options()?
    else {
      return Ok(());
    };
    let state = self.state.lock().map_err(diagnostic)?;
    if state.discovery {
      let plan = state
        .plan
        .as_ref()
        .ok_or_else(|| diagnostic("missing capture plan"))?;
      let seed = Seed {
        schema_version: 1,
        plan_snapshot_id: plan.snapshot_id.clone(),
        target: *target,
        root: context.config.root.clone(),
        sources: state.known.values().cloned().collect(),
        modules: state
          .retry_invalidations
          .union(&state.discovered_modules)
          .cloned()
          .collect(),
      };
      fs::write(seed_path, serde_json::to_vec(&seed).map_err(diagnostic)?).map_err(diagnostic)?;
      return Err(CompilationError::GenericError(
        "[DOCS_INPUT_DISCOVERY] newly discovered watch inputs require an observed retry".into(),
      ));
    }
    Ok(())
  }
}

impl Plugin for FarmPluginDocsSnapshot {
  fn name(&self) -> &str {
    "FarmPluginDocsSnapshot"
  }
  fn priority(&self) -> i32 {
    i32::MAX - 100
  }

  fn plugin_cache_loaded(
    &self,
    bytes: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.state.lock().map_err(diagnostic)?.cached = serde_json::from_slice(bytes).ok();
    Ok(None)
  }

  fn build_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    let plan_path = match self.options()? {
      Options::Capture { plan_path, .. } | Options::Verify { plan_path, .. } => plan_path,
    };
    let plan: DocPlan = read_json(Path::new(plan_path)).map_err(native)?;
    farmfe_docs_plan::verify(&plan).map_err(native)?;
    if let Options::Verify { verification, .. } = self.options()? {
      verify_build(&plan, &context.config.root, verification).map_err(native)?;
      return Ok(None);
    }
    let Options::Capture {
      seed_path,
      target,
      output_path,
      ..
    } = self.options()?
    else {
      return Ok(None);
    };
    if context.config.output.target_env.is_node() != (*target == Target::Server) {
      return Err(diagnostic("capture target differs from compilation target"));
    }
    let mut state = self.state.lock().map_err(diagnostic)?;
    state.known.clear();
    state.initial_sources.clear();
    state.own_sources.clear();
    state.modules.clear();
    state.discovery = false;
    state.retry_invalidations.clear();
    state.discovered_modules.clear();
    state.hard_roots = vec![PathBuf::from(output_path)];
    state.exclusions = vec![
      PathBuf::from(output_path),
      PathBuf::from(&context.config.root).join("node_modules"),
    ];
    if let Some(path) = context.cache_manager.disk_cache_directory() {
      state.exclusions.push(path.clone());
      state.hard_roots.push(path);
    }
    let cached = state.cached.clone().filter(|cache| {
      cache.schema_version == 1
        && cache.plan_snapshot_id == plan.snapshot_id
        && cache.target == *target
        && cache.root == context.config.root
    });
    state.cached = cached.clone();
    if let Some(cache) = cached {
      for source in cache.modules.values().flatten() {
        // Removed inputs may belong to modules no longer used; invalidate on use.
        if let Ok(record) = Self::observe_input(Path::new(&source.path), &state) {
          state.known.insert(record.path.clone(), record);
        }
      }
    }
    if Path::new(seed_path).exists() {
      let seed: Seed = read_json(Path::new(seed_path)).map_err(native)?;
      if seed.schema_version != 1
        || seed.plan_snapshot_id != plan.snapshot_id
        || seed.target != *target
        || seed.root != context.config.root
      {
        return Err(diagnostic("discovery seed identity differs"));
      }
      for source in seed.sources {
        Self::verify_input(&source, &state)?;
        state.known.insert(source.path.clone(), source);
      }
      state.retry_invalidations = seed.modules;
    }
    state.initial_sources = state
      .known
      .values()
      .map(|source| source.canonical_path.clone())
      .collect();
    state.plan = Some(plan);
    Ok(None)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if matches!(self.options()?, Options::Verify { .. })
      && param.resolved_path == "farm-docs:verify-seal"
    {
      return Ok(Some(PluginLoadHookResult {
        content: "export default 0;".into(),
        module_type: farmfe_core::module::ModuleType::Js,
        source_map: None,
      }));
    }
    if !matches!(self.options()?, Options::Capture { .. }) {
      return Ok(None);
    }
    let path = Path::new(param.resolved_path);
    let mut state = self.state.lock().map_err(diagnostic)?;
    let mut records = vec![];
    if path.is_absolute() && path.is_file() {
      let key = path
        .to_str()
        .ok_or_else(|| diagnostic("non UTF-8 module path"))?;
      let record = if let Some(record) = state.known.get(key) {
        Self::verify_input(record, &state)?;
        record.clone()
      } else {
        let record = Self::observe_input(path, &state)?;
        state.known.insert(record.path.clone(), record.clone());
        record
      };
      state
        .own_sources
        .insert(param.module_id.clone(), record.canonical_path.clone());
      records.push(record);
    }
    state.modules.insert(param.module_id.clone(), records);
    Ok(None)
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
    _hook: &PluginHookContext,
  ) -> Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if matches!(self.options()?, Options::Verify { .. })
      && (param.source == "farm-docs:verify-seal"
        || (matches!(param.kind, farmfe_core::plugin::ResolveKind::Entry(_))
          && param.source == "./farm-docs:verify-seal"))
    {
      return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
        resolved_path: "farm-docs:verify-seal".into(),
        ..Default::default()
      }));
    }
    Ok(None)
  }

  fn handle_persistent_cached_module(
    &self,
    module: &Module,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<bool>> {
    if !matches!(self.options()?, Options::Capture { .. }) {
      return Ok(None);
    }
    let mut state = self.state.lock().map_err(diagnostic)?;
    let id = module.id.to_string();
    if state.retry_invalidations.contains(&id) {
      return Ok(Some(true));
    }
    let records = state
      .cached
      .as_ref()
      .and_then(|cache| cache.modules.get(&id))
      .cloned();
    match records {
      Some(records)
        if records
          .iter()
          .all(|source| Self::verify_input(source, &state).is_ok()) =>
      {
        state.modules.insert(id, records);
        Ok(None)
      }
      _ => Ok(Some(true)),
    }
  }

  fn freeze_module(
    &self,
    param: &mut PluginFreezeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !matches!(self.options()?, Options::Capture { .. }) || param.module.external {
      return Ok(None);
    }
    self.observe_watches(&param.module.id, param.module.external, context)?;
    Ok(None)
  }

  fn module_graph_build_end(
    &self,
    graph: &mut ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !matches!(self.options()?, Options::Capture { .. }) {
      return Ok(None);
    }
    for module in graph.modules() {
      self.observe_watches(&module.id, module.external, context)?;
    }
    self.reject_unobserved_inputs(context)?;
    Ok(None)
  }

  fn finish(&self, _stats: &Stats, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    let Options::Capture {
      receipt_path,
      target,
      ..
    } = self.options()?
    else {
      return Ok(None);
    };
    let modules = {
      let graph = context.module_graph.read();
      graph
        .modules()
        .iter()
        .map(|module| (module.id.clone(), module.external))
        .collect::<Vec<_>>()
    };
    for (id, external) in modules {
      self.observe_watches(&id, external, context)?;
    }
    self.reject_unobserved_inputs(context)?;
    let state = self.state.lock().map_err(diagnostic)?;
    let plan = state
      .plan
      .as_ref()
      .ok_or_else(|| diagnostic("missing capture plan"))?;
    farmfe_docs_plan::verify(plan).map_err(native)?;
    let mut sources = BTreeMap::new();
    for record in state.modules.values().flatten() {
      Self::verify_input(record, &state)?;
      if let Some(previous) = sources.insert(record.path.clone(), record.clone()) {
        if previous != *record {
          return Err(diagnostic("conflicting module observations"));
        }
      }
    }
    let resources = context.resources_map.lock();
    let mut rows = resources
      .values()
      .filter(|resource| !resource.emitted)
      .map(|resource| ResourceRecord {
        name: resource.name.clone(),
        resource_type: resource.resource_type.to_string(),
        length: resource.bytes.len(),
        hash: hash(&resource.bytes),
      })
      .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    let receipt = Receipt {
      schema_version: 1,
      plan_snapshot_id: plan.snapshot_id.clone(),
      target: *target,
      root: context.config.root.clone(),
      sources: sources.into_values().collect(),
      resources: rows,
    };
    receipt
      .validate(&plan.snapshot_id, *target, &context.config.root)
      .map_err(native)?;
    fs::write(
      receipt_path,
      serde_json::to_vec(&receipt).map_err(diagnostic)?,
    )
    .map_err(diagnostic)?;
    Ok(None)
  }

  fn write_plugin_cache(&self, context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>> {
    let Options::Capture { target, .. } = self.options()? else {
      return Ok(None);
    };
    let state = self.state.lock().map_err(diagnostic)?;
    let plan = state
      .plan
      .as_ref()
      .ok_or_else(|| diagnostic("missing capture plan"))?;
    let cache = Cache {
      schema_version: 1,
      plan_snapshot_id: plan.snapshot_id.clone(),
      target: *target,
      root: context.config.root.clone(),
      modules: state.modules.clone(),
    };
    Ok(Some(serde_json::to_vec(&cache).map_err(diagnostic)?))
  }
}
