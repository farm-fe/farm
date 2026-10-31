//! Direct native Docs preparation and verification, with explicit plan leases.
use std::{
  fs,
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use farmfe_core::context::shared_resources::SharedResourceProvider;
use farmfe_docs_plan::shared::PlanResources;
use napi::{
  Env, Status,
  bindgen_prelude::{Buffer, Object, ObjectRef},
};

#[napi(object)]
pub struct DocsTargetResult {
  pub resources: std::collections::HashMap<String, Buffer>,
  pub input_observations: String,
}

type OwnedTargetResult = (std::collections::HashMap<String, Vec<u8>>, String);

/// Own the compiler until cache tasks finish, including unsuccessful builds.
/// This runs outside its Rayon pool so dropping the pool cannot join itself.
fn compile_owned(compiler: Arc<farmfe_compiler::Compiler>) -> Result<OwnedTargetResult, String> {
  let mut profile = farmfe_docs_plan::verify_profile::Timer::new("native.target");
  let compile = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compiler.compile()))
    .map_err(|_| "target compiler panicked".to_owned())
    .and_then(|result| result.map_err(|error| error.to_string()));
  profile.checkpoint("compiler_compile_inclusive", 1);
  let drain = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compiler.flush_cache()))
    .map_err(|_| "cache drain panicked".to_owned())
    .and_then(|result| result.map_err(|error| error.to_string()));
  profile.checkpoint("cache_drain", 1);
  if std::env::var("FARM_DOCS_VERIFY_PROFILE").is_ok_and(|value| value == "1") {
    let context = compiler.context();
    let stats = context.stats.initial_compilation_flow_stats.read();
    // Existing compiler clocks: inclusive graph build and the remaining stages.
    // Failed attempts may not reach a clock; report null rather than invent zero.
    let build = stats
      .build_end_time
      .checked_sub(stats.start_time)
      .filter(|_| stats.start_time != 0 && stats.build_end_time != 0);
    let remaining = stats
      .end_time
      .checked_sub(stats.build_end_time)
      .filter(|_| stats.end_time != 0 && stats.build_end_time != 0);
    let graph = context.module_graph.read();
    let modules = graph.modules();
    let mut physical = std::collections::BTreeSet::new();
    let mut canonical = std::collections::BTreeSet::new();
    let mut file_instances = 0;
    let mut external = 0;
    let mut virtual_modules = 0;
    let mut query_instances = 0;
    let mut unclassified = 0;
    for module in &modules {
      query_instances += usize::from(!module.id.query_string().is_empty());
      if module.external {
        external += 1;
      } else if module
        .id
        .relative_path()
        .starts_with(farmfe_core::module::VIRTUAL_MODULE_PREFIX)
      {
        virtual_modules += 1;
      } else {
        let path = PathBuf::from(module.id.resolved_path(&context.config.root));
        if path.is_file() {
          file_instances += 1;
          physical.insert(path.clone());
          if let Ok(path) = fs::canonicalize(&path) {
            canonical.insert(path);
          }
        } else {
          unclassified += 1;
        }
      }
    }
    eprintln!(
      "[FarmDocsTargetProfile] {}",
      farmfe_core::serde_json::json!({
        "target": context.config.output.target_env,
        "buildInclusiveMilliseconds": build,
        "generateFinishSealCacheInclusiveMilliseconds": remaining,
        "success": compile.is_ok(),
        "hookStatsEnabled": context.config.record,
        "graphModules": modules.len(),
        "graphEdges": graph.edge_count(),
        "corePrefixVirtualModules": virtual_modules,
        "externalModules": external,
        "queryModuleInstances": query_instances,
        "physicalFileModuleInstances": file_instances,
        "distinctPhysicalPaths": physical.len(),
        "distinctCanonicalFiles": canonical.len(),
        "otherNonFileModules": unclassified,
        "classificationNote": "profile-only filesystem classification; query count overlaps other categories; no module paths logged",
      })
    );
  }
  profile.checkpoint("profile_graph_inspection", 1);
  let outcome = match (compile, drain) {
    (Err(primary), Err(drain)) => Err(format!("{primary}; cache drain also failed: {drain}")),
    (Err(primary), _) => Err(primary),
    (_, Err(drain)) => Err(drain),
    (Ok(()), Ok(())) => (|| {
      let context = compiler.context();
      let observations = context
        .source_observations
        .sealed_records()
        .map_err(|error| error.to_string())?;
      let observations =
        farmfe_core::serde_json::to_string(&observations).map_err(|error| error.to_string())?;
      let resources = context
        .resources_map
        .lock()
        .values()
        .filter(|resource| !resource.emitted)
        .map(|resource| (resource.name.clone(), resource.bytes.clone()))
        .collect();
      Ok((resources, observations))
    })(),
  };
  profile.checkpoint("collect_resources_and_observations", 1);
  drop(compiler);
  profile.checkpoint("compiler_dispose", 1);
  outcome
}

