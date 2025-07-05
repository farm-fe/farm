use std::sync::Arc;

use farmfe_core::{
  config::FARM_REQUIRE,
  context::CompilationContext,
  module::{
    meta_data::script::{
      ScriptModuleMetaData, EXPORT_DEFAULT, FARM_RUNTIME_MODULE_HELPER_ID,
      FARM_RUNTIME_MODULE_SYSTEM_ID,
    },
    ModuleId,
  },
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    CallExpr, Callee, Decl, EsVersion, ExportDecl, ExportDefaultExpr, Expr, ExprOrSpread, ExprStmt,
    Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, ImportStarAsSpecifier, Lit,
    Module as SwcModule, ModuleDecl, ModuleExportName, ModuleItem, Stmt, VarDecl, VarDeclKind,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::{
  script::{
    analyze_statement::analyze_statements, concatenate_modules::EXPORT_NAMESPACE, create_call_expr,
    create_top_level_ident, create_var_decl_item, create_var_declarator, is_commonjs_require,
    wrap_farm_runtime,
  },
  swc_ecma_utils::StmtLikeInjector,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use crate::handle_exports::{create_export_decl_items, update_module_export_ident_map};

pub const FARM_REGISTER: &str = "farmRegister";
pub const FARM_INTEROP_REQUIRE: &str = "interopRequireDefault";
pub const FARM_CJS_EXPORTS: &str = "__farm_cjs_exports__";
pub const FARM_NODE_REQUIRE: &str = "__farmNodeRequire";

/// Transform cjs to esm, for example:
/// ```js
/// // export.ts
/// const esm = require('./esm.mjs');
/// const cjs = require('./cjs.cjs');
/// module.exports = {
///   ...esm,
///   ...cjs,
/// }
/// // index.ts
/// import { foo, zoo } from './exports.ts'
/// ```
/// would be transformed to
/// ```js
/// // export.ts
/// import { farmRegister } from '@farmfe/runtime/module-system';
/// import * as mjs_ns from './esm/mjs';
/// // var { farmRegister } = require('@farmfe/runtime/module-system'); // when output.format is cjs
/// var farmRequire = farmRegister(function (module, exports, require) {
///   const esm = mjs_ns; // require('esm') will be replaced by concatenated namespace ident
///   const cjs = require('./cjs.cjs');
///   module.exports = {
///     ...esm,
///     ...cjs,
///   }
/// });
/// var exports = farmRequire();
/// export var foo = exports.foo, var zoo = exports.zoo;
/// // module.exports = exports; // when output.format js cjs
/// ```
pub fn transform_cjs_to_esm(
  module_id: &ModuleId,
  cjs_import_items: Vec<ModuleItem>,
  meta: &mut ScriptModuleMetaData,
  context: &Arc<CompilationContext>,
  is_entry: bool,
  is_required_cjs_module: bool,
  used_helper_idents: &mut HashSet<&str>,
) {
  let mut items = cjs_import_items;

  let unresolved_mark = Mark::from_u32(meta.unresolved_mark);
  let top_level_mark = Mark::from_u32(meta.top_level_mark);

  // insert module system import
  let mut prepend_imports = vec![create_import_decl_item(
    vec![ImportSpecifier::Named(ImportNamedSpecifier {
      span: DUMMY_SP,
      local: create_top_level_ident(FARM_REGISTER, top_level_mark),
      imported: None,
      is_type_only: false,
    })],
    FARM_RUNTIME_MODULE_SYSTEM_ID,
  )];

  if meta.export_ident_map.contains_key(EXPORT_DEFAULT)
    && !used_helper_idents.contains(FARM_INTEROP_REQUIRE)
  {
    used_helper_idents.insert(FARM_INTEROP_REQUIRE);
  }

  let mut sorted_helper_idents = used_helper_idents.iter().collect::<Vec<_>>();
  sorted_helper_idents.sort();

  // inject runtime helpers at the top of the module
  let specifiers = sorted_helper_idents
    .into_iter()
    .map(|ident| {
      ImportSpecifier::Named(ImportNamedSpecifier {
        span: DUMMY_SP,
        local: create_top_level_ident(ident, top_level_mark),
        imported: None,
        is_type_only: false,
      })
    })
    .collect::<Vec<_>>();

  prepend_imports.push(create_import_decl_item(
    specifiers,
    FARM_RUNTIME_MODULE_HELPER_ID,
  ));

  items.prepend_stmts(prepend_imports);

  // wrap the cjs module with farm module system
  let ast = std::mem::take(&mut meta.ast);
  let wrapped_item = wrap_with_farm_register(
    module_id,
    ast,
    unresolved_mark,
    top_level_mark,
    context,
    is_required_cjs_module,
  );
  items.push(wrapped_item);

  // construct the new ast
  let mut new_ast = SwcModule {
    span: DUMMY_SP,
    shebang: None,
    body: vec![],
  };
  new_ast.body.extend(items);

  meta.ast = new_ast;

  // create export items
  let mut export_items = create_export_decl_items(&mut meta.export_ident_map, top_level_mark);

  // 1. export_items.len() > 0 is true, which means the module is imported by esm
  // 2. export_ident_map contains EXPORT_NAMESPACE, which means the module is imported/exported by esm
  let should_add_cjs_exports =
    export_items.len() > 0 || meta.export_ident_map.contains_key(EXPORT_NAMESPACE);

  if should_add_cjs_exports {
    export_items.insert(0, create_cjs_export_decl_item(top_level_mark));
  }

  // if the cjs module is neither required nor exported by esm, we should executed it by default
  if is_entry || (!is_required_cjs_module && !should_add_cjs_exports) {
    let expr = Box::new(create_call_expr(
      Expr::Ident(create_top_level_ident(FARM_REQUIRE, top_level_mark)),
      vec![],
    ));

    if is_entry && !meta.export_ident_map.contains_key(EXPORT_DEFAULT) {
      export_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
        ExportDefaultExpr {
          span: DUMMY_SP,
          expr,
        },
      )));
    } else if !is_required_cjs_module && !should_add_cjs_exports {
      export_items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr,
      })));
    }
  }

  meta.ast.body.extend(export_items);

  meta.statements = analyze_statements(&meta.ast);

  update_module_export_ident_map(
    module_id,
    &mut meta.export_ident_map,
    top_level_mark,
    is_entry,
    is_required_cjs_module,
    should_add_cjs_exports,
  );
}

