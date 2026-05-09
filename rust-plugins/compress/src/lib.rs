#![deny(clippy::all)]

use farmfe_core::error::CompilationError;
use farmfe_core::parking_lot::Mutex;
use farmfe_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use farmfe_core::regex::Regex;
use farmfe_core::resource::{Resource, ResourceType};
use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::hash::sha256;

mod utils;

type CompressionOutput = (
  String,
  farmfe_core::resource::ResourceOrigin,
  farmfe_core::error::Result<Vec<u8>>,
  usize,
  farmfe_core::HashMap<String, String>,
  bool,
  farmfe_core::HashMap<String, String>,
);

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum CompressAlgorithm {
  #[default]
  Brotli,
  Gzip,
  DeflateRaw,
  Deflate,
}

fn default_filter() -> String {
  "\\.(js|mjs|json|css|html)$".to_string()
}

fn default_level() -> u32 {
  6
}

fn default_threshold() -> usize {
  1024
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  #[serde(default)]
  pub algorithm: CompressAlgorithm,
  #[serde(default = "default_level")]
  pub level: u32,
  #[serde(default = "default_threshold")]
  pub threshold: usize,
  #[serde(default = "default_filter")]
  pub filter: String,
  pub delete_origin_file: Option<bool>,
}

#[farm_plugin]
pub struct FarmfePluginCompress {
  options: Options,
  time_cost: Mutex<std::time::Duration>,
  saved: Mutex<usize>,
}

impl FarmfePluginCompress {
  fn new(_config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    Self {
      options,
      time_cost: Default::default(),
      saved: Mutex::new(0),
    }
  }
}

impl Plugin for FarmfePluginCompress {
  fn name(&self) -> &str {
    "FarmfePluginCompress"
  }

  fn priority(&self) -> i32 {
    101
  }

  fn finalize_resources(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let start = std::time::Instant::now();
    let ext_name = utils::get_ext_name(&self.options.algorithm);
    let should_delete_origin = self.options.delete_origin_file.unwrap_or(false);
    let filter = Regex::new(&self.options.filter).map_err(|e| {
      CompilationError::GenericError(format!("Invalid regex expression for compress plugin: {e}"))
    })?;

    let compressed_buffers: Vec<CompressionOutput> = param
      .resources_map
      .par_iter()
      .filter_map(|(resource_id, resource)| {
        // Skip resources already marked as emitted (e.g. Farm's internal runtime
        // which is inlined into HTML and never written to disk as a standalone file).
        // Compressing them would produce orphaned .br/.gz files with no matching
        // original, causing the preview server to 404 or fall back to index.html.
        if resource.emitted
          || !filter.is_match(resource_id)
          || resource.bytes.len() < self.options.threshold
        {
          return None;
        }
        Some((
          resource_id.to_string(),
          resource.origin.clone(),
          utils::compress_buffer(&resource.bytes, &self.options.algorithm, self.options.level),
          resource.bytes.len(),
          resource.meta.clone(),
          resource.should_transform_output_filename,
          resource.special_placeholders.clone(),
        ))
      })
      .collect();

    let mut saved = 0;
    for (
      resource_id,
      origin,
      buffer,
      origin_file_size,
      meta,
      should_transform_output_filename,
      special_placeholders,
    ) in compressed_buffers
    {
      let bytes = buffer?;
      let name = format!("{resource_id}.{ext_name}");
      saved += origin_file_size - bytes.len();
      param.resources_map.insert(
        name.clone(),
        Resource {
          name: name.clone(),
          bytes,
          emitted: false,
          resource_type: ResourceType::Custom(ext_name.to_string()),
          origin,
          name_hash: sha256(name.as_bytes(), 8),
          meta,
          should_transform_output_filename,
          special_placeholders,
        },
      );

      if should_delete_origin {
        if let Some(resource) = param.resources_map.get_mut(&resource_id) {
          resource.emitted = true;
        }
      }
    }

    *self.saved.lock() = saved;
    *self.time_cost.lock() = start.elapsed();

    Ok(None)
  }

  fn finish(
    &self,
    _stat: &farmfe_core::stats::Stats,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    println!(
      "\x1b[1m\x1b[38;2;113;26;95m[ Farm ]\x1b[39m\x1b[0m Compress plugin finished in \x1b[1m\x1b[32m{:.2}ms\x1b[0m \
      and reduced size by \x1b[1m\x1b[32m{:.2}KB\x1b[0m.",
      self.time_cost.lock().as_secs_f64() * 1000.0,
      *self.saved.lock() / 1024
    );
    Ok(None)
  }
}
