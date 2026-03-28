#![deny(clippy::all)]
mod cache;
use std::{path::Path, sync::Arc};

use base64::{engine::general_purpose, Engine};
use cache::WorkerCache;
use farmfe_compiler::Compiler;
use farmfe_core::{
  cache_item,
  config::{
    persistent_cache::PersistentCacheConfig, Config, ModuleFormat, ModuleFormatConfig,
    OutputConfig, TargetEnv,
  },
  context::{CompilationContext, EmitFileParams},
  deserialize,
  module::{ModuleId, ModuleType},
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookResult, PluginResolveHookParam,
    PluginTransformHookResult, ResolveKind,
  },
  resource::{Resource, ResourceOrigin, ResourceType},
  serde,
  serde_json::{self, Map, Value},
  serialize, Cacheable,
};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::fs::{transform_output_filename, TransformOutputFileNameParams};
use farmfe_utils::relative;
use regress::{Match, Regex as JsRegex};
use rustc_hash::FxHashMap;

const WORKER_OR_SHARED_WORKER_RE: &str = r#"(?:\?|&)(worker|sharedworker)(?:&|$)"#;
const WORKER_IMPORT_META_URL_RE: &str = r#"\bnew\s+(?:Worker|SharedWorker)\s*\(\s*(new\s+URL\s*\(\s*('[^']+'|"[^"]+"|`[^`]+`)\s*,\s*import\.meta\.url[^)]*\))"#;

// fn merge_json(a: &mut Value, b: Value, exclude: &HashSet<&str>) {
//   match (a, b) {
//     (Value::Object(ref mut a_map), Value::Object(b_map)) => {
//       for (k, v) in b_map {
//         if !exclude.contains(k.as_str()) {
//           merge_json(a_map.entry(k).or_insert(Value::Null), v, exclude);
//         }
//       }
//     }
//     (Value::Array(ref mut a_arr), Value::Array(b_arr)) => {
//       a_arr.extend(b_arr);
//     }
//     (a, b) => {
//       *a = b;
//     }
//   }
// }
// fn merge_configs(
//   config1: Config,
//   config2: Value,
//   exclude: &HashSet<&str>,
// ) -> Result<Config, serde_json::Error> {
//   let mut val1 = to_value(config1)?;

//   merge_json(&mut val1, config2, exclude);

//   serde_json::from_value(val1)
// }

fn build_worker(
  resolved_path: &str,
  module_id: &str,
  compiler_config: &Config,
  host_config: &Config,
) -> Vec<u8> {
  let (_worker_url, full_file_name) = get_worker_url(resolved_path, module_id, compiler_config);
  let mut input = FxHashMap::default();
  input.insert(full_file_name.clone(), resolved_path.to_string());
  let compiler = Compiler::new(
    Config {
      input,
      persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
      output: Box::new(OutputConfig {
        target_env: TargetEnv::Library,
        entry_filename: full_file_name.clone(),
        ..*compiler_config.output.clone()
      }),
      lazy_compilation: false,
      runtime: host_config.runtime.clone(),
      ..compiler_config.clone()
    },
    vec![],
  )
  .unwrap();
  compiler.compile().unwrap();
  let resources_map = compiler.context().resources_map.lock();
  let resource = resources_map.get(&full_file_name).unwrap();
  let content_bytes = resource.bytes.clone();
  content_bytes
}

fn emit_worker_file(
  module_id: &str,
  file_name: &str,
  content_bytes: Vec<u8>,
  context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
) {
  let params = EmitFileParams {
    resolved_path: module_id.to_string(),
    content: content_bytes,
    name: file_name.to_string(),
    resource_type: ResourceType::Js,
  };
  context.emit_file(params);
}

fn get_worker_url(
  resolved_path: &str,
  module_id: &str,
  compiler_config: &Config,
) -> (String, String) {
  let file_name_ext = Path::new(resolved_path)
    .file_name()
    .map(|x| x.to_string_lossy().to_string())
    .unwrap_or_else(|| "".to_string());
  let (file_name, ext) = file_name_ext.split_once(".").unwrap();
  let assets_filename_config = compiler_config.output.assets_filename.clone();
  let transform_output_file_name_params = TransformOutputFileNameParams {
    filename_config: assets_filename_config,
    name: file_name,
    name_hash: "",
    bytes: &module_id.as_bytes(),
    ext,
    special_placeholders: &Default::default(),
  };
  // hash_bytes = resolved_path + file_name_ext bytes ,make sure that the files of the same name in different directory will not be covered;
  let file_name = transform_output_filename(transform_output_file_name_params);
  // worker.ts -> worker.js
  let file_name = if file_name.ends_with(".ts") {
    file_name.replace(".ts", ".js")
  } else {
    file_name
  };
  let worker_url = if !compiler_config.output.public_path.is_empty() {
    let normalized_public_path = compiler_config.output.public_path.trim_end_matches("/");
    format!("{}/{}", normalized_public_path, file_name)
  } else {
    format!("/{}", file_name)
  };
  (worker_url, file_name)
}
struct ProcessWorkerParam<'a> {
  resolved_path: &'a str,
  module_id: &'a str,
  is_build: bool,
  is_inline: bool,
  compiler_config: &'a Config,
  host_config: &'a Config,
  worker_cache: &'a WorkerCache,
  is_url: bool,
  context: &'a std::sync::Arc<farmfe_core::context::CompilationContext>,
}

