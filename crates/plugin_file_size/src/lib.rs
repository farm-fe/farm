#![deny(clippy::all)]

use colored::*;
use farmfe_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use farmfe_core::{
  config::Config, context::CompilationContext, error::CompilationError, plugin::Plugin,
  stats::Stats,
};

use farmfe_macro_plugin::farm_plugin;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

pub struct FarmPluginFileSize {}

impl FarmPluginFileSize {
  pub fn new(config: &Config) -> Self {
    Self {}
  }
}

impl FarmPluginFileSize {
  fn calculate_gzip_size(&self, content: &[u8]) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content).unwrap();
    encoder.finish().unwrap().len()
  }

  fn format_size(&self, size: usize) -> ColoredString {
    let size_str = if size < 1024 {
      format!("{} B", size)
    } else if size < 1024 * 1024 {
      format!("{:.1} KB", size as f64 / 1024.0)
    } else {
      format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    };

    if size > 300 * 1024 {
      size_str.red().bold()
    } else if size > 100 * 1024 {
      size_str.yellow().bold()
    } else {
      size_str.green()
    }
  }

  fn get_size_color(&self, size: usize, text: &str) -> ColoredString {
    if size > 300 * 1024 {
      // > 300KB
      text.red().bold()
    } else if size > 100 * 1024 {
      // > 100KB
      text.yellow().bold()
    } else {
      text.green()
    }
  }

  fn format_gzip_size(&self, size: usize) -> ColoredString {
    let size_str = if size < 1024 {
      format!("{} B", size)
    } else if size < 1024 * 1024 {
      format!("{:.1} KB", size as f64 / 1024.0)
    } else {
      format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    };

    size_str.normal().dimmed()
  }

  fn format_path_and_name(&self, path: &str, name: &str, size: usize) -> String {
    let normalized_path = Path::new(path)
      .components()
      .filter_map(|comp| match comp {
        std::path::Component::Normal(s) => s.to_str(),
        _ => None,
      })
      .collect::<Vec<_>>()
      .join("/");
    let full_path = format!("{}/{}", normalized_path, name);
    let display_width = UnicodeWidthStr::width(full_path.as_str());

    let padding = if display_width < 45 {
      " ".repeat(45 - display_width)
    } else {
      String::new()
    };

    format!(
      "{}{}{}{}",
      normalized_path.dimmed(),
      "/".dimmed(),
      self.get_size_color(size, name),
      padding
    )
  }
}

impl Plugin for FarmPluginFileSize {
  fn name(&self) -> &str {
    "FarmPluginFileSize"
  }

  fn priority(&self) -> i32 {
    999
  }

  fn finish(
    &self,
    _stat: &Stats,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>, CompilationError> {
    let resources_map = context.resources_map.lock();
    let output_config = context.config.output.clone();
    println!("\n{}", "Output files:".bold());
    println!();

    println!(
      "{}",
      format!("{:<45} {:>12} {:>12}", "File", "Size", "Gzipped").bold()
    );

    let files: Vec<_> = resources_map
      .iter()
      .filter(|(_, resource)| !resource.emitted)
      .map(|(name, resource)| (name.to_string(), resource.bytes.to_vec()))
      .collect();

    let files_with_size: Vec<_> = files
      .par_iter()
      .map(|(name, bytes)| {
        let size = bytes.len();
        let gzip_size = self.calculate_gzip_size(bytes);
        (name.clone(), size, gzip_size)
      })
      .collect();

    let mut files_with_size = files_with_size;
    files_with_size.sort_by(|a, b| a.0.cmp(&b.0));

    let mut total_size = 0;
    let mut total_gzip_size = 0;

    for (name, size, gzip_size) in files_with_size {
      total_size += size;
      total_gzip_size += gzip_size;

      println!(
        "{:<45}  {:>12}  {:>12}",
        self.format_path_and_name(&context.config.output.path, &name, size),
        self.format_size(size),
        self.format_gzip_size(gzip_size)
      );
    }

    println!("\n{}", "Total:".bold());
    println!("  {} {}", "Size:".bold(), self.format_size(total_size));
    println!(
      "  {} {}",
      "Gzipped:".bold(),
      self.format_gzip_size(total_gzip_size)
    );
    println!();

    Ok(None)
  }
}
