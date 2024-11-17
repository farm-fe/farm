use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use absolute_path_handler::AbsolutePathHandler;
use deps_analyzer::{DepsAnalyzer, HtmlInlineModule, HTML_INLINE_ID_PREFIX};
// use farmfe_core::config::minify::MinifyOptions;
use farmfe_core::parking_lot::Mutex;
use farmfe_core::{cache_item, deserialize, serialize};
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  module::{HtmlModuleMetaData, ModuleId, ModuleMetaData, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeResourcesHookParams,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginResolveHookParam, PluginResolveHookResult,
    PluginTransformHookResult, ResolveKind,
  },
  relative_path::RelativePath,
  resource::{
    resource_pot::{RenderedModule, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
};
use farmfe_toolkit::common::{create_swc_source_map, MinifyBuilder, Source};
use farmfe_toolkit::minify::minify_html_module;
use farmfe_toolkit::{
  fs::read_file_utf8,
  get_dynamic_resources_map::get_dynamic_resources_map,
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
      }
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
            meta: std::collections::HashMap::new(),
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

      let meta = ModuleMetaData::Html(HtmlModuleMetaData {
        ast: html_document,
        custom: Default::default(),
      });

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

  fn render_resource_pot_modules(
    &self,
    resource_pot: &ResourcePot,
    context: &std::sync::Arc<CompilationContext>,
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

      let module_graph = context.module_graph.read();
      let html_module = module_graph.module(modules[0]).unwrap();
      let html_module_document = html_module.meta.as_html().ast.clone();

      let code = Arc::new(codegen_html_document(&html_module_document, false));

      Ok(Some(ResourcePotMetaData {
        rendered_modules: std::collections::HashMap::from([(
          modules[0].clone(),
          RenderedModule {
            id: modules[0].clone(),
            rendered_content: code.clone(),
            rendered_map: None,
            rendered_length: html_module.size,
            original_length: code.len(),
          },
        )]),
        rendered_content: code,
        rendered_map_chain: vec![],
        ..Default::default()
      }))
    } else {
      Ok(None)
    }
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      Ok(Some(PluginGenerateResourcesHookResult {
        resource: Resource {
          name: resource_pot.id.to_string(),
          bytes: vec![],
          emitted: false,
          resource_type: ResourceType::Html,
          origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
          info: None,
        },
        source_map: None,
      }))
    } else {
      Ok(None)
    }
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cached_inline_module_map = deserialize!(cache, CachedHtmlInlineModuleMap).map;
    let mut inline_module_map = self.inline_module_map.lock();
    inline_module_map.extend(cached_inline_module_map);

    Ok(Some(()))
  }

  fn write_plugin_cache(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let inline_module_map = self.inline_module_map.lock();
    let cached_inline_module_map = CachedHtmlInlineModuleMap {
      map: inline_module_map
        .iter()
        .filter(|(k, v)| {
          let module_graph = context.module_graph.read();
          module_graph.has_module(&k.as_str().into()) && module_graph.has_module(&v.html_id)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect(),
    };

    let bytes = serialize!(&cached_inline_module_map);

    Ok(Some(bytes))
  }
}

impl FarmPluginHtml {
  pub fn new(_: &Config) -> Self {
    Self {
      inline_module_map: Mutex::new(HashMap::new()),
    }
  }
}

pub struct FarmPluginTransformHtml {
  minify_config: MinifyBuilder,
}

impl Plugin for FarmPluginTransformHtml {
  fn name(&self) -> &str {
    "FarmPluginTransformHtml"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn finalize_resources(
    &self,
    params: &mut PluginFinalizeResourcesHookParams,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // 1. inject runtime as inline <script>
    // 2. inject script and css link in topo order
    // 3. execute direct script module dependency

    let mut runtime_code = Rc::new(String::new());

    for resource in params.resources_map.values() {
      if matches!(resource.resource_type, ResourceType::Runtime) {
        runtime_code = Rc::new(String::from_utf8(resource.bytes.to_vec()).unwrap());
        break;
      }
    }

    let module_graph = context.module_graph.read();
    let html_entries_ids = module_graph
      .entries
      .clone()
      .into_iter()
      .filter(|(m, _)| {
        module_graph
          .module(m)
          .is_some_and(|m| matches!(m.module_type, ModuleType::Html))
      })
      .collect::<Vec<_>>();

    let mut resources_to_inject = HashMap::new();

    for (html_entry_id, _) in &html_entries_ids {
      let module_group_id = html_entry_id.clone();

      let resource_pot_map = context.resource_pot_map.read();
      let module_group_graph = context.module_group_graph.read();
      let module_group = module_group_graph.module_group(&module_group_id).unwrap();

      // Found all resources in this entry html module group
      let mut dep_resources = vec![];
      let mut html_entry_resource = None;

      let sorted_resource_pots =
        module_group.sorted_resource_pots(&module_graph, &resource_pot_map);

      for rp_id in &sorted_resource_pots {
        let rp = resource_pot_map.resource_pot(rp_id).unwrap_or_else(|| {
          panic!(
            "Resource pot {} not found in resource pot map",
            rp_id.to_string()
          )
        });

        for resource in rp.resources() {
          if rp.modules().contains(&html_entry_id) {
            html_entry_resource = Some(resource.clone());
            continue;
          }
        }

        dep_resources.extend(rp.resources().into_iter().map(|r| r.to_string()));
      }

      let dynamic_resources_map = get_dynamic_resources_map(
        &module_group_graph,
        &module_group_id,
        &resource_pot_map,
        &params.resources_map,
        &module_graph,
      );

      resources_to_inject.insert(
        html_entry_resource.unwrap(),
        (dep_resources, dynamic_resources_map),
      );
    }

    let mut already_injected_resources = Vec::new();

    for (html_resource_name, (dep_resources, dynamic_resources_map)) in resources_to_inject {
      let mut resource_pot_map = context.resource_pot_map.write();
      let mut script_resources: Vec<String> = vec![];
      let mut css_resources: Vec<String> = vec![];

      for res_id in dep_resources {
        if !params.resources_map.contains_key(&res_id) {
          continue;
        }
        let res = params.resources_map.get(&res_id).unwrap();

        if matches!(res.resource_type, ResourceType::Js) {
          script_resources.push(res.name.clone());
        } else if matches!(res.resource_type, ResourceType::Css) {
          css_resources.push(res.name.clone());
        }
      }

      let html_resource = params.resources_map.get_mut(&html_resource_name).unwrap();

      let module_graph = context.module_graph.read();
      let current_html_id = resource_pot_map
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
      drop(module_graph);

      let mut resources_injector = ResourcesInjector::new(
        vec![],
        runtime_code.clone(),
        script_resources,
        css_resources,
        script_entries,
        dynamic_resources_map,
        ResourcesInjectorOptions {
          mode: context.config.mode.clone(),
          public_path: context.config.output.public_path.clone(),
          namespace: context.config.runtime.namespace.clone(),
          current_html_id: current_html_id.clone(),
          context: context.clone(),
        },
        &mut already_injected_resources,
      );

      let resource_pot = resource_pot_map
        .resource_pot_mut(html_resource.origin.as_resource_pot())
        .unwrap();
      let mut html_ast =
        parse_html_document(&resource_pot.id, resource_pot.meta.rendered_content.clone())?;

      resources_injector.inject(&mut html_ast);

      // set publicPath prefix
      let mut absolute_path_handler = AbsolutePathHandler {
        public_path: context.config.output.public_path.clone(),
      };
      absolute_path_handler.add_public_path_prefix(&mut html_ast);

      let code = codegen_html_document(
        &html_ast,
        self.minify_config.is_enabled(&html_resource.name),
      );
      html_resource.bytes = code.bytes().collect();

      resources_injector.update_resource(params.resources_map);
    }

    Ok(None)
  }
}

impl FarmPluginTransformHtml {
  pub fn new(config: &Config) -> Self {
    Self {
      minify_config: MinifyBuilder::create_builder(&config.minify, None),
    }
  }
}

pub struct FarmPluginMinifyHtml {
  minify_config: MinifyBuilder,
}

impl FarmPluginMinifyHtml {
  pub fn new(config: &Config) -> Self {
    Self {
      minify_config: MinifyBuilder::create_builder(&config.minify, None),
    }
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
    params: &mut PluginFinalizeResourcesHookParams,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    for resource in params.resources_map.values_mut() {
      if matches!(resource.resource_type, ResourceType::Html) {
        if !self.minify_config.is_enabled(&resource.name) {
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

        let (cm, _) = create_swc_source_map(Source {
          path: PathBuf::from(&resource.name),
          content: html_code.clone(),
        });

        try_with(cm, &context.meta.html.globals, || {
          minify_html_module(&mut html_ast);
        })?;

        let html_code = codegen_html_document(&html_ast, true);
        resource.bytes = html_code.into_bytes();
      }
    }

    Ok(Some(()))
  }
}
