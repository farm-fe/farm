use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  config::ModuleFormat,
  context::CompilationContext,
  error::Result,
  module::{ModuleId, ModuleSystem},
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    self, ArrayLit, BindingIdent, Bool, CallExpr, Decl, Expr, ExprOrSpread, Ident, IdentName,
    KeyValueProp, ModuleItem, ObjectLit, Pat, Prop, PropName, PropOrSpread, Stmt, Str, VarDecl,
    VarDeclKind, VarDeclarator,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_reference::{BundleReference, ExternalReferenceExport},
    reference::{ReferenceExport, ReferenceMap},
    ModuleAnalyzerManager,
  },
  modules_analyzer::module_analyzer::ImportSpecifierInfo,
  polyfill::{Polyfill, SimplePolyfill},
  uniq_name::BundleVariable,
};

use super::{cjs::generate::CjsGenerate, esm::generate::EsmGenerate};

/// namespace
pub fn generate_namespace_by_reference_map(
  module_id: &ModuleId,
  local: usize,
  bundle_variable: &BundleVariable,
  bundle_reference: &mut BundleReference,
  map: &ReferenceMap,
  module_analyzer_manager: &ModuleAnalyzerManager,
  order_index_map: &HashMap<ModuleId, usize>,
  polyfill: &mut SimplePolyfill,
) -> Result<Vec<ModuleItem>> {
  let mut patch_ast_items = vec![];
  let namespace = bundle_variable.name(local);

  let mut props: Vec<PropOrSpread> = vec![];
  let mut commonjs_fns: Vec<Ident> = vec![];
  let mut reexport_namespace: Vec<Ident> = vec![];

  {
    generate_export_as_object_prop(&mut props, &map.export, bundle_variable);
  }

  let mut module_ids = map.reexport_map.keys().collect::<Vec<_>>();

  module_ids.sort_by_key(|a| &order_index_map[a]);

  for module_id in module_ids {
    let reference_export = &map.reexport_map[module_id];

    if module_analyzer_manager.is_external(module_id) {
      if reference_export.is_empty() || reference_export.all {
        let ns_index = module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(module_id)
          .unwrap();

        bundle_reference.add_import(
          &ImportSpecifierInfo::Namespace(ns_index),
          module_id.clone().into(),
          bundle_variable,
        )?;

        reexport_namespace.push(bundle_variable.name(ns_index).as_str().into());
        continue;
      }

      // TODO: export import from external
      generate_export_as_object_prop(&mut props, reference_export, bundle_variable);
    } else if module_analyzer_manager.is_commonjs(module_id) {
      if reference_export.is_empty() || reference_export.all {
        commonjs_fns.push(
          bundle_variable
            .name(
              module_analyzer_manager
                .module_global_uniq_name
                .commonjs_name(module_id)
                .unwrap(),
            )
            .as_str()
            .into(),
        );
        continue;
      }

      generate_export_as_object_prop(&mut props, reference_export, bundle_variable)
    }
  }

  if module_analyzer_manager.is_hybrid_or_esm(module_id) {
    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Ident(IdentName::from("__esModule")),
      value: Box::new(Expr::Lit(swc_ecma_ast::Lit::Bool(Bool {
        span: DUMMY_SP,
        value: true,
      }))),
    }))));
  }

  let declare_init = if matches!(map.export_type, ModuleSystem::EsModule)
    && reexport_namespace.is_empty()
    && commonjs_fns.is_empty()
  {
    Some(Box::new(Expr::Object(ObjectLit {
      span: DUMMY_SP,
      props,
    })))
  } else {
    polyfill.add(Polyfill::MergeNamespace);

    // dynamic
    Some(Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(Ident::from("_mergeNamespaces")))),
      args: vec![
        // static
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
          })),
        },
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Array(ArrayLit {
            span: DUMMY_SP,
            elems: commonjs_fns
              .into_iter()
              .map(|ident| {
                Some(ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(ident))),
                    args: vec![],
                    type_args: None,
                    ctxt: SyntaxContext::empty(),
                  })),
                })
              })
              .chain(reexport_namespace.into_iter().map(|ns| {
                Some(ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Ident(ns)),
                })
              }))
              .collect(),
          })),
        },
      ],
      type_args: None,
      ctxt: SyntaxContext::empty(),
    })))
  };

  patch_ast_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: Ident::new(namespace.as_str().into(), DUMMY_SP, SyntaxContext::empty()),
        type_ann: None,
      }),
      init: declare_init,
      definite: false,
    }],
    ctxt: SyntaxContext::empty(),
  })))));
  Ok(patch_ast_items)
}

