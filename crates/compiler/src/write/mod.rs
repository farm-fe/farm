use crate::Compiler;
use farmfe_core::error::Result;
use farmfe_core::rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use farmfe_core::relative_path::RelativePath;
use farmfe_core::resource::Resource;
use farmfe_core::HashMap;

use std::collections::HashSet;
use std::fs::{copy, create_dir_all, read_dir, File};
use std::io::{BufWriter, Write};
use std::path::Path;

pub type SplitResources<'a> = (
  Vec<(&'a String, &'a Resource)>,
  Vec<(&'a String, &'a Resource)>,
);

// default set to 8192 Memory allocation threshold issues Ensure that the code runs perfectly
// TODO may be different for different platforms threshold will be different linux / macos / windows
const SMALL_FILE_THRESHOLD: usize = 8192;

// TODO use error::{CompilationError} we need refactor Error mod
impl Compiler {
  // pub(crate) fn write(&self) -> Result<()> {
  //   // TODO add writeBundle write hooks plugin_driver
  //   self.write_resources_to_disk()?;

  //   Ok(())
  // }

  pub fn write_resources_to_disk(&self) -> Result<()> {
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_function!();

    let output_dir = Path::new(&self.context.config.output.path);

    let output_dir = if output_dir.is_absolute() {
      output_dir.to_path_buf()
    } else {
      let rel_path = RelativePath::new(&self.context.config.output.path);
      rel_path.to_logical_path(&self.context.config.root)
    };
    let output_dir = output_dir.as_path();

    {
      let resources_map = self.context.resources_map.lock();

      // create output directories for all resources
      self
        .create_output_directories(&resources_map, output_dir)
        .unwrap();

      let (small_files, large_files) = self.split_resources(&resources_map);

      self.write_large_files(&large_files, output_dir).unwrap();

      self.write_small_files(&small_files, output_dir).unwrap();
    }

    self.copy_public_dir(output_dir).unwrap();

    Ok(())
  }

  fn copy_public_dir(&self, output_dir: &Path) -> Result<()> {
    let build_config = &self.context.config;
    let output_dir_path = Path::new(&output_dir);
    let public_dir_path = match &build_config.assets.public_dir {
      Some(dir) => Path::new(dir),
      None => return Ok(()),
    };
    if !public_dir_path.exists() {
      return Ok(());
    }

    if !self.are_separate_folders(output_dir_path, public_dir_path) {
      // TODO: add Farm rust logger
      println!(
        "\n(!) [Farm warn] The public directory feature may not work correctly. \
               outDir {} and publicDir {} are not separate folders.\n",
        output_dir_path.display(),
        public_dir_path.display()
      );
    }

    Self::copy_dir(public_dir_path, output_dir_path);

    Ok(())
  }

  fn are_separate_folders(&self, dir1: &Path, dir2: &Path) -> bool {
    if let Ok(relative) = dir2.strip_prefix(dir1) {
      if !relative.as_os_str().is_empty() && !relative.to_string_lossy().starts_with("..") {
        return false;
      }
    }

    if let Ok(relative) = dir1.strip_prefix(dir2) {
      if !relative.as_os_str().is_empty() && !relative.to_string_lossy().starts_with("..") {
        return false;
      }
    }

    true
  }

  fn copy_dir(from: &Path, to: &Path) {
    if !from.exists() {
      return;
    }

    if !to.exists() {
      create_dir_all(to).unwrap();
    }

    for entry in read_dir(from).unwrap() {
      let entry = entry.unwrap();
      let file_type = entry.file_type().unwrap();
      let to_path = to.join(entry.file_name());

      if file_type.is_dir() {
        Self::copy_dir(&entry.path(), &to_path);
      } else {
        // do not overwrite existing file
        if to_path.exists() {
          println!(
            "[Farm warn] public directory file {:?} already exists in outputDir, skip copy",
            entry.file_name()
          );
          continue;
        }

        copy(entry.path(), to_path).unwrap();
      }
    }
  }

  fn create_output_directories(
    &self,
    resources_map: &HashMap<String, Resource>,
    output_dir: &Path,
  ) -> Result<()> {
    let mut dirs = HashSet::new();
    for name in resources_map.keys() {
      let path = Path::new(output_dir).join(name.split(['?', '#']).next().unwrap());
      if let Some(parent) = path.parent() {
        dirs.insert(parent.to_path_buf());
      }
    }

    // TODO try catch error
    for dir in dirs {
      create_dir_all(&dir).unwrap();
    }

    Ok(())
  }

  fn split_resources<'a>(
    &self,
    resources_map: &'a HashMap<String, Resource>,
  ) -> SplitResources<'a> {
    let mut small_files = Vec::new();
    let mut large_files = Vec::new();

    for (name, resource) in resources_map.iter() {
      if resource.emitted {
        continue;
      }

      if resource.bytes.len() < SMALL_FILE_THRESHOLD {
        small_files.push((name, resource));
      } else {
        large_files.push((name, resource));
      }
    }

    (small_files, large_files)
  }

  fn write_large_files(
    &self,
    large_files: &[(&String, &Resource)],
    output_dir: &Path,
  ) -> Result<()> {
    // temporary use try_for_each to avoid panic and ok return instead of Result
    large_files.par_iter().for_each(|&(name, resource)| {
      let path = output_dir.join(name.split(['?', '#']).next().unwrap());
      self.write_file(&path, &resource.bytes);
    });

    Ok(())
  }

  fn write_small_files(
    &self,
    small_files: &[(&String, &Resource)],
    output_dir: &Path,
  ) -> Result<()> {
    for (name, resource) in small_files {
      let path = output_dir.join(name.split(['?', '#']).next().unwrap());
      self.write_file(&path, &resource.bytes);
    }
    Ok(())
  }

  // TODO use error::{CompilationError} we need refactor Error mod
  fn write_file(&self, path: &Path, content: &[u8]) {
    let file = File::create(path).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(content).unwrap();
    writer.flush().unwrap();
  }
}
