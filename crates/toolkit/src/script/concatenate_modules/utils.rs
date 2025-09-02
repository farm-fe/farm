use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{Statement, SwcId},
      ModuleExportIdent, ModuleExportIdentType, AMBIGUOUS_EXPORT_ALL,
      FARM_RUNTIME_MODULE_HELPER_ID, FARM_RUNTIME_MODULE_SYSTEM_ID,
    },
    ModuleId,
  },
  regex::Regex,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    BinExpr, BinaryOp, BindingIdent, BlockStmt, Bool, CallExpr, Callee, Decl, EmptyStmt, ExportAll,
    ExportNamedSpecifier, ExportSpecifier, Expr, ExprOrSpread, ExprStmt, GetterProp, Ident,
    IdentName, ImportDecl, ImportNamedSpecifier, ImportSpecifier, ImportStarAsSpecifier,
    KeyValueProp, Lit, MemberExpr, MemberProp, ModuleDecl, ModuleExportName, ModuleItem,
    NamedExport, ObjectLit, Pat, Prop, PropName, PropOrSpread, ReturnStmt, Stmt, Str, VarDecl,
    VarDeclKind, VarDeclarator,
  },
  HashMap, HashSet,
};

use super::{
  strip_module_decl::StripModuleDeclResult,
  unique_idents::{TopLevelIdentsRenameHandler, EXPORT_DEFAULT, EXPORT_NAMESPACE},
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
  export_ident_map: &HashMap<String, ModuleExportIdent>,
  cyclic_idents: &HashSet<(Option<SwcId>, ModuleExportIdent)>,
  rename_handler: &TopLevelIdentsRenameHandler,
) -> ModuleItem {
  let mut key_ident_vec = export_ident_map.iter().collect::<Vec<_>>();
  key_ident_vec.sort_by_key(|a| a.0);

  let cyclic_idents = cyclic_idents
    .iter()
    .map(|(_, module_export_ident)| module_export_ident)
    .collect::<HashSet<_>>();

  let mut props: Vec<PropOrSpread> = key_ident_vec
    .into_iter()
    .filter(|(key, module_export_ident)| {
      *key != EXPORT_NAMESPACE
        && *key != AMBIGUOUS_EXPORT_ALL
        && matches!(
          module_export_ident.as_internal().export_type,
          ModuleExportIdentType::Declaration
        )
    })
    .map(|(key, org_module_export_ident)| {
      let module_export_ident = org_module_export_ident.as_internal();

      let renamed_ident = rename_handler
        .get_renamed_ident(&module_export_ident.module_id, &module_export_ident.ident)
        .or(rename_handler.get_renamed_ident(module_id, &key.as_str().into()))
        .unwrap_or(module_export_ident.ident.clone());

      let value_expr = Box::new(Expr::Ident(renamed_ident.into()));

      // for cyclic import, using get method
      let prop = if cyclic_idents.contains(&org_module_export_ident) {
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
        id: create_export_namespace_ident(module_id), // It's renamed when handling import/export * as
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

pub fn create_export_default_ident(module_id: &ModuleId) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), EXPORT_DEFAULT).into(),
    DUMMY_SP,
    SyntaxContext::empty(),
  )
}

pub fn create_export_namespace_ident(module_id: &ModuleId) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), EXPORT_NAMESPACE).into(),
    DUMMY_SP,
    SyntaxContext::empty(),
  )
}

pub fn create_ambiguous_export_all_ident(module_id: &ModuleId) -> Ident {
  Ident::new(
    format!("{}_{}", get_filename(module_id), AMBIGUOUS_EXPORT_ALL).into(),
    DUMMY_SP,
    SyntaxContext::empty(),
  )
}

pub fn create_import_stmt(
  specifiers: Vec<ImportSpecifier>,
  source_module_id: &ModuleId,
) -> ModuleItem {
  let import_decl = ImportDecl {
    span: DUMMY_SP,
    specifiers,
    src: Box::new(Str {
      span: DUMMY_SP,
      value: source_module_id.to_string().into(),
      raw: None,
    }),
    type_only: false,
    with: None,
    phase: Default::default(),
  };

  ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl))
}

pub fn create_import_specifiers(idents: Vec<(SwcId, String)>) -> Vec<ImportSpecifier> {
  idents
    .into_iter()
    .map(|(id, name)| {
      ImportSpecifier::Named(ImportNamedSpecifier {
        span: DUMMY_SP,
        imported: if id.sym.as_str() != name.as_str() {
          Some(ModuleExportName::Ident(name.into()))
        } else {
          None
        },
        local: id.into(),
        is_type_only: false,
      })
    })
    .collect()
}

/// insert `import * as external_all_farm_internal_ from 'module';`
pub fn create_import_external_namespace_stmt(
  ident: Ident,
  source_module_id: &ModuleId,
) -> ModuleItem {
  create_import_stmt(
    vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
      span: DUMMY_SP,
      local: ident,
    })],
    source_module_id,
  )
}

