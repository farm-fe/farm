use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex, RwLock},
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleId, ModuleType},
  plugin::{
    Plugin, PluginFinalizeResourcesHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
    PluginUpdateModulesHookParam, ResolveKind,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
  serde_json::{self, Value, json},
};
use farmfe_docs_config::{PublicSiteConfig, ValidatedConfig, parse_config};
use farmfe_docs_plan::{
  ContentCache, DocPlan, application, asset_url, browser_application, prepare, prepare_with_cache,
  read_asset, validate_resource_names, verify,
};
use farmfe_macro_plugin::farm_plugin;
use serde::{Deserialize, Serialize};
mod color_mode;
mod css_resources;
mod development;
mod development_entry;
mod html_template;
mod page_resources;

// Core preserves this prefix when watch dependencies become update paths.
// Public docs:* aliases remain independent of the internal module identity.
const VIRTUAL_ID: &str = "virtual:farm-docs:snapshot";
const SITE_VIRTUAL_ID: &str = "virtual:farm-docs:site";
const APP_VIRTUAL_ID: &str = "virtual:farm-docs:application";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
  #[serde(default)]
  snapshot_path: Option<String>,
  #[serde(default)]
  site: Option<Value>,
  #[serde(default)]
  prepare: bool,
  #[serde(default)]
  development: bool,
  #[serde(default)]
  plan_path: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Snapshot {
  schema_version: u32,
  snapshot_id: String,
  routes: Vec<Value>,
}

fn diagnostic(code: &str, message: impl std::fmt::Display) -> CompilationError {
  CompilationError::GenericError(format!("[{code}] {message}"))
}

fn read_snapshot(file: &Path) -> Result<Snapshot> {
  let bytes = fs::read(file)
    .map_err(|error| diagnostic("DOCS_SNAPSHOT_READ", format!("{}: {error}", file.display())))?;
  let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(|error| {
    diagnostic(
      "DOCS_SNAPSHOT_PARSE",
      format!("{}: {error}", file.display()),
    )
  })?;
  if snapshot.schema_version != 1 || snapshot.snapshot_id.is_empty() {
    return Err(diagnostic(
      "DOCS_SNAPSHOT_SCHEMA",
      "require schemaVersion 1 and a nonempty snapshotId",
    ));
  }
  Ok(snapshot)
}

fn assert_public_module(resolved: &str) -> Result<()> {
  let private = |path: &str| {
    path
      .replace('\\', "/")
      .split('/')
      .any(farmfe_docs_config::is_private_tooling_component)
  };
  if private(resolved)
    || fs::canonicalize(resolved)
      .ok()
      .is_some_and(|path| private(&path.to_string_lossy()))
  {
    return Err(diagnostic(
      "DOCS_PRIVATE_MODULE",
      format!("private tooling module cannot enter the public graph: {resolved}"),
    ));
  }
  Ok(())
}

#[farm_plugin]
pub struct FarmPluginDocs {
  options: std::result::Result<Options, String>,
  snapshot: RwLock<Option<Snapshot>>,
  site: RwLock<Option<ValidatedConfig>>,
  plan: RwLock<Option<DocPlan>>,
  content_cache: Mutex<ContentCache>,
  pending_development_modules: RwLock<Vec<ModuleId>>,
}

impl FarmPluginDocs {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str(&options).map_err(|error| error.to_string()),
      snapshot: RwLock::new(None),
      site: RwLock::new(None),
      plan: RwLock::new(None),
      content_cache: Mutex::new(ContentCache::default()),
      pending_development_modules: RwLock::new(vec![]),
    }
  }

  fn prepare_development(&self, config: &ValidatedConfig) -> Result<DocPlan> {
    let mut cache = self
      .content_cache
      .lock()
      .map_err(|error| diagnostic("DOCS_STATE", error))?;
    prepare_with_cache(config, &mut cache).map_err(CompilationError::GenericError)
  }

  fn snapshot(&self) -> Result<Snapshot> {
    self
      .snapshot
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .clone()
      .ok_or_else(|| diagnostic("DOCS_STATE", "snapshot is not loaded"))
  }

  fn public_site(&self) -> Result<PublicSiteConfig> {
    if let Some(plan) = self
      .plan
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .as_ref()
    {
      return Ok(plan.site.clone());
    }
    self
      .site
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .as_ref()
      .map(|site| site.public.clone())
      .ok_or_else(|| diagnostic("DOCS_STATE", "site configuration is not loaded"))
  }
}

impl Plugin for FarmPluginDocs {
  fn name(&self) -> &str {
    "FarmPluginDocs"
  }

