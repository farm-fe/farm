use std::sync::{
  mpsc::{channel, Sender},
  Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleType,
  plugin::{
    PluginAnalyzeDepsHookParam, PluginLoadHookParam, PluginParseHookParam, PluginResolveHookParam,
    PluginTransformHookParam, ResolveKind,
  },
  rayon,
  rayon::ThreadPool,
  serde_json::from_str,
};

use crate::Compiler;

pub struct BuildModuleParam {
  pub source: String,
  pub importer: Option<String>,
  pub kind: ResolveKind,
}

impl Compiler {
  pub(crate) fn build(&self) -> Result<()> {
    self.context.plugin_driver.build_start(&self.context)?;

    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let (err_sender, err_receiver) = channel::<CompilationError>();

    for source in self.context.config.input.values() {
      Self::build_module(
        thread_pool.clone(),
        BuildModuleParam {
          source: source.clone(),
          importer: None,
          kind: ResolveKind::Entry,
        },
        self.context.clone(),
        err_sender.clone(),
      );
    }

    drop(err_sender);

    if let Ok(err) = err_receiver.recv() {
      return Err(err);
    }

    self.context.plugin_driver.build_end(&self.context)
  }

  /// resolving, loading, transforming and parsing a module in a separate thread
  fn build_module(
    thread_pool: Arc<ThreadPool>,
    param: BuildModuleParam,
    context: Arc<CompilationContext>,
    err_sender: Sender<CompilationError>,
  ) {
    let c_thread_pool = thread_pool.clone();
    thread_pool.spawn(move || {
      let resolve_param = PluginResolveHookParam {
        source: param.source,
        importer: param.importer,
        kind: param.kind,
        caller: None,
      };
      let resolved = match context.plugin_driver.resolve(&resolve_param, &context) {
        Ok(resolved) => match resolved {
          Some(res) => res,
          None => {
            err_sender
              .send(CompilationError::ResolveError {
                importer: resolve_param
                  .importer
                  .unwrap_or_else(|| context.config.root.clone()),
                src: resolve_param.source.clone(),
                source: None,
              })
              .err();
            return;
          }
        },
        Err(e) => {
          err_sender
            .send(CompilationError::ResolveError {
              importer: resolve_param
                .importer
                .unwrap_or_else(|| context.config.root.clone()),
              src: resolve_param.source.clone(),
              source: Some(Box::new(e)),
            })
            .expect("Send resolve error failed");
          return;
        }
      };

      println!("resolved {:?}", resolved);

      let load_param = PluginLoadHookParam {
        id: &resolved.id,
        caller: None,
        query: resolved.query.clone(),
      };

      let loaded = match context.plugin_driver.load(&load_param, &context) {
        Ok(loaded) => match loaded {
          Some(loaded) => loaded,
          None => {
            err_sender
              .send(CompilationError::LoadError {
                id: load_param.id.to_string(),
                source: None,
              })
              .unwrap();
            return;
          }
        },
        Err(e) => {
          err_sender
            .send(CompilationError::LoadError {
              id: load_param.id.to_string(),
              source: Some(Box::new(e)),
            })
            .expect("send load error failed");
          return;
        }
      };

      println!("loaded {:?}", loaded);

      let transform_param = PluginTransformHookParam {
        content: loaded.content,
        id: &resolved.id,
        module_type: loaded.module_type,
        query: resolved.query.clone(),
      };

      let transformed = match context.plugin_driver.transform(transform_param, &context) {
        Ok(transformed) => transformed,
        Err(e) => {
          err_sender
            .send(CompilationError::TransformError {
              id: resolved.id.to_string(),
              source: Some(Box::new(e)),
            })
            .unwrap();
          return;
        }
      };

      println!("transformed {:?}", transformed);

      let parse_param = PluginParseHookParam {
        id: resolved.id,
        query: resolved.query,
        module_type: ModuleType::Js,
        content: transformed.content,
        source_map_chain: vec![],
        side_effects: false,
        package_json_info: from_str(r#"{ "name": "hello" }"#).unwrap(),
        caller: None,
      };
      let mut module = match context.plugin_driver.parse(&parse_param, &context) {
        Ok(module) => match module {
          Some(module) => module,
          None => {
            err_sender
              .send(CompilationError::ParseError {
                id: parse_param.id,
                source: None,
              })
              .unwrap();
            return;
          }
        },
        Err(e) => {
          err_sender
            .send(CompilationError::ParseError {
              id: parse_param.id,
              source: Some(Box::new(e)),
            })
            .unwrap();
          return;
        }
      };

      if let Err(e) = context.plugin_driver.process_module(&mut module, &context) {
        err_sender
          .send(CompilationError::ModuleParsedError {
            id: parse_param.id,
            source: Some(Box::new(e)),
          })
          .unwrap();
        return;
      }

      let mut analyze_deps_param = PluginAnalyzeDepsHookParam {
        module: &module,
        deps: vec![],
      };
      if let Err(e) = context
        .plugin_driver
        .analyze_deps(&mut analyze_deps_param, &context)
      {
        err_sender
          .send(CompilationError::AnalyzeDepsError {
            id: parse_param.id,
            source: Some(Box::new(e)),
          })
          .unwrap();
        return;
      }

      // resolving dependencies recursively
      for dep in analyze_deps_param.deps {
        Self::build_module(
          c_thread_pool.clone(),
          BuildModuleParam {
            source: dep.source,
            importer: Some(parse_param.id.clone()),
            kind: dep.kind,
          },
          context.clone(),
          err_sender.clone(),
        );
      }
    });
  }
}
