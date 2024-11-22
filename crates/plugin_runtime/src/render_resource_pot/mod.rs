use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use farmfe_core::{
  cache::cache_store::CacheStoreKey,
  cache_item,
  config::{minify::MinifyMode, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  deserialize,
  enhanced_magic_string::{
    bundle::{Bundle, BundleOptions},
    magic_string::{MagicString, MagicStringOptions},
  },
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId},
  parking_lot::Mutex,
  plugin::PluginParseHookParam,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::{RenderedModule, ResourcePot},
  serialize,
  swc_common::{comments::SingleThreadedComments, SourceMap, DUMMY_SP},
  swc_ecma_ast::{
    EsVersion, Expr, ExprOrSpread, KeyValueProp, Lit, Module as SwcModule, ObjectLit, Prop,
    PropName, PropOrSpread,
  },
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  common::{build_source_map, collapse_sourcemap, MinifyBuilder, Source},
  html::get_farm_global_this,
  script::{codegen_module, parse_module, CodeGenCommentsConfig, ParseScriptModuleResult},
};

use farmfe_utils::{hash::sha256, parse_query};
use merge_rendered_module::{wrap_resource_pot_ast, RenderResourcePotAstResult};
use render_module::RenderModuleOptions;
use scope_hoisting::build_scope_hoisted_module_groups;

use self::render_module::{render_module, RenderModuleResult};

pub mod merge_rendered_module;
mod render_module;
mod scope_hoisting;
mod source_replacer;
mod transform_async_module;

