#![feature(box_patterns)]
#![feature(path_file_prefix)]

use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::{Config, ModuleFormat, TargetEnv},
  context::CompilationContext,
  error::Result,
  module::{
    CommentsMetaData, ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData,
    VIRTUAL_MODULE_PREFIX,
  },
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::{comments::SingleThreadedComments, Mark, GLOBALS},
  swc_ecma_ast::EsVersion,
};
use farmfe_swc_transformer_import_glob::transform_import_meta_glob;
use farmfe_toolkit::{
  common::{
    create_swc_source_map, generate_source_map_resource, load_source_original_source_map, Source,
  },
  fs::read_file_utf8,
  script::{
    module_type_from_id, parse_module, set_module_system_for_module_meta, swc_try_with::try_with,
    syntax_from_module_type, ParseScriptModuleResult,
  },
  swc_ecma_transforms::resolver,
  swc_ecma_visit::VisitMutWith,
};

use import_meta_visitor::{replace_import_meta_url, ImportMetaVisitor};
#[cfg(feature = "swc_plugin")]
use swc_plugins::{init_plugin_module_cache_once, transform_by_swc_plugins};

mod deps_analyzer;
mod import_meta_visitor;
#[cfg(feature = "swc_plugin")]
mod swc_plugins;
mod swc_script_transforms;
mod transform_import_meta_url;

use transform_import_meta_url::transform_url_with_import_meta_url;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx/... files, support loading, parse, analyze dependencies and code generation.
/// Note that we do not do transforms here, the transforms (e.g. strip types, jsx...) are handled in a separate plugin (farmfe_plugin_swc_transforms).
pub struct FarmPluginScript {}

