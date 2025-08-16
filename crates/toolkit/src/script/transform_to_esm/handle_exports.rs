use crate::script::{
  concatenate_modules::EXPORT_NAMESPACE, create_call_expr, create_export_default_ident,
  create_top_level_ident, create_var_decl_item, create_var_declarator,
};
use farmfe_core::{
  config::FARM_REQUIRE,
  module::{
    meta_data::script::{
      ModuleExportIdent, ModuleExportIdentType, AMBIGUOUS_EXPORT_ALL, EXPORT_DEFAULT,
    },
    ModuleId,
  },
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ExportNamedSpecifier, ExportSpecifier, Expr, ExprOrSpread, Ident, IdentName, MemberExpr,
    MemberProp, ModuleDecl, ModuleExportName, ModuleItem, NamedExport,
  },
  HashMap,
};

use super::transform_cjs::{FARM_CJS_EXPORTS, FARM_INTEROP_REQUIRE};

pub fn create_export_decl_items(
  export_ident_map: &mut HashMap<String, ModuleExportIdent>,
  top_level_mark: Mark,
) -> Vec<ModuleItem> {
  let mut var_decorators = vec![];
  let mut export_specifier = vec![];

  for (export_name, export_ident) in export_ident_map {
    if export_name == EXPORT_NAMESPACE || export_name == AMBIGUOUS_EXPORT_ALL {
      continue;
    }
    let mut export_ident = export_ident.as_internal_mut();

    export_ident.export_type = ModuleExportIdentType::Declaration;

    let (ident, exported) = if export_name == EXPORT_DEFAULT {
      (export_ident.ident.sym.as_str(), Some(export_name))
    } else {
      (export_name.as_str(), None)
    };

    let var_id = create_top_level_ident(ident, top_level_mark);
    export_ident.ident = var_id.to_id().into();

    let cjs_export_ident = Expr::Ident(create_top_level_ident(FARM_CJS_EXPORTS, top_level_mark));
    // __farm_cjs_export__ or interopRequireDefault
    let var_cjs_export = if export_name == EXPORT_DEFAULT {
      // add interopRequireDefault
      create_call_expr(
        Expr::Ident(create_top_level_ident(FARM_INTEROP_REQUIRE, top_level_mark)),
        vec![ExprOrSpread {
          spread: None,
          expr: Box::new(cjs_export_ident),
        }],
      )
    } else {
      cjs_export_ident
    };
    // __farm_cjs_export__.xxx
    let var_export_expr = Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(var_cjs_export),
      prop: MemberProp::Ident(IdentName::new(export_name.as_str().into(), DUMMY_SP)),
    });

    var_decorators.push(create_var_declarator(var_id.clone(), var_export_expr));

    export_specifier.push(ExportSpecifier::Named(ExportNamedSpecifier {
      span: DUMMY_SP,
      orig: ModuleExportName::Ident(var_id),
      exported: exported.map(|e| {
        ModuleExportName::Ident(Ident::new(
          e.as_str().into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        ))
      }),
      is_type_only: false,
    }));
  }

  let mut items = vec![];

  if !var_decorators.is_empty() {
    items.push(create_var_decl_item(var_decorators));
  }

  if !export_specifier.is_empty() {
    items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
      NamedExport {
        span: DUMMY_SP,
        specifiers: export_specifier,
        src: None,
        type_only: false,
        with: None,
      },
    )));
  }

  items
}

pub fn update_module_export_ident_map(
  module_id: &ModuleId,
  export_ident_map: &mut HashMap<String, ModuleExportIdent>,
  top_level_mark: Mark,
  is_entry: bool,
  is_required_cjs_module: bool,
  should_add_cjs_exports: bool,
) {
  // add require export
  if is_required_cjs_module {
    export_ident_map.insert(
      FARM_REQUIRE.to_string(),
      ModuleExportIdent::new(
        module_id.clone(),
        create_top_level_ident(FARM_REQUIRE, top_level_mark)
          .to_id()
          .into(),
        ModuleExportIdentType::Declaration,
      ),
    );
  }

  // set namespace export to FARM_CJS_EXPORTS
  if should_add_cjs_exports {
    export_ident_map.insert(
      EXPORT_NAMESPACE.to_string(),
      ModuleExportIdent::new(
        module_id.clone(),
        create_top_level_ident(FARM_CJS_EXPORTS, top_level_mark)
          .to_id()
          .into(),
        ModuleExportIdentType::VirtualNamespace,
      ),
    );
  }

  if is_entry && !export_ident_map.contains_key(EXPORT_DEFAULT) {
    export_ident_map.insert(
      EXPORT_DEFAULT.to_string(),
      ModuleExportIdent::new(
        module_id.clone(),
        create_export_default_ident(module_id).to_id().into(),
        ModuleExportIdentType::Declaration,
      ),
    );
  }
}
