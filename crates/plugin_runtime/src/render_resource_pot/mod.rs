use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use farmfe_core::{
  config::{
    minify::MinifyMode, FARM_DYNAMIC_REQUIRE, FARM_MODULE, FARM_MODULE_EXPORT, FARM_REQUIRE,
  },
  context::CompilationContext,
  enhanced_magic_string::{
    bundle::{Bundle, BundleOptions},
    magic_string::{MagicString, MagicStringOptions},
  },
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem},
  parking_lot::Mutex,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::{RenderedModule, ResourcePot},
  swc_common::{comments::SingleThreadedComments, util::take::Take, Mark, DUMMY_SP},
  swc_ecma_ast::{BindingIdent, BlockStmt, FnDecl, Function, Module, ModuleItem, Param, Stmt}, // swc_ecma_ast::Function
};
use farmfe_toolkit::{
  common::{build_source_map, create_swc_source_map, PathFilter, Source},
  minify::minify_js_module,
  script::{
    codegen_module,
    swc_try_with::{resolve_module_mark, try_with},
    CodeGenCommentsConfig,
  },
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    fixer,
    helpers::inject_helpers,
    hygiene::{hygiene_with_config, Config as HygieneConfig},
    modules::{
      common_js,
      import_analysis::import_analyzer,
      util::{Config, ImportInterop},
    },
  },
  swc_ecma_transforms_base::fixer::paren_remover,
  swc_ecma_visit::VisitMutWith,
};

use self::source_replacer::{ExistingCommonJsRequireVisitor, SourceReplacer};

// mod farm_module_system;
mod source_replacer;

/// Merge all modules' ast in a [ResourcePot] to Farm's runtime [ObjectLit]. The [ObjectLit] looks like:
/// ```js
/// {
///   // commonjs or hybrid module system
///   "a.js": async function(module, exports, require) {
///       const b = await require('./b');
///       console.log(b);
///    },
///    // esm module system
///    "b.js": async function(module, exports, require) {
///       Promise.all([
///         require('./c'),
///         require('./d')
///       ]).then(([c, d]) => {
///       exports.c = c;
///       exports.d = d;
///     });
///    }
/// }
/// ```
pub fn resource_pot_to_runtime_object(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<RenderedJsResourcePot> {
  let modules = Mutex::new(vec![]);
  let minify_options = context.config.minify.clone().unwrap_or_default();
  let path_filter = PathFilter::new(&minify_options.include, &minify_options.exclude);

  let minify_enabled =
    matches!(minify_options.mode, MinifyMode::Module) && context.config.minify.enabled();
  let is_enabled_minify =
    |module_id: &ModuleId| minify_enabled && path_filter.execute(module_id.relative_path());

  resource_pot
    .modules()
    .into_par_iter()
    .try_for_each(|m_id| {
      let module = module_graph
        .module(m_id)
        .unwrap_or_else(|| panic!("Module not found: {:?}", m_id));

      let mut cloned_module = module.meta.as_script().ast.clone();
      let (cm, _) = create_swc_source_map(Source {
        path: PathBuf::from(m_id.resolved_path_with_query(&context.config.root)),
        content: module.content.clone(),
      });
      let mut external_modules = vec![];
      let comments: SingleThreadedComments = module.meta.as_script().comments.clone().into();
      let minify_enabled = is_enabled_minify(&module.id);

      try_with(cm.clone(), &context.meta.script.globals, || {
        let (unresolved_mark, top_level_mark) = if module.meta.as_script().unresolved_mark == 0
          && module.meta.as_script().top_level_mark == 0
        {
          resolve_module_mark(
            &mut cloned_module,
            module.module_type.is_typescript(),
            context,
          )
        } else {
          let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
          let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
          (unresolved_mark, top_level_mark)
        };

        // replace commonjs require('./xxx') to require('./xxx', true)
        if matches!(
          module.meta.as_script().module_system,
          ModuleSystem::CommonJs | ModuleSystem::Hybrid
        ) {
          cloned_module.visit_mut_with(&mut ExistingCommonJsRequireVisitor::new(
            unresolved_mark,
            top_level_mark,
          ));
        }

        cloned_module.visit_mut_with(&mut paren_remover(Some(&comments)));

        // ESM to commonjs, then commonjs to farm's runtime module systems
        if matches!(
          module.meta.as_script().module_system,
          ModuleSystem::EsModule | ModuleSystem::Hybrid
        ) {
          cloned_module.visit_mut_with(&mut import_analyzer(ImportInterop::Swc, true));
          cloned_module.visit_mut_with(&mut inject_helpers(unresolved_mark));
          cloned_module.visit_mut_with(&mut common_js::<&SingleThreadedComments>(
            unresolved_mark,
            Config {
              ignore_dynamic: true,
              preserve_import_meta: true,
              ..Default::default()
            },
            enable_available_feature_from_es_version(context.config.script.target),
            Some(&comments),
          ));
        }

        // replace import source with module id
        let mut source_replacer = SourceReplacer::new(
          unresolved_mark,
          top_level_mark,
          module_graph,
          m_id.clone(),
          context.config.mode.clone(),
        );
        cloned_module.visit_mut_with(&mut source_replacer);
        cloned_module.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
          top_level_mark,
          ..Default::default()
        }));

        wrap_function(&mut cloned_module, unresolved_mark);

        if matches!(minify_options.mode, MinifyMode::Module)
          && context.config.minify.enabled()
          && path_filter.execute(module.id.relative_path())
        {
          minify_js_module(
            &mut cloned_module,
            cm.clone(),
            &comments,
            unresolved_mark,
            top_level_mark,
            &minify_options,
          );
        }

        cloned_module.visit_mut_with(&mut fixer(Some(&comments)));

        external_modules = source_replacer.external_modules;
      })?;

      // remove shebang
      cloned_module.shebang = None;

      let sourcemap_enabled = context.config.sourcemap.enabled(module.immutable);
      // wrap module function
      // let wrapped_module = wrap_module_ast(cloned_module);
      let mut mappings = vec![];
      let code_bytes = codegen_module(
        &cloned_module,
        context.config.script.target.clone(),
        cm.clone(),
        if sourcemap_enabled {
          Some(&mut mappings)
        } else {
          None
        },
        minify_enabled,
        Some(CodeGenCommentsConfig {
          comments: &comments,
          // preserve all comments when generate module code.
          config: &context.config.comments,
        }),
      )
      .map_err(|e| CompilationError::RenderScriptModuleError {
        id: m_id.to_string(),
        source: Some(Box::new(e)),
      })?;

      let code = Arc::new(String::from_utf8(code_bytes).unwrap());

      let mut rendered_module = RenderedModule {
        id: m_id.clone(),
        rendered_content: code.clone(),
        rendered_map: None,
        rendered_length: code.len(),
        original_length: module.content.len(),
      };
      let mut source_map_chain = vec![];

      if sourcemap_enabled {
        let sourcemap = build_source_map(cm, &mappings);
        let mut buf = vec![];
        sourcemap
          .to_writer(&mut buf)
          .map_err(|e| CompilationError::RenderScriptModuleError {
            id: m_id.to_string(),
            source: Some(Box::new(e)),
          })?;
        let map = Arc::new(String::from_utf8(buf).unwrap());
        rendered_module.rendered_map = Some(map.clone());

        source_map_chain = module.source_map_chain.clone();
        source_map_chain.push(map);
      }

      let mut module = MagicString::new(
        &code,
        Some(MagicStringOptions {
          filename: Some(m_id.resolved_path_with_query(&context.config.root)),
          source_map_chain,
          ..Default::default()
        }),
      );

      module.prepend(&format!("{:?}:", m_id.id(context.config.mode.clone())));
      module.append(",");

      modules.lock().push(RenderedScriptModule {
        id: m_id.clone(),
        module,
        rendered_module,
        external_modules,
      });

      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let mut modules = modules.into_inner();
  modules.sort_by(|a, b| {
    a.id
      .id(context.config.mode.clone())
      .cmp(&b.id.id(context.config.mode.clone()))
  });
  // insert props to the object lit

  let mut bundle = Bundle::new(BundleOptions {
    trace_source_map_chain: Some(true),
    separator: if context.config.minify.enabled() {
      Some('\0')
    } else {
      None
    },
    ..Default::default()
  });
  let mut rendered_modules = HashMap::new();
  let mut external_modules_set = HashSet::new();

  for m in modules {
    bundle.add_source(m.module, None).unwrap();
    rendered_modules.insert(m.id, m.rendered_module);
    external_modules_set.extend(m.external_modules);
  }

  let mut external_modules = external_modules_set.into_iter().collect::<Vec<_>>();
  external_modules.sort();

  bundle.prepend("{");
  bundle.append("}", None);

  Ok(RenderedJsResourcePot {
    bundle,
    rendered_modules,
    external_modules,
  })
}

