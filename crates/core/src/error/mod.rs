use std::{error::Error, sync::LockResult};

use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use crate::resource::resource_pot::ResourcePotType;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompilationError {
  // #[error("Can not resolve `{src}` from {importer}.\nOriginal error: {source:?}.\n\nPotential Causes:\n1.The file that `{src}` points to does not exist.\n2.Install it first if `{src}` is an dependency from node_modules, if you are using pnpm refer to [https://pnpm.io/faq#pnpm-does-not-work-with-your-project-here] for solutions.\n3. If `{src}` is a alias, make sure your alias config is correct.\n")]
  #[error("{}", serde_json::to_string(&serialize_resolve_error(src, importer, source))
  .map_err(|_| "Failed to serialize resolve error type message".to_string())
  .unwrap_or_else(|e| e))]
  ResolveError {
    importer: String,
    src: String,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },
  // TODO, give the specific recommended plugin of this kind of module
  // TODO add potential causes
  #[error("Can not load `{resolved_path}`. Original error: \n{source:?}.\n\nPotential Causes:\n1.This kind of module is not supported, you may need plugins to support it.\n")]
  LoadError {
    resolved_path: String,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  // #[error("Transform `{resolved_path}` failed.\n {msg}")]
  #[error("{}", serde_json::to_string(&serialize_transform_error(resolved_path, msg))
  .map_err(|_| "Failed to serialize transform error type message".to_string())
  .unwrap_or_else(|e| e))]
  TransformError { resolved_path: String, msg: String },

  // TODO, give the specific recommended plugin of this kind of module
  // #[error("Parse `{resolved_path}` failed.\n {msg}\nPotential Causes:\n1.The module have syntax error.\n2.This kind of module is not supported, you may need plugins to support it\n")]
  #[error("{}", serde_json::to_string(&serialize_parse_error(&resolved_path, &msg))
    .map_err(|_| "Failed to serialize parse error type  message".to_string())
    .unwrap_or_else(|e| e))]
  ParseError { resolved_path: String, msg: String },

  #[error("Hook `process_module` execute failed for module `{resolved_path}`.\nOriginal error: {source:?}.")]
  ProcessModuleError {
    resolved_path: String,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error(
    "Hook `analyze_deps` execute failed for module `{resolved_path}`.\nOriginal error: {source:?}."
  )]
  AnalyzeDepsError {
    resolved_path: String,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("{0}")]
  GenericError(String),

  // TODO optimize using source Error
  #[error("{0}")]
  NAPIError(String),

  #[error("Hook `analyze_module_graph` execute failed.\nOriginal error: {source:?}.\n")]
  AnalyzeModuleGraphError {
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("No plugins return valid result for hook {hook_name}. You may need plugins if you are using module types which are not native supported.")]
  PluginHookResultCheckError { hook_name: String },

  #[error("Generate resources for {name}(type: {ty:?}) failed. Original error: {source:?}. This error usually caused by problematic plugin that implement `generate_resources` hook but does not return a valid result")]
  GenerateResourcesError {
    name: String,
    ty: ResourcePotType,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("Render Html Resource Pot {name} failed, A html resource pot should only contains one html module, current containing html modules: {modules:?}")]
  RenderHtmlResourcePotError { name: String, modules: Vec<String> },

  #[error("Load package.json from `{package_json_path}` failed: {err_message}")]
  LoadPackageJsonError {
    package_json_path: String,
    err_message: String,
  },

  #[error("render script module `{id}` failed. Original error: {source:?}")]
  RenderScriptModuleError {
    id: String,
    #[source]
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("generate sourcemap for module `{id}` failed")]
  GenerateSourceMapError {
    id: String,
    // #[source]
    // source: Option<Box<dyn Error + Send + Sync>>,
  },
}

fn serialize_resolve_error(
  src: &str,
  importer: &str,
  source: &Option<Box<dyn Error + Send + Sync>>,
) -> serde_json::Value {
  let mut msg = format!("Can not resolve `{}` from `{}`.", src, importer);
  if let Some(source) = source {
    msg.push_str(&format!("\nOriginal error: {}.", source));
  }

  let cause = format!(
      "Potential Causes:\n\
       1. The file that `{}` points to does not exist.\n\
       2. Install it first if `{}` is a dependency from node_modules. If you are using pnpm, refer to [https://pnpm.io/faq#pnpm-does-not-work-with-your-project-here] for solutions.\n\
       3. If `{}` is an alias, make sure your alias config is correct.",
      src, src, src
  );

  let full_message = format!("{}\n{}", msg, cause);

  json!({
      "id": importer,
      "type": "Resolve Error",
      "errorFrame": msg,
      "message": full_message,
      "cause": cause
  })
}

fn serialize_parse_error(resolved_path: &str, msg: &str) -> serde_json::Value {
  if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg) {
    return parsed;
  }
  json!({
      "id": resolved_path,
      "errorFrame": msg,
      "type": "Parse Error",
      "cause": "Potential Causes:\n1.The module have syntax error.\n2.This kind of module is not supported, you may need plugins to support it.\n",
      "message": format!("Parse `{}` failed.\n {}", resolved_path, msg)
  })
}

fn serialize_transform_error(resolved_path: &str, msg: &str) -> serde_json::Value {
  if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg) {
    return parsed;
  }
  json!({
      "type": "Transform Error",
      "id": resolved_path,
      "errorFrame": msg,
      "cause": "Potential Causes:\n1.This kind of module is not supported, you may need plugins to support it.\n",
      "message": format!("Transform `{}` failed.\n {}", resolved_path, msg)
  })
}

pub type Result<T> = core::result::Result<T, CompilationError>;

pub trait ToResolveError
where
  Self: Error + Sized + Send + Sync + 'static,
{
  fn to_resolve_error(self, src: String, importer: String) -> CompilationError {
    CompilationError::ResolveError {
      importer,
      src,
      source: Some(Box::new(self) as _),
    }
  }
}

impl<T: Error + Sized + Send + Sync + 'static> ToResolveError for T
where
  T: Error,
{
  fn to_resolve_error(self, src: String, importer: String) -> CompilationError {
    CompilationError::ResolveError {
      importer,
      src,
      source: Some(Box::new(self) as _),
    }
  }
}

pub trait MapCompletionError<T> {
  fn map_c_error(self) -> Result<T>;
}

// mutex lock error
impl<T> MapCompletionError<T> for LockResult<T> {
  fn map_c_error(self) -> Result<T> {
    match self {
      Ok(v) => Ok(v),
      Err(e) => Err(CompilationError::GenericError(e.to_string())),
    }
  }
}
