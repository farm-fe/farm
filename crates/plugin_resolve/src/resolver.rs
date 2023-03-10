use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  str::FromStr,
};

use farmfe_core::{
  common::PackageJsonInfo,
  config::ResolveConfig,
  error::{CompilationError, Result},
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};
use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};

pub struct Resolver {
  config: ResolveConfig,
}

const NODE_MODULES: &str = "node_modules";

impl Resolver {
  pub fn new(config: ResolveConfig) -> Self {
    Self { config }
  }

  /// Specifier type supported by now:
  /// * **Relative Path**: './xxx' or '../xxx'
  /// * **Absolute Path**: '/root/xxx' or 'c:\\root\\xxx'
  /// * **Configured Alias**: '@/pages/xxx'
  /// * **Package**:
  ///   * **exports**: refer to [exports](https://nodejs.org/api/packages.html#packages_conditional_exports), if source is end with '.js', also try to find '.ts' file
  ///   * **browser**: refer to [package-browser-field-spec](https://github.com/defunctzombie/package-browser-field-spec)
  ///   * **module/main**: `{ "module": "es/index.mjs", "main": "lib/index.cjs" }`
  pub fn resolve(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Option<PluginResolveHookResult> {
    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: self.config.symlinks,
        resolve_ancestor_dir: true, // only look for current directory
      },
    );

    // check if module is external
    if let Ok(package_json_info) = &package_json_info {
      if !self.is_source_absolute(source)
        && !self.is_source_relative(source)
        && self.is_module_external(package_json_info, source)
      {
        // this is an external module
        return Some(PluginResolveHookResult {
          resolved_path: String::from(source),
          external: true,
          ..Default::default()
        });
      }

      // check browser replace
      if !self.is_source_absolute(source) && !self.is_source_relative(source) {
        if let Some(resolved_path) = self.try_browser_replace(package_json_info, source) {
          let external = self.is_module_external(package_json_info, &resolved_path);
          let side_effects = self.is_module_side_effects(package_json_info, &resolved_path);

          return Some(PluginResolveHookResult {
            resolved_path,
            external,
            side_effects,
            ..Default::default()
          });
        }
      }
    }

