#![deny(clippy::all)]
#![allow(clippy::needless_collect)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::too_many_arguments)]
#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::Config, context::CompilationContext, error::Result, plugin::Plugin, stats::Stats,
};

pub mod build;
pub mod generate;
pub mod update;

pub struct Compiler {
  context: Arc<CompilationContext>,
}

impl Compiler {
  /// The params are [farmfe_core::config::Config] and dynamic load rust plugins and js plugins [farmfe_core::plugin::Plugin]
  pub fn new(config: Config, mut plugin_adapters: Vec<Arc<dyn Plugin>>) -> Result<Self> {
    let mut plugins = vec![
      Arc::new(farmfe_plugin_runtime::FarmPluginRuntime::new(&config)) as _,
      // register internal core plugins
      Arc::new(farmfe_plugin_resolve::FarmPluginResolve::new(&config)) as _,
      Arc::new(farmfe_plugin_script::FarmPluginScript::new(&config)) as _,
      Arc::new(farmfe_plugin_partial_bundling::FarmPluginPartialBundling::new(&config)) as _,
      Arc::new(farmfe_plugin_html::FarmPluginHtml::new(&config)) as _,
      Arc::new(farmfe_plugin_css::FarmPluginCss::new(&config)) as _,
      Arc::new(farmfe_plugin_static_assets::FarmPluginStaticAssets::new(
        &config,
      )) as _,
      Arc::new(farmfe_plugin_json::FarmPluginJson::new(&config)) as _,
    ];

    if config.lazy_compilation {
      plugins.push(
        Arc::new(farmfe_plugin_lazy_compilation::FarmPluginLazyCompilation::new(&config)) as _,
      );
    }

    if config.tree_shaking {
      plugins.push(Arc::new(farmfe_plugin_tree_shake::FarmPluginTreeShake::new(&config)) as _);
    }

    if config.minify {
      plugins.push(Arc::new(farmfe_plugin_minify::FarmPluginMinify::new(&config)) as _);
    }

    if config.preset_env.enabled() {
      plugins.push(Arc::new(farmfe_plugin_polyfill::FarmPluginPolyfill::new(&config)) as _);
    }

    plugins.append(&mut plugin_adapters);
    // sort plugins by priority to make larger priority plugin run first
    plugins.sort_by_key(|b| std::cmp::Reverse(b.priority()));

    let mut context = CompilationContext::new(config, plugins)?;
    context.plugin_driver.config(&mut context.config)?;
    Ok(Self {
      context: Arc::new(context),
    })
  }

  /// Compile the project using the configuration
  pub fn compile(&self) -> Result<()> {
    // triggering build stage
    {
      #[cfg(feature = "profile")]
      farmfe_core::puffin::profile_scope!("Build Stage");
      let start = std::time::Instant::now();
      self.build()?;
      println!("Build cost {:?}", start.elapsed());
    }

    {
      #[cfg(feature = "profile")]
      farmfe_core::puffin::profile_scope!("Generate Stage");
      let start = std::time::Instant::now();
      self.generate()?;
      println!("Generate cost {:?}", start.elapsed());
    }

    if self.context.config.persistent_cache.enabled() {
      // Does not support write cache in update mode for now
      let start = std::time::Instant::now();
      self.context.cache_manager.write_cache();
      println!("Write cache cost {:?}", start.elapsed());

      let module_graph = self.context.module_graph.read();
      let modules_len = module_graph.modules().len();
      let cached_modules_len = self
        .context
        .cache_manager
        .module_cache
        .initial_cache_modules()
        .len();
      println!(
        "{} / {}, Cache hit rate: {}%",
        cached_modules_len,
        modules_len,
        cached_modules_len as f64 / modules_len as f64 * 100.0
      );
    }

    self.context.plugin_driver.finish(&Stats {}, &self.context)
  }

  pub fn context(&self) -> &Arc<CompilationContext> {
    &self.context
  }
}
