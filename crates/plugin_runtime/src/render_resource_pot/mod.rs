use std::sync::Arc;

use farmfe_core::{
  cache::cache_store::CacheStoreKey,
  cache_item,
  config::minify::MinifyMode,
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
  serialize, HashMap, HashSet,
};
use farmfe_toolkit::common::MinifyBuilder;

use farmfe_utils::{hash::sha256, parse_query};
use render_module::RenderModuleOptions;
use scope_hoisting::build_scope_hoisted_module_groups;

use self::render_module::{render_module, RenderModuleResult};

mod render_module;
mod scope_hoisting;
mod source_replacer;
mod transform_async_module;

/// Merge all modules' ast in a [ResourcePot] to Farm's runtime [ObjectLit]. The [ObjectLit] looks like:
/// ```js
/// {
///   // commonjs or hybrid module system
///   "a.js": function(module, exports, require) {
///       const b = require('./b');
///       console.log(b);
///    },
///    // esm module system
///    "b.js": async function(module, exports, require) {
///       const [c, d] = await Promise.all([
///         require('./c'),
///         require('./d')
///       ]);
///
///       exports.c = c;
///       exports.d = d;
///    }
/// }
/// ```
pub fn resource_pot_to_runtime_object(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  async_modules: &HashSet<ModuleId>,
  context: &Arc<CompilationContext>,
) -> Result<RenderedJsResourcePot> {
  let modules = Mutex::new(vec![]);

  let minify_builder =
    MinifyBuilder::create_builder(&context.config.minify, Some(MinifyMode::Module));

  let is_enabled_minify = |module_id: &ModuleId| {
    minify_builder.is_enabled(&module_id.resolved_path(&context.config.root))
  };

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

      let mut cache_store_key = None;

      // enable persistent cache
      if context.config.persistent_cache.enabled() {
        let content_hash = module.content_hash.clone();
        let store_key = CacheStoreKey {
          name: module.id.to_string() + "-resource_pot_to_runtime_object",
          key: sha256(
            format!(
              "resource_pot_to_runtime_object_{}_{}_{}",
              content_hash,
              module.id.to_string(),
              module.used_exports.join(",")
            )
            .as_bytes(),
            32,
          ),
        };
        cache_store_key = Some(store_key.clone());

        // determine whether the cache exists,and store_key not change
        if context.cache_manager.custom.has_cache(&store_key.name)
          && !context.cache_manager.custom.is_cache_changed(&store_key)
        {
          if let Some(cache) = context.cache_manager.custom.read_cache(&store_key.name) {
            let cached_rendered_script_module = deserialize!(&cache, CacheRenderedScriptModule);
            let module = cached_rendered_script_module.to_magic_string(&context);

            modules.lock().push(RenderedScriptModule {
              module,
              id: cached_rendered_script_module.id,
              rendered_module: cached_rendered_script_module.rendered_module,
              external_modules: cached_rendered_script_module.external_modules,
            });
            return Ok(());
          }
        }
      }

      let is_async_module = async_modules.contains(&module.id);
      let RenderModuleResult {
        rendered_module,
        external_modules,
        source_map_chain,
      } = render_module(
        RenderModuleOptions {
          module,
          hoisted_ast,
          module_graph,
          is_enabled_minify,
          minify_builder: &minify_builder,
          is_async_module,
          context,
        },
        comments,
      )?;
      let code = rendered_module.rendered_content.clone();

      // cache the code and sourcemap
      if context.config.persistent_cache.enabled() {
        let cache_rendered_script_module = CacheRenderedScriptModule::new(
          module.id.clone(),
          code.clone(),
          rendered_module.clone(),
          external_modules.clone(),
          source_map_chain.clone(),
        );
        let bytes = serialize!(&cache_rendered_script_module);
        context
          .cache_manager
          .custom
          .write_single_cache(cache_store_key.unwrap(), bytes)
          .expect("failed to write resource pot to runtime object cache");
      }

      let mut magic_string = MagicString::new(
        &code,
        Some(MagicStringOptions {
          filename: Some(module.id.resolved_path_with_query(&context.config.root)),
          source_map_chain,
          ..Default::default()
        }),
      );

      magic_string.prepend(&format!("{:?}:", module.id.id(context.config.mode.clone())));
      magic_string.append(",");

      modules.lock().push(RenderedScriptModule {
        id: module.id.clone(),
        module: magic_string,
        rendered_module,
        external_modules,
      });

      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let mut modules = modules.into_inner();
  modules.sort_by(|a, b| {
    a.id
      .id(context.config.mode.clone())
      .cmp(&b.id.id(context.config.mode.clone()))
  });
  // insert props to the object lit

  let mut bundle = Bundle::new(BundleOptions {
    trace_source_map_chain: Some(true),
    separator: if context.config.minify.enabled() {
      Some('\0')
    } else {
      None
    },
    ..Default::default()
  });
  let mut rendered_modules = HashMap::default();
  let mut external_modules_set = HashSet::default();

  for m in modules {
    bundle.add_source(m.module, None).unwrap();
    rendered_modules.insert(m.id, m.rendered_module);
    external_modules_set.extend(m.external_modules);
  }

  let mut external_modules = external_modules_set.into_iter().collect::<Vec<_>>();
  external_modules.sort();

  bundle.prepend("{");
  bundle.append("}", None);

  Ok(RenderedJsResourcePot {
    bundle,
    rendered_modules,
    external_modules,
  })
}

pub struct RenderedScriptModule {
  pub id: ModuleId,
  pub module: MagicString,
  pub rendered_module: RenderedModule,
  pub external_modules: Vec<String>,
}

pub struct RenderedJsResourcePot {
  pub bundle: Bundle,
  pub rendered_modules: HashMap<ModuleId, RenderedModule>,
  pub external_modules: Vec<String>,
}

#[cache_item]
pub struct CacheRenderedScriptModule {
  pub id: ModuleId,
  pub code: Arc<String>,
  pub rendered_module: RenderedModule,
  pub external_modules: Vec<String>,
  pub source_map_chain: Vec<Arc<String>>,
}

impl CacheRenderedScriptModule {
  fn new(
    id: ModuleId,
    code: Arc<String>,
    rendered_module: RenderedModule,
    external_modules: Vec<String>,
    source_map_chain: Vec<Arc<String>>,
  ) -> Self {
    Self {
      id,
      code,
      rendered_module,
      external_modules,
      source_map_chain,
    }
  }
  fn to_magic_string(&self, context: &Arc<CompilationContext>) -> MagicString {
    let magic_string_option = MagicStringOptions {
      filename: Some(self.id.resolved_path_with_query(&context.config.root)),
      source_map_chain: self.source_map_chain.clone(),
      ..Default::default()
    };
    let mut module = MagicString::new(&self.code, Some(magic_string_option));
    module.prepend(&format!("{:?}:", self.id.id(context.config.mode.clone())));
    module.append(",");
    module
  }
}
