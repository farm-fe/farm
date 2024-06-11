use std::{collections::HashMap, ffi::OsStr};

use farmfe_core::{
  regex::Regex,
  swc_common::{util::take::Take, Mark, DUMMY_SP},
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, CallExpr, Callee, Class, ClassDecl,
    ClassExpr, Decl, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr, Expr,
    ExprOrSpread, ExprStmt, FnDecl, FnExpr, Function, Id, Ident, ImportDecl, ImportSpecifier,
    KeyValueProp, Lit, MemberExpr, MemberProp, Module as SwcModule, ModuleDecl, ModuleExportName,
    ModuleItem, NamedExport, Pat, Prop, SimpleAssignTarget, Stmt, Str, VarDecl, VarDeclKind,
    VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::defined_idents_collector::DefinedIdentsCollector,
  swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith, VisitWith},
};

const FARM_MODULE_SYSTEM_MODULE: &str = "module";
const FARM_MODULE_SYSTEM_REQUIRE: &str = "require";
const FARM_MODULE_SYSTEM_DEFAULT: &str = "default";
const FARM_MODULE_SYSTEM_EXPORTS: &str = "exports";

/// Transform import statement to cjs require/exports. Farm doesn't use swc commonjs transformer because it's output is too large.
/// Example, transform:
/// ```js
/// import { a, c as d } from "./a";
/// import * as b from "./b";
/// import e from "./c";
///
/// export { a, d, b, e };
/// export default 'hello';
/// export const f = 1;
/// export function g() {}
/// export * from "./d";
/// ```
/// To:
/// ```js
/// var ra = require("./a");
/// var b = require("./b");
/// var re = require("./c");
///
/// exports.a = ra.a;
/// exports.d = ra.c;
/// exports.b = b;
/// exports.e = re.default;
/// exports.default = 'hello';
/// exports.f = 1;
/// exports.g = function g() {};
///
/// module._e(exports, require("./d"));
///
/// ```
pub fn transform_module_decls(ast: &mut SwcModule, unresolved_mark: Mark) {
  let mut items = vec![];
  // all import items should be placed at the top of the module
  let mut import_items = vec![];
  let mut import_bindings_map = HashMap::new();
  let mut is_es_module = false;

  for item in ast.body.take() {
    match item {
      ModuleItem::ModuleDecl(module_decl) => {
        is_es_module = true;

        match module_decl {
          ModuleDecl::Import(import_decl) => {
            import_items.extend(transform_import_decl(
              import_decl,
              unresolved_mark,
              &mut import_bindings_map,
            ));
          }
          ModuleDecl::ExportDecl(export_decl) => {
            items.extend(transform_export_decl(export_decl, unresolved_mark));
          }
          ModuleDecl::ExportNamed(export_named) => {
            items.extend(transform_export_named(export_named, unresolved_mark));
          }
          ModuleDecl::ExportDefaultDecl(default_decl) => {
            items.extend(transform_export_default_decl(default_decl, unresolved_mark));
          }
          ModuleDecl::ExportDefaultExpr(export_expr) => {
            items.extend(transform_export_default_expr(export_expr, unresolved_mark));
          }
          ModuleDecl::ExportAll(export_all) => {
            items.extend(transform_export_all(export_all, unresolved_mark));
          }
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }
      ModuleItem::Stmt(stmt) => items.push(ModuleItem::Stmt(stmt)),
    }
  }

  import_items.extend(items);
  let mut items = import_items;

  let mut handler = ImportBindingsHandler::new(import_bindings_map);

  for item in &mut items {
    if let ModuleItem::Stmt(stmt) = item {
      stmt.visit_mut_with(&mut handler);
    }
  }

  if is_es_module {
    items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
      span: DUMMY_SP,
      expr: Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: create_module_helper_callee("_m"),
        args: vec![ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
        }],
        type_args: None,
      })),
    })))
  }

  ast.body = items;
}

