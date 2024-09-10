use std::collections::HashMap;
use std::{
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::regex;
use farmfe_core::{
  common::PackageJsonInfo,
  context::CompilationContext,
  farm_profile_function,
  parking_lot::Mutex,
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};

use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};
use farmfe_utils::relative;

use crate::resolver::browser::try_browser_map;
use crate::resolver::exports::resolve_exports_or_imports;
use crate::resolver::utils::{
  get_field_value_from_package_json_info, is_double_source_dot, is_source_absolute, is_source_dot,
  is_source_relative, ParsePackageSourceResult,
};

use self::browser::{BrowserMapResult, BrowserMapType};

mod browser;
mod exports;
mod utils;

pub use utils::parse_package_source;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResolveCacheKey {
  pub source: String,
  pub base_dir: String,
  pub kind: ResolveKind,
  pub options: ResolveOptions,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct ResolveOptions {
  pub dynamic_extensions: Option<Vec<String>>,
}

pub struct Resolver {
  /// the key is (source, base_dir) and the value is the resolved result
  resolve_cache: Mutex<HashMap<ResolveCacheKey, Option<PluginResolveHookResult>>>,
}

pub const NODE_MODULES: &str = "node_modules";
const BROWSER_SUBPATH_EXTERNAL_ID: &str = "__FARM_BROWSER_SUBPATH_EXTERNAL__";
const REGEX_PREFIX: &str = "$__farm_regex:";
const HIGHEST_PRIORITY_FIELD: &str = "exports";

impl Resolver {
  pub fn new() -> Self {
    Self {
      resolve_cache: Mutex::new(HashMap::new()),
    }
  }

  pub fn resolve(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    let base_dir = if base_dir.is_absolute() {
      base_dir
    } else {
      // using the root as the base dir
      PathBuf::from(&context.config.root)
    };

    let cache_key = ResolveCacheKey {
      source: source.to_string(),
      base_dir: base_dir.to_string_lossy().to_string(),
      kind: kind.clone(),
      options: options.clone(),
    };

    if let Some(result) = self.resolve_cache.lock().get(&cache_key) {
      // None result should not be cached
      if let Some(result) = result {
        return Some(result.clone());
      }
    }

    let result = self._resolve(source, base_dir, kind, options, context);
    self.resolve_cache.lock().insert(cache_key, result.clone());

    result
  }

  /// Specifier type supported by now:
  /// * **Relative Path**: './xxx' or '../xxx'
  /// * **Absolute Path**: '/root/xxx' or 'c:\\root\\xxx'
  /// * **Configured Alias**: '@/pages/xxx'
  /// * **Package**:
  ///   * **exports**: refer to [exports](https://nodejs.org/api/packages.html#packages_conditional_exports), if source is end with '.js', also try to find '.ts' file
  ///   * **browser**: refer to [package-browser-field-spec](https://github.com/defunctzombie/package-browser-field-spec)
  ///   * **module/main**: `{ "module": "es/index.mjs", "main": "lib/index.cjs" }`
  pub fn _resolve(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("resolver::resolve".to_string());

    // 1. try `imports` field(https://nodejs.org/api/packages.html#subpath-imports).
    let resolved_imports = self.try_imports(source, base_dir.clone(), kind, context);
    let (source, base_dir) = if let Some((resolved_imports, package_dir)) = resolved_imports {
      (resolved_imports, PathBuf::from(&package_dir))
    } else {
      (source.to_string(), base_dir)
    };
    let source = source.as_str();

    self
      .try_alias(source, base_dir.clone(), kind, options, context)
      .or_else(|| {
        self.try_relative_or_absolute_path(source, base_dir.clone(), kind, options, context)
      })
      .or_else(|| {
        self.try_browser(
          BrowserMapType::Source(source.to_string()),
          base_dir.clone(),
          kind,
          options,
          context,
        )
      })
      .or_else(|| self.try_node_modules(source, base_dir, kind, options, context))
  }

  fn try_browser(
    &self,
    browser_map_type: BrowserMapType,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("try_browser".to_string());
    if !context.config.output.target_env.is_browser() {
      return None;
    }

    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: true,
      },
    )
    .ok()?;
    let package_path = PathBuf::from(package_json_info.dir());

    if let Some(browser_map_result) = try_browser_map(&package_json_info, browser_map_type.clone())
    {
      match browser_map_result {
        BrowserMapResult::Str(mapped_value) => {
          // alias to external package
          let result = if !is_source_relative(&mapped_value) && !is_source_absolute(&mapped_value) {
            self.try_node_modules(&mapped_value, package_path, kind, options, context)
          } else {
            // alias to local file
            self
              .try_relative_path(&mapped_value, package_path, kind, options, context)
              .map(|resolved_path| PluginResolveHookResult {
                resolved_path,
                external: false,
                ..Default::default()
              })
          };

          return result.map(|result| {
            // find side effects from closest package.json
            let package_json_info = load_package_json(
              PathBuf::from(&result.resolved_path),
              Options {
                follow_symlinks: context.config.resolve.symlinks,
                resolve_ancestor_dir: true,
              },
            )
            .ok();

            let side_effects = package_json_info
              .map(|info| self.is_module_side_effects(&info, &result.resolved_path))
              .unwrap_or(false);

            PluginResolveHookResult {
              side_effects,
              ..result
            }
          });
        }
        BrowserMapResult::External => {
          return Some(PluginResolveHookResult {
            resolved_path: browser_map_type.to_string(),
            external: true,
            side_effects: false,
            ..Default::default()
          });
        }
      }
    }

    None
  }

  fn try_relative_or_absolute_path(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,

    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    // try relative path or absolute path
    self
      .try_absolute_path(source, kind, options, context)
      .or_else(|| self.try_relative_path(source, base_dir.clone(), kind, options, context))
      .map(|resolved_path| {
        // try browser map first
        let browser_map_type = BrowserMapType::ResolvedPath(resolved_path.clone());
        if let Some(result) = self.try_browser(browser_map_type, base_dir, kind, options, context) {
          return result;
        }

        let side_effects = load_package_json(
          PathBuf::from(&resolved_path),
          Options {
            follow_symlinks: context.config.resolve.symlinks,
            resolve_ancestor_dir: true,
          },
        )
        .ok()
        .map(|info| self.is_module_side_effects(&info, &resolved_path))
        .unwrap_or(false);

        PluginResolveHookResult {
          resolved_path,
          external: false,
          side_effects,
          ..Default::default()
        }
      })
  }

  fn try_absolute_path(
    &self,
    path: &str,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_absolute_path".to_string());
    let path_buf = PathBuf::from_str(path).unwrap();

    if !path_buf.is_absolute() {
      return None;
    }

    self
      .try_file(&path_buf, options, context)
      .or_else(|| self.try_directory(&path_buf, kind, false, options, context))
  }

  fn try_relative_path(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    if !is_source_relative(source) {
      return None;
    }

    self
      .try_dot_path(source, base_dir.clone(), kind, options, context)
      .or_else(|| self.try_double_dot(source, base_dir.clone(), kind, options, context))
      .or_else(|| {
        farm_profile_function!("try_relative_path".to_string());
        let normalized_path = RelativePath::new(source).to_logical_path(base_dir);
        let normalized_path = normalized_path.as_path();

        let normalized_path = if context.config.resolve.symlinks {
          follow_symlinks(normalized_path.to_path_buf())
        } else {
          normalized_path.to_path_buf()
        };

        // TODO try read symlink from the resolved path step by step to its parent util the root
        self
          .try_file(&normalized_path, options, context)
          .or_else(|| self.try_directory(&normalized_path, kind, false, options, context))
      })
  }

  fn try_dot_path(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_dot_path".to_string());
    if is_source_dot(source) {
      return self.try_directory(&base_dir, kind, false, options, context);
    }

    None
  }

  fn try_double_dot(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_double_dot".to_string());
    if is_double_source_dot(source) {
      let parent_path = Path::new(&base_dir).parent().unwrap().to_path_buf();
      return self.try_directory(&parent_path, kind, false, options, context);
    }

    None
  }

  /// Try resolve as a file with the configured main fields.
  fn try_directory(
    &self,
    dir: &Path,
    kind: &ResolveKind,
    skip_try_package: bool,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    if !dir.is_dir() {
      return None;
    }

    for main_file in &context.config.resolve.main_files {
      let file = dir.join(main_file);

      if let Some(found) = self.try_file(&file, options, context) {
        return Some(found);
      }
    }

    if !skip_try_package {
      let res = self.try_package_entry(dir.to_path_buf(), kind, options, context);

      if let Some(res) = res {
        return Some(res);
      }
    }

    None
  }

  /// Try resolve as a file with the configured extensions.
  /// If `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension exists (like `/root/index.ts`), return it immediately
  fn try_file(
    &self,
    file: &PathBuf,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    // TODO add a test that for directory imports like `import 'comps/button'` where comps/button is a dir
    if file.exists() && file.is_file() {
      Some(file.to_string_lossy().to_string())
    } else {
      let append_extension = |file: &PathBuf, ext: &str| {
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        file.with_file_name(format!("{}.{}", file_name, ext))
      };
      let extensions = if let Some(ext) = &options.dynamic_extensions {
        ext
      } else {
        &context.config.resolve.extensions
      };
      let ext = extensions.iter().find(|&ext| {
        let new_file = append_extension(file, ext);
        new_file.exists() && new_file.is_file()
      });

      ext.map(|ext| append_extension(file, ext).to_string_lossy().to_string())
    }
  }

  fn try_alias(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("try_alias".to_string());
    // sort the alias by length, so that the longest alias will be matched first
    let mut alias_list: Vec<_> = context.config.resolve.alias.keys().collect();
    alias_list.sort_by(|a, b| b.len().cmp(&a.len()));

    for alias in alias_list {
      let replaced = context.config.resolve.alias.get(alias).unwrap();
      let mut result = None;

      // try regex alias first
      if let Some(alias) = alias.strip_prefix(REGEX_PREFIX) {
        let regex = regex::Regex::new(alias).unwrap();
        if regex.is_match(source) {
          let replaced = regex.replace(source, replaced.as_str()).to_string();
          result = self.resolve(&replaced, base_dir.clone(), kind, options, context);
        }
      } else if alias.ends_with('$') && source == alias.trim_end_matches('$') {
        result = self.resolve(replaced, base_dir.clone(), kind, options, context);
      } else if !alias.ends_with('$') && source.starts_with(alias) {
        // Add absolute path and values in node_modules package

        let source_left = RelativePath::new(source.trim_start_matches(alias));
        let new_source = source_left
          .to_logical_path(replaced)
          .to_string_lossy()
          .to_string();
        if Path::new(&new_source).is_absolute() && !Path::new(&new_source).is_relative() {
          result = self.resolve(&new_source, base_dir.clone(), kind, options, context);
        }
        let (res, _) = self._try_node_modules_internal(
          new_source.as_str(),
          base_dir.clone(),
          kind,
          options,
          context,
        );
        if let Some(resolve_result) = res {
          let resolved_path = resolve_result.resolved_path;
          result = self.resolve(&resolved_path, base_dir.clone(), kind, options, context);
        }
      }

      if result.is_some() {
        return result;
      }
    }

    None
  }

  fn try_node_modules(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    // do not try node_modules for absolute path or relative path
    if is_source_relative(source) || is_source_absolute(source) {
      return None;
    }

    // check if the node modules resolve result is cached
    if let Some(result) = self.resolve_cache.lock().get(&ResolveCacheKey {
      source: source.to_string(),
      base_dir: base_dir.to_string_lossy().to_string(),
      kind: kind.clone(),
      options: options.clone(),
    }) {
      return result.clone();
    }

    let (result, tried_paths) =
      self._try_node_modules_internal(source, base_dir, kind, options, context);
    // cache the result
    for tried_path in tried_paths {
      let mut resolve_node_modules_cache = self.resolve_cache.lock();
      let key = ResolveCacheKey {
        source: source.to_string(),
        base_dir: tried_path.to_string_lossy().to_string(),
        kind: kind.clone(),
        options: options.clone(),
      };

      if !resolve_node_modules_cache.contains_key(&key) {
        resolve_node_modules_cache.insert(key, result.clone());
      }
    }
    result
  }

  /// Resolve the source as a package
  fn _try_node_modules_internal(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    farm_profile_function!("try_node_modules".to_string());
    // find node_modules until root
    let mut current = base_dir;
    // if a dependency is resolved, cache all paths from base_dir to the resolved node_modules
    let mut tried_paths = vec![];

    while current.parent().is_some() {
      let key = ResolveCacheKey {
        source: source.to_string(),
        base_dir: current.to_string_lossy().to_string(),
        kind: kind.clone(),
        options: options.clone(),
      };

      if let Some(result) = self.resolve_cache.lock().get(&key) {
        return (result.clone(), tried_paths);
      }

      tried_paths.push(current.clone());

      let maybe_node_modules_path = current.join(NODE_MODULES);
      if maybe_node_modules_path.exists() && maybe_node_modules_path.is_dir() {
        let parse_package_source_result = utils::parse_package_source(source);
        if parse_package_source_result.is_none() {
          return (None, tried_paths);
        }

        let ParsePackageSourceResult {
          package_name,
          sub_path,
        } = parse_package_source_result.unwrap();

        let package_path = if context.config.resolve.symlinks {
          follow_symlinks(
            RelativePath::new(&package_name).to_logical_path(&maybe_node_modules_path),
          )
        } else {
          RelativePath::new(&package_name).to_logical_path(&maybe_node_modules_path)
        };

        let resolved_path = if let Some(sub_path) = sub_path {
          self.try_package_subpath(&sub_path, package_path.clone(), kind, options, context)
        } else {
          self
            .try_package_entry(package_path.clone(), kind, options, context)
            .map(|resolved_path| {
              // browser map package entry
              let browser_map_type = BrowserMapType::ResolvedPath(resolved_path.clone());
              self
                .try_browser(
                  browser_map_type,
                  package_path.clone(),
                  kind,
                  options,
                  context,
                )
                .map(|res| res.resolved_path)
                .unwrap_or(resolved_path)
            })
        };

        if let Some(resolved_path) = resolved_path {
          let result = if resolved_path == BROWSER_SUBPATH_EXTERNAL_ID {
            PluginResolveHookResult {
              resolved_path,
              external: true,
              side_effects: false,
              ..Default::default()
            }
          } else {
            let side_effects = load_package_json(
              package_path,
              Options {
                follow_symlinks: context.config.resolve.symlinks,
                resolve_ancestor_dir: true,
              },
            )
            .map(|info| self.is_module_side_effects(&info, &resolved_path))
            .unwrap_or(false);
            PluginResolveHookResult {
              resolved_path,
              external: false,
              side_effects,
              ..Default::default()
            }
          };

          return (Some(result), tried_paths);
        }
      }

      current = current.parent().unwrap().to_path_buf();
    }

    // unsupported node_modules resolving type
    (None, tried_paths)
  }

  fn try_package_subpath(
    &self,
    subpath: &str,
    package_path: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_package_subpath".to_string());

    // try package.json under subpath, fix 1402
    let abs_subpath = RelativePath::new(subpath)
      .join("package.json")
      .to_logical_path(package_path.clone());

    if abs_subpath.exists() {
      if let Some(result) = self.try_package_entry(
        abs_subpath.parent().unwrap().to_path_buf(),
        kind,
        options,
        context,
      ) {
        return Some(result.clone());
      }
    }

    let package_json_info = load_package_json(
      package_path.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: false, // only look for current directory
      },
    );

    let relative_path = if let Ok(package_json_info) = package_json_info {
      resolve_exports_or_imports(&package_json_info, subpath, "exports", kind, context)
        .map(|resolve_exports_path| resolve_exports_path.get(0).unwrap().to_string())
        .or_else(|| {
          if context.config.output.target_env.is_browser() {
            try_browser_map(
              &package_json_info,
              BrowserMapType::Source(subpath.to_string()),
            )
            .map(|browser_map_result| match browser_map_result {
              BrowserMapResult::Str(mapped_value) => mapped_value,
              BrowserMapResult::External => BROWSER_SUBPATH_EXTERNAL_ID.to_string(),
            })
          } else {
            None
          }
        })
        .unwrap_or(subpath.to_string())
    } else {
      subpath.to_string()
    };

    if relative_path == BROWSER_SUBPATH_EXTERNAL_ID {
      Some(BROWSER_SUBPATH_EXTERNAL_ID.to_string())
    } else {
      self.try_relative_path(&relative_path, package_path, kind, options, context)
    }
  }

  fn try_package_entry(
    &self,
    package_path: PathBuf,
    kind: &ResolveKind,
    options: &ResolveOptions,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_package".to_string());

    let package_json_info = load_package_json(
      package_path.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: false, // only look for current directory
      },
    )
    .ok()?;

    // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points) by default
    // search normal entry, based on self.config.main_fields, e.g. module/main
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();
    // highest priority: exports field, so we need handle this first
    let entry_point = raw_package_json_info
      .get(HIGHEST_PRIORITY_FIELD)
      .and_then(|_| {
        resolve_exports_or_imports(
          &package_json_info,
          ".",
          HIGHEST_PRIORITY_FIELD,
          kind,
          context,
        )
        .map(|exports_entries| exports_entries.get(0).unwrap().to_string())
      })
      .or_else(|| {
        context
          .config
          .resolve
          .main_fields
          .iter()
          .find_map(|main_field| {
            if main_field == "browser" && !context.config.output.target_env.is_browser() {
              return None;
            }
            raw_package_json_info
              .get(main_field)
              .and_then(|field_value| match field_value {
                Value::Object(_) if main_field == "browser" => {
                  get_field_value_from_package_json_info(&package_json_info, main_field).and_then(
                    |browser| {
                      if let Value::Object(browser) = browser {
                        browser.get(".").and_then(|value| {
                          if let Value::String(value) = value {
                            Some(value.to_string())
                          } else {
                            None
                          }
                        })
                      } else {
                        None
                      }
                    },
                  )
                }
                Value::String(str) => Some(str.to_string()),
                _ => None,
              })
          })
      });
    if let Some(entry_point) = entry_point {
      let dir = package_json_info.dir();
      let entry_point = if !entry_point.starts_with("./") && !entry_point.starts_with("../") {
        format!("./{}", entry_point)
      } else {
        entry_point
      };
      return self.try_relative_path(&entry_point, PathBuf::from(dir), kind, options, context);
    }

    // no main field found, try to resolve index.js file
    self.try_directory(
      Path::new(package_json_info.dir()),
      kind,
      true,
      options,
      context,
    )
  }

  fn try_imports(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<(String, String)> {
    farm_profile_function!("try_imports".to_string());
    if !source.starts_with('#') {
      return None;
    }

    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: true, // only look for current directory
      },
    )
    .ok()?;

    let imports_paths =
      resolve_exports_or_imports(&package_json_info, source, "imports", kind, context);

    imports_paths
      .map(|result| result.get(0).unwrap().to_string())
      .map(|imports_path| (imports_path, package_json_info.dir().to_string()))
  }

  fn is_module_side_effects(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
  ) -> bool {
    farm_profile_function!("is_module_side_effects".to_string());
    match package_json_info.side_effects() {
      Some(side_effect) => match side_effect {
        farmfe_core::common::SideEffects::Bool(b) => *b,
        farmfe_core::common::SideEffects::Array(arr) => arr
          .iter()
          .any(|s| s.is_match(relative(package_json_info.dir(), resolved_path))),
      },
      None => true,
    }
  }
}
