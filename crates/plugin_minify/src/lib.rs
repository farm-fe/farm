use std::sync::Arc;

use farmfe_core::{
  config::{minify::MinifyOptions, Config},
  context::CompilationContext,
  plugin::Plugin,
  resource::resource_pot::{ResourcePot, ResourcePotType},
};
use minify_resource_pot::{minify_css, minify_js};

// use ident_generator::MinifiedIdentsGenerator;
// use imports_minifier::{IdentReplacer, ImportsMinifier};
// use top_level_idents_collector::{TopLevelIdentsCollector, UnresolvedIdentCollector};
// use util::is_module_contains_export;
// use exports_minifier::ExportsMinifier;

// mod exports_minifier;
// mod ident_generator;
// mod imports_minifier;
mod minify_resource_pot;
// mod mangle_exports;
// mod top_level_idents_collector;
// mod util;

pub struct FarmPluginMinify {
  minify_options: MinifyOptions,
}

impl FarmPluginMinify {
  pub fn new(config: &Config) -> Self {
    Self {
      minify_options: config
        .minify
        .clone()
        .map(|val| MinifyOptions::from(val))
        .unwrap_or_default(),
    }
  }
}

impl Plugin for FarmPluginMinify {
  fn name(&self) -> &'static str {
    "FarmPluginMinify"
  }

  // /// minify imports and exports of a module
  // fn optimize_module_graph(
  //   &self,
  //   module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
  //   context: &Arc<CompilationContext>,
  // ) -> Result<Option<()>> {
  //   // if config.runtime.concatenate_modules is true, we don't need to minify the module decls cause it will be handled by swc_minifier
  //   // module decls will be handled when concatenate modules
  //   if !self.minify_options.module_decls || context.config.concatenate_modules {
  //     return Ok(None);
  //   }

  //   // skip minify imports/exports if any of the entry modules contains export statement
  //   // TODO remove this guard and handle entries in imports/exports minifier
  //   if module_graph
  //     .entries
  //     .keys()
  //     .into_iter()
  //     .any(|module_id| is_module_contains_export(module_id, module_graph))
  //   {
  //     return Ok(None);
  //   }

  //   // 1. rename all the exports of a module
  //   let mut minified_module_exports_map = HashMap::new();
  //   let ident_generator_map: Mutex<HashMap<ModuleId, MinifiedIdentsGenerator>> =
  //     Mutex::new(HashMap::new());

  //   module_graph
  //     .modules_mut()
  //     .into_par_iter()
  //     .for_each(|module| {
  //       if !module.module_type.is_script() {
  //         return;
  //       }

  //       let meta = module.meta.as_script_mut();
  //       let ast = &mut meta.ast;
  //       // collect used idents to avoid duplicate variable declaration
  //       let mut top_level_idents_collector = TopLevelIdentsCollector::new();
  //       ast.visit_mut_with(&mut top_level_idents_collector);

  //       let mut top_level_idents = top_level_idents_collector.top_level_idents;

  //       let (cm, _) = create_swc_source_map(Source {
  //         path: PathBuf::from(module.id.to_string()),
  //         content: module.content.clone(),
  //       });
  //       try_with(cm, &context.meta.script.globals, || {
  //         let mut collector = UnresolvedIdentCollector::new(Mark::from_u32(meta.unresolved_mark));
  //         ast.visit_mut_with(&mut collector);
  //         top_level_idents.extend(collector.unresolved_idents);
  //       })
  //       .unwrap();
  //       let ident_generator = MinifiedIdentsGenerator::new(top_level_idents);
  //       ident_generator_map
  //         .lock()
  //         .insert(module.id.clone(), ident_generator);
  //     });

  //   let mut ident_generator_map = ident_generator_map.into_inner();

  //   let get_export_from_deps =
  //     |module_id: &ModuleId, module_graph: &farmfe_core::module::module_graph::ModuleGraph| {
  //       module_graph
  //         .dependencies(module_id)
  //         .into_iter()
  //         .filter(|(_, edge)| edge.contains_export_from())
  //         .map(|(dep_id, _)| dep_id)
  //         .collect::<Vec<_>>()
  //     };
  //   let get_require_or_dynamic_deps =
  //     |module_id: &ModuleId, module_graph: &farmfe_core::module::module_graph::ModuleGraph| {
  //       module_graph
  //         .dependencies(module_id)
  //         .into_iter()
  //         .filter(|(_, edge)| edge.contains_require() || edge.contains_dynamic())
  //         .map(|(dep_id, _)| dep_id)
  //         .collect::<Vec<_>>()
  //     };

  //   // modules that are required by cjs will always not be minified
  //   // TODO handle skipped module ids the same as module_graph.entries
  //   let mut skipped_module_ids = HashSet::new();

  //   // Handle conflicting export * from. e.g:
  //   // ```
  //   // export * from 'vue';
  //   // function original() {}
  //   // export const h = original;
  //   // ```
  //   // minified h conflicts with the exported ident of vue. we should rename h to h1 to avoid conflict.
  //   // topo sort the module graph, and traverse the module graph from bottom to top
  //   let (mut sorted_module_ids, cyclic_module_ids) = module_graph.toposort();
  //   // skip cyclic module ids and its dependencies. TODO: handle cyclic module ids in the future
  //   skipped_module_ids.extend(cyclic_module_ids.into_iter().flatten().into_iter());

  //   // traverse the module graph from top to bottom to avoid export * from name conflict
  //   for module_id in &sorted_module_ids {
  //     if !module_graph
  //       .module(&module_id)
  //       .unwrap()
  //       .module_type
  //       .is_script()
  //     {
  //       continue;
  //     }

  //     // if module is skipped, all the dependencies are skipped
  //     if skipped_module_ids.contains(module_id) {
  //       let deps = module_graph
  //         .dependencies(module_id)
  //         .into_iter()
  //         .map(|(dep_id, _)| dep_id);
  //       skipped_module_ids.extend(deps);
  //       // continue;
  //     }

  //     skipped_module_ids.extend(get_require_or_dynamic_deps(module_id, &module_graph));

  //     let deps = get_export_from_deps(module_id, &module_graph);
  //     let current_used_idents = ident_generator_map
  //       .get(&module_id)
  //       .unwrap()
  //       .used_idents()
  //       .clone();

  //     for dep in deps {
  //       if let Some(ident_generator) = ident_generator_map.get_mut(&dep) {
  //         ident_generator.extend_used_idents(current_used_idents.clone());
  //       }
  //     }
  //   }

  //   // reverse to make the module graph traverse from bottom to top
  //   sorted_module_ids.reverse();

  //   let mut id_to_replace: HashMap<ModuleId, HashMap<Id, String>> = HashMap::new();

  //   for module_id in &sorted_module_ids {
  //     if !module_graph
  //       .module(&module_id)
  //       .unwrap()
  //       .module_type
  //       .is_script()
  //       || skipped_module_ids.contains(module_id)
  //     {
  //       continue;
  //     }

  //     let deps = get_export_from_deps(module_id, &module_graph);

  //     let dep_used_idents = deps
  //       .into_iter()
  //       .fold(HashSet::<String>::new(), |mut acc, dep| {
  //         if let Some(ident_generator) = ident_generator_map.get(&dep) {
  //           acc.extend(ident_generator.used_idents().clone());
  //         }
  //         acc
  //       });

  //     let module = module_graph.module_mut(&module_id).unwrap();
  //     let meta = module.meta.as_script_mut();
  //     let ast = &mut meta.ast;

  //     let mut ident_generator = ident_generator_map.get_mut(&module_id).unwrap();
  //     ident_generator.extend_used_idents(dep_used_idents);

  //     let mut exports_minifier = ExportsMinifier::new(&mut ident_generator);

  //     ast.visit_mut_with(&mut exports_minifier);

  //     minified_module_exports_map.insert(module.id.clone(), exports_minifier.minified_exports_map);
  //     id_to_replace
  //       .entry(module.id.clone())
  //       .or_default()
  //       .extend(exports_minifier.ident_to_replace);
  //   }

  //   // 2. rename all the imports of a module, handle export * from carefully
  //   for module_id in sorted_module_ids {
  //     let module = module_graph.module_mut(&module_id).unwrap();

  //     if !module.module_type.is_script() || skipped_module_ids.contains(&module_id) {
  //       continue;
  //     }

  //     let mut ast = module.meta.as_script_mut().take_ast();
  //     let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);

  //     let ident_generator = ident_generator_map.get_mut(&module_id).unwrap();
  //     let (cm, _) = create_swc_source_map(Source {
  //       path: PathBuf::from(module.id.to_string()),
  //       content: module.content.clone(),
  //     });
  //     try_with(cm, &context.meta.script.globals, || {
  //       // minify imports, handle export { xxx } from and export * from carefully
  //       let mut imports_minifier = ImportsMinifier::new(
  //         &module_id,
  //         &mut minified_module_exports_map,
  //         module_graph,
  //         ident_generator,
  //         unresolved_mark,
  //       );
  //       ast.visit_mut_with(&mut imports_minifier);
  //       id_to_replace
  //         .entry(module_id.clone())
  //         .or_default()
  //         .extend(imports_minifier.id_to_replace_map);
  //     })?;

  //     let module = module_graph.module_mut(&module_id).unwrap();
  //     module.meta.as_script_mut().set_ast(ast);
  //   }

  //   let id_to_replace = Mutex::new(id_to_replace);

  //   module_graph
  //     .modules_mut()
  //     .into_par_iter()
  //     .for_each(|module| {
  //       if !module.module_type.is_script() {
  //         return;
  //       }

  //       let meta = module.meta.as_script_mut();
  //       let ast = &mut meta.ast;

  //       let (cm, _) = create_swc_source_map(Source {
  //         path: PathBuf::from(module.id.to_string()),
  //         content: module.content.clone(),
  //       });
  //       try_with(cm, &context.meta.script.globals, || {
  //         if let Some(id_to_replace) = id_to_replace.lock().remove(&module.id) {
  //           let mut ident_replacer = IdentReplacer::new(id_to_replace);
  //           ast.visit_mut_with(&mut ident_replacer);
  //         }
  //       })
  //       .unwrap();
  //     });

  //   // update used exports of the module
  //   module_graph.modules_mut().into_iter().for_each(|module| {
  //     if let Some(minified_exports_map) = minified_module_exports_map.get(&module.id) {
  //       for (export, minified) in minified_exports_map {
  //         if module.used_exports.contains(export) {
  //           // the minified may be changed even if the export is not changed
  //           // we need to add the minified export to the used exports to make sure cache works as expected
  //           module.used_exports.push(minified.clone());
  //         }
  //       }
  //     }
  //   });

  //   Ok(Some(()))
  // }

  fn optimize_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let enable_minify = context.config.minify.enabled();

    if !enable_minify {
      return Ok(None);
    }

    if matches!(resource_pot.resource_pot_type, ResourcePotType::Js) {
      minify_js(resource_pot, &self.minify_options, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      minify_css(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      // html minify is handled in plugin html after all resources are injected in finalize_resources hook
    }

    Ok(None)
  }
}