fn transform_import_decl(
  import_decl: ImportDecl,
  unresolved_mark: Mark,
  import_bindings_map: &mut HashMap<Id, MemberExpr>,
) -> Vec<ModuleItem> {
  let mut items = vec![];

  if import_decl.specifiers.is_empty() {
    items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
      span: DUMMY_SP,
      expr: create_require_call_expr(*import_decl.src, unresolved_mark),
    })));
    return items;
  }

  // 1. push var val_name = require(src);
  let (require_item, val_name_ident) = create_require_stmt(*import_decl.src, unresolved_mark);
  items.push(require_item);

  for specifier in import_decl.specifiers {
    match specifier {
      ImportSpecifier::Named(specifier) => {
        // 2. push var specifier.local = val_name.imported;
        let specifier_ident = specifier.local;
        let init = if let Some(imported) = specifier.imported {
          let imported_ident = get_ident_from_module_export_name(imported);
          MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(val_name_ident.clone())),
            prop: MemberProp::Ident(imported_ident),
          }
        } else {
          MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(val_name_ident.clone())),
            prop: MemberProp::Ident(Ident::new(specifier_ident.sym.clone(), DUMMY_SP)),
          }
        };

        import_bindings_map.insert(specifier_ident.to_id(), init);
      }
      ImportSpecifier::Default(specifier) => {
        let init = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(val_name_ident.clone())),
          prop: MemberProp::Ident(Ident::new(FARM_MODULE_SYSTEM_DEFAULT.into(), DUMMY_SP)),
        };
        import_bindings_map.insert(specifier.local.to_id(), init);
      }
      ImportSpecifier::Namespace(specifier) => {
        items.push(create_var_decl_stmt(
          specifier.local,
          Box::new(Expr::Ident(val_name_ident.clone())),
        ));
      }
    }
  }

  items
}

fn transform_export_decl(export_decl: ExportDecl, unresolved_mark: Mark) -> Vec<ModuleItem> {
  let mut items = vec![];

  match export_decl.decl {
    Decl::Class(class_decl) => items.extend(create_export_class_decl_stmts(
      Some(class_decl.ident.clone()),
      class_decl.ident,
      class_decl.class,
      unresolved_mark,
    )),
    Decl::Fn(fn_decl) => items.extend(create_export_fn_decl_stmts(
      Some(fn_decl.ident.clone()),
      fn_decl.ident,
      fn_decl.function,
      unresolved_mark,
    )),
    Decl::Var(var_decls) => {
      let mut local_items = vec![];

      for var_decl in &var_decls.decls {
        let mut idents_collector = DefinedIdentsCollector::new();
        var_decl.name.visit_with(&mut idents_collector);

        for ident in idents_collector.defined_idents {
          let exports_assign_left =
            create_exports_assign_left(Ident::new(ident.0.clone(), DUMMY_SP), unresolved_mark);
          local_items.push(create_exports_assign_stmt(
            exports_assign_left,
            Expr::Ident(Ident::new(ident.0, DUMMY_SP.with_ctxt(ident.1))),
          ))
        }
      }

      items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decls))));
      items.extend(local_items);
    }
    _ => unreachable!("invalid export decl when rendering module system"),
  }

  items
}

