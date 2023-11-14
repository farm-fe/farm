#![deny(clippy::all)]
#![allow(clippy::needless_collect)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::too_many_arguments)]
#![feature(box_patterns)]

use std::sync::Arc;

use farmfe_core::{
  config::{Config, Mode},
  context::CompilationContext,
  error::Result,
  plugin::Plugin,
  stats::Stats,
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

    self
      .context
      .plugin_driver
      .finish(&Stats {}, &self.context)?;

    if self.context.config.persistent_cache.enabled() {
      self
        .context
        .plugin_driver
        .write_plugin_cache(&self.context)
        .unwrap_or_else(|err| {
          eprintln!("write plugin cache error: {:?}", err);
        });

      if matches!(self.context.config.mode, Mode::Development) {
        write_cache_async(self.context.clone());
      } else {
        write_cache(self.context.clone());
      }
    }

    Ok(())
  }

  pub fn context(&self) -> &Arc<CompilationContext> {
    &self.context
  }
}

fn write_cache(context: Arc<CompilationContext>) {
  let start = std::time::Instant::now();
  context.cache_manager.write_cache();
  println!("Write cache cost {:?}", start.elapsed());
}

pub fn write_cache_async(context: Arc<CompilationContext>) {
  std::thread::spawn(move || {
    write_cache(context);
  });
}
