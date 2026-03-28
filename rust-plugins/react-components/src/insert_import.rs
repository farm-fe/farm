use farmfe_core::{config::TargetEnv, swc_common::{DUMMY_SP, SyntaxContext}, swc_ecma_ast::*};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};
use farmfe_utils::relative;
use std::{
  collections::HashSet,
  path::Path,
  sync::{Arc, Mutex},
};

use crate::{
  find_local_components::{ComponentInfo, ExportType},
  resolvers::ImportStyle,
  ImportMode,
};
pub struct ImportModifier {
  components: Arc<Mutex<HashSet<ComponentInfo>>>,
  pub used_components: HashSet<ComponentInfo>,
}

impl ImportModifier {
  pub fn new(components: Arc<Mutex<HashSet<ComponentInfo>>>) -> Self {
    Self {
      components,
      used_components: HashSet::new(),
    }
  }
}

impl VisitMut for ImportModifier {
  fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
    let mut components = self.components.lock().unwrap();
    for specifier in &n.specifiers {
      match specifier {
        ImportSpecifier::Default(default_spec) => {
          let imported_name = default_spec.local.sym.as_ref();
          components.retain(|c: &ComponentInfo| &c.name != imported_name);
        }

        ImportSpecifier::Named(named_spec) => {
          let imported_name = match &named_spec.imported {
            Some(imported) => match imported {
              ModuleExportName::Ident(ident) => ident.sym.as_ref(),
              ModuleExportName::Str(str) => str.value.as_ref(),
            },
            None => named_spec.local.sym.as_ref(),
          };
          components
            .retain(|c| &c.name != imported_name || c.name != named_spec.local.sym.as_ref());
        }
        _ => {}
      }
    }
  }
  fn visit_mut_jsx_opening_element(&mut self, n: &mut JSXOpeningElement) {
    if let JSXElementName::Ident(ident) = &n.name {
      let component_name = ident.sym.to_string();
      if component_name
        .chars()
        .next()
        .map_or(false, |c| c.is_uppercase())
      {
        let item = self
          .components
          .lock()
          .unwrap()
          .iter()
          .find(|c| c.name == component_name)
          .cloned();
        if let Some(item) = item {
          self.used_components.insert(item);
        }
      }
    }

    n.visit_mut_children_with(self);
  }
}

pub struct InsertImportModifier {
  pub components: HashSet<ComponentInfo>,
  pub target_env: TargetEnv,
  pub import_mode: ImportMode,
  pub file_path: String,
}
impl InsertImportModifier {
  pub fn new(
    target_env: TargetEnv,
    import_mode: ImportMode,
    file_path: String,
    components: HashSet<ComponentInfo>,
  ) -> Self {
    Self {
      components,
      target_env,
      import_mode,
      file_path,
    }
  }
}
impl VisitMut for InsertImportModifier {
  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    let mut last_import_index = None;
    for (index, item) in items.iter().enumerate() {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(_)) = item {
        last_import_index = Some(index);
      }
    }
    let mut new_imports = Vec::new();
    for component in &self.components {
      let mut component_path = component.path.clone();
      if component.is_local {
        component_path = match self.import_mode {
          ImportMode::Relative => format!(
            "./{}",
            relative(
              Path::new(&self.file_path)
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
              &component.path,
            )
          ),
          ImportMode::Absolute => component_path,
        };
      }
      let imported = {
        if component.original_name != component.name {
          Some(ModuleExportName::Ident(Ident::new(
            component.original_name.clone().into(),
            DUMMY_SP,
            SyntaxContext::empty(),
          )))
        } else {
          None
        }
      };
      let specifier = match component.export_type {
        ExportType::Default => ImportSpecifier::Default(ImportDefaultSpecifier {
          local: Ident::new(component.name.clone().into(), DUMMY_SP, SyntaxContext::empty()),
          span: DUMMY_SP,
        }),
        ExportType::Named => ImportSpecifier::Named(ImportNamedSpecifier {
          local: Ident::new(component.name.clone().into(), DUMMY_SP, SyntaxContext::empty()),
          imported,
          span: DUMMY_SP,
          is_type_only: false,
        }),
      };
      let import_decl = ImportDecl {
        src: Box::new(Str {
          value: component_path.into(),
          span: DUMMY_SP,
          raw: None,
        }),
        specifiers: vec![specifier],
        type_only: false,
        span: DUMMY_SP,
        with: Default::default(),
        phase: Default::default(),
      };

      new_imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
      if ImportStyle::Bool(false) != component.import_style {
        let target_dir = match self.target_env {
          TargetEnv::Browser => "es",
          TargetEnv::Node => "lib",
          _ => "es",
        };
        // {module}/{lib|es}/{Button}/style/index.css|js
        // module antd
        // target env [lib/es]
        // Button ComponentName
        // ImportStyle string
        let full_component_path = format!(
          "{}/{}/{}",
          component.path, target_dir, component.original_name
        );

        match &component.import_style {
          ImportStyle::Bool(res) => {
            if *res {
              let import_decl = ImportDecl {
                src: Box::new(Str {
                  value: format!("{}/{}", full_component_path, "style").into(),
                  span: DUMMY_SP,
                  raw: None,
                }),
                specifiers: vec![],
                type_only: false,
                span: DUMMY_SP,
                with: Default::default(),
                phase: Default::default(),
              };
              new_imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
            }
          }
          ImportStyle::String(res) => {
            let import_decl = ImportDecl {
              src: Box::new(Str {
                value: format!("{}/{}", full_component_path, res).into(),
                span: DUMMY_SP,
                raw: None,
              }),
              specifiers: vec![],
              type_only: false,
              span: DUMMY_SP,
              with: Default::default(),
              phase: Default::default(),
            };
            new_imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
          }
        }
      }
    }

    match last_import_index {
      Some(index) => items.splice(index + 1..index + 1, new_imports.iter().cloned()),
      None => items.splice(0..0, new_imports.iter().cloned()),
    };
  }
}
