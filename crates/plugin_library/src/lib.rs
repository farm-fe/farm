use farmfe_core::{
  config::Config,
  error::CompilationError,
  plugin::Plugin,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::ResourcePotType,
  },
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ExportNamedSpecifier, ExportSpecifier, Ident, ModuleDecl, ModuleExportName, ModuleItem,
    NamedExport,
  },
};
use farmfe_toolkit::script::{
  concatenate_modules::{concatenate_modules_ast, ConcatenateModulesAstResult, EXPORT_NAMESPACE},
  sourcemap::merge_comments,
};

#[derive(Default)]
pub struct FarmPluginLibrary {}

impl FarmPluginLibrary {
  pub fn new(_: &Config) -> Self {
    Self::default()
  }
}

impl Plugin for FarmPluginLibrary {
  fn name(&self) -> &str {
    "FarmPluginLibrary"
  }

  // TODO: add a hook collect resource pot info before render resource pot

  fn render_resource_pot(
    &self,
    resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let module_graph = context.module_graph.read();

    let ConcatenateModulesAstResult {
      mut ast,
      module_ids,
      external_modules,
    } = concatenate_modules_ast(&resource_pot.modules, &module_graph, context)
      .map_err(|e| CompilationError::GenericError(e.to_string()))?;

    // handle import/export between resource pots
    if let Some(entry) = &resource_pot.entry_module {
      let entry_module = module_graph.module(entry).unwrap();
      // add export statement from export_ident_map
      let export_ident_map = &entry_module.meta.as_script().export_ident_map;
      let mut export_idents = export_ident_map
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
      export_idents.sort_by_key(|a| a.0.clone());

      let mut specifiers = vec![];

      for (name, id) in export_idents {
        // skip internal namespace
        if name == EXPORT_NAMESPACE {
          continue;
        }

        specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
          span: DUMMY_SP,
          orig: ModuleExportName::Ident(Ident::new(id.sym.clone(), DUMMY_SP, id.ctxt())),
          exported: Some(ModuleExportName::Ident(Ident::new(
            name.as_str().into(),
            DUMMY_SP,
            SyntaxContext::empty(),
          ))),
          is_type_only: false,
        }));
      }

      let export_item = ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
        span: DUMMY_SP,
        specifiers,
        src: None,
        type_only: false,
        with: None,
      }));

      ast.body.push(export_item);
    }

    // TODO find exports in this resource pot

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast,
      external_modules,
      rendered_modules: module_ids,
      comments: Default::default(), // TODO: merge comments
    })))
  }
}
