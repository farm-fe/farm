use std::{
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_core::{
  common::PackageJsonInfo,
  config::TargetEnv,
  context::CompilationContext,
  error::{CompilationError, Result},
  hashbrown::HashMap,
  parking_lot::Mutex,
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};
use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};

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
    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
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
      return Some(result);
    } else if self.is_source_absolute(source) {
      let path_buf = PathBuf::from_str(source).unwrap();

      return self
        .try_file(&path_buf, context)
        .or_else(|| self.try_directory(&path_buf, kind, false, context))
        .map(|resolved_path| {
          self.get_resolve_result(&package_json_info, resolved_path, kind, context)
        });
    } else if self.is_source_relative(source) {
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
        .or_else(|| self.try_directory(&normalized_path, kind, false, context))
        .ok_or(CompilationError::GenericError(format!(
          "File `{:?}` does not exist",
          normalized_path
        )));

      if let Some(resolved_path) = resolved_path.ok() {
        return Some(self.get_resolve_result(&package_json_info, resolved_path, kind, context));
      } else {
        None
      }
    } else if self.is_source_dot(source) {
      // import xx from '.'
      return self
        .try_directory(&base_dir, kind, false, context)
        .map(|resolved_path| {
          self.get_resolve_result(&package_json_info, resolved_path, kind, context)
        });
    } else if self.is_double_source_dot(source) {
      // import xx from '..'
      let parent_path = Path::new(&base_dir).parent().unwrap().to_path_buf();
      return self
        .try_directory(&parent_path, kind, false, context)
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

      let (result, tried_paths) = self.try_node_modules(source, base_dir.clone(), kind, context);
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
    dir: &PathBuf,
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
        let (res, _) = self.try_package(&package_json_info, kind, vec![], context);

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
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    // sort the alias by length, so that the longest alias will be matched first
    let mut alias_list: Vec<_> = context
      .config
      .resolve
      .alias
      .iter()
      .map(|(alias, _)| alias)
      .collect();
    alias_list.sort_by(|a, b| b.len().cmp(&a.len()));

    for alias in alias_list {
      let replaced = context.config.resolve.alias.get(alias).unwrap();

      if alias.ends_with("$") && source == alias.trim_end_matches('$') {
        return self.resolve(replaced, base_dir, kind, context);
      } else if !alias.ends_with("$") && source.starts_with(alias) {
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
    // find node_modules until root
    let mut current = base_dir.clone();
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
              .or_else(|| self.try_directory(&package_path, kind, true, context))
            {
              return (
                Some(self.get_resolve_node_modules_result(
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
              if let Ok(_) = package_json_info {
                return (
                  Some(self.get_resolve_node_modules_result(
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
            .or_else(|| self.try_directory(&package_path, kind, true, context))
          {
            return (
              Some(self.get_resolve_node_modules_result(
                package_json_info.ok().as_ref(),
                resolved_path,
                kind,
                context,
              )),
              tried_paths,
            );
          }
        } else if package_path.exists() && package_path.is_dir() {
          if let Err(_) = package_json_info {
            return (None, tried_paths);
          }
          let package_json_info = package_json_info.unwrap();

          let (result, tried_paths) =
            self.try_package(&package_json_info, kind, tried_paths, context);

          if result.is_some() {
            return (result, tried_paths);
          }

          // no main field found, try to resolve index.js file
          return (
            self
              .try_file(&package_path.join("index"), context)
              .map(|resolved_path| {
                self.get_resolve_node_modules_result(
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
    package_json_info: &PackageJsonInfo,
    kind: &ResolveKind,
    tried_paths: Vec<PathBuf>,
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)

    // search normal entry, based on self.config.main_fields, e.g. module/main
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();

    for main_field in &context.config.resolve.main_fields {
      if main_field == "browser" {
        if context.config.output.target_env == TargetEnv::Node {
          continue;
        }
      }

      if let Some(field_value) = raw_package_json_info.get(main_field) {
        if let Value::Object(_) = field_value {
          let resolved_path = Some(self.get_resolve_node_modules_result(
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
                Some(&package_json_info),
                resolved_path,
                kind,
                context,
              )),
              tried_paths,
            ),
            None => (
              self
                .try_directory(&full_path, kind, true, context)
                .map(|resolved_path| {
                  self.get_resolve_node_modules_result(
                    Some(&package_json_info),
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
    package_json_info: &Result<PackageJsonInfo>,
    resolved_path: String,
    _kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    if let Ok(package_json_info) = package_json_info {
      let external = self.is_module_external(&package_json_info, &resolved_path);
      let side_effects = self.is_module_side_effects(&package_json_info, &resolved_path);
      let resolved_path = self
        .try_browser_replace(package_json_info, &resolved_path, context)
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

  fn get_resolve_node_modules_result(
    &self,
    package_json_info: Option<&PackageJsonInfo>,
    resolved_path: String,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    if let Some(package_json_info) = package_json_info {
      let side_effects = self.is_module_side_effects(&package_json_info, &resolved_path);
      let resolved_path = self
        .try_exports_replace(package_json_info, &resolved_path, &kind, context)
        .unwrap_or(resolved_path);
      // fix: not exports field, eg: "@ant-design/icons-svg/es/asn/SearchOutlined"
      let resolved_path_buf = PathBuf::from(&resolved_path);
      let resolved_path = self
        .try_file(&resolved_path_buf, context)
        .or_else(|| self.try_directory(&resolved_path_buf, kind, true, context))
        .unwrap_or_else(|| resolved_path);

      PluginResolveHookResult {
        resolved_path,
        side_effects,
        ..Default::default()
      }
    } else {
      return PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      };
    }
  }

  fn try_exports_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    // resolve exports field
    // TODO: add all cases from https://nodejs.org/api/packages.html
    let exports_field = self.get_field_value_from_package_json_info(package_json_info, "exports");
    if let Some(exports_field) = exports_field {
      let dir = package_json_info.dir();
      let path = Path::new(resolved_path);
      if let Value::Object(obj) = exports_field {
        for (key, value) in obj {
          let key_path = self.get_key_path(&key, &dir);
          if self.are_paths_equal(key_path, resolved_path) {
            match value {
              Value::String(current_field_value) => {
                let dir = package_json_info.dir();
                let path = Path::new(resolved_path);
                if path.is_absolute() {
                  let key_path = self.get_key_path(&key, &dir);

                  if self.are_paths_equal(&key_path, resolved_path) {
                    let value_path =
                      self.get_key_path(&current_field_value, package_json_info.dir());
                    return Some(value_path);
                  }
                }
              }
              Value::Object(current_field_obj) => {
                for (key_word, key_value) in current_field_obj {
                  match kind {
                    // import with node default
                    ResolveKind::Import => {
                      if self.are_paths_equal(&key_word, "default") {
                        if path.is_absolute() {
                          let value_path =
                            self.get_key_path(&key_value.to_string(), package_json_info.dir());
                          return Some(value_path);
                        }
                      }
                      if self.are_paths_equal(&key_word, "import") {
                        match key_value {
                          Value::String(import_value) => {
                            if path.is_absolute() {
                              let value_path =
                                self.get_key_path(&import_value, package_json_info.dir());
                              return Some(value_path);
                            }
                          }
                          Value::Object(import_value) => {
                            for (key_word, key_value) in import_value {
                              match context.config.output.target_env {
                                TargetEnv::Node => {
                                  if self.are_paths_equal(&key_word, "node") {
                                    if path.is_absolute() {
                                      let value_path = self.get_key_path(
                                        &key_value.as_str().unwrap(),
                                        package_json_info.dir(),
                                      );
                                      return Some(value_path);
                                    }
                                  }
                                }
                                TargetEnv::Browser => {
                                  if self.are_paths_equal(key_word, "default") {
                                    if path.is_absolute() {
                                      let value_path = self.get_key_path(
                                        &key_value.as_str().unwrap(),
                                        package_json_info.dir(),
                                      );
                                      return Some(value_path);
                                    }
                                  }
                                }
                              }
                            }
                          }
                          _ => {}
                        }
                      }
                    }
                    ResolveKind::Require => {
                      if key_word.to_lowercase() == "require" {
                        let path = Path::new(resolved_path);
                        match key_value {
                          Value::String(key_value) => {
                            if path.is_absolute() {
                              let value_path =
                                self.get_key_path(&key_value, package_json_info.dir());
                              return Some(value_path);
                            }
                          }
                          Value::Object(key_value) => {
                            if path.is_absolute() {
                              for (key, value) in key_value {
                                match context.config.output.target_env {
                                  TargetEnv::Node => {
                                    if self.are_paths_equal(key, "default") {
                                      if path.is_absolute() {
                                        let value_path = self.get_key_path(
                                          &value.as_str().unwrap(),
                                          package_json_info.dir(),
                                        );
                                        return Some(value_path);
                                      }
                                    }
                                  }
                                  TargetEnv::Browser => {
                                    if self.are_paths_equal(key, "default") {
                                      if path.is_absolute() {
                                        let value_path = self.get_key_path(
                                          &value.as_str().unwrap(),
                                          package_json_info.dir(),
                                        );
                                        return Some(value_path);
                                      }
                                    }
                                  }
                                }
                              }
                            }
                          }
                          _ => {}
                        }
                      }
                    }
                    _ => {}
                  }
                }
              }
              _ => {
                // TODO strict_exports config with error
              }
            }
          }
        }
      }
    }

    None
  }

  fn try_browser_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    if context.config.output.target_env != TargetEnv::Browser {
      return None;
    }

    let browser_field = self.get_field_value_from_package_json_info(package_json_info, "browser");
    if let Some(browser_field) = browser_field {
      if let Value::Object(obj) = browser_field {
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
    }

    None
  }

  fn try_imports_replace(
    &self,
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    if resolved_path.starts_with('#') {
      let imports_field = self.get_field_value_from_package_json_info(package_json_info, "imports");
      if let Some(imports_field) = imports_field {
        if let Value::Object(imports_field_map) = imports_field {
          for (key, value) in imports_field_map {
            if self.are_paths_equal(&key, resolved_path) {
              if let Value::String(str) = &value {
                return self.get_string_value_path(&str, package_json_info);
              }

              if let Value::Object(str) = &value {
                for (key, value) in str {
                  match context.config.output.target_env {
                    TargetEnv::Browser => {
                      if self.are_paths_equal(&key, "default") {
                        if let Value::String(str) = value {
                          return self.get_string_value_path(&str, package_json_info);
                        }
                      }
                    }
                    TargetEnv::Node => {
                      if self.are_paths_equal(&key, "node") {
                        if let Value::String(str) = value {
                          return self.get_string_value_path(&str, package_json_info);
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
              let key_path = self.get_key_path(&key, package_json_info.dir());

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
    let path = Path::new(&str);
    if path.extension().is_none() {
      // resolve imports field import other deps. import can only use relative paths
      return Some(path.to_string_lossy().to_string());
    } else {
      let value_path = self.get_key_path(&str, package_json_info.dir());
      return Some(value_path);
    }
  }
}
