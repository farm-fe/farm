use std::sync::Arc;

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
  resource::{meta_data::js::RenderModuleResult, resource_pot::ResourcePot},
  serialize,
  swc_common::{comments::SingleThreadedComments, util::take::Take, SourceMap, DUMMY_SP},
  swc_ecma_ast::{
    EsVersion, Expr, ExprOrSpread, KeyValueProp, Lit, Module as SwcModule, ObjectLit, Prop,
    PropName, PropOrSpread,
  },
  swc_ecma_parser::{EsSyntax, Syntax},
  HashMap,
};
use farmfe_toolkit::{
  html::get_farm_global_this,
  script::{codegen_module, parse_module, CodeGenCommentsConfig, ParseScriptModuleResult},
  source_map::{build_source_map, collapse_sourcemap},
};

use farmfe_utils::{hash::sha256, parse_query};
use merge_rendered_module::wrap_resource_pot_ast;
use render_module::RenderModuleOptions;
use scope_hoisting::build_scope_hoisted_module_groups;

use self::render_module::render_module;

pub(crate) mod external;
pub(crate) mod merge_rendered_module;
mod render_module;
mod scope_hoisting;
mod source_replacer;
mod transform_async_module;

type RenderResourcePotResult = (Vec<RenderModuleResult>, HashMap<ModuleId, Vec<ModuleId>>);

pub fn render_resource_pot_modules(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<RenderResourcePotResult> {
  let modules = Mutex::new(vec![]);

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

      let mut hoisted_ast = if hoisted_group.hoisted_module_ids.len() > 1 {
        Some(hoisted_group.concatenate_modules(module_graph, context)?)
      } else {
        None
      };

      let hoisted_modules = hoisted_ast
        .as_mut()
        .map(|item| item.rendered_modules.take());

      let render_module_result = render_module(RenderModuleOptions {
        module,
        module_graph,
        hoisted_ast,
        context,
      })?;

      // modules.lock().push((render_module_result, hoisted_modules));
      modules
        .lock()
        .push((render_module_result, Vec::<ModuleId>::new()));

      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let modules = modules.into_inner();

  let (mut modules, hoisted_map) = modules.into_iter().fold(
    (vec![], HashMap::default()),
    |(mut modules, mut hoisted_map), (result, hosited_modules)| {
      // if let Some(hosited_modules) = hosited_modules {
      //   hoisted_map.insert(result.module_id.clone(), hosited_modules);
      // }

      modules.push(result);

      (modules, hoisted_map)
    },
  );

  modules.sort_by(|a, b| {
    a.module_id
      .id(context.config.mode.clone())
      .cmp(&b.module_id.id(context.config.mode.clone()))
  });

  Ok((modules, hoisted_map))
}
