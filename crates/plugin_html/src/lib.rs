use std::{mem, sync::Arc};

use absolute_path_handler::AbsolutePathHandler;
use deps_analyzer::{DepsAnalyzer, HtmlInlineModule, HTML_INLINE_ID_PREFIX};
use farmfe_core::module::meta_data::html::HtmlModuleMetaData;

use farmfe_core::parking_lot::Mutex;
use farmfe_core::plugin::GeneratedResource;
use farmfe_core::resource::meta_data::html::HtmlResourcePotMetaData;
use farmfe_core::resource::meta_data::ResourcePotMetaData;
use farmfe_core::swc_common::Globals;
use farmfe_core::{cache_item, deserialize, serialize, HashMap};
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  module::{ModuleId, ModuleMetaData, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeResourcesHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookResult, ResolveKind,
  },
  relative_path::RelativePath,
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
};
use farmfe_toolkit::minify::minify_html_module;
use farmfe_toolkit::plugin_utils::path_filter::PathFilter;
use farmfe_toolkit::sourcemap::create_swc_source_map;
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::{codegen_html_document, parse_html_document},
  script::{module_type_from_id, swc_try_with::try_with},
};
use resources_injector::{ResourcesInjector, ResourcesInjectorOptions};

mod absolute_path_handler;
mod deps_analyzer;
mod resources_injector;
mod utils;

const BASE_HTML_CHILDREN_PLACEHOLDER: &str = "{{children}}";
pub const UNRESOLVED_SLASH_MODULE: &str = "FARM_HTML_UNRESOLVED_SLASH_MODULE";

#[cache_item(farmfe_core)]
struct CachedHtmlInlineModuleMap {
  map: HashMap<String, HtmlInlineModule>,
}

pub struct FarmPluginHtml {
  inline_module_map: Mutex<HashMap<String, HtmlInlineModule>>,
}

impl Plugin for FarmPluginHtml {
  fn name(&self) -> &str {
    "FarmPluginHtml"
  }

  fn build_start(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if context.cache_manager.enable() {
      if let Some(v) = context
        .cache_manager
        .plugin_cache
        .read_cache_item::<CachedHtmlInlineModuleMap>(self.name())
      {
        let mut inline_module_map = self.inline_module_map.lock();
        inline_module_map.extend(v.map);
      };
    }

    Ok(None)
  }