/// Wrap the module ast to follow Farm's commonjs-style module system.
/// Note: this function won't render the esm to commonjs, if you want to render esm to commonjs, see [common_js].
///
/// For example:
/// ```js
/// const b = farmRequire('./b');
/// console.log(b);
/// exports.b = b;
/// ```
/// will be rendered to
/// ```js
/// function(module, exports, farmRequire) {
///   const b = farmRequire('./b');
///   console.log(b);
///   exports.b = b;
/// }
/// ```
fn wrap_function(module: &mut Module, unresolved_mark: Mark) {
  let body = module.body.take();

  module.body.push(ModuleItem::Stmt(Stmt::Decl(
    farmfe_core::swc_ecma_ast::Decl::Fn(FnDecl {
      ident: " ".into(),
      declare: false,
      function: Box::new(Function {
        params: vec![
          Param {
            span: DUMMY_SP.apply_mark(unresolved_mark),
            decorators: vec![],
            pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
              id: FARM_MODULE.into(),
              type_ann: None,
            }),
          },
          Param {
            span: DUMMY_SP.apply_mark(unresolved_mark),
            decorators: vec![],
            pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
              id: FARM_MODULE_EXPORT.into(),
              type_ann: None,
            }),
          },
          Param {
            span: DUMMY_SP.apply_mark(unresolved_mark),
            decorators: vec![],
            pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
              id: FARM_REQUIRE.into(),
              type_ann: None,
            }),
          },
          Param {
            span: DUMMY_SP.apply_mark(unresolved_mark),
            decorators: vec![],
            pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
              id: FARM_DYNAMIC_REQUIRE.into(),
              type_ann: None,
            }),
          },
        ],
        decorators: vec![],
        span: DUMMY_SP.apply_mark(unresolved_mark),
        body: Some(BlockStmt {
          span: DUMMY_SP.apply_mark(unresolved_mark),
          stmts: body
            .into_iter()
            .map(|body| match body {
              ModuleItem::ModuleDecl(_) => unreachable!(),
              ModuleItem::Stmt(stmt) => stmt,
            })
            .collect(),
        }),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
      }),
    }),
  )));
}

pub struct RenderedScriptModule {
  pub id: ModuleId,
  pub module: MagicString,
  pub rendered_module: RenderedModule,
  pub external_modules: Vec<String>,
}

pub struct RenderedJsResourcePot {
  pub bundle: Bundle,
  pub rendered_modules: HashMap<ModuleId, RenderedModule>,
  pub external_modules: Vec<String>,
}
