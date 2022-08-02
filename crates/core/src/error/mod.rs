use std::error::Error;

use thiserror::Error;

use crate::resource::resource_pot::ResourcePotType;

#[derive(Debug, Error)]
pub enum CompilationError {
  #[error("Can not resolve `{src}` from {importer}.\nOriginal error: {source:?}.\n\nPotential Causes:\n1.The file that `{src}` points to does not exist.\n2.Install it first if `{src}` is an dependency from node_modules, if you are using pnpm refer to [https://pnpm.io/faq#pnpm-does-not-work-with-your-project-here] for solutions.\n3. If `{src}` is a alias, make sure your alias config is correct.\n")]
  ResolveError {
    importer: String,
    src: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },
  // TODO, give the specific recommended plugin of this kind of module
  #[error("Can not load `{id}`. Original error: {source:?}.\n\nPotential Causes:\n1.This kind of module is not supported, you may need plugins to support it.\n")]
  LoadError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("Transform `{id}` failed.\nOriginal error: {source:?}")]
  TransformError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },
  // TODO, give the specific recommended plugin of this kind of module
  #[error("Parse `{id}` failed.\nOriginal error: {source:?}.\n\nPotential Causes:\n1.The module have syntax error.\n2.This kind of module is not supported, you may need plugins to support it\n")]
  ParseError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("Hook `module_parsed` execute failed.\nOriginal error: {source:?}.")]
  ModuleParsedError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("Hook `analyze_deps` execute failed.\nOriginal error: {source:?}.")]
  AnalyzeDepsError {
    id: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
  },

  #[error("{0}")]
  GenericError(String),
  #[error("{0}")]
  NAPIError(String),

  #[error("Generate resources for {name}(type: {ty:?}) failed. This error usually caused by problematic plugin that implement `generate_resources` hook but does not return a valid result")]
  GenerateResourcesError { name: String, ty: ResourcePotType },
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
