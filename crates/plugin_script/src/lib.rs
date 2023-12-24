#![feature(box_patterns)]
#![feature(path_file_prefix)]

use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::{Config, ModuleFormat, TargetEnv},
  context::CompilationContext,
  enhanced_magic_string::collapse_sourcemap::collapse_sourcemap_chain,
  error::Result,
  module::{ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::{Mark, GLOBALS},
  swc_ecma_ast::ModuleItem,
};
use farmfe_swc_transformer_import_glob::transform_import_meta_glob;
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  fs::read_file_utf8,
  script::{
    module_system_from_deps, module_type_from_id, parse_module, swc_try_with::try_with,
    syntax_from_module_type,
  },
  sourcemap::SourceMap,
  swc_ecma_transforms::resolver,
  swc_ecma_visit::VisitMutWith,
};

use import_meta_visitor::ImportMetaVisitor;
#[cfg(feature = "swc_plugin")]
use swc_plugins::{init_plugin_module_cache_once, transform_by_swc_plugins};

mod deps_analyzer;
mod import_meta_visitor;
#[cfg(feature = "swc_plugin")]
mod swc_plugins;
mod swc_script_transforms;

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

        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
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
      let mut swc_module = parse_module(
        &param.module_id.to_string(),
        &param.content,
        syntax,
        context.config.script.target,
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
      let ast = &mut param.meta.as_script_mut().ast;
      let cur_dir = PathBuf::from(&param.module_id.resolved_path(&context.config.root));
      transform_import_meta_glob(
        ast,
        context.config.root.clone(),
        cur_dir.parent().unwrap().to_string_lossy().to_string(),
      )?;
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
      // TODO deal with dynamic import, when dynamic import and static import are mixed, using static import
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
    // default to commonjs
    let module_system = if !param.deps.is_empty() {
      module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect())
    } else {
      ModuleSystem::CommonJs
    };
    param.module.meta.as_script_mut().module_system = module_system.clone();

    let ast = &param.module.meta.as_script().ast;

    if module_system != ModuleSystem::Hybrid {
      // if the ast contains ModuleDecl, it's a esm module
      for item in ast.body.iter() {
        if let ModuleItem::ModuleDecl(_) = item {
          if module_system == ModuleSystem::CommonJs && !param.deps.is_empty() {
            param.module.meta.as_script_mut().module_system = ModuleSystem::Hybrid;
          } else {
            param.module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
          }
          break;
        }
      }
    }

    // find and replace `import.meta.xxx` to `module.meta.xxx` and detect hmr_accepted
    // skip transform import.meta when targetEnv is node
    if matches!(context.config.output.target_env, TargetEnv::Browser)
      || matches!(context.config.output.format, ModuleFormat::CommonJs)
    {
      // transform `import.meta.xxx` to `module.meta.xxx`
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut import_meta_v = ImportMetaVisitor::new();
      ast.visit_mut_with(&mut import_meta_v);
    }

    if matches!(context.config.output.target_env, TargetEnv::Browser) {
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut hmr_accepted_v =
        import_meta_visitor::HmrAcceptedVisitor::new(param.module.id.clone(), context.clone());
      ast.visit_mut_with(&mut hmr_accepted_v);
      param.module.meta.as_script_mut().hmr_self_accepted = hmr_accepted_v.is_hmr_self_accepted;
      param.module.meta.as_script_mut().hmr_accepted_deps = hmr_accepted_v
        .hmr_accepted_deps
        .into_iter()
        .map(|dep| dep.into())
        .collect();
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
        let source_map_chain = resource_pot
          .meta
          .rendered_map_chain
          .iter()
          .map(|s| SourceMap::from_slice(s.as_bytes()).unwrap())
          .collect::<Vec<_>>();
        let collapsed_sourcemap = collapse_sourcemap_chain(source_map_chain, Default::default());
        let mut src_map = vec![];
        collapsed_sourcemap
          .to_writer(&mut src_map)
          .expect("failed to write sourcemap");
        let map = Resource {
          bytes: src_map,
          name: resource.name.clone(),
          emitted: false,
          resource_type: ResourceType::SourceMap(resource_pot.id.to_string()),
          origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
          info: None,
        };

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
  pub fn new(config: &Config) -> Self {
    #[cfg(feature = "swc_plugin")]
    init_plugin_module_cache_once(config);
    Self {}
  }
}