pub fn create_define_export_star_ident() -> Ident {
  Ident::new("defineExportStar".into(), DUMMY_SP, SyntaxContext::empty())
}

pub fn create_farm_register_ident() -> Ident {
  Ident::new("farmRegister".into(), DUMMY_SP, SyntaxContext::empty())
}

/// `import { defineExportStar } from '@farmfe/runtime/src/modules/module-helper`
pub fn create_import_farm_define_export_helper_stmt() -> ModuleItem {
  create_import_stmt(
    vec![ImportSpecifier::Named(ImportNamedSpecifier {
      span: DUMMY_SP,
      local: create_define_export_star_ident(),
      imported: None,
      is_type_only: false,
    })],
    &FARM_RUNTIME_MODULE_HELPER_ID.into(),
  )
}

/// `import { farmRegister } from '@farmfe/runtime/src/module-system`
pub fn create_import_farm_register_helper_stmt() -> ModuleItem {
  create_import_stmt(
    vec![ImportSpecifier::Named(ImportNamedSpecifier {
      span: DUMMY_SP,
      local: create_farm_register_ident(),
      imported: None,
      is_type_only: false,
    })],
    &FARM_RUNTIME_MODULE_SYSTEM_ID.into(),
  )
}

/// `defineExportStar(module_namespace, node_fs_external_namespace_farm_internal_)`
pub fn create_define_export_star_item(module_id: &ModuleId, ident: Ident) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      ctxt: SyntaxContext::empty(),
      type_args: None,
      callee: Callee::Expr(Box::new(Expr::Ident(create_define_export_star_ident()))),
      args: vec![
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(create_export_namespace_ident(module_id))),
        },
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(ident)),
        },
      ],
    })),
  }))
}

/// export * from 'external-module'
pub fn create_export_all_item(module_id: &ModuleId) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
    span: DUMMY_SP,
    src: Box::new(Str {
      span: DUMMY_SP,
      value: module_id.to_string().into(),
      raw: None,
    }),
    type_only: false,
    with: None,
  }))
}

pub fn generate_export_decl_item(
  module_export_ident: Vec<(String, ModuleExportIdent)>,
  rename_handler: &TopLevelIdentsRenameHandler,
) -> ModuleItem {
  // add preserved export decl
  let mut specifiers = vec![];

  for (name, id) in module_export_ident {
    let id = id.as_internal();

    if !matches!(
      id.export_type,
      ModuleExportIdentType::Declaration
        | ModuleExportIdentType::External
        | ModuleExportIdentType::Unresolved
    ) {
      continue;
    }

    let renamed_ident = rename_handler
      .get_renamed_ident(&id.module_id, &id.ident)
      .unwrap_or(id.ident.clone());
    let ctxt = renamed_ident.ctxt();

    specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
      span: DUMMY_SP,
      orig: ModuleExportName::Ident(Ident::new(renamed_ident.sym, DUMMY_SP, ctxt)),
      exported: Some(ModuleExportName::Ident(Ident::new(
        name.as_str().into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      ))),
      is_type_only: false,
    }));
  }

  ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
    span: DUMMY_SP,
    specifiers,
    src: None,
    type_only: false,
    with: None,
  }))
}

pub fn should_add_namespace_ident(
  module_id: &ModuleId,
  export_ident_map: &HashMap<String, ModuleExportIdent>,
) -> bool {
  // append:
  // ```js
  // var module_namespace = {
  //    default: a,
  //    ns: e_js_namespace_farm_internal_
  // }
  // ```
  // if module is used by export * as or import * as or import('...')
  let should_add_export_namespace_item =
    if let Some(module_export_ident) = export_ident_map.get(EXPORT_NAMESPACE) {
      let module_export_ident = module_export_ident.as_internal();
      // the ident should equal to the default ident, otherwise, it means the namespace ident is existed and should not be added
      module_export_ident.ident == create_export_namespace_ident(&module_id).to_id().into()
        && module_export_ident.module_id == *module_id
    } else {
      false
    };

  should_add_export_namespace_item
}

pub fn create_bin_expr(mut exprs: Vec<Expr>, op: BinaryOp) -> Expr {
  if exprs.len() == 1 {
    return exprs.remove(0);
  }

  let left = exprs.remove(0);

  Expr::Bin(BinExpr {
    span: DUMMY_SP,
    op,
    left: Box::new(left),
    right: Box::new(create_bin_expr(exprs, op)),
  })
}

pub fn create_member_expr(ident: &SwcId, prop: &str) -> Expr {
  Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Ident(Ident::new(
      ident.sym.clone(),
      DUMMY_SP,
      ident.ctxt(),
    ))),
    prop: MemberProp::Ident(IdentName::new(prop.into(), DUMMY_SP)),
  })
}
