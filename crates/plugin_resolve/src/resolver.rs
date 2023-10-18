use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  config::{Mode, TargetEnv},
  context::CompilationContext,
  error::{CompilationError, Result as CompResult},
  farm_profile_function, farm_profile_scope,
  hashbrown::{HashMap, HashSet},
  parking_lot::Mutex,
  plugin::{PluginResolveHookResult, ResolveKind},
  regex,
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};

use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};

enum Entry {
  Exports(String),
  Imports(String),
}

#[derive(Debug, Eq, PartialEq, Hash)]
enum Condition {
  Default,
  Require,
  Import,
  Browser,
  Node,
  Development,
  Module,
  Production,
}

impl FromStr for Condition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "default" => Ok(Condition::Default),
      "require" => Ok(Condition::Require),
      "import" => Ok(Condition::Import),
      "browser" => Ok(Condition::Browser),
      "node" => Ok(Condition::Node),
      "development" => Ok(Condition::Development),
      "production" => Ok(Condition::Production),
      "module" => Ok(Condition::Module),
      _ => Err(format!("Invalid Condition: {}", s)),
      // _ => {}
    }
  }
}

#[derive(Debug)]
struct ConditionOptions {
  pub unsafe_flag: bool,
  pub require: bool,
  pub browser: bool,
  pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResolveNodeModuleCacheKey {
  pub source: String,
  pub base_dir: String,
  pub kind: ResolveKind,
}

pub struct Resolver {
  /// the key is (source, base_dir) and the value is the resolved result
  resolve_node_modules_cache:
    Mutex<HashMap<ResolveNodeModuleCacheKey, Option<PluginResolveHookResult>>>,
}

const NODE_MODULES: &str = "node_modules";

impl Resolver {
  pub fn new() -> Self {
    Self {
      resolve_node_modules_cache: Mutex::new(HashMap::new()),
    }
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
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("resolver::resolve".to_string());
    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: true, // only look for current directory
      },
    );
    // check if module is external
    if let Ok(package_json_info) = &package_json_info {
      farm_profile_scope!("resolve.check_external".to_string());
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

      if !self.is_source_absolute(source) && !self.is_source_relative(source) {
        // check browser replace
        if let Some(resolved_path) = self.try_browser_replace(package_json_info, source, context) {
          let external = self.is_module_external(package_json_info, &resolved_path);
          let side_effects = self.is_module_side_effects(package_json_info, &resolved_path);
          return Some(PluginResolveHookResult {
            resolved_path,
            external,
            side_effects,
            ..Default::default()
          });
        }

        // check imports replace
        if let Some(resolved_path) = self.try_imports_replace(package_json_info, source, context) {
          if Path::new(&resolved_path).extension().is_none() {
            let parent_path = Path::new(&package_json_info.dir())
              .parent()
              .unwrap()
              .to_path_buf();
            return self.resolve(&resolved_path, parent_path, kind, context);
          }
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

    // try alias first
    if let Some(result) = self.try_alias(source, base_dir.clone(), kind, context) {
      Some(result)
    } else if self.is_source_absolute(source) {
      let path_buf = PathBuf::from_str(source).unwrap();

      return self
        .try_file(&path_buf, context)
        .or_else(|| self.try_directory(source, &path_buf, kind, false, context))
        .map(|resolved_path| {
          self.get_resolve_result(&package_json_info, resolved_path, kind, context)
        });
    } else if self.is_source_relative(source) {
      farm_profile_scope!("resolve.relative".to_string());
      // if it starts with './' or '../, it is a relative path
      let normalized_path = RelativePath::new(source).to_logical_path(base_dir);
      let normalized_path = normalized_path.as_path();

      let normalized_path = if context.config.resolve.symlinks {
        follow_symlinks(normalized_path.to_path_buf())
      } else {
        normalized_path.to_path_buf()
      };

      // TODO try read symlink from the resolved path step by step to its parent util the root
      let resolved_path = self
        .try_file(&normalized_path, context)
        .or_else(|| self.try_directory(source, &normalized_path, kind, false, context))
        .ok_or(CompilationError::GenericError(format!(
          "File `{:?}` does not exist",
          normalized_path
        )));

      if let Ok(resolved_path) = resolved_path {
        return Some(self.get_resolve_result(&package_json_info, resolved_path, kind, context));
      } else {
        None
      }
    } else if self.is_source_dot(source) {
      // import xx from '.'
      return self
        .try_directory(source, &base_dir, kind, false, context)
        .map(|resolved_path| {
          self.get_resolve_result(&package_json_info, resolved_path, kind, context)
        });
    } else if self.is_double_source_dot(source) {
      // import xx from '..'
      let parent_path = Path::new(&base_dir).parent().unwrap().to_path_buf();
      return self
        .try_directory(source, &parent_path, kind, false, context)
        .map(|resolved_path| {
          self.get_resolve_result(&package_json_info, resolved_path, kind, context)
        });
    } else {
      // check if the result is cached
      if let Some(result) = self
        .resolve_node_modules_cache
        .lock()
        .get(&ResolveNodeModuleCacheKey {
          source: source.to_string(),
          base_dir: base_dir.to_string_lossy().to_string(),
          kind: kind.clone(),
        })
      {
        return result.clone();
      }

      let (result, tried_paths) = self.try_node_modules(source, base_dir, kind, context);
      // cache the result
      for tried_path in tried_paths {
        let mut resolve_node_modules_cache = self.resolve_node_modules_cache.lock();
        let key = ResolveNodeModuleCacheKey {
          source: source.to_string(),
          base_dir: tried_path.to_string_lossy().to_string(),
          kind: kind.clone(),
        };

        if !resolve_node_modules_cache.contains_key(&key) {
          resolve_node_modules_cache.insert(key, result.clone());
        }
      }

      result
    }
  }

  /// Try resolve as a file with the configured main fields.
  fn try_directory(
    &self,
    source: &str,
    dir: &Path,
    kind: &ResolveKind,
    skip_try_package: bool,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    if !dir.is_dir() {
      return None;
    }

    for main_file in &context.config.resolve.main_files {
      let file = dir.join(main_file);

      if let Some(found) = self.try_file(&file, context) {
        return Some(found);
      }
    }

    let package_path = dir.join("package.json");

    if package_path.exists() && package_path.is_file() && !skip_try_package {
      let package_json_info = load_package_json(
        package_path,
        Options {
          follow_symlinks: context.config.resolve.symlinks,
          resolve_ancestor_dir: true, // only look for current directory
        },
      );

      if let Ok(package_json_info) = package_json_info {
        let (res, _) = self.try_package(source, &package_json_info, kind, vec![], context);

        if let Some(res) = res {
          return Some(res.resolved_path);
        }
      }
    }

    None
  }

  /// Try resolve as a file with the configured extensions.
  /// If `/root/index` exists, return `/root/index`, otherwise try `/root/index.[configured extension]` in order, once any extension exists (like `/root/index.ts`), return it immediately
  fn try_file(&self, file: &PathBuf, context: &Arc<CompilationContext>) -> Option<String> {
    // TODO add a test that for directory imports like `import 'comps/button'` where comps/button is a dir
    if file.exists() && file.is_file() {
      Some(file.to_string_lossy().to_string())
    } else {
      let append_extension = |file: &PathBuf, ext: &str| {
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        file.with_file_name(format!("{}.{}", file_name, ext))
      };
      let ext = context.config.resolve.extensions.iter().find(|&ext| {
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
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("try_alias".to_string());
    // sort the alias by length, so that the longest alias will be matched first
    let mut alias_list: Vec<_> = context.config.resolve.alias.keys().collect();
    alias_list.sort_by(|a, b| b.len().cmp(&a.len()));

    for alias in alias_list {
      let replaced = context.config.resolve.alias.get(alias).unwrap();

      if alias.ends_with('$') && source == alias.trim_end_matches('$') {
        return self.resolve(replaced, base_dir, kind, context);
      } else if !alias.ends_with('$') && source.starts_with(alias) {
        let source_left = RelativePath::new(source.trim_start_matches(alias));
        let new_source = source_left
          .to_logical_path(replaced)
          .to_string_lossy()
          .to_string();

        return self.resolve(&new_source, base_dir, kind, context);
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
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    farm_profile_function!("try_node_modules".to_string());
    // find node_modules until root
    let mut current = base_dir;
    // if a dependency is resolved, cache all paths from base_dir to the resolved node_modules
    let mut tried_paths = vec![];

    while current.parent().is_some() {
      let key = ResolveNodeModuleCacheKey {
        source: source.to_string(),
        base_dir: current.to_string_lossy().to_string(),
        kind: kind.clone(),
      };

      if let Some(result) = self.resolve_node_modules_cache.lock().get(&key) {
        return (result.clone(), tried_paths);
      }

      tried_paths.push(current.clone());

      let maybe_node_modules_path = current.join(NODE_MODULES);
      if maybe_node_modules_path.exists() && maybe_node_modules_path.is_dir() {
        let package_path = if context.config.resolve.symlinks {
          follow_symlinks(RelativePath::new(source).to_logical_path(&maybe_node_modules_path))
        } else {
          RelativePath::new(source).to_logical_path(&maybe_node_modules_path)
        };
        let package_json_info = load_package_json(
          package_path.clone(),
          Options {
            follow_symlinks: context.config.resolve.symlinks,
            resolve_ancestor_dir: false, // only look for current directory
          },
        );
        /*
         * TODO: fix that exports and sub package.json exists at the same time. e.g. `@swc/helpers/_/_interop_require_default`.
         * This may need to refactor the exports resolve logic.
         * Refer to https://github.com/npm/validate-npm-package-name/blob/main/lib/index.js#L3 for the package name recognition and determine the sub path,
         * instead of judging the existence of package.json.
         */
        if !package_path.join("package.json").exists() {
          // check if the source is a directory or file can be resolved
          if matches!(&package_path, package_path if package_path.exists()) {
            if let Some(resolved_path) = self
              .try_file(&package_path, context)
              .or_else(|| self.try_directory(source, &package_path, kind, true, context))
            {
              return (
                Some(self.get_resolve_node_modules_result(
                  source,
                  package_json_info.ok().as_ref(),
                  resolved_path,
                  kind,
                  context,
                )),
                tried_paths,
              );
            }
          }
          // split source loop find package.json
          // Arranged according to the priority from back to front
          let source_parts: Vec<&str> = source.split('/').filter(|s| !s.is_empty()).collect();
          let split_source_result = source_parts
            .iter()
            .scan(String::new(), |prev_path, &single_source| {
              let new_path = format!("{}/{}", prev_path, single_source);
              *prev_path = new_path.clone();
              Some(new_path)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<String>>();
          let package_json_info = load_package_json(
            package_path.clone(),
            Options {
              follow_symlinks: context.config.resolve.symlinks,
              resolve_ancestor_dir: false, // only look for current directory
            },
          );
          for item_source in &split_source_result {
            let package_path_dir = if context.config.resolve.symlinks {
              follow_symlinks(
                RelativePath::new(item_source).to_logical_path(&maybe_node_modules_path),
              )
            } else {
              RelativePath::new(item_source).to_logical_path(&maybe_node_modules_path)
            };
            if package_path_dir.exists() && package_path_dir.is_dir() {
              let package_json_info = load_package_json(
                package_path_dir.clone(),
                Options {
                  follow_symlinks: context.config.resolve.symlinks,
                  resolve_ancestor_dir: false, // only look for current directory
                },
              );
              if package_json_info.is_ok() {
                return (
                  Some(self.get_resolve_node_modules_result(
                    source,
                    package_json_info.ok().as_ref(),
                    package_path.to_str().unwrap().to_string(),
                    kind,
                    context,
                  )),
                  tried_paths,
                );
              }
            }
          }
          if let Some(resolved_path) = self
            .try_file(&package_path, context)
            .or_else(|| self.try_directory(source, &package_path, kind, true, context))
          {
            return (
              Some(self.get_resolve_node_modules_result(
                source,
                package_json_info.ok().as_ref(),
                resolved_path,
                kind,
                context,
              )),
              tried_paths,
            );
          }
        } else if package_path.exists() && package_path.is_dir() {
          if package_json_info.is_err() {
            return (None, tried_paths);
          }
          let package_json_info = package_json_info.unwrap();

          let (result, tried_paths) =
            self.try_package(source, &package_json_info, kind, tried_paths, context);

          if result.is_some() {
            return (result, tried_paths);
          }

          // no main field found, try to resolve index.js file
          return (
            self
              .try_file(&package_path.join("index"), context)
              .map(|resolved_path| {
                self.get_resolve_node_modules_result(
                  source,
                  Some(&package_json_info),
                  resolved_path,
                  kind,
                  context,
                )
              }),
            tried_paths,
          );
        }
      }

      current = current.parent().unwrap().to_path_buf();
    }

    // unsupported node_modules resolving type
    (None, tried_paths)
  }

  fn try_package(
    &self,
    source: &str,
    package_json_info: &PackageJsonInfo,
    kind: &ResolveKind,
    tried_paths: Vec<PathBuf>,
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    farm_profile_function!("try_package".to_string());
    // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)

    // search normal entry, based on self.config.main_fields, e.g. module/main
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();

    for main_field in &context.config.resolve.main_fields {
      if main_field == "browser" && context.config.output.target_env == TargetEnv::Node {
        continue;
      }

      if let Some(field_value) = raw_package_json_info.get(main_field) {
        if let Value::Object(_) = field_value {
          let resolved_path = Some(self.get_resolve_node_modules_result(
            source,
            Some(package_json_info),
            package_json_info.dir().to_string(),
            kind,
            context,
          ));
          let result = resolved_path.as_ref().unwrap();
          let path = Path::new(result.resolved_path.as_str());
          if let Some(_extension) = path.extension() {
            return (resolved_path, tried_paths);
          }
        } else if let Value::String(str) = field_value {
          let dir = package_json_info.dir();
          let full_path = RelativePath::new(str).to_logical_path(dir);
          // the main fields can be a file or directory
          return match self.try_file(&full_path, context) {
            Some(resolved_path) => (
              Some(self.get_resolve_node_modules_result(
                source,
                Some(package_json_info),
                resolved_path,
                kind,
                context,
              )),
              tried_paths,
            ),
            None => (
              self
                .try_directory(source, &full_path, kind, true, context)
                .map(|resolved_path| {
                  self.get_resolve_node_modules_result(
                    source,
                    Some(package_json_info),
                    resolved_path,
                    kind,
                    context,
                  )
                }),
              tried_paths,
            ),
          };
        }
      }
    }

    (None, tried_paths)
  }

  fn get_resolve_result(
    &self,
    package_json_info: &CompResult<PackageJsonInfo>,
    resolved_path: String,
    _kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    farm_profile_function!("get_resolve_result".to_string());
    if let Ok(package_json_info) = package_json_info {
      let external = self.is_module_external(package_json_info, &resolved_path);
      let side_effects = self.is_module_side_effects(package_json_info, &resolved_path);
      let resolved_path = self
        .try_browser_replace(package_json_info, &resolved_path, context)
        .unwrap_or(resolved_path);
      PluginResolveHookResult {
        resolved_path,
        external,
        side_effects,
        ..Default::default()
      }
    } else {
      PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      }
    }
  }

  fn get_resolve_node_modules_result(
    &self,
    source: &str,
    package_json_info: Option<&PackageJsonInfo>,
    resolved_path: String,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    farm_profile_function!("get_resolve_node_modules_result".to_string());
    if let Some(package_json_info) = package_json_info {
      let side_effects = self.is_module_side_effects(package_json_info, &resolved_path);
      let resolved_path = self
        .try_exports_replace(source, package_json_info, &resolved_path, kind, context)
        .unwrap_or(resolved_path);
      // fix: not exports field, eg: "@ant-design/icons-svg/es/asn/SearchOutlined"
      let resolved_path_buf = PathBuf::from(&resolved_path);
      let resolved_path = self
        .try_file(&resolved_path_buf, context)
        .or_else(|| self.try_directory(source, &resolved_path_buf, kind, true, context))
        .unwrap_or(resolved_path);
      PluginResolveHookResult {
        resolved_path,
        side_effects,
        ..Default::default()
      }
    } else {
      PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      }
    }
  }

  fn try_exports_replace(
    &self,
    source: &str,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_exports_replace".to_string());
    // TODO: add all cases from https://nodejs.org/api/packages.html
    let re = regex::Regex::new(r"^(?P<group1>[^@][^/]*)/|^(?P<group2>@[^/]+/[^/]+)/").unwrap();
    let is_matched = re.is_match(source);
    if let Some(resolve_exports_path) = self.resolve_exports_or_imports(
      package_json_info,
      source,
      "exports",
      kind,
      context,
      is_matched,
    ) {
      let resolved_id = resolve_exports_path.get(0).unwrap();
      let value_path = self.get_key_path(resolved_id, package_json_info.dir());
      return Some(value_path);
    }

    None
  }

  fn try_browser_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_browser_replace".to_string());
    if context.config.output.target_env != TargetEnv::Browser {
      return None;
    }

    let browser_field = self.get_field_value_from_package_json_info(package_json_info, "browser");
    if let Some(Value::Object(obj)) = browser_field {
      for (key, value) in obj {
        let path = Path::new(resolved_path);
        // resolved path
        if path.is_absolute() {
          let key_path = self.get_key_path(&key, package_json_info.dir());
          if self.are_paths_equal(key_path, resolved_path) {
            if let Value::String(str) = value {
              let value_path = self.get_key_path(&str, package_json_info.dir());
              return Some(value_path);
            }
          }
        } else {
          // TODO: this is not correct, it should remap the package name
          // source, e.g. 'foo' in require('foo')
          if self.are_paths_equal(&key, resolved_path) {
            if let Value::String(str) = value {
              let value_path = self.get_key_path(&str, package_json_info.dir());
              return Some(value_path);
            }
          }
        }
      }
    }

    None
  }

  fn try_imports_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_imports_replace".to_string());
    if resolved_path.starts_with('#') {
      let imports_field = self.get_field_value_from_package_json_info(package_json_info, "imports");
      if let Some(Value::Object(imports_field_map)) = imports_field {
        for (key, value) in imports_field_map {
          if self.are_paths_equal(&key, resolved_path) {
            if let Value::String(str) = &value {
              return self.get_string_value_path(str, package_json_info);
            }

            if let Value::Object(str) = &value {
              for (key, value) in str {
                match context.config.output.target_env {
                  TargetEnv::Browser => {
                    if self.are_paths_equal(key, "default") {
                      if let Value::String(str) = value {
                        return self.get_string_value_path(str, package_json_info);
                      }
                    }
                  }
                  TargetEnv::Node => {
                    if self.are_paths_equal(key, "node") {
                      if let Value::String(str) = value {
                        return self.get_string_value_path(str, package_json_info);
                      }
                    }
                  }
                }
              }
              // }
            }
          }
        }
      }
    }

    None
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
    farm_profile_function!("is_module_side_effects".to_string());
    match package_json_info.side_effects() {
      farmfe_core::common::ParsedSideEffects::Bool(b) => *b,
      farmfe_core::common::ParsedSideEffects::Array(arr) => arr.iter().any(|s| s == resolved_path),
    }
  }

  fn is_module_external(&self, package_json_info: &PackageJsonInfo, resolved_path: &str) -> bool {
    farm_profile_function!("is_module_external".to_string());
    let browser_field = self.get_field_value_from_package_json_info(package_json_info, "browser");

    if let Some(Value::Object(obj)) = browser_field {
      for (key, value) in obj {
        let path = Path::new(resolved_path);

        if matches!(value, Value::Bool(false)) {
          // resolved path
          if path.is_absolute() {
            let key_path = self.get_key_path(&key, package_json_info.dir());

            if key_path == resolved_path {
              return true;
            }
          } else {
            // source, e.g. 'foo' in require('foo')
            if key == resolved_path {
              return true;
            }
          }
        }
      }
    }

    false
  }

  fn is_source_relative(&self, source: &str) -> bool {
    // fix: relative path start with .. or ../
    source.starts_with("./") || source.starts_with("../")
  }

  fn is_source_absolute(&self, source: &str) -> bool {
    if let Ok(sp) = PathBuf::from_str(source) {
      sp.is_absolute()
    } else {
      false
    }
  }

  fn is_source_dot(&self, source: &str) -> bool {
    source == "."
  }

  fn is_double_source_dot(&self, source: &str) -> bool {
    source == ".."
  }

  /**
   * check if two paths are equal
   * Prevent path carrying / cause path resolution to fail
   */

  fn are_paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(&self, path1: P1, path2: P2) -> bool {
    farm_profile_function!("are_paths_equal".to_string());
    let path1 = PathBuf::from(path1.as_ref());
    let path2 = PathBuf::from(path2.as_ref());
    let path1_suffix = path1.strip_prefix("/").unwrap_or(&path1);
    let path2_suffix = path2.strip_prefix("/").unwrap_or(&path2);
    path1_suffix == path2_suffix
  }

  /**
   * get key path with other different key
   * TODO need add a argument (default | node) to determine the key
   */

  fn get_key_path(&self, key: &str, dir: &String) -> String {
    farm_profile_function!("get_key_path".to_string());
    let key_path = match Path::new(&key).is_relative() {
      true => {
        let resolve_key = &key.trim_matches('\"');
        RelativePath::new(resolve_key).to_logical_path(dir)
      }
      false => RelativePath::new("").to_logical_path(dir),
    };
    key_path.to_string_lossy().to_string()
  }

  /**
   * get normal path_value
   */
  fn get_string_value_path(
    &self,
    str: &str,
    package_json_info: &PackageJsonInfo,
  ) -> Option<String> {
    farm_profile_function!("get_string_value_path".to_string());
    let path = Path::new(&str);
    if path.extension().is_none() {
      // resolve imports field import other deps. import can only use relative paths
      return Some(path.to_string_lossy().to_string());
    } else {
      let value_path = self.get_key_path(str, package_json_info.dir());
      Some(value_path)
    }
  }

  fn exports(
    self: &Self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    config: &ConditionOptions,
  ) -> Option<Vec<String>> {
    if let Some(exports_field) =
      self.get_field_value_from_package_json_info(package_json_info, "exports")
    {
      // TODO If the current package does not have a name, then look up for the name of the folder
      let name = match self.get_field_value_from_package_json_info(package_json_info, "name") {
        Some(n) => n,
        None => {
          eprintln!(
            "Missing \"name\" field in package.json {:?}",
            package_json_info
          );
          return None;
        }
      };
      let mut map: HashMap<String, Value> = HashMap::new();
      match exports_field {
        Value::String(string_value) => {
          map.insert(".".to_string(), Value::String(string_value.clone()));
        }
        Value::Object(object_value) => {
          for (k, v) in &object_value {
            if !k.starts_with('.') {
              map.insert(".".to_string(), Value::Object(object_value.clone()));
              break;
            } else {
              map.insert(k.to_string(), v.clone());
            }
          }
        }
        _ => {}
      }
      if !map.is_empty() {
        return Some(self.walk(name.as_str().unwrap(), &map, source, config));
      }
    }

    None
  }

  fn imports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    config: &ConditionOptions,
  ) -> Option<Vec<String>> {
    if let Some(imports_field) =
      self.get_field_value_from_package_json_info(package_json_info, "imports")
    {
      // TODO If the current package does not have a name, then look up for the name of the folder
      let name = match self.get_field_value_from_package_json_info(package_json_info, "name") {
        Some(n) => n,
        None => {
          // 如果 name 找不到 也要解析一下错误情况处理返回
          eprintln!(
            "Missing \"name\" field in package.json {:?}",
            package_json_info
          );
          return None;
        }
      };
      let mut imports_map: HashMap<String, Value> = HashMap::new();

      match imports_field {
        Value::Object(object_value) => {
          imports_map.extend(object_value.clone());
        }
        _ => {
          eprintln!("Unexpected imports field format");
          return None;
        }
      }
      return Some(self.walk(name.as_str().unwrap(), &imports_map, source, config));
    }
    None
  }

  fn resolve_exports_or_imports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    field_type: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
    is_matched: bool,
  ) -> Option<Vec<String>> {
    farm_profile_function!("resolve_exports_or_imports".to_string());
    let mut additional_conditions: HashSet<String> = vec![
      String::from("development"),
      String::from("production"),
      String::from("module"),
    ]
    .into_iter()
    .collect();

    let resolve_conditions: HashSet<String> = context
      .config
      .resolve
      .conditions
      .clone()
      .into_iter()
      .collect();
    additional_conditions.extend(resolve_conditions);

    let filtered_conditions: Vec<String> = additional_conditions
      .clone()
      .into_iter()
      .filter(|condition| match condition.as_str() {
        "production" => {
          let mode = &context.config.mode;
          matches!(mode, Mode::Production)
        }
        "development" => {
          let mode = &context.config.mode;
          matches!(mode, Mode::Development)
        }
        _ => true,
      })
      .collect();

    // resolve exports field
    let is_browser = TargetEnv::Browser == context.config.output.target_env;
    let is_require = match kind {
      ResolveKind::Require => true,
      _ => false,
    };
    let condition_config = ConditionOptions {
      browser: is_browser && !additional_conditions.contains("node"),
      require: is_require && !additional_conditions.contains("import"),
      conditions: filtered_conditions,
      // set default unsafe_flag to insert require & import field
      unsafe_flag: false,
    };
    let id: &str = if is_matched { source } else { "." };
    let result: Option<Vec<String>> = if field_type == "imports" {
      self.imports(package_json_info, source, &condition_config)
    } else {
      self.exports(package_json_info, id, &condition_config)
    };
    return result;
  }

  // TODO ---------------------------------------------
  fn conditions(self: &Self, options: &ConditionOptions) -> HashSet<Condition> {
    let mut out: HashSet<Condition> = HashSet::new();
    out.insert(Condition::Default);
    // TODO resolver other conditions
    // for condition in options.conditions.iter() {
    //   out.insert(condition.parse().unwrap());
    // }
    for condition_str in &options.conditions {
      match Condition::from_str(condition_str) {
        Ok(condition_enum) => {
          out.insert(condition_enum);
        }
        Err(error) => {
          // TODO resolve error
          eprintln!("Error: {}", error);
        }
      }
    }
    if !options.unsafe_flag {
      if options.require {
        out.insert(Condition::Require);
      } else {
        out.insert(Condition::Import);
      }

      if options.browser {
        out.insert(Condition::Browser);
      } else {
        out.insert(Condition::Node);
      }
    }
    out
  }

  fn injects(self: &Self, items: &mut Vec<String>, value: &str) -> Option<Vec<String>> {
    let rgx1: regex::Regex = regex::Regex::new(r#"\*"#).unwrap();
    let rgx2: regex::Regex = regex::Regex::new(r#"/$"#).unwrap();

    for item in items.iter_mut() {
      let tmp = item.clone();
      if rgx1.is_match(&tmp) {
        *item = rgx1.replace(&tmp, value).to_string();
      } else if rgx2.is_match(&tmp) {
        *item += value;
      }
    }

    return items.clone().into_iter().map(|s| Some(s)).collect();
  }

  fn loop_value(
    self: &Self,
    m: Value,
    keys: &HashSet<Condition>,
    mut result: &mut Option<HashSet<String>>,
  ) -> Option<Vec<String>> {
    match m {
      Value::String(s) => {
        if let Some(result_set) = result {
          result_set.insert(s.clone());
        }
        Some(vec![s])
      }
      Value::Array(values) => {
        let arr_result = result.clone().unwrap_or_else(|| HashSet::new());
        for item in values {
          if let Some(item_result) = self.loop_value(item, keys, &mut Some(arr_result.clone())) {
            return Some(item_result);
          }
        }

        // 如果使用了初始化的结果集，返回结果
        if result.is_none() && !arr_result.is_empty() {
          return Some(arr_result.into_iter().collect());
        } else {
          None
        }
      }
      Value::Object(map) => {
        // TODO Temporarily define the order problem
        let property_order: Vec<String> = vec![
          String::from("browser"),
          String::from("development"),
          String::from("module"),
          String::from("import"),
          String::from("require"),
          String::from("default"),
        ];

        for key in &property_order {
          if let Some(value) = map.get(key) {
            if let Ok(condition) = Condition::from_str(&key) {
              if keys.contains(&condition) {
                return self.loop_value(value.clone(), keys, result);
              }
            }
          }
        }
        None
      }
      Value::Null => None,
      _ => None,
    }
  }

  fn to_entry(
    self: &Self,
    name: &str,
    ident: &str,
    externals: Option<bool>,
  ) -> Result<String, String> {
    if name == ident || ident == "." {
      return Ok(".".to_string());
    }

    let root = format!("{}/", name);
    let len = root.len();
    let bool = ident.starts_with(&root);
    let output = if bool {
      ident[len..].to_string()
    } else {
      ident.to_string()
    };

    if output.starts_with('#') {
      return Ok(output);
    }

    if bool || externals.unwrap_or(false) {
      if output.starts_with("./") {
        Ok(output)
      } else {
        Ok(format!("./{}", output))
      }
    } else {
      Err(output)
    }
  }

  fn throws(self: &Self, name: &str, entry: &str, condition: Option<i32>) {
    let message = if let Some(cond) = condition {
      if cond != 0 {
        format!(
          "No known conditions for \"{}\" specifier in \"{}\" package",
          entry, name
        )
      } else {
        format!("Missing \"{}\" specifier in \"{}\" package", entry, name)
      }
    } else {
      format!("Missing \"{}\" specifier in \"{}\" package", entry, name)
    };
    eprintln!("{}", message);
  }

  fn walk(
    self: &Self,
    name: &str,
    mapping: &HashMap<String, Value>,
    input: &str,
    options: &ConditionOptions,
  ) -> Vec<String> {
    let entry_result: Result<String, String> = self.to_entry(name, input, Some(true));
    let entry: String = match entry_result {
      Ok(entry) => entry.to_string(),
      Err(error) => {
        eprintln!("Error resolve {} package error: {}", name, error);
        String::from(name)
      }
    };
    let c: HashSet<Condition> = self.conditions(options);
    let mut m: Option<&Value> = mapping.get(&entry);
    let mut result: Option<Vec<String>> = None;
    let mut replace: Option<String> = None;
    if m.is_none() {
      let mut longest: Option<&str> = None;

      for (key, value) in mapping.iter() {
        if let Some(cur_longest) = &longest {
          if key.len() < cur_longest.len() {
            // do not allow "./" to match if already matched "./foo*" key
            continue;
          }
        }

        if key.ends_with('/') && entry.starts_with(key) {
          replace = Some(entry[key.len()..].to_string());
          longest = Some(key.as_str());
        } else if key.len() > 1 {
          if let Some(tmp) = key.find('*') {
            let pattern = format!("^{}(.*){}", &key[..tmp], &key[tmp + 1..]);
            let regex = regex::Regex::new(&pattern).unwrap();

            if let Some(captures) = regex.captures(&entry) {
              if let Some(match_group) = captures.get(1) {
                replace = Some(match_group.as_str().to_string());
                longest = Some(key.as_str());
              }
            }
          }
        }
      }

      if let Some(longest_key) = longest {
        m = mapping.get(&longest_key.to_string());
      }
    }
    if m.is_none() {
      self.throws(name, &entry, None);
      return Vec::new(); // 返回一个空 Vec 作为错误处理的默认值
    }
    let v = self.loop_value(m.unwrap().clone(), &c, &mut None);
    if v.is_none() {
      self.throws(name, &entry, Some(1));
      return Vec::new(); // 返回一个空 Vec 作为错误处理的默认值
    }
    let mut cloned_v = v.clone();
    if let Some(replace) = replace {
      if let Some(v1) = self.injects(&mut cloned_v.unwrap(), &replace) {
        return v1;
      }
    }
    v.unwrap()
  }
}