enum State {
  Empty,
  Preparing,
  Ready(Arc<PlanResources>),
  Failed,
  Closed,
}

fn failure(message: impl std::fmt::Display) -> napi::Error {
  napi::Error::new(
    Status::GenericFailure,
    format!("[DOCS_NATIVE_HOST] {message}"),
  )
}

/// A process/Env-local native snapshot. No pointer or large content is returned to JS.
#[napi]
pub struct DocsSnapshot {
  root: String,
  site: String,
  state: Arc<Mutex<State>>,
}

impl DocsSnapshot {
  fn ready(&self) -> napi::Result<Arc<PlanResources>> {
    let state = self.state.lock().map_err(failure)?;
    match &*state {
      State::Ready(resources) => Ok(resources.clone()),
      _ => Err(failure("snapshot is not prepared or has been closed")),
    }
  }

  pub(crate) fn provider(&self) -> napi::Result<Arc<dyn SharedResourceProvider>> {
    Ok(self.ready()?)
  }
}

fn private_plan_path(root: &str, path: &str) -> Result<PathBuf, String> {
  let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
  let path = Path::new(path);
  if !path.is_absolute() || path.file_name().is_none() {
    return Err("private plan must be an absolute file path".into());
  }
  let parent = path.parent().ok_or("private plan requires a parent")?;
  let canonical_parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
  let private_root = root.join(".farm-docs");
  if !canonical_parent.starts_with(&private_root) {
    return Err("private plan must remain in the site's real .farm-docs staging directory".into());
  }
  Ok(canonical_parent.join(path.file_name().ok_or("private plan filename is missing")?))
}

#[napi]
impl DocsSnapshot {
  #[napi(constructor)]
  pub fn new(root: String, site: String) -> napi::Result<Self> {
    if !Path::new(&root).is_absolute() {
      return Err(failure("root must be absolute"));
    }
    Ok(Self {
      root,
      site,
      state: Arc::new(Mutex::new(State::Empty)),
    })
  }

