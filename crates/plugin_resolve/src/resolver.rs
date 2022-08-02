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
    specifier: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Result<PluginResolveHookResult> {
    // TODO: try load package.json first, the relative resolve may also need to use browser/exports field in package.json

    // just used for error reporting
    let importer = base_dir.to_string_lossy().to_string();

    let is_specifier_absolute = if let Ok(sp) = PathBuf::from_str(specifier) {
      sp.is_absolute()
    } else {
      false
    };

    let resolved_path = if is_specifier_absolute {
      specifier.to_string()
    } else if specifier.starts_with(".") {
      // if it starts with '.', it is a relative path
      let normalized_path = RelativePath::new(specifier).to_logical_path(base_dir);
      println!("{:?}", normalized_path);
      let normalized_path = normalized_path.as_path();

      let normalized_path = if normalized_path.is_symlink() && self.config.symlinks {
        normalized_path
          .read_link()
          .map_err(|e| CompilationError::GenericError(format!("Read symlink error: {:?}", e)))?
      } else {
        normalized_path.to_path_buf()
      };

      let try_file = self.resolve_file(&normalized_path);

      if let Some(file) = try_file {
        file
      } else {
        // try dir
        let try_dir = self.resolve_directory(&normalized_path);

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
      // TODO support absolute path specifier
      unimplemented!("resolving non-relative specifiers is not supported by now!");
    };

    Ok(PluginResolveHookResult {
      id: resolved_path,
      ..Default::default()
    })
  }

  pub fn resolve_directory(&self, dir: &PathBuf) -> Option<String> {
    None
  }

  /// Try resolve as a file with the configured extensions.
  /// ## Example
  /// if `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension like `/root/index.ts` exists, return it
  pub fn resolve_file(&self, file: &PathBuf) -> Option<String> {
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

  pub fn resolve_alias(&self, specifier: &str, base_dir: PathBuf) -> Option<String> {
    None
  }

  /// Resolve the specifier as a package, return (resolve_path, package_info)
  pub fn resolve_node_modules(
    &self,
    specifier: &str,
    kind: &ResolveKind,
  ) -> Option<(String, Value)> {
    None
  }
}
