use std::collections::{HashMap, HashSet};

use farmfe_core::{
  error::Result,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    ExportAll, ExportNamedSpecifier, ExportNamespaceSpecifier, ExportSpecifier, ImportDecl,
    ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier, ImportStarAsSpecifier,
    ModuleDecl, ModuleExportName, ModuleItem, NamedExport, Str,
  },
};
use farmfe_toolkit::itertools::Itertools;

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_reference::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleAnalyzerManager,
  },
  common::with_bundle_reference_slot_name,
  uniq_name::BundleVariable,
  ShareBundleOptions,
};

pub struct EsmGenerate {}

impl EsmGenerate {
  pub fn generate_export(
    should_reexport_raw: bool,
    source: Option<&ReferenceKind>,
    export: &ExternalReferenceExport,
    bundle_variable: &BundleVariable,
    module_analyzer_manager: &ModuleAnalyzerManager,
    options: &ShareBundleOptions,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut specifiers = vec![];

    let source_url = source.map(|target_id| {
      if let ReferenceKind::Module(target_id) = target_id {
        return if let Some(target_module_analyzer) =
          module_analyzer_manager.module_analyzer(target_id)
        {
          with_bundle_reference_slot_name(
            &target_module_analyzer.bundle_group_id,
            options.reference_slot,
          )
        } else {
          options.format(target_id)
        };
      }

      target_id.to_module_id().to_string()
    });

    let mut uniq_sets = HashSet::new();

    for exported in export
      .named
      .keys()
      .sorted_by_key(|a| bundle_variable.render_name(**a))
    {
      let local = &export.named[exported];
      if bundle_variable.var_by_index(*local).removed {
        continue;
      }

      let named_render_name = bundle_variable.render_name(*local);
      let exported_name = bundle_variable.name(*exported);

      if uniq_sets.contains(&named_render_name) {
        continue;
      }
      uniq_sets.insert(named_render_name.clone());

      let exported_name = if !should_reexport_raw || named_render_name == exported_name {
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
          src: Box::new(source_url.as_ref().unwrap().as_str().into()),
          type_only: false,
          with: None,
        })));
      }
    }

    if let Some(default) = export.default.as_ref() {
      let name = bundle_variable.render_name(*default).as_str().into();

      specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
        span: DUMMY_SP,
        orig: ModuleExportName::Ident(name),
        exported: should_reexport_raw.then(|| ModuleExportName::Ident("default".into())),
        is_type_only: false,
      }));
    }

    if !specifiers.is_empty() {
      stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
        NamedExport {
          span: DUMMY_SP,
          specifiers,
          src: source_url.map(|source| Box::new(source.as_str().into())),
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
    options: &ShareBundleOptions,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut generate_import_specifies: HashMap<String, ImportItem> = HashMap::new();

    for source in import_map.keys().sorted() {
      let mut is_import_uniq_name = false;

      let (_, url) = match source {
        ReferenceKind::Bundle(_) => continue,
        ReferenceKind::Module(m) => {
          if module_analyzer_manager.is_external(m) {
            (m.clone(), m.to_string())
          } else {
            if module_analyzer_manager.contain(m) && !module_analyzer_manager.is_entry(m) {
              is_import_uniq_name = true;
            }

            (
              m.clone(),
              with_bundle_reference_slot_name(
                &module_analyzer_manager
                  .group_id(m)
                  .map(|id| id.to_string())
                  // maybe using group
                  .unwrap_or_else(|| options.format(m)),
                options.reference_slot,
              ),
            )
          }
        }
      };

      let import = &import_map[source];

      let import_item = init_import_specify(&mut generate_import_specifies, &url);

      if import.is_empty() {
        continue;
      }

      for imported in import.named.keys().sorted() {
        let local = import.named[imported];
        let local_named = bundle_variable.render_name(local);

        let imported = if is_import_uniq_name || imported == &local_named {
          None
        } else {
          Some(imported.as_str())
        };

        let used_name = imported.unwrap_or(&local_named).to_string();

        if import_item.contains_key(&used_name) {
          continue;
        }

        import_item.insert(
          used_name,
          ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: local_named.as_str().into(),
            imported: imported.map(|s| ModuleExportName::Ident(s.into())),
            is_type_only: false,
          }),
        );
      }

      if let Some(default) = import.default.as_ref() {
        let name = bundle_variable.render_name(*default);
        if import_item.contains_key(&name) {
          continue;
        }

        import_item.insert(
          name.clone(),
          if is_import_uniq_name {
            ImportSpecifier::Named(ImportNamedSpecifier {
              span: DUMMY_SP,
              local: name.as_str().into(),
              imported: None,
              is_type_only: false,
            })
          } else {
            ImportSpecifier::Default(ImportDefaultSpecifier {
              span: DUMMY_SP,
              local: name.as_str().into(),
            })
          },
        );
      }

      if let Some(namespace) = import.namespace.as_ref() {
        let namespace_name = bundle_variable.render_name(*namespace).as_str().into();
        if import_item.is_empty() {}
        import_item.insert(
          namespace_name,
          ImportSpecifier::Namespace(ImportStarAsSpecifier {
            span: DUMMY_SP,
            local: bundle_variable.render_name(*namespace).as_str().into(),
          }),
        );
      }
    }

    for (url, import_item) in generate_import_specifies
      .into_iter()
      .sorted_by(|a, b| a.0.cmp(&b.0))
    {
      let partial_specifiers = import_item
        .into_iter()
        .sorted_by(|a, b| a.0.cmp(&b.0))
        .fold([vec![], vec![]], |mut res, (_, specifier)| {
          match specifier {
            ImportSpecifier::Namespace(_) => {
              res[1] = vec![specifier];
            }

            _ => res[0].push(specifier),
          }

          res
        });

      let mut partial_specifiers = partial_specifiers
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();

      if partial_specifiers.is_empty() {
        partial_specifiers.push(vec![]);
      }

      for item in partial_specifiers {
        stmts.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
          span: DUMMY_SP,
          specifiers: item,
          src: Box::new(Str {
            span: DUMMY_SP,
            value: url.clone().into(),
            raw: None,
          }),
          type_only: false,
          with: None,
          phase: farmfe_core::swc_ecma_ast::ImportPhase::Evaluation,
        })));
      }
    }

    Ok(stmts)
  }
}

fn init_import_specify<'a>(
  generate_import_specifies: &'a mut HashMap<String, HashMap<String, ImportSpecifier>>,
  url: &String,
) -> &'a mut ImportItem {
  if !generate_import_specifies.contains_key(url) {
    generate_import_specifies.insert(url.clone(), Default::default());
  }

  generate_import_specifies.get_mut(url).unwrap()
}

type ImportItem = HashMap<String, ImportSpecifier>;
