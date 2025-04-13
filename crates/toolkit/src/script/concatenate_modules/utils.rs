use farmfe_core::{
  config::TargetEnv,
  module::{
    meta_data::script::{statement::Statement, ModuleExportIdent, EXPORT_EXTERNAL_NAMESPACE},
    ModuleId,
  },
  regex::Regex,
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, BlockStmt, Bool, CallExpr, Callee, ComputedPropName, Decl, EmptyStmt, Expr,
    ExprOrSpread, ExprStmt, GetterProp, Ident, IdentName, KeyValueProp, Lit, MemberExpr,
    MemberProp, ModuleItem, ObjectLit, Pat, Prop, PropName, PropOrSpread, ReturnStmt, Stmt, Str,
    VarDecl, VarDeclKind, VarDeclarator,
  },
  HashMap, HashSet,
};

use super::{
  strip_module_decl::StripModuleDeclResult,
  unique_idents::{EXPORT_DEFAULT, EXPORT_NAMESPACE},
};

// replace the module decl statement to empty statement
pub fn replace_module_decl(
  statement: &Statement,
  result: &mut StripModuleDeclResult,
) -> ModuleItem {
  std::mem::replace(
    &mut result.ast.body[statement.id],
    ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })),
  )
}

pub fn create_var_decl_item(ident: Ident, init: Box<Expr>) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: ident,
        type_ann: None,
      }),
      init: Some(init),
      definite: false,
    }],
    ctxt: SyntaxContext::empty(),
    declare: false,
  }))))
}

/// export default '123' => var module_default = '123';
pub fn create_export_default_expr_item(expr: Box<Expr>, default_ident: Ident) -> ModuleItem {
  create_var_decl_item(default_ident, expr)
}

pub(crate) fn create_var_namespace_item(
  module_id: &ModuleId,
  top_level_mark: Mark,
  export_ident_map: &HashMap<String, ModuleExportIdent>,
  cyclic_idents: &HashSet<ModuleExportIdent>,
  delayed_rename: &mut HashMap<ModuleId, HashSet<ModuleExportIdent>>,
) -> ModuleItem {
  let mut key_ident_vec = export_ident_map.iter().collect::<Vec<_>>();
  key_ident_vec.sort_by_key(|a| a.0);

  let mut props: Vec<PropOrSpread> = key_ident_vec
    .into_iter()
    .filter(|(key, _)| *key != EXPORT_NAMESPACE && *key != EXPORT_EXTERNAL_NAMESPACE)
    .map(|(key, module_export_ident)| {
      delayed_rename
        .entry(module_id.clone())
        .or_default()
        .insert(module_export_ident.clone());

      let ident = module_export_ident.ident.clone();

      let value_expr = Box::new(Expr::Ident(Ident::new(
        ident.sym.clone(),
        DUMMY_SP,
        ident.ctxt(),
      )));

      // for cyclic import, using get method
      let prop = if cyclic_idents.contains(&module_export_ident) {
        Prop::Getter(GetterProp {
          span: DUMMY_SP,
          key: PropName::Ident(key.as_str().into()),
          type_ann: None,
          body: Some(BlockStmt {
            span: DUMMY_SP,
            stmts: vec![Stmt::Return(ReturnStmt {
              span: DUMMY_SP,
              arg: Some(value_expr),
            })],
            ctxt: SyntaxContext::empty(),
          }),
        })
      } else {
        Prop::KeyValue(KeyValueProp {
          key: PropName::Ident(key.as_str().into()),
          value: value_expr,
        })
      };

      PropOrSpread::Prop(Box::new(prop))
    })
    .collect();

  // append __esModule
  let es_module_prop = PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: PropName::Ident("__esModule".into()),
    value: Box::new(Expr::Lit(Lit::Bool(Bool {
      span: DUMMY_SP,
      value: true,
    }))),
  })));

  props.push(es_module_prop);

  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: create_export_namespace_ident(module_id, top_level_mark),
        type_ann: None,
      }),
      init: Some(Box::new(Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props,
      }))),
      definite: false,
    }],
  }))))
}

/// Get the filename from the module id. Replace all non-alphanumeric characters with `_`.
/// For example, `/root/a/b/c.js` will be `c_js` and `a.js` will be `a_js`.
pub fn get_filename(module_id: &ModuleId) -> String {
  Regex::new("[^0-9a-zA-Z]")
    .unwrap()
    .replace_all(module_id.relative_path().split("/").last().unwrap(), "_")
    .to_string()
}

pub fn create_export_default_ident(module_id: &ModuleId, top_level_mark: Mark) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), EXPORT_DEFAULT).into(),
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(top_level_mark),
  )
}

pub fn create_export_namespace_ident(module_id: &ModuleId, top_level_mark: Mark) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), EXPORT_NAMESPACE).into(),
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(top_level_mark),
  )
}

pub fn create_export_external_namespace_ident(module_id: &ModuleId) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), EXPORT_EXTERNAL_NAMESPACE).into(),
    DUMMY_SP,
    SyntaxContext::empty(),
  )
}

// `window[xxx].defineExportStar(module_namespace, node_fs_external_namespace_farm_internal_)`
pub fn create_define_export_star_item(
  namespace: &str,
  target_env: &TargetEnv,
  module_id: &ModuleId,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  export_ident_map: &HashMap<String, ModuleExportIdent>,
) -> ModuleItem {
  let export_external_ident = export_ident_map.get(EXPORT_EXTERNAL_NAMESPACE).unwrap();

  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      ctxt: SyntaxContext::empty(),
      type_args: None,
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            if target_env.is_node() {
              "global".into()
            } else {
              "window".into()
            },
            DUMMY_SP,
            SyntaxContext::empty().apply_mark(unresolved_mark),
          ))),
          prop: MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
              span: DUMMY_SP,
              value: namespace.into(),
              raw: None,
            }))),
          }),
        })),
        prop: MemberProp::Ident(IdentName::new("defineExportStar".into(), DUMMY_SP)),
      }))),
      args: vec![
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(create_export_namespace_ident(
            module_id,
            top_level_mark,
          ))),
        },
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(Ident::new(
            export_external_ident.ident.sym.clone(),
            DUMMY_SP,
            export_external_ident.ident.ctxt(),
          ))),
        },
      ],
    })),
  }))
}
