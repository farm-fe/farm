use farmfe_core::{
  common::PackageJsonInfo,
  config::{Mode, TargetEnv},
  context::CompilationContext,
  error::{CompilationError, Result},
  farm_profile_function, farm_profile_scope,
  hashbrown::{HashMap, HashSet},
  plugin::{PluginResolveHookResult, ResolveKind},
  regex,
  relative_path::RelativePath,
  serde_json::{from_str, Map, Value},
};
use std::{
  fs,
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use farmfe_toolkit::resolve::{follow_symlinks, load_package_json, package_json_loader::Options};

use crate::resolver_common::{
  are_values_equal, get_directory_path, get_field_value_from_package_json_info, get_key_path,
  get_real_path, get_result_path, is_bare_import_path, is_double_source_dot, is_in_node_modules,
  is_module_external, is_module_side_effects, is_source_absolute, is_source_dot,
  is_source_relative, map_with_browser_field, split_file_and_postfix, try_file, walk,
  ConditionOptions, NODE_MODULES,
};
use crate::{
  resolver_cache::{ResolveCache, ResolveNodeModuleCacheKey},
  resolver_common::DEEP_IMPORT_RE,
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

  // TODO builtIns NODE BUN DENO
  // TODO  /@fs/xxx
  // TODO /foo try_to /@fs/root/foo
  // TODO Data URL skip resolve
  // TODO fs paths need resolve diff os “Windows”

  // TODO 整合 options 全局变量把  context 都放在一起
  pub fn resolve(
    &self,
    id: &str,
    importer: PathBuf,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    farm_profile_function!("resolver::resolve".to_string());
    let mut source: String = id.to_string();
    // Load the package.json file located under base_dir
    let package_json_info = load_package_json(
      importer.clone(),
      Options {
        follow_symlinks: context.config.resolve.symlinks,
        resolve_ancestor_dir: true, // only look for current directory
      },
    );
    if let Ok(package_json_info) = &package_json_info {
      // check source start_with "#" prepare resolve imports fields
      let resolved_imports_path =
        self.resolve_sub_path_imports(package_json_info, id, kind, context);
      if let Some(resolved_imports_path) = resolved_imports_path {
        // TODO need base dir to check the relative path
        // Combine pkg_dir and imports_path
        source = resolved_imports_path;
      }

      // check if module is external
      let is_source_module_external = is_module_external(package_json_info, source.as_str());
      if !is_source_absolute(source.as_str())
        && !is_source_relative(source.as_str())
        && is_source_module_external
      {
        // this is an external module
        farm_profile_scope!("resolve.check_external".to_string());
        return Some(PluginResolveHookResult {
          resolved_path: String::from(source),
          external: true,
          ..Default::default()
        });
      }

      // if !is_source_absolute(source.as_str()) && !is_source_relative(source.as_str()) {
      //   // check browser replace
      //   if let Some(resolved_path) = self.try_browser_replace(package_json_info, source.as_str(), context) {
      //     let external = is_module_external(package_json_info, &resolved_path);
      //     let side_effects = is_module_side_effects(package_json_info, &resolved_path);
      //     return Some(PluginResolveHookResult {
      //       resolved_path,
      //       external,
      //       side_effects,
      //       ..Default::default()
      //     });
      //   }
      // }
    }
    // Execution resolve strategy
    self.resolve_strategy(source.as_str(), importer, kind, context, package_json_info)
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
          .or_else(|| self.try_directory(&path_buf, source, kind, false, context))
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      source if is_source_relative(source) => {
        farm_profile_scope!("resolve.relative".to_string());
        // if it starts with './' or '../, it is a relative path
        let normalized_path = RelativePath::new(source).to_logical_path(&base_dir);
        let normalized_path = normalized_path.as_path();
        let normalized_path = if context.config.resolve.symlinks {
          follow_symlinks(normalized_path.to_path_buf())
        } else {
          normalized_path.to_path_buf()
        };

        // TODO try read symlink from the resolved path step by step to its parent util the root
        // if let Some(normalized_path) =
        //   self.try_fs_resolve(normalized_path.to_string_lossy().to_string(), true, context)
        // {
        //   return Some(self.get_resolve_result(&package_json_info, normalized_path, kind, context));
        // }
        // None

        let resolved_path = try_file(&normalized_path, context)
          .or_else(|| self.try_directory(&normalized_path, source, kind, false, context))
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
          .try_directory(&base_dir, source, kind, false, context)
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      _source if is_double_source_dot(source) => {
        // import xx from '..'
        let parent_path = Path::new(&base_dir).parent().unwrap().to_path_buf();
        return self
          .try_directory(&parent_path, source, kind, false, context)
          .map(|resolved_path| {
            self.get_resolve_result(&package_json_info, resolved_path, kind, context)
          });
      }
      _source if is_bare_import_path(source) => {
        // check if the result is cached
        if let Ok(Some(result)) = self.resolve_module_cache.get(&ResolveNodeModuleCacheKey {
          source: source.to_string(),
          base_dir: base_dir.to_string_lossy().to_string(),
          kind: kind.clone(),
        }) {
          return Some(result.clone());
        }

        let (result, tried_paths) = self.try_node_resolve(source, base_dir, kind, context);
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
              eprintln!("{}", format!("Error checking cache: {:?}", err));
            }
          }
        }
        result
      }
      _ => {
        eprintln!("Unsupported source type: {}", source);
        None
      }
    }
  }

  /// Try resolve as a file with the configured main fields.
  fn try_directory(
    &self,
    dir: &Path,
    source: &str,
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
        let mut deep_match = DEEP_IMPORT_RE.is_match(source);
        if !is_bare_import_path(source) && is_source_absolute(source) {
          deep_match = false;
        }

        if is_source_absolute(dir.to_str().unwrap()) {
          if let Some(resolved_path) =
            self.find_entry_package_point(&package_json_info, kind, context)
          {
            let resolved_path = get_result_path(&resolved_path, package_json_info.dir());
            return resolved_path;
          }
        }
        let (res, _) = self.try_package(
          "",
          deep_match,
          &package_json_info,
          source,
          kind,
          vec![],
          context,
        );

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
  fn try_node_resolve(
    &self,
    source: &str,
    base_dir: PathBuf,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    farm_profile_function!("try_node_resolve".to_string());
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
      // check deepImport source
      let deep_match = DEEP_IMPORT_RE.is_match(source);
      let mut package_id = source;
      let captures = DEEP_IMPORT_RE.captures(source);
      package_id = match captures {
        Some(captures) => captures
          .get(1)
          .map_or_else(|| captures.get(2).unwrap().as_str(), |m| m.as_str()),
        None => source,
      };
      if maybe_node_modules_path.exists() && maybe_node_modules_path.is_dir() {
        let package_path = if context.config.resolve.symlinks {
          follow_symlinks(RelativePath::new(package_id).to_logical_path(&maybe_node_modules_path))
        } else {
          RelativePath::new(package_id).to_logical_path(&maybe_node_modules_path)
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
              .or_else(|| self.try_directory(&package_path, source, kind, true, context))
            {
              return (
                Some(self.get_resolve_node_modules_result(
                  package_json_info.ok().as_ref(),
                  source,
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
                    source,
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
            .or_else(|| self.try_directory(&package_path, source, kind, true, context))
          {
            return (
              Some(self.get_resolve_node_modules_result(
                package_json_info.ok().as_ref(),
                source,
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
          let (result, tried_paths) = self.try_package(
            package_id,
            deep_match,
            &package_json_info,
            source,
            kind,
            tried_paths,
            context,
          );

          if result.is_some() {
            return (result, tried_paths);
          }

          // no main field found, try to resolve index.js file
          return (
            try_file(&package_path.join("index"), context).map(|resolved_path| {
              self.get_resolve_node_modules_result(
                Some(&package_json_info),
                source,
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
    package_id: &str,
    deep_match: bool,
    package_json_info: &PackageJsonInfo,
    source: &str,
    kind: &ResolveKind,
    tried_paths: Vec<PathBuf>,
    context: &Arc<CompilationContext>,
  ) -> (Option<PluginResolveHookResult>, Vec<PathBuf>) {
    farm_profile_function!("try_package".to_string());
    // exports should take precedence over module/main according to node docs (https://nodejs.org/api/packages.html#package-entry-points)
    // search normal entry, based on self.config.main_fields, e.g. module/main
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();
    let resolve_id = self.unresolved_id(deep_match, source, package_id);
    if let Some(resolved_path) = self.resolve_id_logic(
      deep_match,
      resolve_id,
      package_json_info,
      kind,
      context,
      source,
    ) {
      if source.contains("is-string") {
        println!("source: {}", source);
        println!("resolved_path: {}", resolved_path);
      }
      let resolved_path = if is_source_absolute(&resolved_path) {
        resolved_path
      } else {
        get_result_path(&resolved_path, package_json_info.dir()).unwrap()
      };
      return (
        Some(PluginResolveHookResult {
          resolved_path,
          // external,
          // side_effects,
          ..Default::default()
        }),
        tried_paths,
      );
    }
    // for main_field in &context.config.resolve.main_fields {
    //   if main_field == "browser" && context.config.output.target_env == TargetEnv::Node {
    //     continue;
    //   }

    //   if let Some(field_value) = raw_package_json_info.get(main_field) {
    //     if let Value::Object(_) = field_value {
    //       let resolved_path = Some(self.get_resolve_node_modules_result(
    //         Some(package_json_info),
    //         source,
    //         package_json_info.dir().to_string(),
    //         kind,
    //         context,
    //       ));
    //       let result = resolved_path.as_ref().unwrap();
    //       let path = Path::new(result.resolved_path.as_str());
    //       if let Some(_extension) = path.extension() {
    //         return (resolved_path, tried_paths);
    //       }
    //     } else if let Value::String(str) = field_value {
    //       let dir = package_json_info.dir();
    //       let full_path = RelativePath::new(&str).to_logical_path(dir);
    //       // the main fields can be a file or directory
    //       return match try_file(&full_path, context) {
    //         Some(resolved_path) => (
    //           {
    //             Some(self.get_resolve_node_modules_result(
    //               Some(package_json_info),
    //               source,
    //               resolved_path,
    //               kind,
    //               context,
    //             ))
    //           },
    //           tried_paths,
    //         ),
    //         None => (
    //           self
    //             .try_directory(&full_path, source, kind, true, context)
    //             .map(|resolved_path| {
    //               self.get_resolve_node_modules_result(
    //                 Some(package_json_info),
    //                 source,
    //                 resolved_path,
    //                 kind,
    //                 context,
    //               )
    //             }),
    //           tried_paths,
    //         ),
    //       };
    //     }
    //   }
    // }

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
    source: &str,
    resolved_path: String,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> PluginResolveHookResult {
    farm_profile_function!("get_resolve_node_modules_result".to_string());
    if let Some(package_json_info) = package_json_info {
      let side_effects = is_module_side_effects(package_json_info, &resolved_path);
      let resolve_exports_path = self
        .resolve_exports_or_imports(package_json_info, source, "exports", kind, context)
        .unwrap_or(vec![resolved_path.clone()]);
      let current_resolve_base_dir = package_json_info.dir();
      let resolved_path_buf: PathBuf;
      let resolved_id = resolve_exports_path.get(0).unwrap();
      if is_source_absolute(&resolved_id) {
        resolved_path_buf = PathBuf::from(resolved_id);
      } else {
        let resolved_full_id = get_result_path(&resolved_id, current_resolve_base_dir);
        resolved_path_buf = PathBuf::from(resolved_full_id.clone().unwrap());
      }
      let resolved_path = try_file(&resolved_path_buf, context)
        .or_else(|| self.try_directory(&resolved_path_buf, source, kind, true, context))
        .unwrap_or(resolved_path);
      return PluginResolveHookResult {
        resolved_path,
        side_effects,
        ..Default::default()
      };
    } else {
      PluginResolveHookResult {
        resolved_path,
        ..Default::default()
      }
    }
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

  fn resolve_sub_path_imports(
    &self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    farm_profile_function!("resolve_sub_path_imports".to_string());
    if source.starts_with('#') {
      if let Some(resolved_path) =
        self.resolve_exports_or_imports(package_json_info, source, "imports", kind, context)
      {
        if let Some(first_item) = resolved_path.into_iter().next() {
          return Some(first_item);
        }
      }
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
    let result: Option<Vec<String>> = if field_type == "imports" {
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
    config: &ConditionOptions,
  ) -> Option<Vec<String>> {
    if let Some(imports_field) =
      get_field_value_from_package_json_info(package_json_info, "imports")
    {
      // TODO If the current package does not have a name, then look up for the name of the folder
      let name = match get_field_value_from_package_json_info(package_json_info, "name") {
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
      return Some(walk(name.as_str().unwrap(), &imports_map, source, config));
    }
    None
  }

  fn exports(
    self: &Self,
    package_json_info: &PackageJsonInfo,
    source: &str,
    config: &ConditionOptions,
  ) -> Option<Vec<String>> {
    if let Some(exports_field) =
      get_field_value_from_package_json_info(package_json_info, "exports")
    {
      // TODO If the current package does not have a name, then look up for the name of the folder
      let name = match get_field_value_from_package_json_info(package_json_info, "name") {
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
        return Some(walk(name.as_str().unwrap(), &map, source, config));
      }
    }

    None
  }

  fn find_entry_package_point(
    self: &Self,
    package_json_info: &PackageJsonInfo,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    let mut entry_point: Option<String> = None;
    let raw_package_json_info: Map<String, Value> = from_str(package_json_info.raw()).unwrap();
    if let Some(exports) = raw_package_json_info.get("exports") {
      if let Some(point) =
        self.resolve_exports_or_imports(package_json_info, ".", "exports", kind, context)
      {
        if !point.is_empty() {
          entry_point = Some(point[0].clone());
        }
      }
    }
    let resolved_from_exports = entry_point.is_some();
    let is_browser = TargetEnv::Browser == context.config.output.target_env;
    let is_require = matches!(kind, ResolveKind::Require);
    if is_browser && entry_point.is_none() {
      let mut browser_entry: Option<String> = None;

      if let Some(browser_value) = raw_package_json_info.get("browser") {
        if let Some(browser_string) = browser_value.as_str() {
          // If browser_value is a string, assign it to browser_entry
          browser_entry = Some(browser_string.to_string());
        } else if let Some(browser_object) = browser_value.as_object() {
          if let Some(dot_value) = browser_object.get(".") {
            if let Some(dot_string) = dot_value.as_str() {
              // If "." is present and its value is a string, assign it to browser_entry
              browser_entry = Some(dot_string.to_string());
            }
          }
        }
      }

      let module_fields = raw_package_json_info.get("module");
      if let Some(browser_entry) = browser_entry {
        if !is_require
          && context
            .config
            .resolve
            .main_fields
            .contains(&"module".to_string())
          && module_fields.is_some()
        {
          // println!("进来了 开始解析了 browserEntry {:?}", browser_entry);
        } else {
          entry_point = Some(browser_entry);
        }
      }
    }
    if !resolved_from_exports && entry_point.is_none() {
      // If browser_entry is not present, try to resolve the main field
      for field in &context.config.resolve.main_fields {
        if field == "browser" {
          // 已在上面检查过，跳过
          continue;
        }

        if let Some(field_value) = raw_package_json_info.get(field).and_then(|v| v.as_str()) {
          // 如果 data[field] 是字符串类型，将其赋值给 entryPoint，并退出循环
          entry_point = Some(field_value.to_string());
          break;
        }
      }
    }

    entry_point = match entry_point {
      Some(ep) => Some(ep), // 如果 entry_point 有值，直接使用它
      None => raw_package_json_info
        .get("main")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()), // 如果 entry_point 为 None，使用 data.main
    };

    let entry_points: Vec<String> = if let Some(entry_point) = entry_point.clone() {
      vec![entry_point]
    } else {
      vec![String::from("index.js"), String::from("index.json")]
    };
    for mut entry in entry_points {
      if let Some(browser_data) = raw_package_json_info.get("browser") {
        if is_browser && browser_data.is_object() {
          entry = map_with_browser_field(&entry, browser_data)
            .map(|s| s.to_string())
            .unwrap_or_else(|| entry.to_string());
        }
      }
      let entry_point_path = Path::new(package_json_info.dir()).join(&entry);
      if entry_point_path.exists() {
        // entry_point = Some(entry);
        break;
      }
    }

    match entry_point {
      Some(entry) => {
        let dir = PathBuf::from(package_json_info.dir());
        let normalized_key = get_key_path(&entry, package_json_info.dir());
        if PathBuf::from(&entry).extension().is_none() {
          return try_file(&dir.join(Path::new(&normalized_key)), context).or_else(|| {
            self.try_directory(
              &dir.join(Path::new(&normalized_key)),
              // TODO  NO SOURCE
              "",
              kind,
              false,
              context,
            )
          });
          // return try_file(&dir.join(Path::new(&normalized_key)), context);
        }
        return Some(entry);
      }
      None => return None,
    }
  }

  fn resolve_deep_import(
    self: &Self,
    resolve_id: String,
    package_json_info: &PackageJsonInfo,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
    source: &str,
  ) -> Option<String> {
    if source.contains("is-string") {
      println!("source: {}", source);
      println!("resolve_id: {}", resolve_id);
    }
    let mut relative_id = Some(resolve_id.clone());
    let exports_data = get_field_value_from_package_json_info(package_json_info, "exports");
    let browser_data = get_field_value_from_package_json_info(package_json_info, "browser");
    let module_data = get_field_value_from_package_json_info(package_json_info, "module");
    let is_browser = TargetEnv::Browser == context.config.output.target_env;
    if let Some(export_data) = exports_data {
      if export_data.is_object() && !export_data.is_array() {
        if let Some(resolve_id) =
          self.resolve_exports_or_imports(package_json_info, &resolve_id, "exports", kind, context)
        {
          if let Some(value) = resolve_id.get(0) {
            relative_id = Some(value.to_string());
          } else {
            relative_id = None;
          }
        } else {
          relative_id = None;
        }
        if relative_id.is_none() {
          let error_message = format!(
            "Package subpath '{:?}' is not defined by \"exports\" in {}.",
            relative_id,
            package_json_info.dir()
          );

          eprintln!("{}", error_message);
        }
      }
    } else if is_browser && browser_data.is_some() && browser_data.clone().unwrap().is_object() {
      let mapped = map_with_browser_field(
        relative_id.clone().unwrap().as_str(),
        &browser_data.unwrap(),
      );
      if mapped.is_some() {
        relative_id = mapped;
      }
    }
    let dir = PathBuf::from(package_json_info.dir());
    // if relative_id.clone().unwrap().len() < 2 {
    //   let info: Value = from_str(package_json_info.raw()).unwrap();
    //   relative_id = Some(info.get("module").unwrap().to_string())
    // }
    if relative_id.clone().unwrap() == "." && module_data.is_some() {
      relative_id = module_data.map(|s| s.to_string());
    }
    if source.contains("is-string") {
      println!("relative_id: {:?}", relative_id);
      println!("dir: {:?}", dir);
      println!("package_json_info: {:#?}", package_json_info);
    }
    let dir_path = get_result_path(&relative_id.unwrap(), &dir.to_str().unwrap().to_string());
    if let Some(resolved_path) = self.resolve_fs(
      &PathBuf::from(dir_path.clone().unwrap()),
      is_browser,
      kind,
      context,
    ) {
      return Some(resolved_path);
    } else {
      if let Some(dir_path) = self.find_existing_directory(&PathBuf::from(dir_path.unwrap())) {
        if let Some(resolved_path) =
          self.resolve_fs(&PathBuf::from(&dir_path), is_browser, kind, context)
        {
          // if source.contains("is-string") {
          //   println!("&PathBuf::from(dir_path): {:?}", &PathBuf::from(dir_path));
          //   println!("resolved_path 213: {}", resolved_path);
          // }
          return Some(resolved_path);
        }
      }
    }
    // if let Some(relative_id) = &relative_id {
    //   if PathBuf::from(relative_id).extension().is_none() {
    //     return try_file(&dir.join(&relative_id), context);
    //   }
    //   return Some(relative_id.to_string());
    // }

    None
  }

  fn resolve_fs(
    self: &Self,
    path: &PathBuf,
    is_browser: bool,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    let hash_index = path.to_str().unwrap().find('#');
    if let Some(hash_index) = hash_index {
      if is_in_node_modules(&path.to_str().unwrap()) {
        let query_index = path.to_str().unwrap().find('?');

        if let Some(query_index) = query_index {
          if query_index < hash_index {
            let file = &path.to_str().unwrap()[..query_index];
            // TODO  Dependencies like es5-ext use `#` in their paths.
          }
        }
      }
    }
    if PathBuf::from(&path).is_file() && PathBuf::from(&path).exists() {
      return Some(path.to_str().unwrap().to_string());
    }
    return try_file(&path, context)
      .or_else(|| self.try_directory(&path, "", &kind, false, context))
      .map(|resolved_path| {
        return resolved_path;
      });
    // get_result_path(&resolved_id, current_resolve_base_dir);
  }

  fn resolve_id_logic(
    self: &Self,
    deep_match: bool,
    resolve_id: String,
    package_json_info: &PackageJsonInfo,
    kind: &ResolveKind,
    context: &Arc<CompilationContext>,
    source: &str,
  ) -> Option<String> {
    // if deep_match && is_source_absolute(&resolve_id) {
    if deep_match {
      return self.resolve_deep_import(resolve_id, package_json_info, kind, context, source);
    } else {
      return self.find_entry_package_point(package_json_info, kind, context);
    }
  }

  fn unresolved_id(self: &Self, deep_match: bool, id: &str, pkg_id: &str) -> String {
    if deep_match {
      format!(".{}", &id[pkg_id.len()..])
    } else {
      id.to_string()
    }
  }

  fn try_browser_mapping(
    source: &str,
    importer_dir: PathBuf,
    kind: ResolveKind,
    context: Arc<CompilationContext>,
  ) -> Option<PluginResolveHookResult> {
    None
  }

  fn find_existing_directory(self: &Self, path: &Path) -> Option<PathBuf> {
    if path.exists() && path.is_dir() {
      Some(path.to_path_buf())
    } else {
      let parent = path.parent()?;
      self.find_existing_directory(parent)
    }
  }

  fn try_fs_resolve(
    self: &Self,
    base_fs_path: String,
    skip_try_package: bool,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    let (file, postfix) = split_file_and_postfix(&base_fs_path);
    if let Some(resolved_path) = self.try_clean_fs_resolve(&file, context) {
      return Some(resolved_path);
    }
    None
  }

  fn try_clean_fs_resolve(
    self: &Self,
    path: &String,
    // is_browser: bool,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    // let resolved_path = try_file(&normalized_path, context)
    //   .or_else(|| self.try_directory(&normalized_path, source, kind, false, context))
    //   .ok_or(CompilationError::GenericError(format!(
    //     "File `{:?}` does not exist",
    //     normalized_path
    //   )));
    if let Ok(metadata) = fs::metadata(path) {
      if metadata.is_file() {
        let real_path = get_real_path(path, context.config.resolve.symlinks);
        return Some(real_path);
      }
    }
    let dir_path = get_directory_path(path);

    if let Some(resolved_path) =
      self.try_resolve_real_file_with_extensions(PathBuf::from(path), context)
    {
      return Some(resolved_path);
    }
    None
  }

  fn try_resolve_real_file_with_extensions(
    self: &Self,
    file: PathBuf,
    context: &Arc<CompilationContext>,
  ) -> Option<String> {
    let extensions = context.config.resolve.extensions.clone();
    let resolved_path: Option<String> = None;
    for extension in extensions {
      let mut file_path = file.clone();
      file_path.set_extension(extension);
      // self.try_resolve_real_file(file_path, context.config.resolve.symlinks);
      if let Some(resolved_path) = self.try_resolve_real_file(
        &file_path.to_str().unwrap(),
        context.config.resolve.symlinks,
      ) {
        return Some(resolved_path);
      }
    }
    resolved_path
  }
  fn try_resolve_real_file(self: &Self, file: &str, preserve_symlinks: bool) -> Option<String> {
    if let Ok(metadata) = fs::metadata(file) {
      if metadata.is_file() {
        return Some(get_real_path(file, preserve_symlinks));
      }
    }
    None
  }
}
