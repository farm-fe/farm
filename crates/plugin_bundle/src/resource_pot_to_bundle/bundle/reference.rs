use std::collections::HashMap;

use farmfe_core::module::{ModuleId, ModuleSystem};

use crate::resource_pot_to_bundle::{
  modules_analyzer::module_analyzer::ExportSpecifierInfo, uniq_name::BundleVariable,
};

#[derive(Debug, Clone, Default)]
pub struct ReferenceExport {
  pub named: HashMap<usize, usize>,
  pub default: Option<usize>,
  pub all: bool,
  pub namespace: Option<usize>,
}

impl ReferenceExport {
  pub fn new() -> Self {
    Self {
      named: HashMap::new(),
      default: None,
      all: false,
      namespace: None,
    }
  }

  // fn contains(&self, export: &ExportSpecifierInfo) -> bool {
  //   match export {
  //     ExportSpecifierInfo::Named(named) => self.named.contains_key(&named.local()),
  //     ExportSpecifierInfo::Default(_) => self.default.is_some(),
  //     ExportSpecifierInfo::All(_) => self.all.0,
  //     ExportSpecifierInfo::Namespace(_) => self.namespace.is_some(),
  //   }
  // }

  fn insert(&mut self, export: &ExportSpecifierInfo) {
    match export {
      ExportSpecifierInfo::Named(named) => {
        self.named.insert(named.export_as(), named.local());
      }
      ExportSpecifierInfo::Default(local) => {
        self.default = Some(*local);
      }
      ExportSpecifierInfo::All(_) => {
        self.all = true;
      }
      ExportSpecifierInfo::Namespace(name) => {
        self.namespace = Some(*name);
      }
    }
  }

  // reexport should ignore default
  // sub export should ignore all, because it's already to be named
  fn merge_by_export_all(&mut self, from: &ReferenceExport) {
    self.named.extend(from.named.iter());

    if let Some(ref ns) = from.namespace {
      self.named.insert(*ns, *ns);
    }
  }

  // external reexport should save all
  pub fn merge_by_external(&mut self, from: &ReferenceExport) {
    self.named.extend(from.named.iter());

    if let Some(ref ns) = from.namespace {
      self.named.insert(*ns, *ns);
    }

    if let Some(ref default) = from.default {
      self.default = Some(*default);
    }

    if from.all {
      self.all = true;
    }
  }

  pub fn is_empty(&self) -> bool {
    self.named.is_empty() && self.default.is_none() && !self.all && self.namespace.is_none()
  }

  pub fn query(&self, export_from: &String, bundle_variable: &BundleVariable) -> Option<usize> {
    if let Some(index) = self.namespace {
      if &bundle_variable.name(index) == export_from {
        return Some(index);
      }
    }

    for export_as in self.named.keys() {
      if &bundle_variable.name(*export_as) == export_from {
        return Some(self.named[export_as]);
      }
    }

    None
  }

  pub fn raw_query(
    &self,
    export_from: &String,
    bundle_variable: &BundleVariable,
    find_default: bool,
  ) -> Option<usize> {
    if find_default {
      return self.default;
    }

    if let Some(index) = self.namespace {
      if &bundle_variable.name(index) == export_from {
        return Some(index);
      }
    }

    for export_as in self.named.keys() {
      if &bundle_variable.name(*export_as) == export_from {
        return Some(self.named[export_as]);
      }
    }

    None
  }
}

#[derive(Debug, Clone)]
pub struct ReferenceMap {
  pub reexport_map: HashMap<ModuleId, ReferenceExport>,
  pub export: ReferenceExport,
  pub export_type: ModuleSystem,
}

pub struct ReferenceQueryResult {
  pub index: usize,
  pub is_reexport: bool,
}

impl ReferenceMap {
  pub fn new(module_system: ModuleSystem) -> Self {
    Self {
      export: ReferenceExport::default(),
      reexport_map: Default::default(),
      export_type: module_system,
    }
  }

  pub fn add_reference(&mut self, module_id: &ModuleId, export: &ExportSpecifierInfo) {
    if !self.reexport_map.contains_key(module_id) {
      self
        .reexport_map
        .insert(module_id.clone(), ReferenceExport::new());
    }

    let reference = self.reexport_map.get_mut(module_id).unwrap();
    reference.insert(export);
  }

  pub fn add_commonjs(&mut self, module_id: &ModuleId, export: &ExportSpecifierInfo) {
    self.add_reference(module_id, export);
    self.export_type.merge(ModuleSystem::CommonJs);
  }

  pub fn add_local(&mut self, export: &ExportSpecifierInfo) {
    self.export.insert(export);
  }

  pub fn query_by_var_str_and_meta(
    &self,
    export_from: &String,
    bundle_variable: &BundleVariable,
  ) -> Option<ReferenceQueryResult> {
    if let Some(r) = self.export.query(export_from, bundle_variable) {
      Some(ReferenceQueryResult {
        index: r,
        is_reexport: false,
      })
    } else {
      self
        .reexport_map
        .values()
        .find_map(|item| item.query(export_from, bundle_variable))
        .map(|r| ReferenceQueryResult {
          index: r,
          is_reexport: true,
        })
    }
  }

  pub fn query(&self, index: usize, bundle_variable: &BundleVariable) -> Option<usize> {
    let export_from = bundle_variable.name(index);

    let find_default = export_from == "default";

    if let Some(r) = self
      .export
      .raw_query(&export_from, bundle_variable, find_default)
    {
      Some(r)
    } else {
      self
        .reexport_map
        .values()
        .find_map(|item| item.query(&export_from, bundle_variable))
    }
  }

  pub fn extends(&mut self, other: &ReferenceMap) {
    for item in &other.reexport_map {
      if !self.reexport_map.contains_key(item.0) {
        self
          .reexport_map
          .insert(item.0.clone(), ReferenceExport::new());
      }

      if let Some(map) = self.reexport_map.get_mut(item.0) {
        map.merge_by_export_all(item.1);
      }
    }

    self.export_type.merge(other.export_type.clone());

    self.export.merge_by_export_all(&other.export);
  }

  pub fn print(&self, bundle_variable: &BundleVariable) {
    let print_export = |export: &ReferenceExport, source: Option<&ModuleId>| {
      if !export.named.is_empty() {
        print!("export {{");
        for (export_as, local) in &export.named {
          print!(
            "{{ {:?} as {:?}, }}",
            bundle_variable.name(*export_as),
            bundle_variable.name(*local)
          );
        }
        if let Some(source) = source {
          println!("}} from \"{}\"", source.to_string());
        } else {
          println!("}}");
        }
      }

      if let Some(default) = export.default {
        print!("export default {:?}", bundle_variable.name(default));

        println!(
          "{}",
          if let Some(source) = source {
            source.to_string()
          } else {
            "".to_string()
          }
        )
      }
    };

    print_export(&self.export, None);
    for (module_id, reference) in &self.reexport_map {
      print_export(reference, Some(module_id));
    }
  }
}