impl Plugin for FarmPluginScript {
  fn name(&self) -> &str {
    "FarmPluginScript"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if module_type.is_script() {
        let content = read_file_utf8(param.resolved_path)?;

        let map =
          load_source_original_source_map(&content, param.resolved_path, "//# sourceMappingURL");

        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
          source_map: map,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ModuleMetaData>> {
    if let Some(syntax) =
      syntax_from_module_type(&param.module_type, context.config.script.parser.clone())
    {
      let ParseScriptModuleResult {
        ast: mut swc_module,
        comments,
      } = parse_module(
        &param.module_id.to_string(),
        &param.content,
        syntax,
        EsVersion::EsNext,
      )?;

      GLOBALS.set(&context.meta.script.globals, || {
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        swc_module.visit_mut_with(&mut resolver(
          unresolved_mark,
          top_level_mark,
          param.module_type.is_typescript(),
        ));

        let meta = ScriptModuleMetaData {
          ast: swc_module,
          top_level_mark: top_level_mark.as_u32(),
          unresolved_mark: unresolved_mark.as_u32(),
          // set module_system to unknown, it will be detected in `finalize_module`
          module_system: ModuleSystem::Custom(String::from("unknown")),
          hmr_self_accepted: false,
          hmr_accepted_deps: Default::default(),
          comments: CommentsMetaData::from(comments),
          custom: Default::default(),
        };

        Ok(Some(ModuleMetaData::Script(meta)))
      })
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module_type.is_script() {
      return Ok(None);
    }

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&param.module_id.to_string()),
      content: param.content.clone(),
    });

    // transform decorators if needed
    // this transform should be done before strip typescript cause it may need to access the type information
    if (param.module_type.is_typescript() && context.config.script.parser.ts_config.decorators)
      || (param.module_type.is_script() && context.config.script.parser.es_config.decorators)
    {
      swc_script_transforms::transform_decorators(param, &cm, context)?;
    }

    // strip typescript
    if param.module_type.is_typescript() {
      swc_script_transforms::strip_typescript(param, &cm, context)?;
    }

    // execute swc plugins
    #[cfg(feature = "swc_plugin")]
    if param.module_type.is_script() && !context.config.script.plugins.is_empty() {
      try_with(cm.clone(), &context.meta.script.globals, || {
        transform_by_swc_plugins(param, context).unwrap()
      })?;
    }

    if param.module_type.is_script() {
      // transform vite-style `import.meta.glob`
      let script = param.meta.as_script_mut();
      let comments: SingleThreadedComments = script.take_comments().into();
      let ast = &mut script.ast;
      let resolved_path = param.module_id.resolved_path(&context.config.root);
      let cur_dir = if resolved_path.starts_with(VIRTUAL_MODULE_PREFIX) {
        context.config.root.clone()
      } else {
        Path::new(&resolved_path)
          .parent()
          .unwrap()
          .to_string_lossy()
          .to_string()
      };

      transform_url_with_import_meta_url(ast, &comments);

      transform_import_meta_glob(
        ast,
        context.config.root.clone(),
        cur_dir,
        &context.config.resolve.alias,
      )?;
      script.set_comments(comments.into())
    }

    Ok(Some(()))
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let module = param.module;

    if module.module_type.is_script() {
      let module_ast = &module.meta.as_script().ast;

      let mut analyzer = DepsAnalyzer::new(
        &module.id,
        module_ast,
        Mark::from_u32(module.meta.as_script().unresolved_mark),
        Mark::from_u32(module.meta.as_script().top_level_mark),
      );

      GLOBALS.set(&context.meta.script.globals, || {
        let deps = analyzer.analyze_deps();
        param.deps.extend(deps);
      });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  /// detect [ModuleSystem] for a script module based on its dependencies' [ResolveKind] and detect hmr_accepted
  fn finalize_module(
    &self,
    param: &mut PluginFinalizeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }
    // all jsx, js, ts, tsx modules should be transformed to js module for now
    // cause the partial bundling is not support other module type yet
    param.module.module_type = ModuleType::Js;
    // set param.module.meta.module_system
    set_module_system_for_module_meta(param, context);

    let target_env = context.config.output.target_env.clone();
    let format = context.config.output.format;

    let is_library = target_env.is_library();

    if is_library && matches!(format, ModuleFormat::CommonJs) {
      replace_import_meta_url(&mut param.module.meta.as_script_mut().ast)
    };

    // find and replace `import.meta.xxx` to `module.meta.xxx` and detect hmr_accepted
    // skip transform import.meta when targetEnv is node
    if !is_library && (target_env.is_browser() || matches!(format, ModuleFormat::CommonJs)) {
      // transform `import.meta.xxx` to `module.meta.xxx`
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut import_meta_v = ImportMetaVisitor::new();
      ast.visit_mut_with(&mut import_meta_v);
    }

    if matches!(target_env, TargetEnv::Browser) {
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut hmr_accepted_v =
        import_meta_visitor::HmrAcceptedVisitor::new(param.module.id.clone(), context.clone());
      ast.visit_mut_with(&mut hmr_accepted_v);
      param.module.meta.as_script_mut().hmr_self_accepted = hmr_accepted_v.is_hmr_self_accepted;
      param.module.meta.as_script_mut().hmr_accepted_deps = hmr_accepted_v.hmr_accepted_deps;
    }

    Ok(None)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      let buf = resource_pot.meta.rendered_content.as_bytes().to_vec();

      let resource = Resource {
        bytes: buf,
        name: resource_pot.name.to_string(),
        emitted: false,
        resource_type: ResourceType::Js,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
        info: None,
      };
      let mut source_map = None;

      if context.config.sourcemap.enabled(resource_pot.immutable)
        && !resource_pot.meta.rendered_map_chain.is_empty()
      {
        // collapse source map chain
        let map = generate_source_map_resource(resource_pot);
        source_map = Some(map);
      }

      Ok(Some(PluginGenerateResourcesHookResult {
        resource,
        source_map,
      }))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginScript {
  pub fn new(_config: &Config) -> Self {
    #[cfg(feature = "swc_plugin")]
    init_plugin_module_cache_once(_config);
    Self {}
  }
}
