use farmfe_core::{
  common::PackageJsonInfo,
  config::{Mode, TargetEnv},
  context::CompilationContext,
  error::{CompilationError, Result},
  farm_profile_function, farm_profile_scope,
  hashbrown::HashSet,
  plugin::{PluginResolveHookResult, ResolveKind},
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};
use std::{
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};

use crate::resolver_cache::{ResolveCache, ResolveNodeModuleCacheKey};
use crate::resolver_common::{
  are_values_equal, find_mapping, find_request_diff_entry_path,
  get_field_value_from_package_json_info, get_key_path, get_result_path, get_string_value_path,
  is_double_source_dot, is_module_external, is_module_side_effects, is_source_absolute,
  is_source_dot, is_source_relative, try_file, NODE_MODULES,
};

pub struct Resolver {
  /// the key is (source, base_dir) and the value is the resolved result
  resolve_module_cache: ResolveCache,
}

impl Resolver {
  pub fn new() -> Self {
    Self {
      resolve_module_cache: ResolveCache::new(),
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

  /// check current env
  /// 1. browser(object)
  /// 2. exports
  /// 3. browser(string)
  /// 4. module
  /// 5. main
  /// browser is string instead main field
  pub fn resolve(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("resolver::resolve".to_string());
    // Load the package.json file located under base_dir
    let package_json_info = load_package_json(
      base_dir.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: true, // only look for current directory
      },
    );
    // TODO check browser fields
    // check if module is external
    if let Ok(package_json_info) = &package_json_info {
      farm_profile_scope!("resolve.check_external".to_string());
      let is_source_module_external = is_module_external(package_json_info, source);
      if !is_source_absolute(source) && !is_source_relative(source) && is_source_module_external {
        // this is an external module
        return Some(PluginResolveHookResult {
          resolved_path: String::from(source),
          external: true,
          ..Default::default()
        });
      }

      if !is_source_absolute(source) && !is_source_relative(source) {
        // check browser replace
        if let Some(resolved_path) = self.try_browser_replace(package_json_info, source, context) {
          let external = is_module_external(package_json_info, &resolved_path);
          let side_effects = is_module_side_effects(package_json_info, &resolved_path);
          return Some(PluginResolveHookResult {
            resolved_path,
            external,
            side_effects,
            ..Default::default()
          });
        }
        if let Some(resolved_path) =
          self.resolve_sub_path_imports(package_json_info, source, kind, context)
        {
          println!("resolved_path: {:?}", resolved_path);
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
          let external = is_module_external(package_json_info, &resolved_path);
          let side_effects = is_module_side_effects(package_json_info, &resolved_path);
          return Some(PluginResolveHookResult {
            resolved_path,
            external,
            side_effects,
            ..Default::default()
          });
        }
      }
    }
    // Execution resolve strategy
    self.resolve_strategy(source, base_dir, kind, context, package_json_info)
  }

  /// Resolve a module source code path based on the provided parameters and strategies.
  fn resolve_strategy(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
    package_json_info: Result<PackageJsonInfo>,
  ) -> Option<PluginResolveHookResult> {
    match source {
      source
        if self
          .try_alias(source, base_dir.clone(), kind, context)
          .is_some() =>
      {
        // Handle the alias case
        self.try_alias(source, base_dir.clone(), kind, context)
      }
      source if is_source_absolute(source) => {
        // Handle the absolute source case
        let path_buf = PathBuf::from_str(source).unwrap();

        return try_file(&path_buf, context)
          .or_else(|| self.try_directory(&path_buf, kind, false, context))
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      source if is_source_relative(source) => {
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
        let resolved_path = try_file(&normalized_path, context)
          .or_else(|| self.try_directory(&normalized_path, kind, false, context))
          .ok_or(CompilationError::GenericError(format!(
            "File `{:?}` does not exist",
            normalized_path
          )));

        if let Ok(resolved_path) = resolved_path {
          return Some(self.get_resolve_result(&package_json_info, resolved_path, kind, context));
        } else {
          None
        }
      }
      _source if is_source_dot(source) => {
        // import xx from '.'
        return self
          .try_directory(&base_dir, kind, false, context)
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      _source if is_double_source_dot(source) => {
        // import xx from '..'
        let parent_path = Path::new(&base_dir).parent().unwrap().to_path_buf();
        return self
          .try_directory(&parent_path, kind, false, context)
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      _ => {
        // check if the result is cached
        if let Ok(Some(result)) = self.resolve_module_cache.get(&ResolveNodeModuleCacheKey {
          source: source.to_string(),
          base_dir: base_dir.to_string_lossy().to_string(),
          kind: kind.clone(),
        }) {
          return Some(result.clone());
        }

        let (result, tried_paths) = self.try_node_modules(source, base_dir, kind, context);
        // cache the result
        for tried_path in tried_paths {
          let resolve_module_cache = &self.resolve_module_cache;
          let key = ResolveNodeModuleCacheKey {
            source: source.to_string(),
            base_dir: tried_path.to_string_lossy().to_string(),
            kind: kind.clone(),
          };

          // insert
          match resolve_module_cache.contains(&key) {
            Ok(true) => {}
            Ok(false) => {
              let _ = resolve_module_cache.insert(key, result.clone());
            }
            Err(err) => {
              println!("{}", format!("Error checking cache: {:?}", err));
            }
          }
        }
        result
      }
    }
  }

  /// Try resolve as a file with the configured main fields.
  fn try_directory(
    &self,
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

      if let Some(found) = try_file(&file, context) {
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

      if let Ok(Some(result)) = self.resolve_module_cache.get(&key) {
        return (Some(result.clone()), tried_paths);
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
            if let Some(resolved_path) = try_file(&package_path, context)
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
              if package_json_info.is_ok() {
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
          if let Some(resolved_path) = try_file(&package_path, context)
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
          if package_json_info.is_err() {
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
            try_file(&package_path.join("index"), context).map(|resolved_path| {
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
          let full_path = RelativePath::new(&str).to_logical_path(dir);
          // the main fields can be a file or directory
          return match try_file(&full_path, context) {
            Some(resolved_path) => (
              {
                Some(self.get_resolve_node_modules_result(
                  Some(package_json_info),
                  resolved_path,
                  kind,
                  context,
                ))
              },
              tried_paths,
            ),
            None => (
              self
                .try_directory(&full_path, kind, true, context)
                .map(|resolved_path| {
                  self.get_resolve_node_modules_result(
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
    package_json_info: &Result<PackageJsonInfo>,
    resolved_path: String,
    _kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    farm_profile_function!("get_resolve_result".to_string());
    if let Ok(package_json_info) = package_json_info {
      let external = is_module_external(package_json_info, &resolved_path);
      let side_effects = is_module_side_effects(package_json_info, &resolved_path);
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
    package_json_info: Option<&PackageJsonInfo>,
    resolved_path: String,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    farm_profile_function!("get_resolve_node_modules_result".to_string());
    if let Some(package_json_info) = package_json_info {
      let side_effects = is_module_side_effects(package_json_info, &resolved_path);

      let resolved_path = self
        .try_exports_replace(package_json_info, &resolved_path, kind, context)
        .unwrap_or(resolved_path);
      // fix: not exports field, eg: "@ant-design/icons-svg/es/asn/SearchOutlined"
      let resolved_path_buf = PathBuf::from(&resolved_path);
      let resolved_path = try_file(&resolved_path_buf, context)
        .or_else(|| self.try_directory(&resolved_path_buf, kind, true, context))
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
    package_json_info: &PackageJsonInfo,
    resolved_path: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_exports_replace".to_string());
    // resolve exports field
    // TODO: add all cases from https://nodejs.org/api/packages.html
    // resolve current package.json Catalogue
    let current_resolve_base_dir = package_json_info.dir();
    // find user defined imports exports field
    let resolved_field = self.resolve_request_result(&resolved_path, current_resolve_base_dir);
    // resolve current exports fields mapping
    let exports_field = get_field_value_from_package_json_info(package_json_info, "exports");
    // if has exports fields
    if let Some(exports_field) = exports_field {
      match exports_field {
        // export field is a string
        Value::String(string_value) => {
          return get_result_path(&string_value.as_str(), current_resolve_base_dir);
        }
        // export field is a object
        Value::Object(object_value) => {
          // TODO resolved_field.as_str() e.g: node deno browser development
          match find_mapping(resolved_field.as_str(), &object_value) {
            Some(value) => {
              match value {
                Value::String(string_value) => {
                  return get_result_path(&string_value.as_str(), current_resolve_base_dir);
                }
                Value::Object(object_value) => {
                  for (key_word, key_value) in object_value {
                    match kind {
                      // import with node default
                      ResolveKind::Import => {
                        if are_values_equal(key_word, "default")
                          || are_values_equal(key_word, "import")
                        {
                          return self.process_mapping_value(
                            key_value,
                            current_resolve_base_dir,
                            kind,
                            context,
                          );
                        }
                      }
                      ResolveKind::Require => {
                        if are_values_equal(key_word, "default")
                          || are_values_equal(key_word, "require")
                        {
                          return self.process_mapping_value(
                            key_value,
                            current_resolve_base_dir,
                            kind,
                            context,
                          );
                        }
                      }
                      ResolveKind::ExportFrom => {
                        if are_values_equal(key_word, "default")
                          || are_values_equal(key_word, "import")
                        {
                          return self.process_mapping_value(
                            key_value,
                            current_resolve_base_dir,
                            kind,
                            context,
                          );
                        }
                      }
                      _ => {
                        // TODO resolve other kind
                      }
                    }
                  }
                }
                _ => {}
              }
            }
            None => {
              // Handle wildcard matching here
              // TODO Whether to consider supporting e.g:. / dist/*:. / dist/*
            }
          }
        }
        _ => {}
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
    farm_profile_function!("try_browser_replace".to_string());
    if context.config.output.target_env != TargetEnv::Browser {
      return None;
    }

    let browser_field = get_field_value_from_package_json_info(package_json_info, "browser");
    if let Some(Value::Object(obj)) = browser_field {
      for (key, value) in obj {
        let path = Path::new(resolved_path);
        // resolved path
        if path.is_absolute() {
          let key_path = get_key_path(&key, package_json_info.dir());
          if are_values_equal(key_path, resolved_path) {
            if let Value::String(str) = value {
              let value_path = get_key_path(&str, package_json_info.dir());
              return Some(value_path);
            }
          }
        } else {
          // TODO: this is not correct, it should remap the package name
          // source, e.g. 'foo' in require('foo')
          if are_values_equal(&key, resolved_path) {
            if let Value::String(str) = value {
              let value_path = get_key_path(&str, package_json_info.dir());
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
      let imports_field = get_field_value_from_package_json_info(package_json_info, "imports");
      if let Some(Value::Object(imports_field_map)) = imports_field {
        for (key, value) in imports_field_map {
          if are_values_equal(&key, resolved_path) {
            if let Value::String(str) = &value {
              return get_string_value_path(str, package_json_info);
            }

            if let Value::Object(str) = &value {
              for (key, value) in str {
                match context.config.output.target_env {
                  TargetEnv::Browser => {
                    if are_values_equal(key, "default") {
                      if let Value::String(str) = value {
                        return get_string_value_path(str, package_json_info);
                      }
                    }
                  }
                  TargetEnv::Node => {
                    if are_values_equal(key, "node") {
                      if let Value::String(str) = value {
                        return get_string_value_path(str, package_json_info);
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    None
  }

  fn process_target_env_logic(
    self: &Self,
    kind: ResolveKind,
    key: &str,
    value: &Value,
    current_resolve_base_dir: &String,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    match kind {
      ResolveKind::Import | ResolveKind::Require => {
        let value_str = if let Value::String(s) = value {
          s
        } else {
          return None;
        };

        match context.config.output.target_env {
          TargetEnv::Node if are_values_equal(key, "node") => {
            Some(get_key_path(value_str, current_resolve_base_dir))
          }
          TargetEnv::Browser
            if are_values_equal(key, "default") || are_values_equal(key, "browser") =>
          {
            Some(get_key_path(value_str, current_resolve_base_dir))
          }
          _ => None,
        }
      }
      // TODO resolve other ResolveKind logic
      _ => None,
    }
  }

  fn resolve_request_result(
    self: &Self,
    resolved_path: &str,
    current_resolve_base_dir: &String,
  ) -> String {
    match find_request_diff_entry_path(&resolved_path, current_resolve_base_dir) {
      Some(diff) => {
        if diff.origin_request.is_empty() {
          ".".to_string()
        } else {
          format!("./{}", diff.origin_request)
        }
      }
      None => "".to_string(),
    }
  }

  fn process_mapping_value(
    self: &Self,
    key_value: &Value,
    current_resolve_base_dir: &String,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    match key_value {
      Value::String(string_value) => {
        return Some(get_result_path(
          string_value.as_str(),
          current_resolve_base_dir,
        )?);
      }
      Value::Object(import_value) => {
        for (key_word, key_value) in import_value {
          if let Some(result) = self.process_target_env_logic(
            kind.clone(),
            key_word,
            key_value,
            current_resolve_base_dir,
            context,
          ) {
            return Some(result);
          }
        }
      }
      _ => {}
    }
    None
  }

  fn resolve_sub_path_imports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("try_imports_replace".to_string());
    if source.starts_with('#') {
      if let Some(resolved_path) =
        self.resolve_exports_or_imports(package_json_info, source, "imports", kind, context)
      {
        println!("resolved_path: {:?}", resolved_path);
      }
      let imports_field = get_field_value_from_package_json_info(package_json_info, "imports");
      println!("imports_field: {:?}", imports_field);
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
  ) -> Option<String> {
    farm_profile_function!("resolve_exports_or_imports".to_string());
    let additional_conditions: HashSet<String> = context
      .config
      .resolve
      .conditions
      .clone()
      .into_iter()
      .collect();

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
    let condition_config = ConditionConfig {
      browser: is_browser && !additional_conditions.contains("node"),
      require: is_require && !additional_conditions.contains("import"),
      conditions: filtered_conditions,
    };

    let result = if field_type == "imports" {
      self.imports(package_json_info, source, &condition_config)
    } else {
      self.exports(package_json_info, source, &condition_config)
    };
    return result;
  }

  fn imports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    config: &ConditionConfig,
  ) -> Option<String> {
    // 在这里实现imports方法的逻辑
    Some("TODO".to_string())
  }

  fn exports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    config: &ConditionConfig,
  ) -> Option<String> {
    // 在这里实现exports方法的逻辑
    Some("TODO".to_string())
  }
}

struct ConditionConfig {
  browser: bool,
  require: bool,
  conditions: Vec<String>,
}
