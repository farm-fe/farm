use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    Expr, ExprOrSpread, Ident, IdentName, ImportNamedSpecifier, ImportSpecifier, MemberExpr,
    MetaPropExpr, MetaPropKind, ModuleItem, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::{
    create_call_expr, create_var_decl_item,
    swc_try_with::try_with,
    transform_to_esm::transform_cjs::{create_import_decl_item, FARM_NODE_REQUIRE},
  },
  swc_ecma_utils::StmtLikeInjector,
};

use crate::{
  formats::GenerateLibraryFormatResourcesOptions,
  utils::{add_format_to_generated_resources, emit_resource_pot},
};

pub fn emit_esm_resources(
  resource_pot: &mut ResourcePot,
  options: &GenerateLibraryFormatResourcesOptions,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  // prepend:
  //  `import { createRequire as __farmNodeCreateRequire } from 'module';`
  //  `const __farmNodeRequire = __farmNodeCreateRequire(import.meta.url);`

  if options.should_add_farm_node_require {
    let local = create_farm_node_create_require_ident();
    let import_decl = create_import_decl_item(
      vec![ImportSpecifier::Named(ImportNamedSpecifier {
        span: DUMMY_SP,
        local: local.clone(),
        imported: Some(Ident::new("createRequire".into(), DUMMY_SP, SyntaxContext::empty()).into()),
        is_type_only: false,
      })],
      "module",
    );

    let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
    let globals = context.meta.get_resource_pot_globals(&resource_pot.id);

    try_with(cm, globals.value(), || {
      let meta = resource_pot.meta.as_js_mut();
      let require_decl = create_farm_node_require_item(
        create_node_create_require_call_expr(local),
        Mark::from_u32(meta.top_level_mark),
      );

      meta.ast.body.prepend_stmts(vec![import_decl, require_decl]);
    })
    .unwrap();
  }

  let mut resources = emit_resource_pot(resource_pot, context, hook_context)?;

  add_format_to_generated_resources(&mut resources, "esm");

  if options.should_add_farm_node_require {
    // revert ast
    let ast = &mut resource_pot.meta.as_js_mut().ast;
    ast.body.remove(1);
    ast.body.remove(0);
  }

  Ok(resources)
}

fn create_farm_node_create_require_ident() -> Ident {
  Ident::new(
    "__farmNodeCreateRequire".into(),
    DUMMY_SP,
    SyntaxContext::empty(),
  )
}

fn create_node_create_require_call_expr(local: Ident) -> Expr {
  create_call_expr(
    Expr::Ident(local),
    vec![ExprOrSpread {
      spread: None,
      expr: Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::MetaProp(MetaPropExpr {
          span: DUMMY_SP,
          kind: MetaPropKind::ImportMeta,
        })),
        prop: farmfe_core::swc_ecma_ast::MemberProp::Ident(IdentName {
          sym: "url".into(),
          span: DUMMY_SP,
        }),
      })),
    }],
  )
}

pub fn create_farm_node_require_item(expr: Expr, top_level_mark: Mark) -> ModuleItem {
  create_var_decl_item(vec![VarDeclarator {
    span: DUMMY_SP,
    name: farmfe_core::swc_ecma_ast::Pat::Ident(
      Ident::new(
        FARM_NODE_REQUIRE.into(),
        DUMMY_SP,
        SyntaxContext::empty().apply_mark(top_level_mark),
      )
      .into(),
    ),
    init: Some(Box::new(expr)),
    definite: false,
  }])
}
