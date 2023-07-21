use std::sync::Arc;

use farmfe_core::{
  error::{CompilationError, Result},
  swc_common::{errors::HANDLER, Globals, SourceMap, GLOBALS},
};
use swc_ecma_transforms::helpers::{Helpers, HELPERS};
use swc_error_reporters::handler::try_with_handler;

pub fn try_with<F>(cm: Arc<SourceMap>, globals: &Globals, op: F) -> Result<()>
where
  F: FnOnce(),
{
  GLOBALS
    .set(globals, || {
      try_with_handler(cm, Default::default(), |handler| {
        HELPERS.set(&Helpers::new(true), || HANDLER.set(handler, op));
        Ok(())
      })
    })
    .map_err(|e| CompilationError::GenericError(e.to_string()))
}
