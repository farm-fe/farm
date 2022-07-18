use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilationError {
  #[error("Can not resolve `{specifier}` from {importer}.\nOriginal error: {source:?}.\n\nPotential Causes:\n1.The file that `{specifier}` points to does not exist.\n2.Install it first if `{specifier}` is an dependency from node_modules, if you are using pnpm refer to [https://pnpm.io/faq#pnpm-does-not-work-with-your-project-here] for solutions.\n3. If `{specifier}` is a alias, make sure your alias config is correct.\n")]
  ResolveError {
    importer: String,
    specifier: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },
  // TODO, give the specific recommended plugin of this kind of module
  #[error("Can not load `{id}`. Original error: {source:?}.\n\nPotential Causes:\n1.This kind of module is not supported, you may need plugins to support it.\n")]
  LoadError {
    id: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },

  #[error("Transform `{id}` failed.\nOriginal error: {source:?}")]
  TransformError {
    id: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },
  // TODO, give the specific recommended plugin of this kind of module
  #[error("Parse `{id}` failed.\nOriginal error: {source:?}.\n\nPotential Causes:\n1.The module have syntax error.\n2.This kind of module is not supported, you may need plugins to support it\n")]
  ParseError {
    id: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },

  #[error("Hook `module_parsed` execute failed.\nOriginal error: {source:?}.")]
  ModuleParsedError {
    id: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },

  #[error("Hook `analyze_deps` execute failed.\nOriginal error: {source:?}.")]
  AnalyzeDepsError {
    id: String,
    #[source]
    source: Option<Box<CompilationError>>,
  },

  #[error("{0}")]
  GenericError(String),
  #[error("{0}")]
  NAPIError(String),
}

pub type Result<T> = core::result::Result<T, CompilationError>;
