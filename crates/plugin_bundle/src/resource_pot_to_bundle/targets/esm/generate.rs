use std::collections::HashMap;

use farmfe_core::{
  error::Result,
  module::ModuleId,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    ExportAll, ExportDefaultExpr, ExportNamedSpecifier, ExportNamespaceSpecifier, Expr, ImportDecl,
    ImportDefaultSpecifier, ImportNamedSpecifier, ImportStarAsSpecifier, ModuleDecl, ModuleItem,
    NamedExport, Str,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_external::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleAnalyzerManager,
  },
  uniq_name::BundleVariable,
};

pub struct EsmGenerate {}

impl EsmGenerate {
  pub fn generate_export(
    source: Option<&ModuleId>,
    export: &ExternalReferenceExport,
    bundle_variable: &BundleVariable,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut specifiers = vec![];
    let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

    ordered_keys.sort_by_key(|a| bundle_variable.name(**a));

    for exported in ordered_keys {
      let local = &export.named[exported];

      let named_render_name = bundle_variable.render_name(*local);
      let exported_name = bundle_variable.name(*exported);

      let exported_name = if named_render_name == exported_name {
        None
      } else {
        Some(exported_name.as_str().into())
      };

      specifiers.push(farmfe_core::swc_ecma_ast::ExportSpecifier::Named(
        ExportNamedSpecifier {
          span: DUMMY_SP,
          orig: farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
            named_render_name.as_str().into(),
          ),
          exported: exported_name.map(farmfe_core::swc_ecma_ast::ModuleExportName::Ident),
          is_type_only: false,
        },
      ));
    }

    if let Some(namespace) = export.namespace.as_ref() {
      specifiers.push(farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(
        ExportNamespaceSpecifier {
          span: DUMMY_SP,
          name: farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
            bundle_variable.name(*namespace).as_str().into(),
          ),
        },
      ));
    }

    if let Some(source) = source {
      if export.all.0 && !module_analyzer_manager.is_commonjs(source) {
        stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
          span: DUMMY_SP,
          src: Box::new(source.to_string().as_str().into()),
          type_only: false,
          with: None,
        })));
      }
    }

    if !specifiers.is_empty() {
      stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
        NamedExport {
          span: DUMMY_SP,
          specifiers,
          src: source.map(|source| Box::new(source.to_string().as_str().into())),
          type_only: false,
          with: None,
        },
      )));
    }

    if let Some(default) = export.default.as_ref() {
      stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
        ExportDefaultExpr {
          span: DUMMY_SP,
          expr: Box::new(Expr::Ident(
            bundle_variable.render_name(*default).as_str().into(),
          )),
        },
      )));
    }

    Ok(stmts)
  }

  pub fn generate_import(
    bundle_variable: &BundleVariable,
    import_map: &HashMap<ReferenceKind, ExternalReferenceImport>,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_import = import_map.keys().collect::<Vec<_>>();
    ordered_import.sort();

    for source in ordered_import {
      let import = &import_map[source];

      if import.named.is_empty() && import.namespace.is_none() && import.default.is_none() {
        continue;
      }

      let create_import = |specifiers: Vec<farmfe_core::swc_ecma_ast::ImportSpecifier>| {
        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
          span: DUMMY_SP,
          specifiers,
          src: Box::new(Str {
            span: DUMMY_SP,
            value: source.to_string().as_str().into(),
            raw: None,
          }),
          type_only: false,
          with: None,
          phase: farmfe_core::swc_ecma_ast::ImportPhase::Evaluation,
        }))
      };

      let mut specifiers = vec![];

      let mut ordered_named_keys = import.named.keys().collect::<Vec<_>>();
      ordered_named_keys.sort();
      for imported in ordered_named_keys {
        let local = &import.named[imported];
        let local_named = bundle_variable.render_name(*local);

        specifiers.push(farmfe_core::swc_ecma_ast::ImportSpecifier::Named(
          ImportNamedSpecifier {
            span: DUMMY_SP,
            local: local_named.as_str().into(),
            imported: if imported == &local_named {
              None
            } else {
              Some(farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
                imported.as_str().into(),
              ))
            },
            is_type_only: false,
          },
        ));
      }

      if let Some(namespace) = import.namespace.as_ref() {
        stmts.push(create_import(vec![
          farmfe_core::swc_ecma_ast::ImportSpecifier::Namespace(ImportStarAsSpecifier {
            span: DUMMY_SP,
            local: bundle_variable.render_name(*namespace).as_str().into(),
          }),
        ]));
      }

      if let Some(default) = import.default.as_ref() {
        specifiers.push(farmfe_core::swc_ecma_ast::ImportSpecifier::Default(
          ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: bundle_variable.render_name(*default).as_str().into(),
          },
        ));
      }

      if !specifiers.is_empty() {
        stmts.push(create_import(specifiers));
      }
    }

    Ok(stmts)
  }
}