  fn finish(
    &self,
    _stat: &farmfe_core::stats::Stats,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if context.cache_manager.enable() {
      let module_graph = context.module_graph.read();

      context.cache_manager.plugin_cache.write_cache_item(
        self.name(),
        CachedHtmlInlineModuleMap {
          map: self
            .inline_module_map
            .lock()
            .iter()
            .filter(|(k, v)| {
              module_graph.has_module(&k.as_str().into()) && module_graph.has_module(&v.html_id)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        },
      );
    }

    Ok(None)
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if hook_context.caller == Some(self.name().to_string()) {
      return Ok(None);
    }

    if param.source.starts_with(HTML_INLINE_ID_PREFIX) {
      return Ok(Some(PluginResolveHookResult {
        resolved_path: param.source.clone(),
        external: false,
        side_effects: false,
        ..Default::default()
      }));
    }

    // try resolve `/xxx` as `./xxx`, for example: `/src/index.ts` to `./src/index.ts`
    if matches!(param.kind, ResolveKind::ScriptSrc | ResolveKind::LinkHref)
      && param.source.starts_with("/")
    {
      let resolve_result = context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source: format!(".{}", param.source),
          importer: param.importer.clone(),
          kind: param.kind.clone(),
        },
        context,
        &PluginHookContext {
          caller: Some(self.name().to_string()),
          meta: hook_context.meta.clone(),
        },
      )?;

      return Ok(Some(resolve_result.unwrap_or(PluginResolveHookResult {
        resolved_path: UNRESOLVED_SLASH_MODULE.to_string(),
        external: true,
        ..Default::default()
      })));
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if param.resolved_path.starts_with(HTML_INLINE_ID_PREFIX) {
      let inline_module_map = self.inline_module_map.lock();
      if let Some(inline_module) = inline_module_map.get(param.resolved_path) {
        return Ok(Some(PluginLoadHookResult {
          content: inline_module.code.clone(),
          module_type: inline_module.module_type.clone(),
          source_map: None,
        }));
      };
    }

    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Html) {
        Ok(Some(PluginLoadHookResult {
          content: read_file_utf8(param.resolved_path)?,
          module_type,
          source_map: None,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  /// Inherit base html
  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    if param.module_type != ModuleType::Html {
      return Ok(None);
    }

    if let Some(base) = &context.config.html.base {
      let base_html = self
        .load(
          &PluginLoadHookParam {
            resolved_path: RelativePath::new(base)
              .to_logical_path(&context.config.root)
              .to_str()
              .unwrap(),
            query: vec![],
            meta: HashMap::default(),
            module_id: param.module_id.clone(),
          },
          context,
          &PluginHookContext::default(),
        )
        .map_err(|e| CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: format!("Load base html({base}) fail. Error: {e:?}"),
        })?
        .ok_or(CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: format!("Load base html({base}) fail: Base html file does not exist"),
        })?;

      return Ok(Some(PluginTransformHookResult {
        content: base_html
          .content
          .replace(BASE_HTML_CHILDREN_PLACEHOLDER, &param.content),
        module_type: None,
        source_map: None,
        ignore_previous_source_map: false,
      }));
    }

    Ok(None)
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::module::ModuleMetaData>> {
    if matches!(param.module_type, ModuleType::Html) {
      // Ignore query string when parsing html. HTML should not be affected by query string.
      let module_id = ModuleId::new(&param.resolved_path, "", &context.config.root);
      let html_document =
        parse_html_document(module_id.to_string().as_str(), param.content.clone())?;

      let meta = ModuleMetaData::Html(Box::new(HtmlModuleMetaData {
        ast: html_document,
        custom: Default::default(),
      }));

      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    _context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(param.module.module_type, ModuleType::Html) {
      let document = &param.module.meta.as_html().ast;
      let mut deps_analyzer = DepsAnalyzer::new(param.module.id.clone());

      param.deps.extend(deps_analyzer.analyze_deps(document));

      self
        .inline_module_map
        .lock()
        .extend(deps_analyzer.inline_deps_map);

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      let modules = resource_pot.modules();

      if modules.len() != 1 {
        return Err(CompilationError::RenderHtmlResourcePotError {
          name: resource_pot.id.to_string(),
          modules: modules.into_iter().map(|m| m.to_string()).collect(),
        });
      }

      return Ok(Some(ResourcePotMetaData::Html(HtmlResourcePotMetaData {
        custom: Default::default(),
      })));
    }

    Ok(None)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      Ok(Some(PluginGenerateResourcesHookResult {
        resources: vec![GeneratedResource {
          resource: Resource {
            name: resource_pot.id.to_string(),
            bytes: vec![],
            emitted: false,
            resource_type: ResourceType::Html,
            origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
            should_transform_output_filename: true,
            meta: Default::default(),
          },
          source_map: None,
        }],
      }))
    } else {
      Ok(None)
    }
  }

  // fn plugin_cache_loaded(
  //   &self,
  //   cache: &Vec<u8>,
  //   _context: &Arc<CompilationContext>,
  // ) -> farmfe_core::error::Result<Option<()>> {
  //   let cached_inline_module_map = deserialize!(cache, CachedHtmlInlineModuleMap).map;
  //   let mut inline_module_map = self.inline_module_map.lock();
  //   inline_module_map.extend(cached_inline_module_map);

  //   Ok(Some(()))
  // }

  // fn write_plugin_cache(
  //   &self,
  //   context: &Arc<CompilationContext>,
  // ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
  //   let inline_module_map = self.inline_module_map.lock();
  //   let cached_inline_module_map = CachedHtmlInlineModuleMap {
  //     map: inline_module_map
  //       .iter()
  //       .filter(|(k, v)| {
  //         let module_graph = context.module_graph.read();
  //         module_graph.has_module(&k.as_str().into()) && module_graph.has_module(&v.html_id)
  //       })
  //       .map(|(k, v)| (k.clone(), v.clone()))
  //       .collect(),
  //   };

  //   let bytes = serialize!(&cached_inline_module_map);

  //   Ok(Some(bytes))
  // }
}

impl FarmPluginHtml {
  pub fn new(_: &Config) -> Self {
    Self {
      inline_module_map: Mutex::new(HashMap::default()),
    }
  }
}

pub struct FarmPluginTransformHtml {
  // minify_config: MinifyBuilder,
}

impl Plugin for FarmPluginTransformHtml {
  fn name(&self) -> &str {
    "FarmPluginTransformHtml"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn handle_entry_resource(
    &self,
    param: &mut farmfe_core::plugin::PluginHandleEntryResourceHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !matches!(param.resource.resource_type, ResourceType::Html) {
      return Ok(None);
    }

    let mut script_resources: Vec<String> = vec![];
    let mut css_resources: Vec<String> = vec![];

    for (name, ty) in &param.initial_resources {
      if matches!(ty, ResourceType::Js) {
        script_resources.push(name.clone());
      } else if matches!(ty, ResourceType::Css) {
        css_resources.push(name.clone());
      }
    }

    let html_resource = &mut param.resource;

    let module_graph = param.module_graph;
    let current_html_id = param
      .resource_pot_map
      .resource_pot(html_resource.origin.as_resource_pot())
      .unwrap()
      .modules()[0];
    let script_entries = module_graph
      .dependencies(current_html_id)
      .into_iter()
      .filter_map(|dep| {
        let dep_module = module_graph.module(&dep.0).unwrap();

        if dep_module.module_type.is_script() {
          Some(dep.0.id(context.config.mode.clone()))
        } else {
          None
        }
      })
      .collect();

    let mut resources_injector = ResourcesInjector::new(
      vec![],
      param.runtime_code,
      script_resources,
      css_resources,
      script_entries,
      &param.dynamic_resources,
      &param.dynamic_module_resources_map,
      ResourcesInjectorOptions {
        public_path: context.config.output.public_path.clone(),
        namespace: context.config.runtime.namespace.clone(),
        current_html_id: current_html_id.clone(),
        context: context.clone(),
      },
    );

    let html_module = module_graph.module(&current_html_id).unwrap();
    let mut html_ast = html_module.meta.as_html().ast.clone();
    resources_injector.inject(&mut html_ast);

    // set publicPath prefix
    let mut absolute_path_handler = AbsolutePathHandler {
      public_path: context.config.output.public_path.clone(),
    };
    absolute_path_handler.add_public_path_prefix(&mut html_ast);

    let code = codegen_html_document(
      &html_ast,
      should_minify_html(&html_resource.name, &context.config),
    );
    html_resource.bytes = code.bytes().collect();

    for resource in resources_injector.additional_inject_resources {
      if !param
        .additional_inject_resources
        .contains_key(&resource.name)
      {
        param
          .additional_inject_resources
          .insert(resource.name.clone(), resource);
      }
    }

    Ok(None)
  }
}

impl FarmPluginTransformHtml {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

pub struct FarmPluginMinifyHtml {}

impl FarmPluginMinifyHtml {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginMinifyHtml {
  fn name(&self) -> &str {
    "FarmPluginMinifyHtml"
  }

  fn priority(&self) -> i32 {
    -99
  }

  fn finalize_resources(
    &self,
    params: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    for resource in params.resources_map.values_mut() {
      if matches!(resource.resource_type, ResourceType::Html) {
        if !should_minify_html(&resource.name, &context.config) {
          continue;
        }

        let bytes = mem::take(&mut resource.bytes);
        let html_code = Arc::new(String::from_utf8(bytes).unwrap());
        let mut html_ast = match parse_html_document(&resource.name, html_code.clone()) {
          Ok(ast) => ast,
          Err(err) => {
            let farm_debug_html_minify = "FARM_DEBUG_HTML_MINIFY";

            if let Ok(_) = std::env::var(farm_debug_html_minify) {
              println!(
                "Can not minify html {} due to html syntax error: {}",
                resource.name,
                err.to_string()
              );
            } else {
              println!("Can not minify html {} due to html syntax error. Try {farm_debug_html_minify}=1 to see error details", resource.name);
            }
            resource.bytes = Arc::try_unwrap(html_code).unwrap().into_bytes();
            return Ok(Some(()));
          }
        };

        let (cm, _) = create_swc_source_map(&resource.name.as_str().into(), html_code.clone());
        let globals = Globals::new();
        try_with(cm, &globals, || {
          minify_html_module(&mut html_ast);
        })?;

        let html_code = codegen_html_document(&html_ast, true);
        resource.bytes = html_code.into_bytes();
      }
    }

    Ok(Some(()))
  }
}

fn should_minify_html(name: &str, config: &Config) -> bool {
  let default_minify_options = Default::default();
  let minify_options = config.minify.as_obj().unwrap_or(&default_minify_options);
  let filter = PathFilter::new(&minify_options.include, &minify_options.exclude);

  filter.execute(name)
}