pub struct ReplaceCjsRequireResult {
  pub cjs_require_items: Vec<ModuleItem>,
  pub should_add_farm_node_require: bool,
}

pub fn replace_cjs_require(
  module_id: &ModuleId,
  cjs_require_map: &HashMap<(ModuleId, String), ModuleId>,
  meta: &mut ScriptModuleMetaData,
) -> ReplaceCjsRequireResult {
  let unresolved_mark = Mark::from_u32(meta.unresolved_mark);
  let top_level_mark = Mark::from_u32(meta.top_level_mark);

  let mut replacer = RequireEsmReplacer::new(
    module_id.clone(),
    cjs_require_map,
    unresolved_mark,
    top_level_mark,
  );
  meta.ast.visit_mut_with(&mut replacer);

  // insert extra namespace import
  let mut items = replacer
    .extra_import_sources
    .into_iter()
    .map(|(source, ident)| {
      create_import_decl_item(
        vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
          span: DUMMY_SP,
          local: ident,
        })],
        source.as_str(),
      )
    })
    .collect::<Vec<_>>();
  // insert extra import require
  items.extend(
    replacer
      .extra_import_require_sources
      .into_iter()
      .map(|(source, ident)| {
        create_import_decl_item(
          vec![ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: ident,
            imported: Some(ModuleExportName::Ident(create_top_level_ident(
              FARM_REQUIRE,
              top_level_mark,
            ))),
            is_type_only: false,
          })],
          source.as_str(),
        )
      }),
  );

  ReplaceCjsRequireResult {
    cjs_require_items: items,
    should_add_farm_node_require: replacer.should_add_farm_node_require,
  }
}

struct RequireEsmReplacer<'a> {
  module_id: ModuleId,
  cjs_require_map: &'a HashMap<(ModuleId, String), ModuleId>,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  counter: usize,
  require_counter: usize,

  /// import * as __farm_require_replacer_ident__0 from 'source';
  pub extra_import_sources: HashMap<String, Ident>,
  /// import { farmRequire } from 'source';
  pub extra_import_require_sources: HashMap<String, Ident>,
  pub should_add_farm_node_require: bool,
}