pub fn render_resource_pot_modules(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  async_modules: &HashSet<ModuleId>,
  context: &Arc<CompilationContext>,
) -> Result<Vec<RenderModuleResult>> {
  let modules = Mutex::new(vec![]);

  // let minify_builder =
  //   MinifyBuilder::create_builder(&context.config.minify, Some(MinifyMode::Module));

  // let is_enabled_minify = |module_id: &ModuleId| {
  //   minify_builder.is_enabled(&module_id.resolved_path(&context.config.root))
  // };

  // group modules in the same group that can perform scope hoisting
  let scope_hoisting_module_groups =
    build_scope_hoisted_module_groups(resource_pot, module_graph, context);

  scope_hoisting_module_groups
    .into_par_iter()
    .try_for_each(|hoisted_group| {
      let module = module_graph
        .module(&hoisted_group.target_hoisted_module_id)
        .unwrap_or_else(|| {
          panic!(
            "Module not found: {:?}",
            &hoisted_group.target_hoisted_module_id
          )
        });

      let (hoisted_ast, comments) = if hoisted_group.hoisted_module_ids.len() > 1 {
        let hoisted_code_bundle = hoisted_group.render(module_graph, context)?;
        let code = hoisted_code_bundle.to_string();

        // println!(
        //   "module_id: {}\nmodules: {:#?}\ncode: {}\n\nend module_id: {}",
        //   hoisted_group.target_hoisted_module_id.to_string(),
        //   hoisted_group.hoisted_module_ids,
        //   code,
        //   hoisted_group.target_hoisted_module_id.to_string(),
        // );

        let mut meta = context
          .plugin_driver
          .parse(
            &PluginParseHookParam {
              module_id: module.id.clone(),
              resolved_path: module.id.resolved_path(&context.config.root),
              query: parse_query(&module.id.query_string()),
              module_type: module.module_type.clone(),
              content: Arc::new(code),
            },
            context,
            &Default::default(),
          )
          .unwrap()
          .unwrap();
        (
          Some(meta.as_script_mut().take_ast()),
          Some(meta.as_script_mut().take_comments().into()),
        )
      } else {
        (None, None)
      };

      // let mut cache_store_key = None;

      // // enable persistent cache
      // if context.config.persistent_cache.enabled() {
      //   let content_hash = module.content_hash.clone();
      //   let store_key = CacheStoreKey {
      //     name: m_id.to_string() + "-resource_pot_to_runtime_object",
      //     key: sha256(
      //       format!(
      //         "resource_pot_to_runtime_object_{}_{}_{}",
      //         content_hash,
      //         m_id.to_string(),
      //         module.used_exports.join(",")
      //       )
      //       .as_bytes(),
      //       32,
      //     ),
      //   };
      //   cache_store_key = Some(store_key.clone());

      //   // determine whether the cache exists,and store_key not change
      //   if context.cache_manager.custom.has_cache(&store_key.name)
      //     && !context.cache_manager.custom.is_cache_changed(&store_key)
      //   {
      //     if let Some(cache) = context.cache_manager.custom.read_cache(&store_key.name) {
      //       let cached_rendered_script_module = deserialize!(&cache, CacheRenderedScriptModule);
      //       let module = cached_rendered_script_module.to_magic_string(&context);

      //       modules.lock().push(RenderedScriptModule {
      //         module,
      //         id: cached_rendered_script_module.id,
      //         rendered_module: cached_rendered_script_module.rendered_module,
      //         external_modules: cached_rendered_script_module.external_modules,
      //       });
      //       return Ok(());
      //     }
      //   }
      // }

      let is_async_module = async_modules.contains(&module.id);
      let render_module_result = render_module(
        RenderModuleOptions {
          module,
          module_graph,
          hoisted_ast,
          is_async_module,
          context,
        },
        comments,
      )?;
      // let code = rendered_module.rendered_content.clone();

      // // cache the code and sourcemap
      // if context.config.persistent_cache.enabled() {
      //   let cache_rendered_script_module = CacheRenderedScriptModule::new(
      //     m_id.clone(),
      //     code.clone(),
      //     rendered_module.clone(),
      //     external_modules.clone(),
      //     source_map_chain.clone(),
      //   );
      //   let bytes = serialize!(&cache_rendered_script_module);
      //   context
      //     .cache_manager
      //     .custom
      //     .write_single_cache(cache_store_key.unwrap(), bytes)
      //     .expect("failed to write resource pot to runtime object cache");
      // }

      // let mut module = MagicString::new(
      //   &code,
      //   Some(MagicStringOptions {
      //     filename: Some(m_id.resolved_path_with_query(&context.config.root)),
      //     source_map_chain,
      //     ..Default::default()
      //   }),
      // );

      // module.prepend(&format!("{:?}:", m_id.id(context.config.mode.clone())));
      // module.append(",");

      modules.lock().push(render_module_result);

      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let mut modules = modules.into_inner();
  modules.sort_by(|a, b| {
    a.module_id
      .id(context.config.mode.clone())
      .cmp(&b.module_id.id(context.config.mode.clone()))
  });

  Ok(modules)
}

pub fn generate_code_and_sourcemap(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  wrapped_resource_pot_ast: &SwcModule,
  merged_sourcemap: Arc<SourceMap>,
  merged_comments: SingleThreadedComments,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<String>)> {
  let sourcemap_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

  let mut mappings = vec![];
  let code_bytes = codegen_module(
    &wrapped_resource_pot_ast,
    context.config.script.target.clone(),
    merged_sourcemap.clone(),
    if sourcemap_enabled {
      Some(&mut mappings)
    } else {
      None
    },
    context.config.minify.enabled(),
    Some(CodeGenCommentsConfig {
      comments: &merged_comments,
      // preserve all comments when generate module code.
      config: &context.config.comments,
    }),
  )
  .map_err(|e| CompilationError::RenderScriptModuleError {
    id: resource_pot.id.to_string(),
    source: Some(Box::new(e)),
  })?;

  let mut map = None;
  if sourcemap_enabled {
    let sourcemap = build_source_map(merged_sourcemap, &mappings);
    // trace sourcemap chain of each module
    let sourcemap = collapse_sourcemap(sourcemap, module_graph);
    let mut buf = vec![];
    sourcemap
      .to_writer(&mut buf)
      .map_err(|e| CompilationError::RenderScriptModuleError {
        id: resource_pot.id.to_string(),
        source: Some(Box::new(e)),
      })?;
    let sourcemap = String::from_utf8(buf).unwrap();

    map = Some(sourcemap);
  }

  let code = String::from_utf8(code_bytes).unwrap();

  Ok((code, map))
}

pub fn resource_pot_to_runtime_object(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  async_modules: &HashSet<ModuleId>,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<Arc<String>>, Vec<ModuleId>)> {
  let modules = render_resource_pot_modules(resource_pot, module_graph, async_modules, context)?;

  let RenderResourcePotAstResult {
    rendered_resource_pot_ast,
    mut external_modules,
    merged_sourcemap,
    merged_comments,
  } = merge_rendered_module::merge_rendered_module(modules, context);

  let wrapped_resource_pot_ast = wrap_resource_pot_ast(
    rendered_resource_pot_ast,
    &resource_pot.id,
    merged_sourcemap.clone(),
    context,
  );

  // sort external modules by module id to make sure the order is stable
  external_modules.sort();

  let (code, map) = generate_code_and_sourcemap(
    resource_pot,
    module_graph,
    &wrapped_resource_pot_ast,
    merged_sourcemap,
    merged_comments,
    context,
  )?;

  Ok((code, map.map(|m| Arc::new(m)), external_modules))
}

// pub struct RenderedScriptModule {
//   pub id: ModuleId,
//   pub module: MagicString,
//   pub rendered_module: RenderedModule,
//   pub external_modules: Vec<String>,
// }

// pub struct RenderedJsResourcePot {
//   pub bundle: Bundle,
//   pub rendered_modules: HashMap<ModuleId, RenderedModule>,
//   pub external_modules: Vec<String>,
// }

// #[cache_item]
// pub struct CacheRenderedScriptModule {
//   pub id: ModuleId,
//   pub code: Arc<String>,
//   pub rendered_module: RenderedModule,
//   pub external_modules: Vec<String>,
//   pub source_map_chain: Vec<Arc<String>>,
// }

// impl CacheRenderedScriptModule {
//   fn new(
//     id: ModuleId,
//     code: Arc<String>,
//     rendered_module: RenderedModule,
//     external_modules: Vec<String>,
//     source_map_chain: Vec<Arc<String>>,
//   ) -> Self {
//     Self {
//       id,
//       code,
//       rendered_module,
//       external_modules,
//       source_map_chain,
//     }
//   }
//   fn to_magic_string(&self, context: &Arc<CompilationContext>) -> MagicString {
//     let magic_string_option = MagicStringOptions {
//       filename: Some(self.id.resolved_path_with_query(&context.config.root)),
//       source_map_chain: self.source_map_chain.clone(),
//       ..Default::default()
//     };
//     let mut module = MagicString::new(&self.code, Some(magic_string_option));
//     module.prepend(&format!("{:?}:", self.id.id(context.config.mode.clone())));
//     module.append(",");
//     module
//   }
// }
