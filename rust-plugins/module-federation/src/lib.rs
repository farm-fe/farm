//! Experimental native Federation graph adapter. Remote requests become async
//! Farm modules; the official JavaScript runtime owns remote protocol handling.
#![deny(clippy::all)]

mod script_entry;

use farmfe_core::{
  config::{Config, SourcemapConfig},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleType,
  plugin::{
    Plugin, PluginFinalizeResourcesHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult,
  },
  serde_json::{self, json},
};
use farmfe_macro_plugin::farm_plugin;
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path, sync::Arc};

const ADAPTER: &str = "farm-mf-native:adapter";
const CONTAINER: &str = "farm-mf-native:container";

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum EntryFormat {
  #[default]
  Global,
  Esm,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Remote {
  name: String,
  entry: String,
  #[serde(default)]
  global_name: Option<String>,
  #[serde(default)]
  format: EntryFormat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Shared {
  version: String,
  required_version: String,
  #[serde(default)]
  singleton: bool,
  #[serde(default)]
  strict_version: bool,
  fallback: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
  name: String,
  runtime_path: String,
  #[serde(default)]
  hot: bool,
  #[serde(default)]
  public_path: Option<String>,
  #[serde(default)]
  global_name: Option<String>,
  #[serde(default)]
  format: EntryFormat,
  #[serde(default = "default_filename")]
  filename: String,
  #[serde(default)]
  remotes: BTreeMap<String, Remote>,
  #[serde(default)]
  exposes: BTreeMap<String, String>,
  #[serde(default)]
  shared: BTreeMap<String, Shared>,
}
fn default_filename() -> String {
  "remoteEntry.js".into()
}
fn error(message: impl Into<String>) -> CompilationError {
  CompilationError::GenericError(format!("[Farm MF] {}", message.into()))
}
fn quoted(value: &str) -> String {
  json!(value).to_string()
}
fn valid_name(value: &str) -> bool {
  let mut chars = value.chars();
  chars
    .next()
    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
    && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[farm_plugin]
pub struct FarmPluginModuleFederation {
  options: std::result::Result<Options, String>,
}

impl FarmPluginModuleFederation {
  fn new(_: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str(&options).map_err(|e| e.to_string()),
    }
  }
  fn options(&self) -> Result<&Options> {
    self
      .options
      .as_ref()
      .map_err(|e| error(format!("Invalid options: {e}")))
  }
  fn virtual_root(config: &Config) -> String {
    Path::new(&config.root)
      .join(".farm-mf-native")
      .to_string_lossy()
      .replace('\\', "/")
  }
  fn remote_request(&self, source: &str) -> Result<bool> {
    Ok(
      self
        .options()?
        .remotes
        .keys()
        .any(|alias| source.starts_with(&format!("{alias}/"))),
    )
  }
  fn adapter_source(&self, config: &Config) -> Result<String> {
    let options = self.options()?;
    let remotes: Vec<_> = options
      .remotes
      .iter()
      .map(|(alias, remote)| {
        json!({
          "name":remote.name,"alias":alias,"entry":remote.entry,
          "entryGlobalName":remote.global_name.as_ref().unwrap_or(&remote.name),
          "type":if remote.format==EntryFormat::Global {"global"} else {"module"}
        })
      })
      .collect();
    let shared = options.shared.iter().map(|(key, value)| {
      format!("{}:{{version:{},scope:['default'],shareConfig:{{singleton:{},requiredVersion:{},strictVersion:{}}},get:async()=>{{const ns=await import({});return ()=>ns;}}}}",
        quoted(key), quoted(&value.version), value.singleton, quoted(&value.required_version), value.strict_version, quoted(&value.fallback))
    }).collect::<Vec<_>>().join(",");
    let configuration = json!({"name":options.name,"namespace":config.runtime.namespace,"remotes":remotes,"hot":options.hot && config.mode.is_dev()});
    Ok(
      include_str!("adapter.js")
        .replace("__MF_RUNTIME_PATH__", &quoted(&options.runtime_path))
        .replace("__MF_CONFIGURATION__", &configuration.to_string())
        .replace("__MF_SHARED__", &format!("{{{shared}}}"))
        .replace("__MF_HOT_ADAPTER__", include_str!("hot.js")),
    )
  }
  fn container_source(&self, config: &Config) -> Result<String> {
    let options = self.options()?;
    let global_name = options.global_name.as_ref().unwrap_or(&options.name);
    let loaders = options
      .exposes
      .iter()
      .map(|(key, value)| {
        let path = Path::new(&config.root)
          .join(value)
          .to_string_lossy()
          .replace('\\', "/");
        format!("{}:()=>import({})", quoted(key), quoted(&path))
      })
      .collect::<Vec<_>>()
      .join(",");
    let hot_accepts = if options.hot && config.mode.is_dev() {
      let boundaries = options
        .exposes
        .iter()
        .map(|(key, value)| {
          let path = Path::new(&config.root)
            .join(value)
            .to_string_lossy()
            .replace('\\', "/");
          format!(
            "import.meta.hot.accept({}, async value => bridge.publish({}, await value));",
            quoted(&path),
            quoted(key)
          )
        })
        .collect::<Vec<_>>()
        .join("\n");
      format!("if(import.meta.hot){{bridge.installHotBoundary(import.meta.hot);{boundaries}}}")
    } else {
      String::new()
    };
    Ok(format!(
      r#"
      import {{bridge}} from {adapter};
      globalThis[{namespace}].m.sp([{public_path}]);
      const container=bridge.createContainer({{{loaders}}});
      if (globalThis[{global_name}]) throw new Error({conflict});
      globalThis[{global_name}]=container;
      export const get=container.get;
      export const init=container.init;
      {hot_accepts}
    "#,
      hot_accepts = hot_accepts,
      adapter = quoted(ADAPTER),
      namespace = quoted(&config.runtime.namespace),
      public_path = quoted(
        options
          .public_path
          .as_deref()
          .unwrap_or(&config.output.public_path)
      ),
      global_name = quoted(global_name),
      conflict = quoted(&format!(
        "[Farm MF {}] Duplicate container global: {global_name}",
        options.name
      ))
    ))
  }
}

impl Plugin for FarmPluginModuleFederation {
  fn name(&self) -> &str {
    "FarmPluginModuleFederation"
  }
  fn priority(&self) -> i32 {
    1100
  }
  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    let options = self.options()?;
    if !valid_name(&options.name)
      || !valid_name(options.global_name.as_ref().unwrap_or(&options.name))
    {
      return Err(error("name/globalName must be JavaScript identifiers"));
    }
    if !config.output.target_env.is_browser() {
      return Err(error("Browser target required"));
    }
    if !Path::new(&options.runtime_path).is_absolute() {
      return Err(error("runtimePath must be absolute"));
    }
    if options.filename.contains(['/', '\\']) || !options.filename.ends_with(".js") {
      return Err(error("filename must be a plain .js filename"));
    }
    for (alias, remote) in &options.remotes {
      if !valid_name(alias) || !valid_name(&remote.name) {
        return Err(error("Remote alias/name must be identifiers"));
      }
      if !(remote.entry.starts_with("http://") || remote.entry.starts_with("https://")) {
        return Err(error("Remote entry must be HTTP(S)"));
      }
    }
    for (key, shared) in &options.shared {
      if key.is_empty()
        || key.starts_with('.')
        || key.starts_with('/')
        || key.split('/').any(|part| part == "..")
      {
        return Err(error("Shared keys must be bare package requests"));
      }
      if !Path::new(&shared.fallback).is_absolute() {
        return Err(error("Shared fallback must be an absolute module path"));
      }
    }
    config.runtime.namespace = format!("farm_mf_{}", options.name);
    config.runtime.isolate = true;
    if !options.exposes.is_empty() {
      if options.hot && config.mode.is_dev() && config.css.transform_to_script == Some(false) {
        return Err(error(
          "Federation HMR requires css.transformToScript:true in remote development builds",
        ));
      }
      let public_path = options
        .public_path
        .as_deref()
        .unwrap_or(&config.output.public_path);
      if !(public_path.starts_with("http://") || public_path.starts_with("https://")) {
        return Err(error(
          "Native remote requires absolute publicPath in this stage",
        ));
      }
      if options.format == EntryFormat::Global
        && !matches!(*config.sourcemap, SourcemapConfig::Bool(false))
      {
        return Err(error(
          "Experimental global entry assembly requires sourcemap:false; ESM preserves source maps",
        ));
      }
      for key in options.exposes.keys() {
        if !key.starts_with("./") {
          return Err(error("Expose keys must start with ./"));
        }
      }
      config.input.clear();
      config.input.insert("remoteEntry".into(), CONTAINER.into());
      config.output.entry_filename = options.filename.clone();
    }
    Ok(Some(()))
  }
  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    _: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    let source = if param.source.starts_with("./farm-mf-native:") {
      &param.source[2..]
    } else {
      &param.source
    };
    let root = Self::virtual_root(&context.config);
    let path = match source {
      ADAPTER | "farm:module-federation" => Some(format!("{root}/adapter.js")),
      CONTAINER => Some(format!("{root}/container.js")),
      _ if self.options()?.shared.contains_key(source) => {
        Some(format!("{root}/shared/{source}.js"))
      }
      _ if self.remote_request(source)? => {
        if source.split('/').any(|s| s == ".." || s.is_empty()) {
          return Err(error("Invalid remote request"));
        }
        Some(format!("{root}/remotes/{source}.js"))
      }
      _ => None,
    };
    Ok(path.map(|resolved_path| PluginResolveHookResult {
      resolved_path,
      ..Default::default()
    }))
  }
  fn load(
    &self,
    param: &PluginLoadHookParam,
    context: &Arc<CompilationContext>,
    _: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    let root = Self::virtual_root(&context.config);
    let content = if param.resolved_path == format!("{root}/adapter.js") {
      Some(self.adapter_source(&context.config)?)
    } else if param.resolved_path == format!("{root}/container.js") {
      Some(self.container_source(&context.config)?)
    } else if let Some(request) = param
      .resolved_path
      .strip_prefix(&format!("{root}/shared/"))
      .and_then(|s| s.strip_suffix(".js"))
    {
      Some(format!(
        "import {{bridge}} from {}; module.exports = await bridge.loadShared({});",
        quoted(ADAPTER),
        quoted(request)
      ))
    } else {
      // TLA is a real async graph node, not an asynchronous bootstrap hook.
      param
        .resolved_path
        .strip_prefix(&format!("{root}/remotes/"))
        .and_then(|s| s.strip_suffix(".js"))
        .map(|request| {
          format!(
            "import {{bridge}} from {}; module.exports = await bridge.loadRemote({});",
            quoted(ADAPTER),
            quoted(request)
          )
        })
    };
    Ok(content.map(|content| PluginLoadHookResult {
      content,
      module_type: ModuleType::Js,
      source_map: None,
    }))
  }
  fn finalize_resources(
    &self,
    param: &mut PluginFinalizeResourcesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let options = self.options()?;
    if options.exposes.is_empty() || options.format == EntryFormat::Esm {
      return Ok(None);
    }
    let sources = param
      .resources_map
      .iter()
      .filter(|(_, resource)| {
        matches!(
          resource.resource_type,
          farmfe_core::resource::ResourceType::Js | farmfe_core::resource::ResourceType::Runtime
        )
      })
      .map(|(name, resource)| {
        String::from_utf8(resource.bytes.clone())
          .map(|source| (name.clone(), source))
          .map_err(|e| error(e.to_string()))
      })
      .collect::<Result<farmfe_core::HashMap<_, _>>>()?;
    // Guard before evaluating the bundled Farm runtime: a late container guard
    // would already have replaced the namespace of an existing application.
    let global_name = options.global_name.as_ref().unwrap_or(&options.name);
    let mut bytes = format!(
      "if(globalThis[{global}])throw new Error({message});if(globalThis[{namespace}])throw new Error({namespace_message});\n",
      global=quoted(global_name),
      message=quoted(&format!("[Farm MF {}] Duplicate container global: {global_name}", options.name)),
      namespace=quoted(&context.config.runtime.namespace),
      namespace_message=quoted(&format!("[Farm MF {}] Duplicate runtime namespace: {}", options.name, context.config.runtime.namespace)),
    ).into_bytes();
    bytes.extend(script_entry::bundle(&options.filename, &sources, context)?);
    let resource = param
      .resources_map
      .get_mut(&options.filename)
      .ok_or_else(|| error("Container entry not emitted"))?;
    resource.bytes = bytes;
    Ok(Some(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn invalid_options_are_diagnostics_not_panics() {
    let mut config = Config::default();
    let plugin = FarmPluginModuleFederation::new(&config, "{broken".into());
    assert!(plugin.config(&mut config).is_err());
  }
  #[test]
  fn names_are_safe_global_keys() {
    assert!(valid_name("remote_A"));
    assert!(!valid_name("a';alert(1)"));
    assert!(!valid_name(""));
  }
  #[test]
  fn relative_dependencies_do_not_match_remote_aliases() {
    let config = Config::default();
    let plugin = FarmPluginModuleFederation::new(&config, json!({"name":"host","runtimePath":"/runtime.js","remotes":{"remote":{"name":"provider","entry":"http://localhost/remoteEntry.js"}}}).to_string());
    assert!(
      plugin
        .remote_request("remote/Button")
        .expect("valid options")
    );
    assert!(
      !plugin
        .remote_request("./remote/index.js")
        .expect("valid options")
    );
    assert!(
      !plugin
        .remote_request("/remote/index.js")
        .expect("valid options")
    );
  }
  #[test]
  fn relative_shared_fallback_is_rejected() {
    let mut config = Config::default();
    let plugin = FarmPluginModuleFederation::new(&config, json!({"name":"host","runtimePath":"/runtime.js","shared":{"react":{"version":"18.3.1","requiredVersion":"^18.3.1","fallback":"react"}}}).to_string());
    assert!(plugin.config(&mut config).is_err());
  }
  #[test]
  fn hot_boundaries_are_explicit_and_development_only() {
    let mut config = Config::default();
    config.root = "/project".into();
    config.mode = farmfe_core::config::Mode::Development;
    let plugin = FarmPluginModuleFederation::new(&config, json!({"name":"provider","runtimePath":"/runtime.js","hot":true,"exposes":{"./Button":"./Button.js"}}).to_string());
    let development = plugin.container_source(&config).expect("valid container");
    assert!(development.contains("import.meta.hot.accept(\"/project/./Button.js\""));
    assert!(development.contains("bridge.installHotBoundary"));
    config.mode = farmfe_core::config::Mode::Production;
    assert!(
      !plugin
        .container_source(&config)
        .expect("valid container")
        .contains("import.meta.hot")
    );
    assert!(
      plugin
        .adapter_source(&config)
        .expect("valid adapter")
        .contains("\"hot\":false")
    );
  }
  #[test]
  fn default_format_is_global() {
    let options: Options =
      serde_json::from_value(json!({"name":"host","runtimePath":"/runtime.js"}))
        .expect("valid options");
    assert_eq!(options.format, EntryFormat::Global);
  }
}