    if self.is_source_absolute(source) {
      if let Some(resolved_path) = self.try_file(&PathBuf::from_str(source).unwrap()) {
        return Some(self.get_resolve_result(&package_json_info, resolved_path));
      } else {
        return None;
      }
    } else if self.is_source_relative(source) {
      // if it starts with '.', it is a relative path
      let normalized_path = RelativePath::new(source).to_logical_path(base_dir);
      let normalized_path = normalized_path.as_path();

      let normalized_path = if self.config.symlinks {
        follow_symlinks(normalized_path.to_path_buf())
      } else {
        normalized_path.to_path_buf()
      };

      // TODO try read symlink from the resolved path step by step to its parent util the root
      let resolved_path = self
        .try_file(&normalized_path)
        .or_else(|| self.try_directory(&normalized_path))
        .ok_or(CompilationError::GenericError(format!(
          "File `{:?}` does not exist",
          normalized_path
        )));

      if let Some(resolved_path) = resolved_path.ok() {
        return Some(self.get_resolve_result(&package_json_info, resolved_path));
      } else {
        None
      }
    } else {
      // try alias first
      self
        .try_alias(source, base_dir.clone(), kind)
        .or_else(|| self.try_node_modules(source, base_dir, kind))
    }
  }

  /// Try resolve as a file with the configured main fields.  
  fn try_directory(&self, dir: &PathBuf) -> Option<String> {
    if !dir.is_dir() {
      return None;
    }

    for main_file in &self.config.main_files {
      let file = dir.join(main_file);

      if let Some(found) = self.try_file(&file) {
        return Some(found);
      }
    }

    None
  }

  /// Try resolve as a file with the configured extensions.
  /// If `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension exists (like `/root/index.ts`), return it immediately
  fn try_file(&self, file: &PathBuf) -> Option<String> {
    // TODO add a test that for directory imports like `import 'comps/button'` where comps/button is a dir
    if file.exists() && file.is_file() {
      Some(file.to_string_lossy().to_string())
    } else {
      let append_extension = |file: &PathBuf, ext: &str| {
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        file.with_file_name(format!("{}.{}", file_name, ext))
      };
      let ext = self.config.extensions.iter().find(|&ext| {
        let new_file = append_extension(file, ext);
        new_file.exists() && new_file.is_file()
      });

      if let Some(ext) = ext {
        Some(append_extension(file, ext).to_string_lossy().to_string())
      } else {
        None
      }
    }
  }

  fn try_alias(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Option<PluginResolveHookResult> {
    for (alias, replaced) in &self.config.alias {
      if alias.ends_with("$") && source == alias.trim_end_matches('$') {
        return self.resolve(replaced, base_dir, kind);
      } else if !alias.ends_with("$") && source.starts_with(alias) {
        let source_left = RelativePath::new(source.trim_start_matches(alias));
        let new_source = source_left
          .to_logical_path(replaced)
          .to_string_lossy()
          .to_string();
        return self.resolve(&new_source, base_dir, kind);
      }
    }

    None
  }

  /// Resolve the source as a package
  fn try_node_modules(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Option<PluginResolveHookResult> {
    // find node_modules until root
    let mut current = base_dir.clone();

    // TODO if a dependency is resolved, cache all paths from base_dir to the resolved node_modules
    while current.parent().is_some() {
      let maybe_node_modules_path = current.join(NODE_MODULES);

      if maybe_node_modules_path.exists() && maybe_node_modules_path.is_dir() {
        let package_path = if self.config.symlinks {
          follow_symlinks(RelativePath::new(source).to_logical_path(maybe_node_modules_path))
        } else {
          RelativePath::new(source).to_logical_path(maybe_node_modules_path)
        };

        let package_json_info = load_package_json(
          package_path.clone(),
          Options {
            follow_symlinks: self.config.symlinks,
            resolve_ancestor_dir: false, // only look for current directory
          },
        );

        if !package_path.join("package.json").exists() {
          if let Some(resolved_path) = self
            .try_file(&package_path)
            .or_else(|| self.try_directory(&package_path))
          {
            return Some(self.get_resolve_result(&package_json_info, resolved_path));
          }
        } else if package_path.exists() && package_path.is_dir() {
          if let Err(_) = package_json_info {
            return None;
          }

          let package_json_info = package_json_info.unwrap();
          // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)

          // search normal entry, based on self.config.main_fields, e.g. module/main
          let raw_package_json_info: Map<String, Value> =
            from_str(package_json_info.raw()).unwrap();

          for main_field in &self.config.main_fields {
            if let Some(field_value) = raw_package_json_info.get(main_field) {
              if let Value::String(str) = field_value {
                let dir = package_json_info.dir();
                let full_path = RelativePath::new(str).to_logical_path(dir);

                return self.try_file(&full_path).map(|resolved_path| {
                  self.get_resolve_result(&Ok(package_json_info), resolved_path)
                });
              }
            }
          }
        }
      }

      current = current.parent().unwrap().to_path_buf();
    }

    // unsupported node_modules resolving type
    None
  }

  fn get_resolve_result(
    &self,
    package_json_info: &Result<PackageJsonInfo>,
    resolved_path: String,
  ) -> PluginResolveHookResult {
    if let Ok(package_json_info) = package_json_info {
      let external = self.is_module_external(&package_json_info, &resolved_path);
      let side_effects = self.is_module_side_effects(&package_json_info, &resolved_path);

      let resolved_path = self
        .try_browser_replace(package_json_info, &resolved_path)
        .unwrap_or(resolved_path);

      return PluginResolveHookResult {
        resolved_path,
        external,
        side_effects,
        ..Default::default()
      };
    } else {
      return PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      };
    }
  }

  fn get_field_value_from_package_json_info(
    &self,
    package_json_info: &PackageJsonInfo,
    field: &str,
  ) -> Option<Value> {
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();

    if let Some(field_value) = raw_package_json_info.get(field) {
      return Some(field_value.clone());
    }

    None
  }

  fn is_module_side_effects(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
  ) -> bool {
    match package_json_info.side_effects() {
      farmfe_core::common::ParsedSideEffects::Bool(b) => *b,
      farmfe_core::common::ParsedSideEffects::Array(arr) => {
        if arr.iter().any(|s| s == resolved_path) {
          true
        } else {
          false
        }
      }
    }
  }

  fn is_module_external(&self, package_json_info: &PackageJsonInfo, resolved_path: &str) -> bool {
    let browser_field = self.get_field_value_from_package_json_info(package_json_info, "browser");

    if let Some(browser_field) = browser_field {
      if let Value::Object(obj) = browser_field {
        for (key, value) in obj {
          let path = Path::new(resolved_path);

          if matches!(value, Value::Bool(false)) {
            // resolved path
            if path.is_absolute() {
              let key_path = RelativePath::new(&key)
                .to_logical_path(package_json_info.dir())
                .to_string_lossy()
                .to_string();

              return &key_path == resolved_path;
            } else {
              // source, e.g. 'foo' in require('foo')
              return &key == resolved_path;
            }
          }
        }
      }
    }

    false
  }

  fn is_source_relative(&self, source: &str) -> bool {
    source.starts_with("./") || source.starts_with("../")
  }

  fn is_source_absolute(&self, source: &str) -> bool {
    if let Ok(sp) = PathBuf::from_str(source) {
      sp.is_absolute()
    } else {
      false
    }
  }

  fn try_browser_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
  ) -> Option<String> {
    let browser_field = self.get_field_value_from_package_json_info(package_json_info, "browser");

    if let Some(browser_field) = browser_field {
      if let Value::Object(obj) = browser_field {
        for (key, value) in obj {
          let path = Path::new(resolved_path);

          // resolved path
          if path.is_absolute() {
            let key_path = RelativePath::new(&key)
              .to_logical_path(package_json_info.dir())
              .to_string_lossy()
              .to_string();

            if &key_path == resolved_path {
              if let Value::String(str) = value {
                let value_path = RelativePath::new(&str)
                  .to_logical_path(package_json_info.dir())
                  .to_string_lossy()
                  .to_string();
                return Some(value_path);
              }
            }
          } else {
            // source, e.g. 'foo' in require('foo')
            if &key == resolved_path {
              if let Value::String(str) = value {
                let value_path = RelativePath::new(&str)
                  .to_logical_path(package_json_info.dir())
                  .to_string_lossy()
                  .to_string();
                return Some(value_path);
              }
            }
          }
        }
      }
    }

    None
  }
}