fn transform_export_named(named_export: NamedExport, unresolved_mark: Mark) -> Vec<ModuleItem> {
  let mut items = vec![];
  let mut export_from_item = None;

  if let Some(src) = named_export.src {
    let (require_item, val_name_ident) = create_require_stmt(*src, unresolved_mark);
    items.push(require_item);
    export_from_item = Some(val_name_ident);
  }

  for export_specifier in named_export.specifiers {
    match export_specifier {
      farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(specifier) => {
        let ident = get_ident_from_module_export_name(specifier.name);
        items.push(create_var_decl_stmt(
          ident,
          Box::new(Expr::Ident(export_from_item.as_ref().unwrap().clone())),
        ));
      }
      farmfe_core::swc_ecma_ast::ExportSpecifier::Named(specifier) => {
        let (exported_ident, local_ident) = if let Some(exported) = specifier.exported {
          let exported_ident = get_ident_from_module_export_name(exported);
          let local_ident = get_ident_from_module_export_name(specifier.orig);
          (exported_ident, local_ident)
        } else {
          let orig_ident = get_ident_from_module_export_name(specifier.orig);
          (orig_ident.clone(), orig_ident)
        };

        if let Some(export_from_ident) = export_from_item.as_ref() {
          // module._(exports, exported_ident, export_from_ident, local_ident)
          let callee = create_module_helper_callee("_");
          let call_expr = Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee,
            args: vec![
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
              },
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                  span: DUMMY_SP,
                  value: exported_ident.sym,
                  raw: None,
                }))),
              },
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(export_from_ident.clone())),
              },
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                  span: DUMMY_SP,
                  value: local_ident.sym,
                  raw: None,
                }))),
              },
            ],
            type_args: None,
          });
          items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(call_expr),
          })));
        } else {
          let exports_assign_left = create_exports_assign_left(exported_ident, unresolved_mark);
          items.push(create_exports_assign_stmt(
            exports_assign_left,
            Expr::Ident(local_ident),
          ));
        }
      }
      farmfe_core::swc_ecma_ast::ExportSpecifier::Default(_) => {
        unreachable!("`export v from 'mod'` is invalid")
      }
    }
  }

  items
}

fn transform_export_default_decl(
  default_decl: ExportDefaultDecl,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];

  match default_decl.decl {
    farmfe_core::swc_ecma_ast::DefaultDecl::Class(class_decl) => {
      let exported_ident = Ident::new(FARM_MODULE_SYSTEM_DEFAULT.into(), DUMMY_SP);
      items.extend(create_export_class_decl_stmts(
        class_decl.ident,
        exported_ident,
        class_decl.class,
        unresolved_mark,
      ))
    }
    farmfe_core::swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
      let exported_ident = Ident::new(FARM_MODULE_SYSTEM_DEFAULT.into(), DUMMY_SP);
      items.extend(create_export_fn_decl_stmts(
        fn_decl.ident,
        exported_ident,
        fn_decl.function,
        unresolved_mark,
      ));
    }
    farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
  }

  items
}

fn transform_export_default_expr(
  export_expr: ExportDefaultExpr,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];
  let exports_assign_left = create_exports_assign_left(
    Ident::new(FARM_MODULE_SYSTEM_DEFAULT.into(), DUMMY_SP),
    unresolved_mark,
  );
  items.push(create_exports_assign_stmt(
    exports_assign_left,
    *export_expr.expr,
  ));
  items
}

fn transform_export_all(export_all: ExportAll, unresolved_mark: Mark) -> Vec<ModuleItem> {
  let mut items = vec![];
  let (require_item, val_name_ident) = create_require_stmt(*export_all.src, unresolved_mark);
  items.push(require_item);

  // module._e(exports, val_name_ident)
  let callee = create_module_helper_callee("_e");
  let call_expr = Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee,
    args: vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(val_name_ident.clone())),
      },
    ],
    type_args: None,
  });
  items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(call_expr),
  })));

  items
}

fn get_name_from_src(src: &str) -> String {
  let path = std::path::PathBuf::from(src);
  let regex = Regex::new("[^A-Za-z0-9_]").unwrap();
  let val_name = path
    .file_stem()
    .unwrap_or(OsStr::new("_"))
    .to_string_lossy()
    .to_string();
  let val_name = regex.replace_all(&val_name, "_");

  assert!(val_name.len() > 0);

  format!("_f_{}", val_name)
}

fn create_var_decl_stmt(val_name_ident: Ident, init: Box<Expr>) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: val_name_ident,
        type_ann: None,
      }),
      init: Some(init),
      definite: false,
    }],
  }))))
}

