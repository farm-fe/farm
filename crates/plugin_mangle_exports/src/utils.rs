use farmfe_core::{
  module::{
    meta_data::script::{ModuleReExportIdentType, AMBIGUOUS_EXPORT_ALL, EXPORT_DEFAULT},
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ExportNamedSpecifier, ExportSpecifier, Ident, ModuleDecl, ModuleExportName, ModuleItem,
    NamedExport,
  },
  HashMap,
};

pub fn is_reexport_all(
  reexport_ident_map: &HashMap<String, ModuleReExportIdentType>,
  export: &String,
) -> bool {
  reexport_ident_map
    .get(export)
    .map(|reexport_ident_type| {
      matches!(reexport_ident_type, ModuleReExportIdentType::FromExportAll)
    })
    .unwrap_or(false)
}

pub fn get_reexport_named_local(
  reexport_ident_map: &HashMap<String, ModuleReExportIdentType>,
  export: &String,
) -> Option<String> {
  reexport_ident_map
    .get(export)
    .map(|reexport_ident_type| match reexport_ident_type {
      ModuleReExportIdentType::FromExportAll => None,
      ModuleReExportIdentType::FromExportNamed { local } => Some(local.clone()),
    })
    .unwrap_or(None)
}

/// To avoid name conflict of reexported mangled ident, we should transform export * from to export { xxx }
/// Example:
/// ```js
/// export * from './a'; // Foo, Bar is reexported
/// // =>
/// export { Foo, Bar } from './a';
/// ```
pub fn transform_export_all_to_export_named(module_id: ModuleId, module_graph: &mut ModuleGraph) {
  let module = module_graph.module(&module_id).unwrap();

  if module.external || !module.module_type.is_script() {
    return;
  }

  let meta = module.meta.as_script();
  let mut items_to_replace: HashMap<usize, NamedExport> = HashMap::default();
  let mut items_to_append: Vec<ModuleItem> = vec![];

  for (i, item) in meta.ast.body.iter().enumerate() {
    if let ModuleItem::ModuleDecl(module_decl) = item {
      if let ModuleDecl::ExportAll(export_all) = module_decl {
        let dep_module_id = module_graph.get_dep_by_source(
          &module_id,
          &export_all.src.value,
          Some(ResolveKind::ExportFrom),
        );
        let dep_module = module_graph.module(&dep_module_id).unwrap();

        if dep_module.external || !dep_module.module_type.is_script() {
          continue;
        }

        let dep_meta = dep_module.meta.as_script();

        for (export, _) in &dep_meta.export_ident_map {
          if export == EXPORT_DEFAULT {
            continue;
          }

          let replace_item = items_to_replace.entry(i).or_insert(NamedExport {
            span: DUMMY_SP,
            specifiers: vec![],
            src: Some(export_all.src.clone()),
            type_only: false,
            with: None,
          });
          replace_item
            .specifiers
            .push(ExportSpecifier::Named(ExportNamedSpecifier {
              span: DUMMY_SP,
              orig: ModuleExportName::Ident(Ident::new(
                export.as_str().into(),
                DUMMY_SP,
                SyntaxContext::empty(),
              )),
              exported: None,
              is_type_only: false,
            }));
        }

        // should preserve ambiguous export all statement
        if dep_meta
          .ambiguous_export_ident_map
          .get(AMBIGUOUS_EXPORT_ALL)
          .is_some()
        {
          items_to_append.push(ModuleItem::ModuleDecl(ModuleDecl::ExportAll(
            export_all.clone(),
          )));
        }
      }
    }
  }

  let module = module_graph.module_mut(&module_id).unwrap();
  let meta = module.meta.as_script_mut();

  for (i, named_export) in items_to_replace {
    meta.ast.body[i] = ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named_export));
  }

  meta.ast.body.extend(items_to_append);
}
