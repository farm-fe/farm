use std::{path::PathBuf, str::FromStr};

use farmfe_core::{
  config::ResolveConfig,
  error::{CompilationError, Result},
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::Value,
};

pub struct Resolver {
  config: ResolveConfig,
}

impl Resolver {
  pub fn new(config: ResolveConfig) -> Self {
    Self { config }
  }

  /// Specifier type supported by now:
  /// * **Relative Path**: './xxx' or '../xxx'
  /// * **Absolute Path**: '/root/xxx' or 'c:\\root\\xxx'
  /// * **Configured Alias**: '@/pages/xxx'
  /// * **Package**:
  ///   * **exports**: refer to [exports](https://nodejs.org/api/packages.html#packages_conditional_exports)
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

    let resolved_path = if is_source_absolute {
      source.to_string()
    } else if source.starts_with(".") {
      // if it starts with '.', it is a relative path
      let normalized_path = RelativePath::new(source).to_logical_path(base_dir);
      println!("{:?}", normalized_path);
      let normalized_path = normalized_path.as_path();

      let normalized_path = if normalized_path.is_symlink() && self.config.symlinks {
        normalized_path
          .read_link()
          .map_err(|e| CompilationError::GenericError(format!("Read symlink error: {:?}", e)))?
      } else {
        normalized_path.to_path_buf()
      };

      // TODO try read symlink from the resolved path step by step to its parent util the root
      let try_file = self.try_file(&normalized_path);

      if let Some(file) = try_file {
        file
      } else {
        let try_dir = self.try_directory(&normalized_path);

        if let Some(file) = try_dir {
          file
        } else {
          return Err(CompilationError::GenericError(format!(
            "File `{:?}` does not exist",
            normalized_path
          )));
        }
      }
    } else {
      // TODO support absolute path source
      panic!(
        "resolving non-relative source({}) is not supported by now!",
        source
      );
    };

    Ok(PluginResolveHookResult {
      id: resolved_path,
      ..Default::default()
    })
  }

  fn try_directory(&self, dir: &PathBuf) -> Option<String> {
    None
  }

  /// Try resolve as a file with the configured extensions.
  /// If `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension exists (like `/root/index.ts`), return it immediately
  fn try_file(&self, file: &PathBuf) -> Option<String> {
    if file.exists() {
      Some(file.to_string_lossy().to_string())
    } else {
      let ext = self.config.extensions.iter().find(|&ext| {
        let file = file.with_extension(ext);
        file.exists()
      });

      if let Some(ext) = ext {
        Some(file.with_extension(ext).to_string_lossy().to_string())
      } else {
        None
      }
    }
  }

  fn try_alias(&self, source: &str, base_dir: PathBuf) -> Option<String> {
    None
  }

  /// Resolve the source as a package, return (resolve_path, package_info)
  fn resolve_node_modules(&self, source: &str, kind: &ResolveKind) -> Option<(String, Value)> {
    None
  }
}
