use std::{collections::HashMap, path::PathBuf, str::FromStr};

use farmfe_core::{
  config::ResolveConfig,
  error::{CompilationError, Result},
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};
use farmfe_toolkit::resolve::load_package_json;

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

    if is_source_absolute {
      Ok(PluginResolveHookResult {
        id: source.to_string(),
        ..Default::default()
      })
    } else if source.starts_with(".") {
      // if it starts with '.', it is a relative path
      let normalized_path = RelativePath::new(source).to_logical_path(base_dir);
      let normalized_path = normalized_path.as_path();

      let normalized_path = if normalized_path.is_symlink() && self.config.symlinks {
        normalized_path
          .read_link()
          .map_err(|e| CompilationError::GenericError(format!("Read symlink error: {:?}", e)))?
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
        id: resolved_path,
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

    for main_field in &self.config.main_fields {
      let file = dir.join(main_field);

      if let Some(found) = self.try_file(&file) {
        return Some(found);
      }
    }

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

  fn try_alias(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
  ) -> Result<PluginResolveHookResult> {
    for (alias, replaced) in &self.config.alias {
      if alias.ends_with("$") && source == alias {
        return self.resolve(replaced, base_dir, kind);
      } else if !alias.ends_with("$") && source.starts_with(alias) {
        let new_source = source.replace(alias, replaced);
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
    let mut current = base_dir;
    let mut found_node_modules_path = None;

    while current.parent().is_some() {
      let maybe_node_modules_path = current.join(NODE_MODULES);

      if maybe_node_modules_path.exists() && maybe_node_modules_path.is_dir() {
        found_node_modules_path = Some(maybe_node_modules_path);
        break;
      }

      current = current.parent().unwrap().to_path_buf();
    }

    if found_node_modules_path == None {
      return Err(CompilationError::GenericError(
        "can not found node_modules".to_string(),
      ));
    }

    // TODO follow symlink
    let package_dir = RelativePath::new(source).to_logical_path(found_node_modules_path.unwrap());
    let package_json_info = load_package_json(package_dir)?;
    println!("package json info {:?}", package_json_info);
    // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)

    // search normal entry, based on self.config.main_fields, e.g. module/main
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();

    for main_field in &self.config.main_fields {
      if let Some(field_value) = raw_package_json_info.get(main_field) {
        if let Value::String(str) = field_value {
          let dir = package_json_info.dir();
          let full_path = RelativePath::new(str).to_logical_path(dir);

          if full_path.exists() {
            return Ok(PluginResolveHookResult {
              id: full_path.to_string_lossy().to_string(),
              external: false,       // TODO read this from browser
              side_effects: false,   // TODO read this from side_effects in package.json
              query: HashMap::new(), // TODO parse this from the source
              package_json_info: Some(package_json_info),
            });
          }
        }
      }
    }

    // try search node_modules
    panic!(
      "resolving node modules source({}) is not supported by now!",
      source
    );
  }
}
