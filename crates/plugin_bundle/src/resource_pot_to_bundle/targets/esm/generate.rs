use std::collections::{HashMap, HashSet};

use farmfe_core::{
  error::Result,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    ExportAll, ExportDefaultExpr, ExportNamedSpecifier, ExportNamespaceSpecifier, ExportSpecifier,
    Expr, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier,
    ImportStarAsSpecifier, ModuleDecl, ModuleExportName, ModuleItem, NamedExport, Str,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_reference::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleAnalyzerManager,
  },
  common::with_bundle_reference_slot_name,
  uniq_name::BundleVariable,
};

pub struct EsmGenerate {}

impl EsmGenerate {
  pub fn generate_export(
    source: Option<&ReferenceKind>,
    export: &ExternalReferenceExport,
    bundle_variable: &BundleVariable,
    module_analyzer_manager: &ModuleAnalyzerManager,
    // should_reexport_uniq: bool,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut specifiers = vec![];
    let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

    ordered_keys.sort_by_key(|a| bundle_variable.name(**a));
    let mut uniq_sets = HashSet::new();

    let index_is_entry = |i: usize| {
      bundle_variable
        .module_id_by_var_index(i)
        .is_some_and(|m| !module_analyzer_manager.is_entry(m))
    };

    for exported in ordered_keys {
      let local = &export.named[exported];
      if bundle_variable.var_by_index(*local).removed {
        continue;
      }

      let should_reexport_uniq = index_is_entry(*local);

      let named_render_name = bundle_variable.render_name(*local);
      let exported_name = bundle_variable.name(*exported);

      if uniq_sets.contains(&named_render_name) {
        continue;
      }
      uniq_sets.insert(named_render_name.clone());

      let exported_name = if should_reexport_uniq || named_render_name == exported_name {
        None
      } else {
        Some(exported_name.as_str().into())
      };

      specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
        span: DUMMY_SP,
        orig: ModuleExportName::Ident(named_render_name.as_str().into()),
        exported: exported_name.map(ModuleExportName::Ident),
        is_type_only: false,
      }));
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

    if let Some(ReferenceKind::Module(source)) = source {
      if export.all.0 && !module_analyzer_manager.is_commonjs(source) {
        stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
          span: DUMMY_SP,
          src: Box::new(source.to_string().as_str().into()),
          type_only: false,
          with: None,
        })));
      }
    }

    if let Some(default) = export.default.as_ref() {
      let name = bundle_variable.render_name(*default).as_str().into();
      if index_is_entry(*default) {
        specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
          span: DUMMY_SP,
          orig: ModuleExportName::Ident(name),
          exported: None,
          is_type_only: false,
        }));
      } else {
        stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
          ExportDefaultExpr {
            span: DUMMY_SP,
            expr: Box::new(Expr::Ident(name)),
          },
        )));
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

    Ok(stmts)
  }

  pub fn generate_import(
    bundle_variable: &BundleVariable,
    import_map: &HashMap<ReferenceKind, ExternalReferenceImport>,
    module_analyzer_manager: &ModuleAnalyzerManager,
    resource_pot_name: &str,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_import_keys = import_map.keys().collect::<Vec<_>>();
    ordered_import_keys.sort();
    let mut generate_import_specifies = HashMap::new();

    for source in ordered_import_keys {
      let mut is_import_uniq_name = false;

      let url = match source {
        ReferenceKind::Bundle(s) => with_bundle_reference_slot_name(s),
        ReferenceKind::Module(m) => {
          if module_analyzer_manager.is_external(m) {
            m.to_string()
          } else {
            if !module_analyzer_manager.is_entry(m) {
              is_import_uniq_name = true;
            }

            with_bundle_reference_slot_name(
              &module_analyzer_manager
                .module_analyzer(m)
                .map(|m| m.resource_pot_id.clone())
                .unwrap(),
            )
          }
        }
      };

      let import = &import_map[source];
      fn init_import_specify<'a>(
        generate_import_specifies: &'a mut HashMap<String, Vec<ImportSpecifier>>,
        url: &String,
      ) -> &'a mut Vec<ImportSpecifier> {
        if generate_import_specifies.contains_key(url) {
          generate_import_specifies.get_mut(url).unwrap()
        } else {
          generate_import_specifies.insert(url.clone(), vec![]);
          generate_import_specifies.get_mut(url).unwrap()
        }
      }

      let specifiers = init_import_specify(&mut generate_import_specifies, &url);

      if import.is_empty() {
        continue;
      }

      let mut ordered_named_keys = import.named.keys().collect::<Vec<_>>();
      ordered_named_keys.sort();

      for imported in ordered_named_keys {
        let local = import.named[imported];
        let local_named = bundle_variable.render_name(local);

        let imported = imported.to_string();

        specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
          span: DUMMY_SP,
          local: local_named.as_str().into(),
          imported: if is_import_uniq_name || imported == local_named {
            None
          } else {
            Some(ModuleExportName::Ident(imported.as_str().into()))
          },
          is_type_only: false,
        }));
      }

      if let Some(namespace) = import.namespace.as_ref() {
        specifiers.push(ImportSpecifier::Namespace(ImportStarAsSpecifier {
          span: DUMMY_SP,
          local: bundle_variable.render_name(*namespace).as_str().into(),
        }));
      }

      if let Some(default) = import.default.as_ref() {
        if is_import_uniq_name {
          specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: bundle_variable.render_name(*default).as_str().into(),
            imported: None,
            is_type_only: false,
          }));
        } else {
          specifiers.push(ImportSpecifier::Default(ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: bundle_variable.render_name(*default).as_str().into(),
          }));
        }
      }

      // add_import(specifiers)
    }

    for (url, specifiers) in generate_import_specifies {
      stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers,
        src: Box::new(Str {
          span: DUMMY_SP,
          value: url.into(),
          raw: None,
        }),
        type_only: false,
        with: None,
        phase: farmfe_core::swc_ecma_ast::ImportPhase::Evaluation,
      })));
    }

    Ok(stmts)
  }
}