fn create_require_call_expr(src: Str, unresolved_mark: Mark) -> Box<Expr> {
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
      FARM_MODULE_SYSTEM_REQUIRE.into(),
      DUMMY_SP.apply_mark(unresolved_mark),
    )))),
    args: vec![ExprOrSpread {
      spread: None,
      expr: Box::new(Expr::Lit(Lit::Str(src))),
    }],
    type_args: None,
  }))
}

fn create_require_stmt(src: Str, unresolved_mark: Mark) -> (ModuleItem, Ident) {
  let val_name_ident = create_require_val_ident(src.value.as_str());
  (
    create_var_decl_stmt(
      val_name_ident.clone(),
      create_require_call_expr(src, unresolved_mark),
    ),
    val_name_ident,
  )
}

fn create_require_val_ident(src: &str) -> Ident {
  let val_name = get_name_from_src(src);
  Ident::new(val_name.into(), DUMMY_SP.apply_mark(Mark::new()))
}

fn create_exports_assign_left(exported_ident: Ident, unresolved_mark: Mark) -> AssignTarget {
  AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
    prop: MemberProp::Ident(exported_ident),
  }))
}

fn create_exports_ident(unresolved_mark: Mark) -> Ident {
  Ident::new(
    FARM_MODULE_SYSTEM_EXPORTS.into(),
    DUMMY_SP.apply_mark(unresolved_mark),
  )
}

fn create_exports_assign_stmt(
  exports_assign_left: AssignTarget,
  export_assign_right: Expr,
) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Assign(AssignExpr {
      span: DUMMY_SP,
      op: AssignOp::Assign,
      left: exports_assign_left,
      right: Box::new(export_assign_right),
    })),
  }))
}

fn create_export_fn_decl_stmts(
  fn_ident: Option<Ident>,
  exports_ident: Ident,
  function: Box<Function>,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];
  // 1. create fn decl item
  let exports_assign_right = if let Some(ident) = fn_ident {
    let fn_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
      ident: ident.clone(),
      declare: false,
      function,
    })));
    items.push(fn_decl);

    Expr::Ident(ident)
  } else {
    Expr::Fn(FnExpr {
      ident: None,
      function,
    })
  };

  // 2. create exports assign item
  let exports_assign_left = create_exports_assign_left(exports_ident, unresolved_mark);
  items.push(create_exports_assign_stmt(
    exports_assign_left,
    exports_assign_right,
  ));

  items
}

fn create_export_class_decl_stmts(
  class_ident: Option<Ident>,
  exports_ident: Ident,
  class: Box<Class>,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];
  // 1. create class decl item
  let exports_assign_right = if let Some(ident) = class_ident {
    let fn_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
      ident: ident.clone(),
      declare: false,
      class,
    })));
    items.push(fn_decl);

    Expr::Ident(ident)
  } else {
    Expr::Class(ClassExpr { ident: None, class })
  };

  // 2. create exports assign item
  let exports_assign_left = create_exports_assign_left(exports_ident, unresolved_mark);
  items.push(create_exports_assign_stmt(
    exports_assign_left,
    exports_assign_right,
  ));

  items
}

fn create_module_helper_callee(helper: &str) -> Callee {
  let prop = Ident::new(helper.into(), DUMMY_SP);
  Callee::Expr(Box::new(Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Ident(Ident::new(
      FARM_MODULE_SYSTEM_MODULE.into(),
      DUMMY_SP,
    ))),
    prop: MemberProp::Ident(prop),
  })))
}

fn get_ident_from_module_export_name(name: ModuleExportName) -> Ident {
  match name {
    ModuleExportName::Ident(ident) => ident,
    ModuleExportName::Str(_) => unreachable!("invalid `str` export as"),
  }
}

struct ImportBindingsHandler {
  import_bindings_map: HashMap<Id, MemberExpr>,
}

impl ImportBindingsHandler {
  pub fn new(import_bindings_map: HashMap<Id, MemberExpr>) -> Self {
    Self {
      import_bindings_map,
    }
  }
}

