use std::{
  fs,
  path::{Path, PathBuf},
};

use farmfe_core::{
  config::{bool_or_obj::BoolOrObj, Mode},
  HashMap,
};

mod common;

use common::create_compiler_with_args;

fn create_test_directory_structure(base_path: &Path) -> std::io::Result<()> {
  // Create public directory with test file
  let public_dir = base_path.join("public");
  fs::create_dir_all(&public_dir)?;
  fs::write(public_dir.join("test.txt"), "test content")?;
  
  // Create a subdirectory in public
  let public_subdir = public_dir.join("assets");
  fs::create_dir_all(&public_subdir)?;
  fs::write(public_subdir.join("image.png"), "fake image")?;

  // Create source files
  fs::write(base_path.join("index.html"), "<html><body>Test</body></html>")?;
  fs::write(base_path.join("index.js"), "console.log('test');")?;

  Ok(())
}

#[test]
fn test_public_dir_copy_with_separate_folders() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let test_dir = crate_path
    .join("tests")
    .join("fixtures")
    .join("public_dir_test_separate");

  // Clean up if exists
  if test_dir.exists() {
    fs::remove_dir_all(&test_dir).ok();
  }

  // Create test directory structure
  fs::create_dir_all(&test_dir).unwrap();
  create_test_directory_structure(&test_dir).unwrap();

  let compiler = create_compiler_with_args(
    test_dir.clone(),
    crate_path.clone(),
    |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(
        "index".to_string(),
        test_dir.join("index.html").to_string_lossy().to_string(),
      )]);
      config.output.path = "./dist".to_string();
      config.assets.public_dir = Some(test_dir.join("public").to_string_lossy().to_string());
      config.mode = Mode::Production;
      config.minify = Box::new(BoolOrObj::Bool(false));

      (config, plugins)
    },
  );

  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();

  // Verify that public files were copied to output directory
  let output_dir = test_dir.join("dist");
  assert!(output_dir.join("test.txt").exists(), "test.txt should be copied");
  assert!(
    output_dir.join("assets").join("image.png").exists(),
    "image.png should be copied"
  );

  // Clean up
  fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_public_dir_copy_with_nested_output_in_public() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let test_dir = crate_path
    .join("tests")
    .join("fixtures")
    .join("public_dir_test_nested");

  // Clean up if exists
  if test_dir.exists() {
    fs::remove_dir_all(&test_dir).ok();
  }

  // Create test directory structure
  fs::create_dir_all(&test_dir).unwrap();
  create_test_directory_structure(&test_dir).unwrap();

  let compiler = create_compiler_with_args(
    test_dir.clone(),
    crate_path.clone(),
    |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(
        "index".to_string(),
        test_dir.join("index.html").to_string_lossy().to_string(),
      )]);
      // Set output inside public directory - this should trigger the warning
      config.output.path = "./public/build".to_string();
      config.assets.public_dir = Some(test_dir.join("public").to_string_lossy().to_string());
      config.mode = Mode::Production;
      config.minify = Box::new(BoolOrObj::Bool(false));

      (config, plugins)
    },
  );

  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();

  // Verify that output directory was created
  let output_dir = test_dir.join("public").join("build");
  assert!(output_dir.exists(), "Output directory should be created");

  // Verify that public files were NOT copied (to avoid infinite nesting)
  // The original public/test.txt should NOT be copied into public/build/
  let copied_test_file = output_dir.join("test.txt");
  assert!(
    !copied_test_file.exists(),
    "test.txt should NOT be copied when output is inside public"
  );

  // Clean up
  fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_public_dir_copy_with_output_as_parent_of_public() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let test_dir = crate_path
    .join("tests")
    .join("fixtures")
    .join("public_dir_test_parent");

  // Clean up if exists
  if test_dir.exists() {
    fs::remove_dir_all(&test_dir).ok();
  }

  // Create test directory structure
  fs::create_dir_all(&test_dir).unwrap();
  
  // Create a nested public directory structure
  let nested_public = test_dir.join("src").join("public");
  fs::create_dir_all(&nested_public).unwrap();
  fs::write(nested_public.join("test.txt"), "test content").unwrap();

  fs::write(test_dir.join("index.html"), "<html><body>Test</body></html>").unwrap();
  fs::write(test_dir.join("index.js"), "console.log('test');").unwrap();

  let compiler = create_compiler_with_args(
    test_dir.clone(),
    crate_path.clone(),
    |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(
        "index".to_string(),
        test_dir.join("index.html").to_string_lossy().to_string(),
      )]);
      // Set output as parent of public directory
      config.output.path = "./src".to_string();
      config.assets.public_dir = Some(nested_public.to_string_lossy().to_string());
      config.mode = Mode::Production;
      config.minify = Box::new(BoolOrObj::Bool(false));

      (config, plugins)
    },
  );

  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();

  // Verify that output directory was created
  let output_dir = test_dir.join("src");
  assert!(output_dir.exists(), "Output directory should be created");

  // Verify that public files were NOT copied (output contains public)
  let copied_test_file = output_dir.join("test.txt");
  assert!(
    !copied_test_file.exists(),
    "test.txt should NOT be copied when public is inside output"
  );

  // Clean up
  fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_are_separate_folders_logic() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let test_dir = crate_path
    .join("tests")
    .join("fixtures")
    .join("separate_folders_test");

  // Clean up if exists
  if test_dir.exists() {
    fs::remove_dir_all(&test_dir).ok();
  }

  fs::create_dir_all(&test_dir).unwrap();
  fs::write(test_dir.join("index.html"), "<html></html>").unwrap();

  // Test case 1: Separate folders (should pass)
  let compiler1 = create_compiler_with_args(
    test_dir.clone(),
    crate_path.clone(),
    |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(
        "index".to_string(),
        test_dir.join("index.html").to_string_lossy().to_string(),
      )]);
      config.output.path = "./dist".to_string();
      config.mode = Mode::Production;
      config.minify = Box::new(BoolOrObj::Bool(false));
      (config, plugins)
    },
  );

  // Use the internal are_separate_folders logic
  // Since it's private, we test the behavior through write_resources_to_disk
  compiler1.compile().unwrap();
  
  // Should succeed without panic
  let result1 = compiler1.write_resources_to_disk();
  assert!(result1.is_ok(), "Should succeed with separate folders");

  // Test case 2: Output inside public (should warn and skip)
  let compiler2 = create_compiler_with_args(
    test_dir.clone(),
    crate_path.clone(),
    |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(
        "index".to_string(),
        test_dir.join("index.html").to_string_lossy().to_string(),
      )]);
      config.output.path = "./public/dist".to_string();
      config.mode = Mode::Production;
      config.minify = Box::new(BoolOrObj::Bool(false));
      (config, plugins)
    },
  );

  compiler2.compile().unwrap();
  
  // Should succeed but skip copying public directory
  let result2 = compiler2.write_resources_to_disk();
  assert!(result2.is_ok(), "Should succeed but skip copying");

  // Clean up
  fs::remove_dir_all(&test_dir).ok();
}