impl<'a> RequireEsmReplacer<'a> {
  pub fn new(
    module_id: ModuleId,
    cjs_require_map: &'a HashMap<(ModuleId, String), ModuleId>,
    unresolved_mark: Mark,
    top_level_mark: Mark,
  ) -> Self {
    Self {
      module_id,
      cjs_require_map,
      unresolved_mark,
      top_level_mark,
      counter: 0,
      require_counter: 0,
      extra_import_sources: HashMap::default(),
      extra_import_require_sources: HashMap::default(),
      should_add_farm_node_require: false,
    }
  }

  fn create_export_namespace_ident(&mut self) -> Ident {
    let ident = format!("__farm_require_esm_ident__{}", self.counter);
    self.counter += 1;

    create_top_level_ident(&ident, self.top_level_mark)
  }

  fn create_require_ident(&mut self) -> Ident {
    let ident = format!("__farm_require_cjs_ident_{}", self.require_counter);
    self.require_counter += 1;

    create_top_level_ident(&ident, self.top_level_mark)
  }
}

impl<'a> VisitMut for RequireEsmReplacer<'a> {
  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if ident.sym == "require" && ident.ctxt.outer() == self.unresolved_mark {
      ident.sym = FARM_NODE_REQUIRE.into();
      self.should_add_farm_node_require = true;
    }
  }

  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Expr::Call(call_expr) = expr {
      if is_commonjs_require(self.unresolved_mark, self.top_level_mark, call_expr) {
        if call_expr.args.len() > 0 {
          if let ExprOrSpread {
            expr: box Expr::Lit(Lit::Str(str)),
            ..
          } = &call_expr.args[0]
          {
            let source = str.value.to_string();

            if self
              .cjs_require_map
              .contains_key(&(self.module_id.clone(), source.clone()))
            {
              // import { require } from 'source';
              let ident = self.create_require_ident();
              self
                .extra_import_require_sources
                .insert(source, ident.clone());
              *expr = create_call_expr(Expr::Ident(ident), vec![]);
            } else {
              let ident = self.create_export_namespace_ident();
              // if the dep module is an es module
              if !self.extra_import_sources.contains_key(&source) {
                self.extra_import_sources.insert(source, ident.clone());
              }
              // transform require expr to a local ident
              *expr = Expr::Ident(ident)
            }
          }
        }
      }
    }

    expr.visit_mut_children_with(self);
  }
}

pub fn create_import_decl_item(specifiers: Vec<ImportSpecifier>, source: &str) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers,
    src: Box::new(source.into()),
    type_only: false,
    with: None,
    phase: Default::default(),
  }))
}

/// ```js
/// const esm = require('./esm.mjs');
/// const cjs = require('./cjs.cjs');
/// module.exports = {
///   ...esm,
///   ...cjs,
/// }
///
/// // =>
/// var farmRequire = farmRegister(function (module, exports) {
///   const esm = mjs_ns; // require('esm') will be replaced by concatenated namespace ident
///   const cjs = cjs_require();
///   module.exports = {
///     ...esm,
///     ...cjs,
///   }
/// });
/// ```
fn wrap_with_farm_register(
  module_id: &ModuleId,
  ast: SwcModule,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  context: &Arc<CompilationContext>,
  is_required_cjs_module: bool,
) -> ModuleItem {
  let wrapped_fn = wrap_farm_runtime::wrap_function(
    ast,
    false,
    context.config.script.target == EsVersion::Es5,
    false,
    unresolved_mark,
  );
  let call_register = Expr::Call(CallExpr {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    callee: Callee::Expr(Box::new(Expr::Ident(create_top_level_ident(
      FARM_REGISTER,
      top_level_mark,
    )))),
    args: vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(
          module_id.id(context.config.mode).into(),
        ))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(wrapped_fn),
      },
    ],
    type_args: None,
  });
  let var_id = create_top_level_ident(FARM_REQUIRE, top_level_mark);
  let var_decl = create_var_declarator(var_id, call_register);

  if is_required_cjs_module {
    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
      span: DUMMY_SP,
      decl: Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![var_decl],
      })),
    }))
  } else {
    create_var_decl_item(vec![var_decl])
  }
}

pub fn create_cjs_export_decl_item(top_level_mark: Mark) -> ModuleItem {
  // create export specifiers
  let var_id = create_top_level_ident(FARM_CJS_EXPORTS, top_level_mark);
  let var_export_decl = create_var_declarator(
    var_id,
    create_call_expr(
      Expr::Ident(create_top_level_ident(FARM_REQUIRE, top_level_mark)),
      vec![],
    ),
  );

  create_var_decl_item(vec![var_export_decl])
}