/// @License MIT SWC Project.
/// This binding handler is modified from swc and modified by brightwu
impl VisitMut for ImportBindingsHandler {
  noop_visit_mut_type!();

  /// replace bar in binding pattern
  /// input:
  /// ```JavaScript
  /// const foo = { bar }
  /// ```
  /// output:
  /// ```JavaScript
  /// const foo = { bar: baz }
  /// ```
  fn visit_mut_prop(&mut self, n: &mut Prop) {
    if let Prop::Shorthand(shorthand) = n {
      if let Some(expr) = self.import_bindings_map.get(&shorthand.to_id()) {
        *n = KeyValueProp {
          key: shorthand.take().into(),
          value: Box::new(Expr::Member(expr.clone())),
        }
        .into()
      }
    } else {
      n.visit_mut_children_with(self)
    }
  }

  fn visit_mut_expr(&mut self, n: &mut Expr) {
    if let Expr::Ident(ident) = n {
      let id = ident.to_id();
      if let Some(member_expr) = self.import_bindings_map.get(&id) {
        *n = Expr::Member(member_expr.clone());
      }
    } else {
      n.visit_mut_children_with(self);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{swc_common::Globals, swc_ecma_ast::EsVersion, swc_ecma_parser::Syntax};
  use farmfe_toolkit::{
    common::{create_swc_source_map, Source},
    script::{codegen_module, parse_module, swc_try_with::try_with},
  };

  use super::*;

  #[test]
  fn test_get_name_from_src() {
    assert_eq!(get_name_from_src("./a"), "_f_a");
    assert_eq!(get_name_from_src("./a.js"), "_f_a");
    assert_eq!(get_name_from_src("./a.jsx"), "_f_a");
    assert_eq!(get_name_from_src("b"), "_f_b");
    assert_eq!(get_name_from_src("src/index.css"), "_f_index");
    assert_eq!(get_name_from_src("src/button-comp.css"), "_f_button_comp");
  }

  #[test]
  fn test_transform_module_decls() {
    let path = "any";
    let content = r#"
import { a, c as d, default as de } from "./a";
import * as b from "./b";
import e from "./c";

console.log(de);

export { a, d, b, e };
export { a1, d1, b1, e1 as e2} from './d';
export * as b2 from './d';

export const f = 1, h = 2;
export function g() {}
export class i {}

export default 'hello';
export default class j {}
export default function k() {}

export * from './e';
    "#;
    let (cm, _) = create_swc_source_map(Source {
      path: std::path::PathBuf::from(path),
      content: Arc::new(content.to_string()),
    });
    let mut ast = parse_module(
      path,
      content,
      Syntax::Es(Default::default()),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;

    try_with(cm.clone(), &Globals::new(), || {
      transform_module_decls(&mut ast, Mark::new());

      let code_bytes =
        codegen_module(&mut ast, EsVersion::latest(), cm, None, false, None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      println!("{}", code);

      assert_eq!(
        code,
        r#"var _f_a = require("./a");
var _f_b = require("./b");
var b = _f_b;
var _f_c = require("./c");
console.log(_f_a.default);
exports.a = _f_a.a;
exports.d = _f_a.c;
exports.b = b;
exports.e = _f_c.default;
var _f_d = require('./d');
module._(exports, "a1", _f_d, "a1");
module._(exports, "d1", _f_d, "d1");
module._(exports, "b1", _f_d, "b1");
module._(exports, "e2", _f_d, "e1");
var _f_d = require('./d');
var b2 = _f_d;
const f = 1, h = 2;
exports.f = f;
exports.h = h;
function g() {}
exports.g = g;
class i {
}
exports.i = i;
exports.default = 'hello';
class j {
}
exports.default = j;
function k() {}
exports.default = k;
var _f_e = require('./e');
module._e(exports, _f_e);
"#
      )
    })
    .unwrap();
  }
}
