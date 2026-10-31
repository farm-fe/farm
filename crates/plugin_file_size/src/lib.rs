#![deny(clippy::all)]

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::resource::Resource;

  #[test]
  fn finish_completes_without_relocking_resources() {
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
    context.resources_map.lock().insert(
      "site.js".into(),
      Resource {
        name: "site.js".into(),
        bytes: b"export default 1".to_vec(),
        resource_type: ResourceType::Js,
        ..Default::default()
      },
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
      let plugin = FarmPluginFileSize::new(&context.config);
      sender
        .send(plugin.finish(&Stats::default(), &context))
        .unwrap();
    });
    receiver
      .recv_timeout(std::time::Duration::from_secs(3))
      .expect("actual finish must complete without recursively taking its resource lock")
      .unwrap();
    worker.join().unwrap();
  }

  #[test]
  fn compression_does_not_hold_shared_resource_lock() {
    let context = CompilationContext::new(Config::default(), vec![]).unwrap();
    context.resources_map.lock().insert(
      "site.js".into(),
      Resource {
        name: "site.js".into(),
        bytes: b"export default 1".to_vec(),
        resource_type: ResourceType::Js,
        ..Default::default()
      },
    );
    let plugin = FarmPluginFileSize::new(&context.config);
    let rows = plugin.resource_sizes(&context, |bytes| {
      assert!(
        context.resources_map.try_lock().is_some(),
        "parallel compression must not hold the lock needed by other finish hooks"
      );
      plugin.calculate_gzip_size(bytes)
    });
    assert_eq!(rows.len(), 1);
    assert_eq!(
      rows[0],
      (
        "site.js".into(),
        16,
        plugin.calculate_gzip_size(b"export default 1")
      )
    );
  }

  #[test]
  fn resource_measurements_keep_filtering_and_real_gzip_sizes() {
    let context = CompilationContext::new(Config::default(), vec![]).unwrap();
    for (name, kind, emitted) in [
      ("a.js", ResourceType::Js, false),
      ("b.css", ResourceType::Css, false),
      ("a.js.map", ResourceType::SourceMap("a.js".into()), false),
      ("done.js", ResourceType::Js, true),
    ] {
      context.resources_map.lock().insert(
        name.into(),
        Resource {
          name: name.into(),
          bytes: b"content".to_vec(),
          resource_type: kind,
          emitted,
          ..Default::default()
        },
      );
    }
    let plugin = FarmPluginFileSize::new(&context.config);
    let mut rows = plugin.resource_sizes(&context, |bytes| plugin.calculate_gzip_size(bytes));
    rows.sort();
    let gzip = plugin.calculate_gzip_size(b"content");
    assert_eq!(
      rows,
      vec![("a.js".into(), 7, gzip), ("b.css".into(), 7, gzip)]
    );
  }
}

use colored::*;
use farmfe_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use farmfe_core::resource::ResourceType;
use farmfe_core::{
  config::Config, context::CompilationContext, error::CompilationError, plugin::Plugin,
  stats::Stats,
};

use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

pub struct FarmPluginFileSize {}

impl FarmPluginFileSize {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl FarmPluginFileSize {
  fn resource_sizes(
    &self,
    context: &CompilationContext,
    calculate_size: impl Fn(&[u8]) -> usize + Sync,
  ) -> Vec<(String, usize, usize)> {
    // Nested Rayon work may run another finish hook that needs this lock.
    let files: Vec<_> = {
      let resources_map = context.resources_map.lock();
      resources_map
        .iter()
        .filter(|(_, resource)| {
          !resource.emitted && !matches!(resource.resource_type, ResourceType::SourceMap(_))
        })
        .map(|(name, resource)| (name.to_string(), resource.bytes.to_vec()))
        .collect()
    };

    files
      .par_iter()
      .map(|(name, bytes)| {
        let size = bytes.len();
        let gzip_size = calculate_size(bytes);
        (name.clone(), size, gzip_size)
      })
      .collect()
  }

  fn calculate_gzip_size(&self, content: &[u8]) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content).unwrap();
    encoder.finish().unwrap().len()
  }

  fn format_size(&self, size: usize) -> ColoredString {
    let size_str = if size < 1024 {
      format!("{size} B")
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
      format!("{size} B")
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
    let full_path = format!("{normalized_path}/{name}");
    let display_width = UnicodeWidthStr::width(full_path.as_str());

    let padding = if display_width < 45 {
      " ".repeat(45 - display_width)
    } else {
      String::new()
    };

    format!(
      "{}{}{}{padding}",
      normalized_path.dimmed(),
      "/".dimmed(),
      self.get_size_color(size, name)
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
    // let output_config = context.config.output.clone();
    println!("\n{}", "Output files:".bold());
    println!();

    println!(
      "{}",
      format!("{:<45} {:<12} {:<12}", "File", " Size", "  Gzipped").bold()
    );

    let mut files_with_size = self.resource_sizes(context, |bytes| self.calculate_gzip_size(bytes));
    files_with_size.sort_by(|a, b| a.0.cmp(&b.0));

    let mut total_size = 0;
    let mut total_gzip_size = 0;

    for (name, size, gzip_size) in files_with_size {
      total_size += size;
      total_gzip_size += gzip_size;

      println!(
        "{:<45}  {:<12}  {:<12}",
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