pub fn generate_export_as_object_prop(
  props: &mut Vec<PropOrSpread>,
  reference_export: &ReferenceExport,
  bundle_variable: &BundleVariable,
) {
  let mut exported_ordered_names = reference_export
    .named
    .keys()
    .map(|i| (bundle_variable.name(*i), i))
    .collect::<Vec<_>>();

  exported_ordered_names.sort_by(|(a, _), (b, _)| a.cmp(b));

  for (exported_name, exported_index) in &exported_ordered_names {
    let local = &reference_export.named[*exported_index];

    let local_ident = bundle_variable.render_name(*local);

    // maybe as short, but need legacy
    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Ident(exported_name.as_str().into()),
      value: Box::new(Expr::Ident(Ident::from(local_ident.as_str()))),
    }))));
  }

  if let Some(default) = reference_export.default {
    let default_ident = bundle_variable.render_name(default);
    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Str(Str::from("default")),
      value: Box::new(Expr::Ident(Ident::from(default_ident.as_str()))),
    }))));
  }

  if let Some(ns) = reference_export.namespace {
    let namespace = bundle_variable.var_by_index(ns);

    let ns_key = namespace.origin_name();
    let ns_value = namespace.render_name();

    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Str(ns_key.as_str().into()),
      value: Box::new(Expr::Ident(ns_value.as_str().into())),
    }))));
  }
}

/// generate bundle export
pub fn generate_export_by_reference_export(
  resource_pot_id: &str,
  bundle_variable: &BundleVariable,
  bundle_reference: &mut BundleReference,
  module_analyzer_manager: &ModuleAnalyzerManager,
  context: &Arc<CompilationContext>,
  polyfill: &mut SimplePolyfill,
) -> Result<Vec<ModuleItem>> {
  let mut patch_export_to_module = vec![];

  if let Some(export) = bundle_reference.export.as_ref() {
    patch_export_to_module.extend(generate_export_as_module_export(
      resource_pot_id,
      None,
      export,
      bundle_variable,
      module_analyzer_manager,
      context,
      polyfill,
    )?);
  }

  let mut ordered_external_export = bundle_reference
    .external_export_map
    .keys()
    .collect::<Vec<_>>();

  ordered_external_export.sort_by_key(|a| a.to_url());

  for source in ordered_external_export {
    let export = &bundle_reference.external_export_map[source];

    patch_export_to_module.extend(generate_export_as_module_export(
      resource_pot_id,
      Some(&source.to_module_id()),
      export,
      bundle_variable,
      module_analyzer_manager,
      context,
      polyfill,
    )?);
  }

  Ok(patch_export_to_module)
}

pub fn generate_export_as_module_export(
  _resource_pot_name: &str,
  source: Option<&ModuleId>,
  export: &ExternalReferenceExport,
  bundle_variable: &BundleVariable,
  module_analyzer_manager: &ModuleAnalyzerManager,
  context: &Arc<CompilationContext>,
  polyfill: &mut SimplePolyfill,
) -> Result<Vec<ModuleItem>> {
  let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

  ordered_keys.sort_by_key(|a| bundle_variable.name(**a));

  match (&export.module_system, context.config.output.format) {
    // hybrid dynamic es module cannot support, if hybrid, only export static export
    (_, ModuleFormat::EsModule) => {
      EsmGenerate::generate_export(source, export, bundle_variable, module_analyzer_manager)
    }

    (_, ModuleFormat::CommonJs) => CjsGenerate::generate_export(
      source,
      export,
      bundle_variable,
      module_analyzer_manager,
      polyfill,
    ),
  }
}

/// generate bundle import

pub fn generate_bundle_import_by_bundle_reference(
  format: &ModuleFormat,
  bundle_variable: &BundleVariable,
  bundle_reference: &BundleReference,
  module_analyzer_manager: &ModuleAnalyzerManager,
  polyfill: &mut SimplePolyfill,
  resource_pot_id: &str,
) -> Result<Vec<ModuleItem>> {
  let mut patch_import_to_module = vec![];

  match format {
    ModuleFormat::CommonJs => {
      patch_import_to_module.extend(CjsGenerate::generate_import(
        bundle_variable,
        &bundle_reference.import_map,
        module_analyzer_manager,
        polyfill,
        resource_pot_id,
      )?);
    }

    ModuleFormat::EsModule => {
      patch_import_to_module.extend(EsmGenerate::generate_import(
        bundle_variable,
        &bundle_reference.import_map,
      )?);
    }
  }

  Ok(patch_import_to_module)
}