fn process_worker(param: ProcessWorkerParam) -> String {
  let ProcessWorkerParam {
    module_id,
    is_build,
    compiler_config,
    worker_cache,
    resolved_path,
    is_url,
    is_inline,
    context,
    host_config,
  } = param;

  let (worker_url, file_name) = get_worker_url(resolved_path, module_id, compiler_config);
  let content_bytes = build_worker(resolved_path, module_id, &compiler_config, &host_config);

  if worker_cache.get(&file_name).is_none() {
    let content_bytes = insert_worker_cache(&worker_cache, file_name.to_string(), content_bytes);
    emit_worker_file(module_id, &file_name, content_bytes, context);
  } else {
    let catch_content_bytes = worker_cache.get(&file_name).unwrap();
    if content_bytes != catch_content_bytes {
      let content_bytes = insert_worker_cache(&worker_cache, file_name.to_string(), content_bytes);
      emit_worker_file(module_id, &file_name, content_bytes, context);
    }
  }

  let worker_match = JsRegex::new(WORKER_OR_SHARED_WORKER_RE)
    .unwrap()
    .find(&param.module_id);
  let worker_constructor = &module_id[worker_match.unwrap().group(1).unwrap()];

  let worker_constructor = match worker_constructor {
    "sharedworker" => "SharedWorker",
    _ => "Worker",
  };

  let worker_type = if is_build {
    "module"
  } else {
    match &compiler_config.output.format {
      ModuleFormatConfig::Single(ModuleFormat::EsModule) => "module",
      _ => "classic",
    }
  };

  let worker_type_option = match worker_type {
    "module" => "{type: 'module', name: options?.name}",
    _ => "{name: options?.name}",
  };
  if is_build {
    if is_inline {
      let content_bytes = worker_cache.get(resolved_path).unwrap();
      let content_base64 = general_purpose::STANDARD.encode(content_bytes);
      let content_base64_code = format!(r#"const encodedJs = "{}";"#, content_base64);
      let code = if worker_constructor == "Worker" {
        let blob_url = if worker_type == "classic" {
          String::from("")
        } else {
          String::from("'URL.revokeObjectURL(import.meta.url);',")
        };

        format!(
          r#"{0}
            const decodeBase64 = (base64) => Uint8Array.from(atob(base64), c => c.charCodeAt(0));
            const blob = typeof self !== "undefined" && self.Blob && new Blob([{1}decodeBase64(encodedJs)], {{ type: "text/javascript;charset=utf-8" }});
            export default function WorkerWrapper(options) {{
              let objURL;
              try {{
                objURL = blob && (self.URL || self.webkitURL).createObjectURL(blob);
                if (!objURL) throw ''
                const worker = new {2}(objURL, {3});
                worker.addEventListener("error", () => {{
                  (self.URL || self.webkitURL).revokeObjectURL(objURL);
                }});
                return worker;
              }} catch(e) {{
                return new {2}(
                  "data:text/javascript;base64," + encodedJs,
                  {3}
                );
              }}{4}
            }}"#,
          content_base64_code,
          blob_url,
          worker_constructor,
          worker_type_option,
          if worker_type == "classic" {
            String::from(
              r#" finally {
                      objURL && (self.URL || self.webkitURL).revokeObjectURL(objURL);
                    }"#,
            )
          } else {
            String::from("")
          }
        )
      } else {
        format!(
          r#"{0}
            export default function WorkerWrapper(options) {{
              return new {1}(
                "data:text/javascript;base64," + encodedJs,
                {2}
              );
            }}"#,
          content_base64_code, worker_constructor, worker_type_option
        )
      };
      return code;
    }
  }
  if is_url {
    return format!(r#"export default "{}""#, worker_url);
  }

  return format!(
    r#"
      export default function WorkerWrapper(options) {{
        return new {0}(
          "{1}",
          {2}
        );
      }}"#,
    worker_constructor, worker_url, worker_type_option
  );
}

fn insert_worker_cache(worker_cache: &WorkerCache, key: String, content_bytes: Vec<u8>) -> Vec<u8> {
  worker_cache.insert(key.clone(), content_bytes);
  worker_cache.get(&key).unwrap()
}

#[cache_item]
struct CachedStaticAssets {
  list: Vec<Resource>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Options {
  is_build: Option<bool>,
  compiler_config: Option<Config>,
  host_config: Option<Config>,
}

#[farm_plugin]
pub struct FarmfePluginWorker {
  options: Options,
  worker_cache: cache::WorkerCache,
}

impl FarmfePluginWorker {
  fn new(config: &Config, options: String) -> Self {
    let options: Value = serde_json::from_str(&options).unwrap_or(Value::Object(Map::new()));
    let mut compiler_config = options
      .get("compilerConfig")
      .unwrap_or(&Value::Object(Map::new()))
      .clone();
    // Add preset_env to compiler_config if it doesn't exist
    if let Value::Object(ref mut map) = compiler_config {
      if !map.contains_key("presetEnv") {
        map.insert("presetEnv".to_string(), Value::Bool(true));
      }
    }
    // TODO merge config
    // let compiler_config =
    //   merge_configs(config.clone(), compiler_config, &HashSet::from([])).unwrap();
    let is_build = options.get("isBuild").and_then(|x| x.as_bool());
    let worker_cache = cache::WorkerCache::new();
    Self {
      options: Options {
        is_build: Some(is_build.unwrap_or(false)),
        compiler_config: serde_json::from_value(compiler_config).ok(),
        host_config: Some(config.clone()),
      },
      worker_cache,
    }
  }
}

impl Plugin for FarmfePluginWorker {
  fn name(&self) -> &str {
    "FarmfePluginWorker"
  }
  fn priority(&self) -> i32 {
    105
  }
  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if JsRegex::new(WORKER_OR_SHARED_WORKER_RE)
      .unwrap()
      .find(&param.module_id)
      .is_some()
    {
      let content = process_worker(ProcessWorkerParam {
        resolved_path: param.resolved_path,
        module_id: &param.module_id,
        is_build: self.options.is_build.unwrap(),
        is_url: param.query.iter().any(|(k, _v)| k == "url"),
        is_inline: param.query.iter().any(|(k, _v)| k == "inline"),
        compiler_config: self.options.compiler_config.as_ref().unwrap(),
        host_config: self.options.host_config.as_ref().unwrap(),
        worker_cache: &self.worker_cache,
        context,
      });

      return Ok(Some(PluginLoadHookResult {
        content,
        module_type: ModuleType::Js,
        source_map: None,
      }));
    }
    return Ok(None);
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    let re = JsRegex::new(WORKER_IMPORT_META_URL_RE).unwrap();
    let matches = re.find_iter(&param.content).collect::<Vec<Match>>();
    if matches.is_empty() {
      return Ok(None);
    }
    let mut content = String::new();
    let mut last_end = 0;
    matches.iter().for_each(|m: &Match| {
      let args = &m.captures[0].clone().unwrap();
      let worker_url = &m.captures[1].clone().unwrap();
      let arg_code = &param.content[args.start..args.end];
      let worker_url_code = &param.content[worker_url.start..worker_url.end];
      if arg_code.contains("`") && arg_code.contains("${") {
        println!("new URL(url, import.meta.url) is not supported in dynamic template string.")
      } else {
        let compiler_config = self.options.compiler_config.as_ref().unwrap();
        let worker_url = &worker_url_code[1..worker_url_code.len() - 1];
        let full_worker_path = match &worker_url[0..1] {
          "/" => context.config.root.to_string() + worker_url,
          _ => {
            context
              .plugin_driver
              .resolve(
                &PluginResolveHookParam {
                  source: worker_url.to_string(),
                  importer: Some(param.module_id.clone().into()),
                  kind: ResolveKind::Import,
                },
                context,
                &PluginHookContext::default(),
              )
              .unwrap()
              .unwrap()
              .resolved_path
          }
        };
        let content_bytes = build_worker(
          &full_worker_path,
          &full_worker_path,
          compiler_config,
          self.options.host_config.as_ref().unwrap(),
        );
        let new_worker_url = relative(&context.config.root, &full_worker_path);
        // update param content
        // worker_url_code -> new_worker_url
        let (worker_url, filename) =
          get_worker_url(&full_worker_path, &new_worker_url, compiler_config);
        emit_worker_file(&param.module_id, &filename, content_bytes, context);
        content.push_str(&param.content[last_end..args.start]);
        content.push_str(&arg_code.replace(worker_url_code, &format!(r#""{}""#, &worker_url)));
        last_end = args.end;
        let worker_module_id = ModuleId::new(full_worker_path.as_str(), "", &context.config.root);
        let self_module_id = ModuleId::new(param.resolved_path, "", &context.config.root);
        let _ = context.add_watch_files(self_module_id, vec![worker_module_id]);
      }
    });
    content.push_str(&param.content[last_end..]);
    return Ok(Some(PluginTransformHookResult {
      content,
      module_type: Some(param.module_type.clone()),
      ..Default::default()
    }));
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cached_static_assets = deserialize!(cache, CachedStaticAssets);

    for asset in cached_static_assets.list {
      if let ResourceOrigin::Module(m) = asset.origin {
        context.emit_file(EmitFileParams {
          resolved_path: m.to_string(),
          name: asset.name,
          content: asset.bytes,
          resource_type: asset.resource_type,
        });
      }
    }

    Ok(Some(()))
  }
  fn write_plugin_cache(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let mut list = vec![];
    let resources_map = context.resources_map.lock();
    for (_, resource) in resources_map.iter() {
      if let ResourceOrigin::Module(m) = &resource.origin {
        if context.cache_manager.module_cache.has_cache(m) {
          list.push(resource.clone());
        }
      }
    }

    if !list.is_empty() {
      let cached_static_assets = CachedStaticAssets { list };
      Ok(Some(serialize!(&cached_static_assets)))
    } else {
      Ok(None)
    }
  }
}
