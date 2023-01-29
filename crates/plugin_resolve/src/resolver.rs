use std::{collections::HashMap, path::PathBuf, str::FromStr};

use farmfe_core::{
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
  ) -> Result<PluginResolveHookResult> {
    // TODO: try load package.json first, the relative resolve may also need to use browser/exports field in package.json
    let is_source_absolute = if let Ok(sp) = PathBuf::from_str(source) {
      sp.is_absolute()
    } else {
      false
    };

    if is_source_absolute {
      if let Some(resolved_path) = self.try_file(&PathBuf::from_str(source).unwrap()) {
        return Ok(PluginResolveHookResult {
          resolved_path,
          ..Default::default()
        });
      } else {
        return Err(CompilationError::GenericError(format!(
          "File `{:?}` does not exist",
          source
        )));
      }
    } else if source.starts_with(".") {
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
        )))?;

      Ok(PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      })
    } else {
      // try alias first
      self
        .try_alias(source, base_dir.clone(), kind)
        .or_else(|_| self.try_node_modules(source, base_dir, kind))
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
  ) -> Result<PluginResolveHookResult> {
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

    Err(CompilationError::GenericError(String::new()))
  }

  /// Resolve the source as a package
  fn try_node_modules(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Result<PluginResolveHookResult> {
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

        // TODO cover it with tests
        if !package_path.join("package.json").exists() {
          if let Some(resolved_path) = self
            .try_file(&package_path)
            .or_else(|| self.try_directory(&package_path))
          {
            return Ok(PluginResolveHookResult {
              resolved_path,
              external: false,       // TODO read this from browser
              side_effects: false,   // TODO read this from side_effects in package.json
              query: HashMap::new(), // TODO parse this from the source
            });
          }
        } else if package_path.exists() && package_path.is_dir() {
          let package_json_info = load_package_json(
            package_path.clone(),
            Options {
              follow_symlinks: self.config.symlinks,
              resolve_ancestor_dir: false, // only look for current directory
            },
          )?;
          // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)

          // search normal entry, based on self.config.main_fields, e.g. module/main
          let raw_package_json_info: Map<String, Value> =
            from_str(package_json_info.raw()).unwrap();

          for main_field in &self.config.main_fields {
            if let Some(field_value) = raw_package_json_info.get(main_field) {
              if let Value::String(str) = field_value {
                let dir = package_json_info.dir();
                let full_path = RelativePath::new(str).to_logical_path(dir);

                if full_path.exists() {
                  return Ok(PluginResolveHookResult {
                    resolved_path: full_path.to_string_lossy().to_string(),
                    external: false,       // TODO read this from browser
                    side_effects: false,   // TODO read this from side_effects in package.json
                    query: HashMap::new(), // TODO parse this from the source
                  });
                }
              }
            }
          }
        }
      }

      current = current.parent().unwrap().to_path_buf();
    }

    // unsupported node_modules resolving type
    panic!(
      "resolving node modules(resolve {} from {:?}) is not supported by now!",
      source, base_dir
    );
  }
}
