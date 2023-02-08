use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::CompilationError,
  hashbrown::HashMap,
  module::{HtmlModuleMetaData, ModuleId, ModuleMetaData, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam,
  },
  resource::{
    resource_pot::{HtmlResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceType,
  },
  swc_html_ast::Document,
};
use farmfe_toolkit::{
  fs::read_file_utf8,
  html::{codegen_html_document, parse_html_document},
  script::module_type_from_id,
};
use resources_injector::ResourcesInjector;

mod deps_analyzer;
mod resources_injector;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginHtml {}

impl Plugin for FarmPluginHtml {
  fn name(&self) -> &str {
    "FarmPluginHtml"
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if matches!(module_type, ModuleType::Html) {
      Ok(Some(PluginLoadHookResult {
        content: read_file_utf8(param.resolved_path)?,
        module_type,
      }))
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &std::sync::Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::module::ModuleMetaData>> {
    if matches!(param.module_type, ModuleType::Html) {
      let module_id = ModuleId::new(&param.resolved_path, &context.config.root);
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
        resource_pot: resource_pot.id.clone(),
      }]))
    } else {
      Ok(None)
    }
  }

  fn write_resources(
    &self,
    resources_map: &mut HashMap<String, Resource>,
    context: &std::sync::Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut runtime_code = String::new();

    for resource in resources_map.values() {
      if matches!(resource.resource_type, ResourceType::Runtime) {
        runtime_code = String::from_utf8(resource.bytes.to_vec()).unwrap();
        break;
      }
    }

    let mut resources_to_inject = HashMap::new();

    for resource in resources_map.values_mut() {
      if matches!(resource.resource_type, ResourceType::Html) {
        let mut resource_pot_graph = context.resource_pot_graph.write();

        // 1. inject runtime as inline <script>
        // 2. inject script and css link in topo order
        // 3. execute direct script module dependency
        let module_group_id = {
          let resource_pot_id = &resource.resource_pot;
          let resource_pot = resource_pot_graph
            .resource_pot_mut(&resource_pot_id)
            .unwrap();

          resource_pot.module_group.clone()
        };

        let module_group_map = context.module_group_map.read();
        let module_group = module_group_map.module_group(&module_group_id).unwrap();

        let mut depend_resources = vec![];

        for rp_id in module_group.resource_pots() {
          let rp = resource_pot_graph.resource_pot(rp_id).unwrap();
          depend_resources.extend(rp.resources().into_iter().map(|r| r.to_string()));
        }

        resources_to_inject.insert(resource.name.clone(), depend_resources);
      }
    }

    for (html_resource_name, depend_resources) in resources_to_inject {
      let mut resource_pot_graph = context.resource_pot_graph.write();
      let mut script_resources: Vec<String> = vec![];
      let mut css_resources: Vec<String> = vec![];

      for res_id in depend_resources {
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
          resource_pot_graph
            .resource_pot(&html_resource.resource_pot)
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
      );

      let resource_pot = resource_pot_graph
        .resource_pot_mut(&html_resource.resource_pot)
        .unwrap();
      let html_ast = &mut resource_pot.meta.as_html_mut().ast;
      resources_injector.inject(html_ast);

      let code = codegen_html_document(html_ast);
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
