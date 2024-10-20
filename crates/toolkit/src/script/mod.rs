use std::{path::PathBuf, sync::Arc};

use swc_ecma_codegen::{
  text_writer::{JsWriter, WriteJs},
  Emitter, Node,
};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax, TsConfig};

use farmfe_core::{
  config::{comments::CommentsConfig, ScriptParserConfig},
  context::CompilationContext,
  error::{CompilationError, Result},
  module::{ModuleSystem, ModuleType},
  plugin::{PluginFinalizeModuleHookParam, ResolveKind},
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    BytePos, FileName, LineCol, Mark, SourceMap,
  },
  swc_ecma_ast::{
    CallExpr, Callee, EsVersion, Expr, Ident, Import, MemberProp, Module as SwcModule, ModuleItem,
    Stmt,
  },
};
use swc_ecma_visit::{Visit, VisitWith};
use swc_error_reporters::handler::try_with_handler;

use crate::common::{create_swc_source_map, minify_comments, Source};

pub use farmfe_toolkit_plugin_types::swc_ast::ParseScriptModuleResult;

use self::swc_try_with::try_with;

pub mod defined_idents_collector;
pub mod swc_try_with;

/// parse the content of a module to [SwcModule] ast.
pub fn parse_module(
  id: &str,
  content: &str,
  syntax: Syntax,
  target: EsVersion,
) -> Result<ParseScriptModuleResult> {
  let (cm, source_file) = create_swc_source_map(Source {
    path: PathBuf::from(id),
    content: Arc::new(content.to_string()),
  });

  let input = StringInput::from(&*source_file);
  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(syntax, target, input, Some(&comments));

  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module();
  let mut recovered_errors = parser.take_errors();

  match module {
    Err(err) => {
      recovered_errors.push(err);
    }
    Ok(m) => {
      return Ok(ParseScriptModuleResult { ast: m, comments });
    }
  }

  try_with_handler(cm, Default::default(), |handler| {
    for err in recovered_errors {
      err.into_diagnostic(handler).emit();
    }

    Err(anyhow::Error::msg("SyntaxError"))
  })
  .map_err(|e| CompilationError::ParseError {
    resolved_path: id.to_string(),
    msg: if let Some(s) = e.downcast_ref::<String>() {
      s.to_string()
    } else if let Some(s) = e.downcast_ref::<&str>() {
      s.to_string()
    } else {
      "failed to handle with unknown panic message".to_string()
    },
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
      resolved_path: id.to_string(),
      msg: format!("{e:?}"),
    })
}

pub struct CodeGenCommentsConfig<'a> {
  pub comments: &'a SingleThreadedComments,
  pub config: &'a CommentsConfig,
}

/// ast codegen, return generated utf8 bytes. using [String::from_utf8] if you want to transform the bytes to string.
/// Example:
/// ```ignore
/// let bytes = codegen(swc_ast, cm);
/// let code = String::from_utf8(bytes).unwrap();
/// ```
pub fn codegen_module(
  ast: &SwcModule,
  target: EsVersion,
  cm: Arc<SourceMap>,
  src_map: Option<&mut Vec<(BytePos, LineCol)>>,
  minify: bool,
  comments_cfg: Option<CodeGenCommentsConfig>,
) -> std::result::Result<Vec<u8>, std::io::Error> {
  let mut buf = vec![];

  {
    let wr = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, src_map)) as Box<dyn WriteJs>;
    let cfg = swc_ecma_codegen::Config::default()
      .with_minify(minify)
      .with_target(target)
      .with_omit_last_semi(true)
      .with_ascii_only(false);

    if let Some(comments_cfg) = &comments_cfg {
      minify_comments(comments_cfg.comments, comments_cfg.config);
    }

    let comments = comments_cfg.map(|c| c.comments as &dyn Comments);

    let mut emitter = Emitter {
      cfg,
      comments,
      cm,
      wr,
    };

    ast.emit_with(&mut emitter)?;
  }

  Ok(buf)
}

/// Get [ModuleType] from the resolved id's extension, return [ModuleType::Custom(ext)] if the extension is not internally supported.
/// Panic if the id do not has a extension.
pub fn module_type_from_id(id: &str) -> Option<ModuleType> {
  let path = PathBuf::from(id);

  path.extension().map(|ext| ext.to_str().unwrap().into())
}

/// return [None] if module type is not script
pub fn syntax_from_module_type(
  module_type: &ModuleType,
  config: ScriptParserConfig,
) -> Option<Syntax> {
  match module_type {
    ModuleType::Js => Some(Syntax::Es(EsConfig {
      jsx: false,
      import_attributes: true,
      ..config.es_config
    })),
    ModuleType::Jsx => Some(Syntax::Es(EsConfig {
      jsx: true,
      import_attributes: true,
      ..config.es_config
    })),
    ModuleType::Ts => Some(Syntax::Typescript(TsConfig {
      tsx: false,
      ..config.ts_config
    })),
    ModuleType::Tsx => Some(Syntax::Typescript(TsConfig {
      tsx: true,
      ..config.ts_config
    })),
    _ => None,
  }
}

