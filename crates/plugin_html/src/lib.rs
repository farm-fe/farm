use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashMap,
  module::{
    module_group::{ModuleGroupGraph, ModuleGroupId},
    HtmlModuleMetaData, ModuleId, ModuleMetaData, ModuleType,
  },
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginTransformHookResult,
  },
  relative_path::RelativePath,
  resource::{
    resource_pot::{HtmlResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    resource_pot_map::ResourcePotMap,
    Resource, ResourceOrigin, ResourceType,
  },
  swc_html_ast::Document,
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::{codegen_html_document, parse_html_document},
  script::module_type_from_id,
};
use resources_injector::{ResourcesInjector, ResourcesInjectorOptions};

mod deps_analyzer;
mod resources_injector;
mod utils;

const BASE_HTML_CHILDREN_PLACEHOLDER: &str = "{{children}}";

pub struct FarmPluginHtml {}

impl Plugin for FarmPluginHtml {
  fn name(&self) -> &str {
    "FarmPluginHtml"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Html) {
        Ok(Some(PluginLoadHookResult {
          content: read_file_utf8(param.resolved_path)?,
          module_type,
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
          },
          context,
          &PluginHookContext::default(),
        )
        .map_err(|e| CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: format!("Load base html({}) fail. Error: {:?}", base, e),
        })?
        .ok_or(CompilationError::TransformError {
          resolved_path: param.resolved_path.to_string(),
          msg: format!(
            "Load base html({}) fail: Base html file does not exist",
            base
          ),
        })?;

      return Ok(Some(PluginTransformHookResult {
        content: base_html
          .content
          .replace(BASE_HTML_CHILDREN_PLACEHOLDER, &param.content),
        module_type: None,
        source_map: None,
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
      let html_document = parse_html_document(
        module_id.to_string().as_str(),
        &param.content,
        context.meta.html.cm.clone(),
      )?;

      let meta = ModuleMetaData::Html(HtmlModuleMetaData { ast: html_document });

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
      let mut deps_analyzer = DepsAnalyzer::new();

      param.deps.extend(deps_analyzer.analyze_deps(document));

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
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
      let html_module_document = &html_module.meta.as_html().ast;

      resource_pot.meta = ResourcePotMetaData::Html(HtmlResourcePotMetaData {
        ast: Document {
          span: html_module_document.span.clone(),
          mode: html_module_document.mode.clone(),
          children: html_module_document.children.to_vec(),
        },
      });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      Ok(Some(vec![Resource {
        name: resource_pot.id.to_string(),
        bytes: vec![],
        emitted: false,
        resource_type: ResourceType::Html,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
      }]))
    } else {
      Ok(None)
    }
  }

  fn finalize_resources(
    &self,
    resources_map: &mut HashMap<String, Resource>,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // 1. inject runtime as inline <script>
    // 2. inject script and css link in topo order
    // 3. execute direct script module dependency

    let mut runtime_code = String::new();

    for resource in resources_map.values() {
      if matches!(resource.resource_type, ResourceType::Runtime) {
        runtime_code = String::from_utf8(resource.bytes.to_vec()).unwrap();
        break;
      }
    }

    let module_graph = context.module_graph.read();
    let html_entries_ids = module_graph
      .entries
      .clone()
      .into_iter()
      .filter(|(m, _)| {
        let module = module_graph.module(m).unwrap();
        matches!(module.module_type, ModuleType::Html)
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
      let mut resource_pots_order_map = HashMap::<String, usize>::new();
      // TODO make the resource pots order execution order when partial bundling
      let mut sorted_resource_pots = module_group.resource_pots().into_iter().collect::<Vec<_>>();
      sorted_resource_pots.iter().for_each(|rp| {
        let rp = resource_pot_map.resource_pot(rp).unwrap();
        let max_order = rp
          .modules()
          .iter()
          .map(|m| {
            let module = module_graph.module(m).unwrap();
            module.execution_order
          })
          .min()
          .unwrap_or(0);

        resource_pots_order_map.insert(rp.id.to_string(), max_order);
      });
      sorted_resource_pots.sort_by(|a, b| {
        let a_order = resource_pots_order_map.get(&a.to_string()).unwrap_or(&0);
        let b_order = resource_pots_order_map.get(&b.to_string()).unwrap_or(&0);

        a_order.cmp(b_order)
      });

      for rp_id in sorted_resource_pots {
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
        &*module_group_graph,
        &module_group_id,
        &*resource_pot_map,
        resources_map,
      );

      resources_to_inject.insert(
        html_entry_resource.unwrap(),
        (dep_resources, dynamic_resources_map),
      );
    }

    for (html_resource_name, (dep_resources, dynamic_resources_map)) in resources_to_inject {
      let mut resource_pot_map = context.resource_pot_map.write();
      let mut script_resources: Vec<String> = vec![];
      let mut css_resources: Vec<String> = vec![];

      for res_id in dep_resources {
        let res = resources_map.get(&res_id).unwrap();

        if matches!(res.resource_type, ResourceType::Js) {
          script_resources.push(res.name.clone());
        } else if matches!(res.resource_type, ResourceType::Css) {
          css_resources.push(res.name.clone());
        }
      }

      let html_resource = resources_map.get_mut(&html_resource_name).unwrap();

      let module_graph = context.module_graph.read();
      let script_entries = module_graph
        .dependencies(
          resource_pot_map
            .resource_pot(html_resource.origin.as_resource_pot())
            .unwrap()
            .modules()[0],
        )
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
        runtime_code.clone(),
        script_resources,
        css_resources,
        script_entries,
        dynamic_resources_map,
        ResourcesInjectorOptions {
          mode: context.config.mode.clone(),
          public_path: context.config.output.public_path.clone(),
          define: context.config.define.clone(),
        },
      );

      let resource_pot = resource_pot_map
        .resource_pot_mut(&html_resource.origin.as_resource_pot())
        .unwrap();
      let html_ast = &mut resource_pot.meta.as_html_mut().ast;
      resources_injector.inject(html_ast);

      let code = codegen_html_document(html_ast, context.config.minify);
      html_resource.bytes = code.bytes().collect();
    }

    Ok(None)
  }
}

impl FarmPluginHtml {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

pub fn get_dynamic_resources_map(
  module_group_graph: &ModuleGroupGraph,
  module_group_id: &ModuleGroupId,
  resource_pot_map: &ResourcePotMap,
  resources_map: &HashMap<String, Resource>,
) -> HashMap<ModuleId, Vec<(String, ResourceType)>> {
  let mut dep_module_groups = vec![];

  module_group_graph.bfs(&module_group_id, &mut |mg_id| {
    if mg_id != module_group_id {
      dep_module_groups.push(mg_id.clone());
    }
  });

  let mut dynamic_resources_map = HashMap::<ModuleId, Vec<(String, ResourceType)>>::new();

  for mg_id in dep_module_groups {
    let mg = module_group_graph.module_group(&mg_id).unwrap();

    for rp_id in mg.resource_pots() {
      let rp = resource_pot_map.resource_pot(rp_id).unwrap_or_else(|| {
        panic!(
          "Resource pot {} not found in resource pot map",
          rp_id.to_string()
        )
      });

      if dynamic_resources_map.contains_key(&mg_id) {
        let resources = dynamic_resources_map.get_mut(&mg_id).unwrap();

        for r in rp.resources() {
          let resource = resources_map.get(r).unwrap();

          // Currently only support js and css
          if !matches!(resource.resource_type, ResourceType::Js | ResourceType::Css) {
            continue;
          }

          resources.push((resource.name.clone(), resource.resource_type.clone()));
        }
      } else {
        let mut resources = vec![];

        for r in rp.resources() {
          let resource = resources_map
            .get(r)
            .unwrap_or_else(|| panic!("{} not found", r));

          // Currently only support js and css
          if !matches!(resource.resource_type, ResourceType::Js | ResourceType::Css) {
            continue;
          }

          resources.push((resource.name.clone(), resource.resource_type.clone()));
        }

        dynamic_resources_map.insert(mg_id.clone(), resources);
      }
    }
  }

  dynamic_resources_map
}
