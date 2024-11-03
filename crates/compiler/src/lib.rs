#![deny(clippy::all)]
#![allow(clippy::needless_collect)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::assigning_clones)]
#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::{Config, Mode},
  context::CompilationContext,
  error::Result,
  farm_profile_function,
  module::ModuleId,
  parking_lot::Mutex,
  plugin::Plugin,
  rayon::{ThreadPool, ThreadPoolBuilder},
};

pub use farmfe_plugin_css::FARM_CSS_MODULES_SUFFIX;
pub use farmfe_plugin_lazy_compilation::DYNAMIC_VIRTUAL_SUFFIX;
pub use farmfe_plugin_runtime::RUNTIME_SUFFIX;

pub mod build;
pub mod generate;
pub mod trace_module_graph;
pub mod update;
pub mod utils;

pub struct Compiler {
  context: Arc<CompilationContext>,
  pub thread_pool: Arc<ThreadPool>,
  pub last_fail_module_ids: Mutex<Vec<ModuleId>>,
}

impl Compiler {
  /// The params are [farmfe_core::config::Config] and dynamic load rust plugins and js plugins [farmfe_core::plugin::Plugin]
  pub fn new(config: Config, mut plugin_adapters: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    let mut plugins = vec![
      Arc::new(farmfe_plugin_runtime::FarmPluginRuntime::new(&config)) as _,
      Arc::new(farmfe_plugin_bundle::FarmPluginBundle::new()) as _,
      // register internal core plugins
      Arc::new(farmfe_plugin_script::FarmPluginScript::new(&config)) as _,
      Arc::new(farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config)) as _,
      Arc::new(farmfe_plugin_html::FarmPluginHtml::new(&config)) as _,
      Arc::new(farmfe_plugin_html::FarmPluginTransformHtml::new(&config)) as _,
      Arc::new(farmfe_plugin_css::FarmPluginCssResolve::new(&config)) as _,
      Arc::new(farmfe_plugin_css::FarmPluginCss::new(&config)) as _,
      Arc::new(farmfe_plugin_static_assets::FarmPluginStaticAssets::new(
        &config,
      )) as _,
      Arc::new(farmfe_plugin_static_assets::FarmPluginRaw::new(&config)) as _,
      Arc::new(farmfe_plugin_json::FarmPluginJson::new(&config)) as _,
      Arc::new(farmfe_plugin_define::FarmPluginDefine::new(&config)) as _,
    ];

    if config.progress {
      plugins.push(Arc::new(farmfe_plugin_progress::FarmPluginProgress::new(&config)) as _);
    }

    if config.lazy_compilation {
      plugins.push(
        Arc::new(farmfe_plugin_lazy_compilation::FarmPluginLazyCompilation::new(&config)) as _,
      );
    }

    if config.tree_shaking.enabled() {
      plugins.push(Arc::new(farmfe_plugin_tree_shake::FarmPluginTreeShake::new(&config)) as _);
    }

    if config.minify.enabled() {
      plugins.push(Arc::new(farmfe_plugin_minify::FarmPluginMinify::new(&config)) as _);
      plugins.push(Arc::new(farmfe_plugin_html::FarmPluginMinifyHtml::new(&config)) as _);
    }

    if config.preset_env.enabled() {
      plugins.push(Arc::new(farmfe_plugin_polyfill::FarmPluginPolyfill::new(&config)) as _);
    }
    // default resolve will be executed at last within internal plugins
    // but it will be executed earlier than external plugins
    plugins.push(Arc::new(farmfe_plugin_resolve::FarmPluginResolve::new(&config)) as _);

    plugins.append(&mut plugin_adapters);

    Self::new_without_internal_plugins(config, plugins)
  }

  pub fn new_without_internal_plugins(
    config: Config,
    mut plugins: Vec<Arc<dyn Plugin>>,
  ) -> Result<Self> {
    // sort plugins by priority to make larger priority plugin run first
    plugins.sort_by_key(|b| std::cmp::Reverse(b.priority()));

    let mut context = CompilationContext::new(config, plugins)?;
    context.plugin_driver.config(&mut context.config)?;

    Ok(Self {
      context: Arc::new(context),
      thread_pool: Arc::new(
        ThreadPoolBuilder::new()
          .num_threads(num_cpus::get())
          .build()
          .unwrap(),
      ),
      last_fail_module_ids: Mutex::new(vec![]),
    })
  }

  pub fn trace_dependencies(&self) -> Result<Vec<String>> {
    self.build()?;

    let module_graph = self.context.module_graph.read();
    let mut dependencies = vec![];

    for module in module_graph.modules() {
      if module.external {
        dependencies.push(module.id.to_string());
      } else {
        dependencies.push(module.id.resolved_path(&self.context.config.root));
      }
    }

    if self.context.config.persistent_cache.enabled() {
      self.context.cache_manager.write_cache();
    }

    Ok(dependencies)
  }

  /// Compile the project using the configuration
  pub fn compile(&self) -> Result<()> {
    self.context.record_manager.set_start_time();
    if self.context.config.persistent_cache.enabled() {
      self
        .context
        .plugin_driver
        .plugin_cache_loaded(&self.context)?;
    }

    // triggering build stage
    {
      #[cfg(feature = "profile")]
      farmfe_core::puffin::profile_scope!("Build Stage");
      self.build()?
    };

    self.context.record_manager.set_build_end_time();
    {
      #[cfg(feature = "profile")]
      farmfe_core::puffin::profile_scope!("Generate Stage");
      self.generate()?;
    }

    self
      .context
      .plugin_driver
      .finish(&self.context.record_manager, &self.context)?;

    if self.context.config.persistent_cache.enabled() {
      self
        .context
        .plugin_driver
        .write_plugin_cache(&self.context)
        .unwrap_or_else(|err| {
          eprintln!("write plugin cache error: {err:?}");
        });

      if matches!(self.context.config.mode, Mode::Development) {
        #[cfg(feature = "profile")]
        write_cache(self.context.clone());
        #[cfg(not(feature = "profile"))]
        write_cache_async(self.context.clone());
      } else {
        write_cache(self.context.clone());
      }
    }

    self.context.record_manager.set_end_time();

    Ok(())
  }

  pub fn context(&self) -> &Arc<CompilationContext> {
    &self.context
  }
}

fn write_cache(context: Arc<CompilationContext>) {
  farm_profile_function!("write_cache".to_string());
  context.cache_manager.write_cache();
  context.cache_manager.custom.write_manifest();
}

pub fn write_cache_async(context: Arc<CompilationContext>) {
  std::thread::spawn(move || {
    write_cache(context);
  });
}