/// Whether the call expr is commonjs require.
/// A call expr is commonjs require if:
/// * callee is an identifier named `require`
/// * arguments is a single string literal
/// * require is a global variable
pub fn is_commonjs_require(
  unresolved_mark: Mark,
  top_level_mark: Mark,
  call_expr: &CallExpr,
) -> bool {
  if let Callee::Expr(box Expr::Ident(Ident { span, sym, .. })) = &call_expr.callee {
    sym == "require"
      && (span.ctxt.outer() == unresolved_mark || span.ctxt.outer() == top_level_mark)
  } else {
    false
  }
}

/// Whether the call expr is dynamic import.
pub fn is_dynamic_import(call_expr: &CallExpr) -> bool {
  matches!(&call_expr.callee, Callee::Import(Import { .. }))
}

pub fn module_system_from_deps(deps: Vec<ResolveKind>) -> ModuleSystem {
  let mut module_system = ModuleSystem::Custom(String::from("unknown"));

  for resolve_kind in deps {
    if matches!(resolve_kind, ResolveKind::Import)
      || matches!(resolve_kind, ResolveKind::DynamicImport)
      || matches!(resolve_kind, ResolveKind::ExportFrom)
    {
      match module_system {
        ModuleSystem::EsModule => continue,
        ModuleSystem::CommonJs => {
          module_system = ModuleSystem::Hybrid;
          break;
        }
        _ => module_system = ModuleSystem::EsModule,
      }
    } else if matches!(resolve_kind, ResolveKind::Require) {
      match module_system {
        ModuleSystem::CommonJs => continue,
        ModuleSystem::EsModule => {
          module_system = ModuleSystem::Hybrid;
          break;
        }
        _ => module_system = ModuleSystem::CommonJs,
      }
    }
  }

  module_system
}

struct ModuleSystemAnalyzer {
  unresolved_mark: Mark,
  contain_module_exports: bool,
  contain_esm: bool,
}

impl Visit for ModuleSystemAnalyzer {
  fn visit_stmts(&mut self, n: &[Stmt]) {
    if self.contain_module_exports || self.contain_esm {
      return;
    }

    n.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, n: &farmfe_core::swc_ecma_ast::MemberExpr) {
    if self.contain_module_exports {
      return;
    }

    if let box Expr::Ident(Ident { sym, span, .. }) = &n.obj {
      if sym == "module" && span.ctxt.outer() == self.unresolved_mark {
        if let MemberProp::Ident(Ident { sym, .. }) = &n.prop {
          if sym == "exports" {
            self.contain_module_exports = true;
          }
        }
      } else if sym == "exports" && span.ctxt.outer() == self.unresolved_mark {
        self.contain_module_exports = true;
      } else {
        n.visit_children_with(self);
      }
    } else {
      n.visit_children_with(self);
    }
  }

  fn visit_module_decl(&mut self, n: &farmfe_core::swc_ecma_ast::ModuleDecl) {
    if self.contain_esm {
      return;
    }

    self.contain_esm = true;

    n.visit_children_with(self);
  }
}

pub fn module_system_from_ast(
  ast: &SwcModule,
  module_system: ModuleSystem,
  has_deps: bool,
) -> ModuleSystem {
  if module_system != ModuleSystem::Hybrid {
    // if the ast contains ModuleDecl, it's a esm module
    for item in ast.body.iter() {
      if let ModuleItem::ModuleDecl(_) = item {
        if module_system == ModuleSystem::CommonJs {
          return ModuleSystem::Hybrid;
        } else {
          return ModuleSystem::EsModule;
        }
      }
    }
  }

  module_system
}

pub fn set_module_system_for_module_meta(
  param: &mut PluginFinalizeModuleHookParam,
  context: &Arc<CompilationContext>,
) {
  // default to commonjs
  let module_system_from_deps_option = if !param.deps.is_empty() {
    module_system_from_deps(param.deps.iter().map(|d| d.kind.clone()).collect())
  } else {
    ModuleSystem::UnInitial
  };

  // param.module.meta.as_script_mut().module_system = module_system.clone();

  let ast = &param.module.meta.as_script().ast;

  let mut module_system_from_ast: ModuleSystem = ModuleSystem::UnInitial;
  {
    // try_with(param.module.meta.as_script().comments.into(), globals, op)

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(&param.module.id.to_string()),
      content: param.module.content.clone(),
    });

    try_with(cm, &context.meta.script.globals, || {
      let unresolved_mark = Mark::from_u32(param.module.meta.as_script().unresolved_mark);
      let mut analyzer = ModuleSystemAnalyzer {
        unresolved_mark,
        contain_module_exports: false,
        contain_esm: false,
      };

      ast.visit_with(&mut analyzer);

      if analyzer.contain_module_exports {
        module_system_from_ast = module_system_from_ast.merge(ModuleSystem::CommonJs);
      }

      if analyzer.contain_esm {
        module_system_from_ast = module_system_from_ast.merge(ModuleSystem::EsModule);
      }
    })
    .unwrap();
  }

  let mut v = [module_system_from_deps_option, module_system_from_ast]
    .into_iter()
    .reduce(|a, b| a.merge(b))
    .unwrap_or(ModuleSystem::UnInitial);

  if matches!(v, ModuleSystem::UnInitial) {
    v = ModuleSystem::Hybrid;
  }

  param.module.meta.as_script_mut().module_system = v;
}