  fn generate_end(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    if context.config.output.target_env.is_node()
      || self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?
        .is_none()
    {
      return Ok(None);
    }
    // Core's HTML minifier runs after our finalize_resources hook. Restore
    // template attribute boundaries only after every finalize hook has ended,
    // before the snapshot plugin observes the final bytes in finish. Do not
    // insert the theme bootstrap again; it is already in the compiled template.
    let mut resources = context.resources_map.lock();
    for resource in resources.values_mut() {
      if matches!(resource.resource_type, ResourceType::Html) {
        html_template::order_page_styles(resource, context.config.minify.enabled(), None)?;
      }
    }
    Ok(Some(()))
  }

  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    // Server rendering produces HTML for browsers: imported assets must have
    // the same web URL in both targets, rather than Node file: URL objects.
    config.assets.mode = Some(farmfe_core::config::asset::AssetFormatMode::Browser);
    Ok(Some(()))
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    assert_public_module(&param.module_id.resolved_path(&context.config.root))?;
    if !matches!(param.module_type, ModuleType::Css) {
      return Ok(None);
    }
    let guard = self
      .plan
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?;
    let Some(plan) = guard.as_ref() else {
      return Ok(None);
    };
    css_resources::rewrite(param, plan).map_err(CompilationError::GenericError)?;
    Ok(Some(()))
  }

  fn module_graph_build_end(
    &self,
    graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    // This also checks modules restored from persistent cache, whose transform
    // and process_module hooks need not run again.
    for module in graph.modules() {
      assert_public_module(&module.id.resolved_path(&context.config.root))?;
    }
    Ok(Some(()))
  }

  fn build_start(&self, context: &Arc<CompilationContext>) -> Result<Option<()>> {
    let options = self
      .options
      .as_ref()
      .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
    if usize::from(options.site.is_some())
      + usize::from(options.snapshot_path.is_some())
      + usize::from(options.plan_path.is_some())
      != 1
      || (options.prepare && options.site.is_none())
      || (options.development && (options.site.is_none() || options.prepare))
    {
      return Err(diagnostic(
        "DOCS_OPTIONS",
        "provide exactly one of site, snapshotPath or planPath; prepare requires site",
      ));
    }
    if let Some(path) = &options.plan_path {
      let file = PathBuf::from(&context.config.root).join(path);
      let plan = farmfe_docs_plan::shared::read_plan(&file, context.shared_resources.as_deref())
        .map_err(|error| diagnostic("DOCS_PLAN_READ", error))?;
      verify(&plan).map_err(CompilationError::GenericError)?;
      *self
        .plan
        .write()
        .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(plan);
      return Ok(Some(()));
    }
    let snapshot_path = match (&options.site, &options.snapshot_path) {
      (Some(site), None) => {
        let validated = parse_config(
          &site.to_string(),
          std::path::Path::new(&context.config.root),
        )
        .map_err(|error| CompilationError::GenericError(error.to_string()))?;
        if options.prepare || options.development {
          if options.prepare && !context.config.output.target_env.is_node() {
            return Err(diagnostic(
              "DOCS_PLAN_TARGET",
              "private preflight requires a Node target",
            ));
          }
          let plan = if options.development {
            self.prepare_development(&validated)?
          } else {
            prepare(&validated).map_err(CompilationError::GenericError)?
          };
          let record = if options.development {
            Some(serde_json::to_string(&plan).map_err(|error| diagnostic("DOCS_SERIALIZE", error))?)
          } else {
            None
          };
          *self
            .plan
            .write()
            .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(plan);
          if let Some(record) = record {
            context.custom.lock().insert(
              farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
              Box::new(record),
            );
          }
        }
        *self
          .site
          .write()
          .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(validated);
        return Ok(Some(()));
      }
      (None, Some(path)) => path,
      _ => {
        return Err(diagnostic(
          "DOCS_OPTIONS",
          "provide exactly one of site or snapshotPath",
        ));
      }
    };
    let file = PathBuf::from(&context.config.root).join(snapshot_path);
    let snapshot = read_snapshot(&file)?;
    *self
      .snapshot
      .write()
      .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(snapshot);
    context.add_watch_files(
      ModuleId::new(VIRTUAL_ID, "", &context.config.root),
      vec![ModuleId::new(
        &file.to_string_lossy(),
        "",
        &context.config.root,
      )],
    )?;
    Ok(Some(()))
  }

  fn update_modules(
    &self,
    params: &mut PluginUpdateModulesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let options = self
      .options
      .as_ref()
      .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
    if options.development {
      return self.update_development(params, context);
    }
    let Some(snapshot_path) = &options.snapshot_path else {
      return Ok(None);
    };
    // The watch graph has already mapped the changed file to its virtual owner.
    // Incremental builds do not invoke build_start, so refresh before load runs.
    let owners: Vec<_> = params
      .paths
      .iter()
      .map(|(path, _)| ModuleId::from_resolved_path_with_query(path, &context.config.root))
      .filter(|id| id.resolved_path(&context.config.root) == VIRTUAL_ID)
      .collect();
    if owners.is_empty() {
      return Ok(None);
    }
    // Validate the entire candidate before publishing state or invalidating a
    // module. A bad file leaves the previous usable snapshot untouched.
    let candidate = read_snapshot(&PathBuf::from(&context.config.root).join(snapshot_path))?;
    *self
      .snapshot
      .write()
      .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(candidate);
    for owner in owners {
      context.cache_manager.module_cache.invalidate_cache(&owner);
    }
    Ok(Some(()))
  }

  fn update_finished(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    if !self
      .options
      .as_ref()
      .map_err(|error| diagnostic("DOCS_OPTIONS", error))?
      .development
    {
      return Ok(None);
    }
    self
      .pending_development_modules
      .write()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .clear();
    Ok(Some(()))
  }

  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _hook: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    // Native updates re-resolve ModuleId::resolved_path_with_query(). Generated
    // content has no filesystem file, so its rooted spelling must return the
    // same Rust-owned identity used by the initial docs:content import.
    if param.source.contains("farm-docs:generated:") {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      if let Some(plan) = guard.as_ref() {
        for module in plan
          .modules
          .iter()
          .filter(|module| module.source_path.starts_with("farm-docs:generated:"))
        {
          let query = format!("?farm_docs={}", module.key);
          let id = ModuleId::new(&module.source_path, &query, &context.config.root);
          if param.source == id.resolved_path_with_query(&context.config.root)
            || param.source == format!("{}{query}", module.source_path)
          {
            return Ok(Some(PluginResolveHookResult {
              resolved_path: module.source_path.clone(),
              query: vec![("farm_docs".into(), module.key.clone())],
              ..Default::default()
            }));
          }
        }
      }
    }
    if param.source == "docs:dev"
      || param.source == development_entry::VIRTUAL_ID
      || (matches!(param.kind, ResolveKind::Entry(_)) && param.source == "./docs:dev")
    {
      let options = self
        .options
        .as_ref()
        .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
      if !options.development {
        return Err(diagnostic(
          "DOCS_OPTIONS",
          "docs:dev requires development mode",
        ));
      }
      return Ok(Some(PluginResolveHookResult {
        resolved_path: development_entry::VIRTUAL_ID.into(),
        side_effects: true,
        ..Default::default()
      }));
    }
    if param.source == "docs:application" || param.source == APP_VIRTUAL_ID {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: APP_VIRTUAL_ID.into(),
        ..Default::default()
      }));
    }
    if let Some(key) = param
      .source
      .trim_start_matches("./")
      .strip_prefix("docs:page:")
      .or_else(|| {
        param
          .source
          .strip_prefix("virtual:farm-docs/pages/page_")
          .and_then(|key| key.strip_suffix(".js"))
      })
    {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let plan = guard
        .as_ref()
        .ok_or_else(|| diagnostic("DOCS_PLAN_MODULE", key))?;
      if !plan.modules.iter().any(|module| module.key == key) {
        return Err(diagnostic("DOCS_PLAN_MODULE", key));
      }
      return Ok(Some(PluginResolveHookResult {
        resolved_path: page_resources::entry_path(key),
        ..Default::default()
      }));
    }
    if let Some(key) = param
      .source
      .trim_start_matches("./")
      .strip_prefix("docs:asset:")
    {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let asset = guard
        .as_ref()
        .and_then(|plan| plan.assets.iter().find(|asset| asset.key == key))
        .ok_or_else(|| diagnostic("DOCS_ASSET_PLAN", key))?;
      return Ok(Some(PluginResolveHookResult {
        resolved_path: asset.source_path.clone(),
        query: vec![("farm_docs_asset".into(), key.into())],
        side_effects: true,
        ..Default::default()
      }));
    }
    if let Some(key) = param
      .source
      .trim_start_matches("./")
      .strip_prefix("docs:content:")
    {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let module = guard
        .as_ref()
        .and_then(|plan| plan.modules.iter().find(|module| module.key == key))
        .ok_or_else(|| diagnostic("DOCS_PLAN_MODULE", key))?;
      return Ok(Some(PluginResolveHookResult {
        resolved_path: module.source_path.clone(),
        query: vec![("farm_docs".into(), key.into())],
        ..Default::default()
      }));
    }
    if param.source == "docs:site"
      || param.source == SITE_VIRTUAL_ID
      || (matches!(param.kind, ResolveKind::Entry(_)) && param.source == "./docs:site")
    {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: SITE_VIRTUAL_ID.into(),
        side_effects: false,
        ..Default::default()
      }));
    }
    Ok(
      (param.source == "docs:snapshot"
        || param.source == VIRTUAL_ID
        || (matches!(param.kind, ResolveKind::Entry(_)) && param.source == "./docs:snapshot"))
        .then(|| PluginResolveHookResult {
          resolved_path: VIRTUAL_ID.to_string(),
          side_effects: false,
          ..Default::default()
        }),
    )
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    _hook: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if param.resolved_path == development_entry::VIRTUAL_ID {
      let options = self
        .options
        .as_ref()
        .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
      if !options.development {
        return Err(diagnostic(
          "DOCS_OPTIONS",
          "development HTML requires development mode",
        ));
      }
      return Ok(Some(PluginLoadHookResult {
        content: development_entry::html(),
        module_type: ModuleType::Html,
        source_map: None,
      }));
    }
    if let Some(key) = param
      .resolved_path
      .strip_prefix("virtual:farm-docs/pages/page_")
      .and_then(|path| path.strip_suffix(".js"))
    {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let plan = guard
        .as_ref()
        .ok_or_else(|| diagnostic("DOCS_PLAN_MODULE", key))?;
      if !plan.modules.iter().any(|module| module.key == key) {
        return Err(diagnostic("DOCS_PLAN_MODULE", key));
      }
      let source = serde_json::to_string(&format!("docs:content:{key}"))
        .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
      let mut content = format!("export {{ default }} from {source};");
      if let Some(view) = plan
        .routes
        .iter()
        .find(|route| route.page.key == key)
        .and_then(|route| route.page.blog_view.as_ref())
        .filter(|view| view.full_content)
      {
        // Static edges keep original article CSS in the listing's actual lazy
        // module group, so SSG and PageLoader agree about its style lifetime.
        for (index, post_key) in view.posts.iter().enumerate() {
          let post_source = serde_json::to_string(&format!("docs:content:{post_key}"))
            .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
          content.push_str(&format!(
            "\nexport {{ default as listingPost{index} }} from {post_source};"
          ));
        }
      }
      return Ok(Some(PluginLoadHookResult {
        content,
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    if param.resolved_path == APP_VIRTUAL_ID {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let plan = guard
        .as_ref()
        .ok_or_else(|| diagnostic("DOCS_STATE", "application requires a prepared plan"))?;
      context.add_watch_files(
        ModuleId::new(APP_VIRTUAL_ID, "", &context.config.root),
        plan
          .assets
          .iter()
          .map(|asset| ModuleId::new(&asset.source_path, "", &context.config.root))
          .chain(
            plan
              .static_files
              .iter()
              .map(|asset| ModuleId::new(&asset.source_path, "", &context.config.root)),
          )
          .chain(
            plan
              .static_inputs
              .iter()
              .map(|directory| ModuleId::new(&directory.path, "", &context.config.root)),
          )
          .collect(),
      )?;
      return Ok(Some(PluginLoadHookResult {
        content: if context.config.output.target_env.is_node() {
          application(plan)
        } else {
          browser_application(plan)
        }
        .map_err(CompilationError::GenericError)?,
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    if let Some((_, key)) = param
      .query
      .iter()
      .find(|(name, _)| name == "farm_docs_asset")
    {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let plan = guard
        .as_ref()
        .ok_or_else(|| diagnostic("DOCS_STATE", "plan is not loaded"))?;
      let asset = plan
        .assets
        .iter()
        .find(|asset| asset.key == *key && asset.source_path == param.resolved_path)
        .ok_or_else(|| diagnostic("DOCS_ASSET_PLAN", key))?;
      read_asset(asset).map_err(CompilationError::GenericError)?;
      let url = serde_json::to_string(&asset_url(&plan.site, asset))
        .map_err(|cause| diagnostic("DOCS_SERIALIZE", cause))?;
      return Ok(Some(PluginLoadHookResult {
        content: format!("export default {url};"),
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    if let Some((_, key)) = param.query.iter().find(|(name, _)| name == "farm_docs") {
      let guard = self
        .plan
        .read()
        .map_err(|error| diagnostic("DOCS_STATE", error))?;
      let module = guard
        .as_ref()
        .and_then(|plan| {
          plan.modules.iter().find(|module| {
            module.key == *key
              && ModuleId::new(&module.source_path, "", &context.config.root)
                == ModuleId::new(param.resolved_path, "", &context.config.root)
          })
        })
        .ok_or_else(|| diagnostic("DOCS_PLAN_MODULE", key))?;
      return Ok(Some(PluginLoadHookResult {
        content: module.code.to_string(),
        module_type: match module.language {
          farmfe_docs_plan::ContentLanguage::Markdown | farmfe_docs_plan::ContentLanguage::Jsx => {
            ModuleType::Jsx
          }
          farmfe_docs_plan::ContentLanguage::Ts => ModuleType::Ts,
          farmfe_docs_plan::ContentLanguage::Tsx => ModuleType::Tsx,
        },
        source_map: module.source_map.as_deref().map(str::to_owned),
      }));
    }
    if param.resolved_path == SITE_VIRTUAL_ID {
      let public = serde_json::to_string(&self.public_site()?)
        .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
      return Ok(Some(PluginLoadHookResult {
        content: format!("export default {public};"),
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    if param.resolved_path != VIRTUAL_ID {
      return Ok(None);
    }
    let snapshot = serde_json::to_string(&self.snapshot()?)
      .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
    Ok(Some(PluginLoadHookResult {
      content: format!("export default {snapshot};"),
      module_type: ModuleType::Js,
      source_map: None,
    }))
  }

  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    use farmfe_docs_plan::resource_owner;
    let timing = (std::env::var_os("FARM_DOCS_UPDATE_TIMING").as_deref()
      == Some(std::ffi::OsStr::new("1")))
    .then(std::time::Instant::now);
    let mark = |stage: &str| {
      if let Some(start) = timing {
        use std::io::Write;
        let _ = writeln!(
          std::io::stderr(),
          "[FARM_UPDATE_TIMING] scope=docs_finalize stage={stage} elapsed_us={} thread={:?}",
          start.elapsed().as_micros(),
          std::thread::current().id()
        );
      }
    };
    mark("begin");
    let mut candidate = param.resources_map.clone();
    candidate.retain(|name, resource| {
      name != &resource.name
        || !resource_owner::is_owned(
          resource
            .meta
            .get(resource_owner::META_KEY)
            .map(String::as_str),
          self.name(),
          name,
          &resource.bytes,
        )
    });
    mark("clone_and_owner_filter_done");
    let options = self
      .options
      .as_ref()
      .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
    let mut names: Vec<_> = candidate.keys().cloned().collect();
    if options.site.is_some() && !options.development {
      names.push("docs-public-config.json".into());
      if options.prepare {
        names.push("docs-plan.json".into());
      }
    } else {
      names.push("docs-manifest.json".into());
    }
    validate_resource_names(&names).map_err(CompilationError::GenericError)?;
    let existing: std::collections::HashSet<_> = candidate.keys().cloned().collect();
    let result = {
      let param = &mut PluginFinalizeResourcesHookParam {
        resources_map: &mut candidate,
        config: param.config,
      };
      (|| -> Result<Option<()>> {
        if options.site.is_some() && !options.development {
          let name = "docs-public-config.json";
          let bytes = serde_json::to_vec(&self.public_site()?)
            .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
          param.resources_map.insert(
            name.into(),
            Resource {
              name: name.into(),
              bytes,
              resource_type: ResourceType::Custom("json".into()),
              origin: ResourceOrigin::Module(ModuleId::new(
                SITE_VIRTUAL_ID,
                "",
                &param.config.root,
              )),
              should_transform_output_filename: false,
              ..Default::default()
            },
          );
          if self
            .options
            .as_ref()
            .map_err(|error| diagnostic("DOCS_OPTIONS", error))?
            .prepare
          {
            let guard = self
              .plan
              .read()
              .map_err(|error| diagnostic("DOCS_STATE", error))?;
            let bytes = serde_json::to_vec(
              guard
                .as_ref()
                .ok_or_else(|| diagnostic("DOCS_STATE", "plan is not loaded"))?,
            )
            .map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
            param.resources_map.insert(
              "docs-plan.json".into(),
              Resource {
                name: "docs-plan.json".into(),
                bytes,
                resource_type: ResourceType::Custom("json".into()),
                origin: ResourceOrigin::Module(ModuleId::new(
                  SITE_VIRTUAL_ID,
                  "",
                  &param.config.root,
                )),
                should_transform_output_filename: false,
                ..Default::default()
              },
            );
          }
          return Ok(Some(()));
        }
        let theme_bootstrap = if !context.config.output.target_env.is_node()
          && context.module_graph.read().has_module(&ModuleId::new(
            APP_VIRTUAL_ID,
            "",
            &context.config.root,
          )) {
          let guard = self
            .plan
            .read()
            .map_err(|error| diagnostic("DOCS_STATE", error))?;
          guard
            .as_ref()
            .map(|plan| color_mode::Bootstrap::new(&plan.site))
            .transpose()?
        } else {
          None
        };
        if let Some(theme) = &theme_bootstrap {
          if param.resources_map.contains_key(&theme.name) {
            return Err(diagnostic("DOCS_RESOURCE_COLLISION", &theme.name));
          }
          param.resources_map.insert(
            theme.name.clone(),
            Resource {
              name: theme.name.clone(),
              bytes: theme.bytes.clone(),
              resource_type: ResourceType::Js,
              origin: ResourceOrigin::Module(ModuleId::new(
                APP_VIRTUAL_ID,
                "",
                &context.config.root,
              )),
              should_transform_output_filename: false,
              ..Default::default()
            },
          );
        }
        {
          let guard = self
            .plan
            .read()
            .map_err(|error| diagnostic("DOCS_STATE", error))?;
          if let Some(plan) = guard.as_ref() {
            let machine_resources = if !context.config.output.target_env.is_node() {
              farmfe_docs_plan::machine_readable::resources(plan)
                .map_err(CompilationError::GenericError)?
            } else {
              Default::default()
            };
            let mut names: Vec<_> = param.resources_map.keys().cloned().collect();
            names.extend(machine_resources.keys().cloned());
            names.extend(
              [
                "docs-plan.json",
                "docs-public-config.json",
                "docs-manifest.json",
                "docs-build-report.json",
              ]
              .iter()
              .map(|name| name.to_string()),
            );
            names.extend(plan.assets.iter().map(|asset| asset.output_path.clone()));
            names.extend(
              plan
                .static_files
                .iter()
                .filter(|file| !plan.assets.iter().any(|bound| bound.key == file.key))
                .map(|file| file.output_path.clone()),
            );
            validate_resource_names(&names).map_err(CompilationError::GenericError)?;
            for asset in &plan.assets {
              let bytes = read_asset(asset).map_err(CompilationError::GenericError)?;
              let module_id = ModuleId::new(
                &asset.source_path,
                &format!("?farm_docs_asset={}", asset.key),
                &param.config.root,
              );
              if context.module_graph.read().module(&module_id).is_none() {
                return Err(diagnostic(
                  "DOCS_ASSET_GRAPH",
                  "asset module is missing from the Farm graph",
                ));
              }
              if param.resources_map.contains_key(&asset.output_path) {
                return Err(diagnostic("DOCS_ASSET_COLLISION", &asset.output_path));
              }
              param.resources_map.insert(
                asset.output_path.clone(),
                Resource {
                  name: asset.output_path.clone(),
                  bytes,
                  resource_type: ResourceType::Asset(
                    asset.output_path.rsplit('.').next().unwrap_or("bin").into(),
                  ),
                  origin: ResourceOrigin::Module(ModuleId::new(
                    &asset.source_path,
                    &format!("?farm_docs_asset={}", asset.key),
                    &param.config.root,
                  )),
                  should_transform_output_filename: false,
                  ..Default::default()
                },
              );
            }
            for asset in plan
              .static_files
              .iter()
              .filter(|file| !plan.assets.iter().any(|bound| bound.key == file.key))
            {
              let bytes = read_asset(asset).map_err(CompilationError::GenericError)?;
              let origin = context
                .module_graph
                .read()
                .entries
                .keys()
                .next()
                .cloned()
                .ok_or_else(|| diagnostic("DOCS_STATIC_GRAPH", "entry module is missing"))?;
              param.resources_map.insert(
                asset.output_path.clone(),
                Resource {
                  name: asset.output_path.clone(),
                  bytes,
                  resource_type: ResourceType::Asset(
                    asset.output_path.rsplit('.').next().unwrap_or("bin").into(),
                  ),
                  origin: ResourceOrigin::Module(origin),
                  should_transform_output_filename: false,
                  ..Default::default()
                },
              );
            }
            for (name, bytes) in machine_resources {
              param.resources_map.insert(
                name.clone(),
                Resource {
                  name: name.clone(),
                  bytes,
                  resource_type: ResourceType::Custom(
                    if name.ends_with(".json") {
                      "json"
                    } else if name.ends_with(".md") {
                      "md"
                    } else {
                      "txt"
                    }
                    .into(),
                  ),
                  origin: ResourceOrigin::Module(ModuleId::new(
                    APP_VIRTUAL_ID,
                    "",
                    &context.config.root,
                  )),
                  should_transform_output_filename: false,
                  ..Default::default()
                },
              );
            }
          }
        }
        let prepared = self
          .plan
          .read()
          .map_err(|error| diagnostic("DOCS_STATE", error))?
          .is_some();
        let templates: Vec<_> = if prepared && !context.config.output.target_env.is_node() {
          param
            .resources_map
            .iter()
            .filter(|(_, resource)| matches!(resource.resource_type, ResourceType::Html))
            .map(|(name, _)| name.clone())
            .collect()
        } else {
          vec![]
        };
        if templates.len() > 1 {
          return Err(diagnostic(
            "DOCS_MANIFEST",
            "expected a single shared HTML template",
          ));
        }
        let template = templates.first();
        if let Some(name) = template {
          let resource = param
            .resources_map
            .get_mut(name)
            .ok_or_else(|| diagnostic("DOCS_TEMPLATE", "template disappeared"))?;
          html_template::order_page_styles(
            resource,
            context.config.minify.enabled(),
            theme_bootstrap.as_ref(),
          )?;
        }
        let mut resources: Vec<_> = param
          .resources_map
          .iter()
          .filter(|(name, resource)| {
            name.as_str() != "docs-manifest.json" && !resource.emitted && template != Some(*name)
          })
          .map(
            |(name, resource)| json!({ "name": name, "type": resource.resource_type.to_string() }),
          )
          .collect();
        resources.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
        mark("assets_and_resource_inventory_done");
        let page_resources = {
          let guard = self
            .plan
            .read()
            .map_err(|error| diagnostic("DOCS_STATE", error))?;
          if let Some(plan) = guard
            .as_ref()
            .filter(|_| !context.config.output.target_env.is_node())
          {
            page_resources::collect(plan, context, param.resources_map)
              .map_err(CompilationError::GenericError)?
          } else {
            None
          }
        };
        mark("page_resources_done");
        let (snapshot_id, not_found, route_paths) = {
          let guard = self
            .plan
            .read()
            .map_err(|error| diagnostic("DOCS_STATE", error))?;
          match guard.as_ref() {
            Some(plan) => (
              plan.snapshot_id.clone(),
              farmfe_docs_plan::not_found_outputs(plan),
              plan
                .routes
                .iter()
                .map(|route| route.page.route_path.clone())
                .collect::<Vec<_>>(),
            ),
            None => (self.snapshot()?.snapshot_id, Vec::new(), Vec::new()),
          }
        };
        let mut manifest = json!({
          "schemaVersion": 1,
          "snapshotId": snapshot_id,
          "resources": resources,
          "template": template,
          "pageResources": page_resources,
          "notFound": not_found,
          "routePaths": route_paths,
        });
        if !context.config.output.target_env.is_node() {
          let guard = self
            .plan
            .read()
            .map_err(|error| diagnostic("DOCS_STATE", error))?;
          if let Some(plan) = guard.as_ref() {
            if let Some(summary) = farmfe_docs_plan::machine_readable::summary(plan)
              .map_err(CompilationError::GenericError)?
            {
              manifest["machineReadable"] = summary;
            }
          }
        }
        let bytes =
          serde_json::to_vec(&manifest).map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
        param.resources_map.insert(
          "docs-manifest.json".to_string(),
          Resource {
            name: "docs-manifest.json".to_string(),
            bytes,
            resource_type: ResourceType::Custom("json".to_string()),
            origin: ResourceOrigin::Module(
              context
                .module_graph
                .read()
                .entries
                .keys()
                .next()
                .cloned()
                .ok_or_else(|| diagnostic("DOCS_STATE", "entry module missing"))?,
            ),
            should_transform_output_filename: false,
            ..Default::default()
          },
        );
        Ok(Some(()))
      })()
    }?;
    for (name, resource) in &mut candidate {
      if !existing.contains(name) {
        let marker = resource_owner::stamp(self.name(), name, &resource.bytes)
          .map_err(|error| diagnostic("DOCS_RESOURCE_OWNER", error))?;
        resource
          .meta
          .insert(resource_owner::META_KEY.into(), marker);
      }
    }
    *param.resources_map = candidate;
    mark("committed");
    Ok(result)
  }
}

#[cfg(test)]
mod preflight_fixture;
#[cfg(test)]
mod preflight_tests;

#[cfg(test)]
mod development_tests;

#[cfg(test)]
mod virtual_id_tests {
  use super::{APP_VIRTUAL_ID, SITE_VIRTUAL_ID, VIRTUAL_ID};
  use farmfe_core::module::ModuleId;

  fn assert_update_identity(id: &str) {
    for name in ["farm-docs-update-root", "farm docs 更新 root"] {
      let root = std::env::temp_dir().join(name);
      let root = root.to_string_lossy();
      assert_eq!(
        ModuleId::new(id, "", &root).resolved_path(&root),
        id,
        "watch-graph updates must retain virtual identity"
      );
      assert_eq!(
        ModuleId::new(id, "?generation=1", &root).resolved_path_with_query(&root),
        format!("{id}?generation=1"),
        "query updates must retain the same virtual identity"
      );
    }
  }

  #[test]
  fn virtual_identity_snapshot() {
    assert_update_identity(VIRTUAL_ID);
  }

  #[test]
  fn virtual_identity_site() {
    assert_update_identity(SITE_VIRTUAL_ID);
  }

  #[test]
  fn virtual_identity_application() {
    assert_update_identity(APP_VIRTUAL_ID);
  }
}

#[cfg(test)]
mod snapshot_update_tests {
  use super::*;
  use farmfe_core::plugin::{PluginUpdateModulesHookParam, UpdateType};

  struct Fixture(PathBuf);
  impl Drop for Fixture {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.0);
    }
  }

  fn fixture(test: impl FnOnce(&FarmPluginDocs, &Arc<CompilationContext>, &PathBuf)) {
    let unique = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "farm-docs-snapshot-{}-{unique}",
      std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    let _owner = Fixture(root.clone());
    let source = root.join("snapshot.json");
    write_snapshot(&source, "initial");
    let mut config = Config::default();
    config.root = root.to_string_lossy().into_owned();
    let plugin = FarmPluginDocs::new(&config, json!({ "snapshotPath": source }).to_string());
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    plugin.build_start(&context).unwrap();
    test(&plugin, &context, &source);
  }

  fn write_snapshot(source: &PathBuf, id: &str) {
    fs::write(
      source,
      json!({ "schemaVersion": 1, "snapshotId": id, "routes": [] }).to_string(),
    )
    .unwrap();
  }

  fn update(path: &str) -> PluginUpdateModulesHookParam {
    PluginUpdateModulesHookParam {
      paths: vec![(path.into(), UpdateType::Updated)],
    }
  }

  #[test]
  fn snapshot_state_refreshes_for_watched_virtual_updates() {
    fixture(|plugin, context, source| {
      for (path, id) in [
        (VIRTUAL_ID.to_string(), "updated"),
        (format!("{VIRTUAL_ID}?generation=1"), "query-update"),
      ] {
        write_snapshot(source, id);
        plugin.update_modules(&mut update(&path), context).unwrap();
        assert_eq!(plugin.snapshot().unwrap().snapshot_id, id);
      }
    });
  }

  #[test]
  fn snapshot_state_rejects_bad_candidate_and_recovers_without_restart() {
    fixture(|plugin, context, source| {
      fs::write(source, "{").unwrap();
      let error = plugin
        .update_modules(&mut update(VIRTUAL_ID), context)
        .expect_err("invalid snapshot must reject the update");
      assert!(error.to_string().contains("DOCS_SNAPSHOT_PARSE"));
      assert_eq!(plugin.snapshot().unwrap().snapshot_id, "initial");
      write_snapshot(source, "recovered");
      plugin
        .update_modules(&mut update(VIRTUAL_ID), context)
        .unwrap();
      assert_eq!(plugin.snapshot().unwrap().snapshot_id, "recovered");
    });
  }

  #[test]
  fn snapshot_state_ignores_unrelated_updates() {
    fixture(|plugin, context, source| {
      fs::write(source, "{").unwrap();
      plugin
        .update_modules(&mut update("virtual:unrelated"), context)
        .unwrap();
      assert_eq!(plugin.snapshot().unwrap().snapshot_id, "initial");
    });
  }

  #[test]
  fn finalize_rejects_foreign_manifest_without_overwriting_it() {
    fixture(|plugin, context, _source| {
      let entry = ModuleId::new("virtual:test-entry", "", &context.config.root);
      context
        .module_graph
        .write()
        .entries
        .insert(entry, "app".into());
      for name in [
        "docs-manifest.json",
        "DOCS-manifest.json",
        "docs-manifest.json/foreign.js",
      ] {
        let mut resources = farmfe_core::HashMap::default();
        resources.insert(
          name.into(),
          Resource {
            name: name.into(),
            bytes: b"foreign".to_vec(),
            ..Default::default()
          },
        );
        let error = plugin
          .finalize_resources(
            &mut PluginFinalizeResourcesHookParam {
              resources_map: &mut resources,
              config: &context.config,
            },
            context,
          )
          .expect_err("foreign manifest namespace must not be overwritten");
        assert!(error.to_string().contains("COLLISION"), "{error}");
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[name].bytes, b"foreign");
      }
    });
  }

  #[test]
  fn finalize_stamps_its_manifest_for_safe_repeated_updates() {
    fixture(|plugin, context, _source| {
      let entry = ModuleId::new("virtual:test-entry", "", &context.config.root);
      context
        .module_graph
        .write()
        .entries
        .insert(entry, "app".into());
      let mut resources = farmfe_core::HashMap::default();
      for _ in 0..2 {
        plugin
          .finalize_resources(
            &mut PluginFinalizeResourcesHookParam {
              resources_map: &mut resources,
              config: &context.config,
            },
            context,
          )
          .unwrap();
        let resource = &resources["docs-manifest.json"];
        assert!(farmfe_docs_plan::resource_owner::is_owned(
          resource
            .meta
            .get(farmfe_docs_plan::resource_owner::META_KEY)
            .map(String::as_str),
          plugin.name(),
          &resource.name,
          &resource.bytes
        ));
        assert_eq!(resources.len(), 1);
      }
    });
  }

  #[test]
  fn finalize_failure_preserves_previous_manifest_and_recovers() {
    fixture(|plugin, context, _source| {
      let entry = ModuleId::new("virtual:test-entry", "", &context.config.root);
      context
        .module_graph
        .write()
        .entries
        .insert(entry, "app".into());
      let mut resources = farmfe_core::HashMap::default();
      plugin
        .finalize_resources(
          &mut PluginFinalizeResourcesHookParam {
            resources_map: &mut resources,
            config: &context.config,
          },
          context,
        )
        .unwrap();
      let before = resources["docs-manifest.json"].clone();
      let snapshot = plugin.snapshot.write().unwrap().take();
      assert!(
        plugin
          .finalize_resources(
            &mut PluginFinalizeResourcesHookParam {
              resources_map: &mut resources,
              config: &context.config
            },
            context
          )
          .is_err()
      );
      assert_eq!(resources.len(), 1);
      assert_eq!(resources["docs-manifest.json"].bytes, before.bytes);
      assert_eq!(resources["docs-manifest.json"].meta, before.meta);
      *plugin.snapshot.write().unwrap() = snapshot;
      plugin
        .finalize_resources(
          &mut PluginFinalizeResourcesHookParam {
            resources_map: &mut resources,
            config: &context.config,
          },
          context,
        )
        .unwrap();
      assert_eq!(resources["docs-manifest.json"].bytes, before.bytes);
    });
  }

  #[test]
  fn finalize_rejects_altered_manifest_instead_of_claiming_it() {
    fixture(|plugin, context, _source| {
      let entry = ModuleId::new("virtual:test-entry", "", &context.config.root);
      context
        .module_graph
        .write()
        .entries
        .insert(entry, "app".into());
      for strip_marker in [true, false] {
        let mut resources = farmfe_core::HashMap::default();
        plugin
          .finalize_resources(
            &mut PluginFinalizeResourcesHookParam {
              resources_map: &mut resources,
              config: &context.config,
            },
            context,
          )
          .unwrap();
        let resource = resources.get_mut("docs-manifest.json").unwrap();
        if strip_marker {
          resource.meta.clear();
        } else {
          resource.bytes.push(b'!');
        }
        let before = resource.clone();
        assert!(
          plugin
            .finalize_resources(
              &mut PluginFinalizeResourcesHookParam {
                resources_map: &mut resources,
                config: &context.config
              },
              context
            )
            .is_err()
        );
        assert_eq!(resources.len(), 1);
        assert_eq!(resources["docs-manifest.json"].bytes, before.bytes);
        assert_eq!(resources["docs-manifest.json"].meta, before.meta);
      }
    });
  }
}