  /// Run the original Rust prepare on an owned worker, keeping Node's loop free.
  #[napi(ts_return_type = "Promise<void>")]
  pub fn prepare(&self, env: Env, plan_path: String) -> napi::Result<ObjectRef> {
    let mut state_guard = self.state.lock().map_err(failure)?;
    if !matches!(*state_guard, State::Empty) {
      return Err(failure("prepare is allowed exactly once"));
    }
    let (deferred, result) =
      env.create_deferred::<(), Box<dyn FnOnce(Env) -> napi::Result<()>>>()?;
    *state_guard = State::Preparing;
    drop(state_guard);
    let root = self.root.clone();
    let site = self.site.clone();
    let state = self.state.clone();
    let spawn_state = self.state.clone();
    let started = std::thread::Builder::new()
      .name("farm-docs-prepare".into())
      .spawn(move || {
        let operation = (|| -> Result<PlanResources, String> {
          let destination = private_plan_path(&root, &plan_path)?;
          let config = farmfe_docs_config::parse_config(&site, Path::new(&root))
            .map_err(|error| error.to_string())?;
          let plan = farmfe_docs_plan::prepare(&config)?;
          let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| format!("create private plan: {error}"))?;
          let mut output = BufWriter::with_capacity(262144, file);
          serde_write(&mut output, &plan)?;
          output.flush().map_err(|error| error.to_string())?;
          drop(output);
          PlanResources::new(plan, &destination)
        })();
        match operation {
          Ok(resources) => match state.lock() {
            Ok(mut state) => {
              *state = State::Ready(Arc::new(resources));
              deferred.resolve(Box::new(|_| Ok(())));
            }
            Err(error) => deferred.reject(failure(error)),
          },
          Err(error) => {
            if let Ok(mut state) = state.lock() {
              *state = State::Failed;
            }
            deferred.reject(failure(error));
          }
        }
      });
    if let Err(error) = started {
      if let Ok(mut state) = spawn_state.lock() {
        *state = State::Failed;
      }
      return Err(failure(format!("start preparation worker: {error}")));
    }
    result.create_ref()
  }

  /// Return only the fields needed by workflow/SSR, not modules or private inputs.
  #[napi]
  pub fn metadata(&self) -> napi::Result<String> {
    self.ready()?.metadata().map_err(failure)
  }

  /// Compile one target with deterministic native ownership, independent of GC.
  #[napi(ts_return_type = "Promise<DocsTargetResult>")]
  pub fn compile_target(&self, env: Env, config: Object) -> napi::Result<ObjectRef> {
    let provider = self.provider()?;
    let compiler = crate::JsCompiler::new(env, config)?.compiler;
    if !compiler
      .context()
      .shared_resources
      .as_ref()
      .is_some_and(|actual| Arc::ptr_eq(actual, &provider))
    {
      compiler.flush_cache().map_err(failure)?;
      return Err(failure("target compiler must use this prepared snapshot"));
    }
    let (deferred, result) = env
      .create_deferred::<DocsTargetResult, Box<dyn FnOnce(Env) -> napi::Result<DocsTargetResult>>>(
      )?;
    // The compiler is sent only after spawning succeeds: a failed spawn never
    // consumes its owner, and can drain/release it without losing an operation.
    let (sender, receiver) = std::sync::mpsc::channel::<Arc<farmfe_compiler::Compiler>>();
    let worker = std::thread::Builder::new()
      .name("farm-docs-target".into())
      .spawn(move || {
        let operation = receiver
          .recv()
          .map_err(|error| error.to_string())
          .and_then(compile_owned);
        match operation {
          Ok((resources, observations)) => deferred.resolve(Box::new(move |_| {
            Ok(DocsTargetResult {
              resources: resources
                .into_iter()
                .map(|(name, bytes)| (name, bytes.into()))
                .collect(),
              input_observations: observations,
            })
          })),
          // Preserve compiler diagnostic identity: the workflow retries only
          // the exact, typed legacy input-discovery failure, never host errors.
          Err(error) => deferred.reject(napi::Error::new(Status::GenericFailure, error)),
        }
      });
    if let Err(error) = worker {
      let drain = compiler.flush_cache();
      drop(compiler);
      return Err(failure(match drain {
        Ok(()) => format!("start target worker: {error}"),
        Err(drain) => format!("start target worker: {error}; cache drain also failed: {drain}"),
      }));
    }
    if let Err(error) = sender.send(compiler) {
      let drain = error.0.flush_cache();
      return Err(failure(match drain {
        Ok(()) => "target worker closed before accepting compiler".to_owned(),
        Err(drain) => format!(
          "target worker closed before accepting compiler; cache drain also failed: {drain}"
        ),
      }));
    }
    result.create_ref()
  }

  /// Delegate to the same verifier as the snapshot plugin; no seal checks are omitted.
  #[napi(ts_return_type = "Promise<void>")]
  pub fn verify(&self, env: Env, verification: String) -> napi::Result<ObjectRef> {
    let resources = self.ready()?;
    let options: farmfe_docs_snapshot::Verification =
      farmfe_core::serde_json::from_str(&verification).map_err(failure)?;
    let root = self.root.clone();
    let (deferred, result) =
      env.create_deferred::<(), Box<dyn FnOnce(Env) -> napi::Result<()>>>()?;
    std::thread::Builder::new()
      .name("farm-docs-verify".into())
      .spawn(move || {
        let mut profile = farmfe_docs_plan::verify_profile::Timer::new("native.verify");
        let operation = (|| {
          resources.verify_private_file()?;
          profile.checkpoint("private_plan_file_sha", 1);
          farmfe_docs_snapshot::verify_build(resources.plan(), &root, &options)?;
          profile.checkpoint("verify_build", 1);
          Ok::<(), String>(())
        })();
        match operation {
          Ok(()) => deferred.resolve(Box::new(|_| Ok(()))),
          Err(error) => deferred.reject(failure(error)),
        }
      })
      .map_err(failure)?;
    result.create_ref()
  }

  /// Release this handle. Existing compiler providers retain their own safe leases.
  #[napi]
  pub fn close(&self) -> napi::Result<()> {
    let mut state = self.state.lock().map_err(failure)?;
    if matches!(*state, State::Preparing) {
      return Err(failure("await preparation before closing"));
    }
    *state = State::Closed;
    Ok(())
  }
}

fn serde_write(writer: &mut impl Write, plan: &farmfe_docs_plan::DocPlan) -> Result<(), String> {
  farmfe_core::serde_json::to_writer(writer, plan).map_err(|error| error.to_string())
}
