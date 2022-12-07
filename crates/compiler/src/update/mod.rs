use std::sync::{mpsc::Sender, Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  module::ModuleId,
  plugin::{PluginHookContext, PluginResolveHookParam},
  rayon::ThreadPool,
};

use crate::{build::resolve::resolve, Compiler};
use farmfe_core::error::Result;

/// The output after the updating process
#[derive(Debug, Default)]
pub struct UpdateOutput {
  pub added_module_ids: Vec<ModuleId>,
  pub updated_module_ids: Vec<ModuleId>,
  pub removed_module_ids: Vec<ModuleId>,
  pub resources: String,
}

pub enum UpdateType {
  // added a new module
  Added,
  // updated a module
  Updated,
  // removed a module
  Removed,
}

impl Compiler {
  pub fn update(&self, paths: Vec<(String, UpdateType)>) -> Result<UpdateOutput> {
    for (path, update_type) in paths {
      match update_type {
        UpdateType::Added => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Added is not supported yet".to_string(),
          ));
        }
        UpdateType::Updated => {
          // self.context.module_graph.update_module(path);
        }
        UpdateType::Removed => {
          return Err(farmfe_core::error::CompilationError::GenericError(
            "Removed is not supported yet".to_string(),
          ));
        }
      }
    }

    Ok(UpdateOutput::default())
  }

  /// Resolving, loading, transforming and parsing a module in a separate thread.
  /// This method is similar to the build_module_graph_threaded method in the build/mod.rs file,
  /// the difference is that this method is used for updating the module graph, only handles the updated and added module, and ignores the existing unchanged module,
  /// while the build_module_threaded method is used for building full module graph and every module is handled.
  fn update_module_graph_threaded(
    thread_pool: Arc<ThreadPool>,
    resolve_param: PluginResolveHookParam,
    context: Arc<CompilationContext>,
    err_sender: Sender<CompilationError>,
  ) {
    thread_pool.spawn(move || {
      // let (module, deps) = Self::build_module(&resolve_param, &context, err_sender);
    });
  }
}
