use std::{path::PathBuf, sync::Arc};

use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax, TsConfig};

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleType},
  resource::resource_pot::ResourcePot,
  swc_common::{comments::SingleThreadedComments, FileName, Mark, SourceMap, DUMMY_SP, GLOBALS},
  swc_ecma_ast::{
    BlockStmt, EsVersion, Expr, FnExpr, Function, Ident, KeyValueProp, Module as SwcModule,
    ModuleItem, ObjectLit, Param, Pat, Prop, PropName, PropOrSpread, Stmt, Str,
  },
};
use swc_ecma_transforms::{
  feature::enable_available_feature_from_es_version,
  fixer,
  modules::{common_js, import_analysis::import_analyzer, util::ImportInterop},
  resolver,
};
use swc_ecma_visit::VisitMutWith;

use self::source_replacer::SourceReplacer;

pub mod source_replacer;

/// parse the content of a module to [SwcModule] ast.
pub fn parse_module(
  id: &str,
  content: &str,
  syntax: Syntax,
  cm: Arc<SourceMap>,
) -> Result<SwcModule> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let input = StringInput::from(&*source_file);
  // TODO support parsing comments
  let mut parser = Parser::new(syntax, input, None);
  parser
    .parse_module()
    .map_err(|e| CompilationError::ParseError {
      id: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
    })
}

/// parse the content of a module to [SwcModule] ast.
pub fn parse_stmt(
  id: &str,
  content: &str,
  syntax: Syntax,
  cm: Arc<SourceMap>,
  top_level: bool,
) -> Result<Stmt> {
  let source_file = cm.new_source_file(FileName::Real(PathBuf::from(id)), content.to_string());
  let input = StringInput::from(&*source_file);
  // TODO support parsing comments
  let mut parser = Parser::new(syntax, input, None);
  parser
    .parse_stmt(top_level)
    .map_err(|e| CompilationError::ParseError {
      id: id.to_string(),
      source: Some(Box::new(CompilationError::GenericError(format!("{:?}", e))) as _),
    })
}

/// ast codegen, return generated utf8 bytes. using [String::from_utf8] if you want to transform the bytes to string.
/// Example:
/// ```ignore
/// let bytes = codegen(swc_ast, cm);
/// let code = String::from_utf8(bytes).unwrap();
/// ```
pub fn codegen_module(
  ast: &SwcModule,
  cm: Arc<SourceMap>,
) -> std::result::Result<Vec<u8>, std::io::Error> {
  let mut buf = vec![];

  {
    // TODO support source map
    let wr = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)) as Box<dyn WriteJs>;

    let mut emitter = Emitter {
      cfg: swc_ecma_codegen::Config {
        target: Default::default(),
        ascii_only: false,
        minify: false,
      },
      comments: None,
      cm,
      wr,
    };

    ast.emit_with(&mut emitter)?;
  }

  Ok(buf)
}

/// Get [ModuleType] from the resolved id's extension, return [ModuleType::Custom(ext)] if the extension is not internally supported.
/// Panic if the id do not has a extension.
pub fn module_type_from_id(id: &str) -> ModuleType {
  let path = PathBuf::from(id);

  match path
    .extension()
    .unwrap_or_else(|| panic!("extension of {} is None", id))
    .to_str()
    .unwrap()
  {
    "ts" => ModuleType::Ts,
    "tsx" => ModuleType::Tsx,
    "js" | "mjs" | "cjs" => ModuleType::Js,
    "jsx" => ModuleType::Jsx,
    "css" => ModuleType::Css,
    "html" => ModuleType::Html,
    ext => ModuleType::Custom(ext.to_string()),
  }
}

/// TODO support custom [EsConfig] and [TsConfig]
/// return [None] if module type is not script
pub fn syntax_from_module_type(module_type: &ModuleType) -> Option<Syntax> {
  match module_type {
    ModuleType::Js => Some(Syntax::Es(Default::default())),
    ModuleType::Jsx => Some(Syntax::Es(EsConfig {
      jsx: true,
      ..Default::default()
    })),
    ModuleType::Ts => Some(Syntax::Typescript(Default::default())),
    ModuleType::Tsx => Some(Syntax::Typescript(TsConfig {
      tsx: true,
      ..Default::default()
    })),
    _ => None,
  }
}

/// Wrap the module ast to follow Farm's commonjs-style module system spec.
/// Note: this function won't render the esm to commonjs, if you want to render esm to commonjs, see [esm_to_commonjs].
///
/// For example:
/// ```js
/// const b = require('./b');
/// console.log(b);
/// exports.b = b;
/// ```
/// will be rendered to
/// ```js
/// async function(module, exports, require) {
///   const b = require('./b');
///   console.log(b);
///   exports.b = b;
/// }
/// ```
pub fn wrap_module_ast(ast: SwcModule) -> Function {
  let params = vec!["module", "exports", "require"]
    .into_iter()
    .map(|ident| Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: Pat::Ident(
        Ident {
          span: DUMMY_SP,
          sym: ident.into(),
          optional: false,
        }
        .into(),
      ),
    })
    .collect();

  let body = Some(BlockStmt {
    span: DUMMY_SP,
    stmts: ast
      .body
      .to_vec()
      .into_iter()
      .map(|item| match item {
        ModuleItem::ModuleDecl(_) => {
          panic!("should transform all esm module item to commonjs first!")
        }
        ModuleItem::Stmt(stmt) => stmt,
      })
      .collect(),
  });

  Function {
    params,
    decorators: vec![],
    span: DUMMY_SP,
    body,
    is_generator: false,
    // TODO, make the module async to support top level await
    is_async: false,
    type_params: None,
    return_type: None,
  }
}

/// merge all modules in a [ResourcePot] ast to Farm's runtime [ObjectLit]
pub fn merge_module_asts_of_resource_pot(
  resource_pot: &mut ResourcePot,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) -> ObjectLit {
  let mut rendered_resource_ast = ObjectLit {
    span: DUMMY_SP,
    props: vec![],
  };

  for m_id in resource_pot.modules() {
    let module = module_graph.module(m_id).unwrap();
    let mut cloned_module = SwcModule {
      shebang: None,
      span: DUMMY_SP,
      body: module.meta.as_script().ast.body.to_vec(),
    };

    GLOBALS.set(&context.meta.script.globals, || {
      // transform esm to commonjs
      let top_level_mark = Mark::new();
      let unresolved_mark = Mark::new();

      cloned_module.visit_mut_with(&mut import_analyzer(ImportInterop::None, true));
      cloned_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
      cloned_module.visit_mut_with(&mut common_js::<SingleThreadedComments>(
        unresolved_mark,
        Default::default(),
        enable_available_feature_from_es_version(EsVersion::Es2017),
        None,
      ));
      // replace import source with module id
      let mut source_replacer = SourceReplacer::new(
        unresolved_mark,
        module_graph,
        m_id.clone(),
        context.config.mode.clone(),
      );
      cloned_module.visit_mut_with(&mut source_replacer);
      cloned_module.visit_mut_with(&mut fixer(None));
    });

    // wrap module function
    let wrapped_module = wrap_module_ast(cloned_module);
    // TODO transform async function for legacy browser

    rendered_resource_ast
      .props
      .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Str(Str {
          span: DUMMY_SP,
          value: module.id.id(context.config.mode.clone()).into(),
          raw: None,
        }),
        value: Box::new(Expr::Fn(FnExpr {
          ident: None,
          function: wrapped_module,
        })),
      }))))
  }

  rendered_resource_ast
}
