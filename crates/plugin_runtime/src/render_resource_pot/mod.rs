use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

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
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::{RenderedModule, ResourcePot},
  serialize,
};
use farmfe_toolkit::common::MinifyBuilder;

use farmfe_utils::hash::sha256;

use self::render_module::{render_module, RenderModuleResult};

mod render_module;
// mod farm_module_system;
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

  resource_pot
    .modules()
    .into_par_iter()
    .try_for_each(|m_id| {
      let module = module_graph
        .module(m_id)
        .unwrap_or_else(|| panic!("Module not found: {:?}", m_id));

      let mut cache_store_key = None;

      // enable persistent cache
      if context.config.persistent_cache.enabled() {
        let content_hash = module.content_hash.clone();
        let store_key = CacheStoreKey {
          name: m_id.to_string() + "-resource_pot_to_runtime_object",
          key: sha256(
            format!(
              "resource_pot_to_runtime_object_{}_{}_{}",
              content_hash,
              m_id.to_string(),
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

      let is_async_module = async_modules.contains(m_id);
      let RenderModuleResult {
        rendered_module,
        external_modules,
        source_map_chain,
      } = render_module(
        module,
        module_graph,
        is_enabled_minify,
        &minify_builder,
        is_async_module,
        context,
      )?;
      let code = rendered_module.rendered_content.clone();

      // cache the code and sourcemap
      if context.config.persistent_cache.enabled() {
        let cache_rendered_script_module = CacheRenderedScriptModule::new(
          m_id.clone(),
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

      let mut module = MagicString::new(
        &code,
        Some(MagicStringOptions {
          filename: Some(m_id.resolved_path_with_query(&context.config.root)),
          source_map_chain,
          ..Default::default()
        }),
      );

      module.prepend(&format!("{:?}:", m_id.id(context.config.mode.clone())));
      module.append(",");

      modules.lock().push(RenderedScriptModule {
        id: m_id.clone(),
        module,
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
  let mut rendered_modules = HashMap::new();
  let mut external_modules_set = HashSet::new();

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
