use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  path::PathBuf,
  str::FromStr,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use farmfe_core::{
  farm_profile_scope,
  module::{ModuleId, ModuleSystem},
  resource::resource_pot::ResourcePotId,
  swc_ecma_ast::Ident,
};
use farmfe_toolkit::fs::normalize_file_name_as_variable;

use super::{
  bundle::{bundle_external::ReferenceKind, ModuleAnalyzerManager},
  Var,
};

#[derive(Debug, Default)]
pub struct UniqName {
  pub name_count_map: HashMap<String, usize>,
}

impl UniqName {
  pub fn new() -> Self {
    Self {
      name_count_map: HashMap::new(),
    }
  }

  pub fn contain(&self, name: &str) -> bool {
    self.name_count_map.contains_key(name)
  }

  pub fn insert(&mut self, name: &str) {
    if let Some(count) = self.name_count_map.get_mut(name) {
      *count += 1;
    } else {
      self.name_count_map.insert(name.to_string(), 1);
    }
  }

  pub fn uniq_name(&mut self, name: &str) -> String {
    farm_profile_scope!("uniq name");

    let base_uniq_name = name.to_string();
    let mut uniq_name = base_uniq_name.clone();

    if let Some(mut count) = self.name_count_map.get_mut(&base_uniq_name).cloned() {
      loop {
        uniq_name = format!("{}${}", base_uniq_name, count);

        if !self.name_count_map.contains_key(&uniq_name) {
          break;
        }

        count += 1;
      }

      self.name_count_map.insert(base_uniq_name, count);
    }

    uniq_name
  }
}

#[derive(Debug, Default)]
pub struct BundleVariable {
  pub index: Arc<AtomicUsize>,
  // TODO(improve) diff vec and hashmap
  pub variables: HashMap<usize, Var>,
  // TODO(improve): maybe record top_level var, and only register same top_level var
  pub module_defined_vars: HashMap<ModuleId, HashMap<String, usize>>,
  pub uniq_name_hash_map: HashMap<ResourcePotId, UniqName>,
  pub namespace: String,
  pub used_names: HashSet<String>,
}

pub fn safe_name_from_module_id(module_id: &ModuleId, root: &str) -> String {
  farm_profile_scope!("safe_name_from_module_id PathBuf");
  let filename = module_id.resolved_path(root);
  let name = PathBuf::from_str(&filename)
    .map(|path| {
      path
        .file_stem()
        .map(|item| Cow::Owned(item.to_string_lossy().to_string()))
        .unwrap_or(Cow::Borrowed(&filename))
    })
    .unwrap_or(Cow::Borrowed(&filename));

  normalize_file_name_as_variable(name.to_string())
}

impl BundleVariable {
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  fn push(&mut self, var: Var, index: usize) {
    self.variables.insert(index, var);
  }

  pub fn is_in_used_name(&self, index: usize) -> bool {
    self.used_names.contains(&self.name(index))
  }

  pub fn uniq_name_mut(&mut self) -> &mut UniqName {
    self.uniq_name_hash_map.get_mut(&self.namespace).unwrap()
  }

  pub fn uniq_name(&mut self) -> &UniqName {
    self.uniq_name_hash_map.get(&self.namespace).unwrap()
  }

  pub fn with_namespace<T, F: FnOnce(&mut Self) -> T>(&mut self, namespace: String, f: F) -> T {
    let prev = self.namespace.clone();

    if self.uniq_name_hash_map.contains_key(&namespace) {
      self
        .uniq_name_hash_map
        .insert(namespace.clone(), UniqName::new());
    }
    self.namespace = namespace;

    let r = f(self);

    self.namespace = prev;

    r
  }

  pub fn set_namespace(&mut self, namespace: String) {
    if !self.uniq_name_hash_map.contains_key(&namespace) {
      self
        .uniq_name_hash_map
        .insert(namespace.clone(), UniqName::new());
    }

    self.namespace = namespace;
  }

  pub fn add_used_name(&mut self, used_name: String) {
    farm_profile_scope!("add used name");
    self.uniq_name_mut().insert(&used_name);
    self.used_names.insert(used_name);
  }

  pub fn register_var(&mut self, module_id: &ModuleId, ident: &Ident, strict: bool) -> usize {
    farm_profile_scope!("register var");
    let var = Var {
      var: ident.to_id(),
      rename: None,
      ..Default::default()
    };

    let mut index = None;
    let mut create_index = || {
      let v = self.index.fetch_add(1, Ordering::Release);
      index = Some(v);
      v
    };

    let var_ident = if strict {
      // a#1
      ident.to_string()
    } else {
      // a
      var.var.0.to_string()
    };

    if let Some(map) = self.module_defined_vars.get_mut(module_id) {
      if !self.used_names.contains(&var_ident) {
        if let Some(exists_index) = map.get(&var_ident) {
          return *exists_index;
        }
      }

      map.insert(var_ident, create_index());
    } else {
      let mut map = HashMap::new();
      map.insert(var_ident, create_index());
      self.module_defined_vars.insert(module_id.clone(), map);
    }

    self.push(var, index.unwrap());

    index.unwrap()
  }

  pub fn branch(&self) -> Self {
    Self {
      index: Arc::clone(&self.index),
      ..Default::default()
    }
  }

  pub fn merge(&mut self, other: Self) {
    for (index, var) in other.variables {
      self.variables.insert(index, var);
    }

    self.module_defined_vars.extend(other.module_defined_vars);
    self.used_names.extend(other.used_names);

    // when merge stage, uniq_name is all unresolved var, so we only record once
    for (resource_pot, uniq_name) in other.uniq_name_hash_map {
      if let Some(self_uniq_name) = self.uniq_name_hash_map.get_mut(&resource_pot) {
        uniq_name.name_count_map.into_iter().for_each(|(name, _)| {
          self_uniq_name.insert(&name);
        });
      } else {
        self.uniq_name_hash_map.insert(resource_pot, uniq_name);
      }
    }
  }

  pub fn register_used_name_by_module_id(
    &mut self,
    module_id: &ModuleId,
    suffix: &str,
    root: &str,
  ) -> usize {
    farm_profile_scope!("register name");
    let module_safe_name = format!("{}{suffix}", safe_name_from_module_id(module_id, root));

    let uniq_name_safe_name = self.uniq_name_mut().uniq_name(&module_safe_name);

    self.add_used_name(uniq_name_safe_name.clone());

    self.register_var(module_id, &uniq_name_safe_name.as_str().into(), false)
  }

  pub fn var_by_index(&self, index: usize) -> &Var {
    &self.variables[&index]
  }

  pub fn var_mut_by_index(&mut self, index: usize) -> &mut Var {
    self.variables.get_mut(&index).unwrap()
  }

  pub fn set_rename(&mut self, index: usize, rename: String) {
    let var = self.var_mut_by_index(index);
    if var.rename.is_none() {
      var.rename = Some(rename);
    }
  }

  pub fn set_rename_force(&mut self, index: usize, rename: String) {
    self.var_mut_by_index(index).rename = Some(rename);
  }

  pub fn rename(&self, index: usize) -> Option<&String> {
    self.var_by_index(index).rename.as_ref()
  }

  pub fn name(&self, index: usize) -> String {
    self.var_by_index(index).var.0.to_string()
  }

  #[inline]
  pub fn is_default_key(&self, index: usize) -> bool {
    self.name(index) == "default"
  }

  pub fn render_name(&self, index: usize) -> String {
    let var = self.var_by_index(index);
    if let Some(rename) = var.rename.as_ref() {
      return rename.clone();
    }

    var.var.0.to_string()
  }

  pub fn set_var_uniq_rename_string(&mut self, index: usize, var_ident: String) {
    let var = self.var_by_index(index);
    if var.rename.is_some() {
      return;
    }

    let uniq_name = if self.uniq_name().contain(&var_ident) {
      self.uniq_name_mut().uniq_name(&var_ident)
    } else {
      var_ident.clone()
    };

    self.set_rename(index, uniq_name.clone());

    self.uniq_name_mut().insert(&var_ident);

    if uniq_name != var_ident {
      self.uniq_name_mut().insert(uniq_name.as_str());
    }
  }

  pub fn set_var_uniq_rename(&mut self, index: usize) {
    self.set_var_uniq_rename_string(index, self.name(index));
  }

  pub fn find_ident_by_index(
    &self,
    index: usize,
    source: &ModuleId,
    module_analyzers: &ModuleAnalyzerManager,
    resource_pot_id: ResourcePotId,
    find_default: bool,
    find_namespace: bool,
  ) -> Option<FindModuleExportResult> {
    let var_ident = self.name(index);

    if module_analyzers.is_external(source) {
      return Some(FindModuleExportResult::External(index, source.clone()));
    }

    if let Some(module_analyzer) = module_analyzers.module_analyzer(source) {
      let module_system = module_analyzer.module_system.clone();

      if find_namespace || module_analyzers.is_commonjs(source) {
        return Some(FindModuleExportResult::Local(
          index,
          source.clone(),
          module_system,
        ));
      }

      let reference_map = module_analyzer.export_names();

      if module_analyzer.resource_pot_id != resource_pot_id {
        if let Some(index) = reference_map.query_by_var_str(&var_ident, self) {
          return Some(FindModuleExportResult::Bundle(
            index,
            module_analyzer.resource_pot_id.clone(),
            // support cjs
            module_system,
          ));
          // TODO: error?
        }
      }

      if find_default {
        if let Some(d) = reference_map
          .export
          .default
          .or_else(|| reference_map.export.query(&"default".to_string(), self))
        {
          return Some(FindModuleExportResult::Local(
            d,
            source.clone(),
            module_system,
          ));
        }

        return None;
      }

      // find from local
      if let Some(d) = reference_map.export.query(&var_ident, self) {
        return Some(FindModuleExportResult::Local(
          d,
          source.clone(),
          module_system,
        ));
      }

      // find from reference external or bundle
      for (module_id, export) in &reference_map.reference_map {
        if let Some(d) = export.query(&var_ident, self) {
          if module_analyzers.is_external(module_id) {
            return Some(FindModuleExportResult::External(d, module_id.clone()));
          } else {
            return Some(FindModuleExportResult::Local(
              d,
              module_id.clone(),
              module_system,
            ));
          }
        }
      }
    }

    None
  }
}

#[derive(Debug)]
pub enum FindModuleExportResult {
  Local(usize, ModuleId, ModuleSystem),
  External(usize, ModuleId),
  Bundle(usize, ResourcePotId, ModuleSystem),
}

impl FindModuleExportResult {
  pub fn is_common_js(&self) -> bool {
    match self {
      FindModuleExportResult::Local(_, _, module_system)
      | FindModuleExportResult::Bundle(_, _, module_system) => {
        matches!(module_system, ModuleSystem::CommonJs | ModuleSystem::Hybrid)
      }

      _ => false,
    }
  }

  pub fn module_system(&self) -> Option<ModuleSystem> {
    match self {
      FindModuleExportResult::Local(_, _, module_system)
      | FindModuleExportResult::Bundle(_, _, module_system) => Some(module_system.clone()),
      FindModuleExportResult::External(_, _) => None,
    }
  }

  pub fn target_source(&self) -> ReferenceKind {
    match self {
      FindModuleExportResult::Local(_, target_source, _) => target_source.clone().into(),
      FindModuleExportResult::External(_, target_source) => target_source.clone().into(),
      FindModuleExportResult::Bundle(_, target_bundle, _) => target_bundle.clone().into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    config::Config,
    context::CompilationContext,
    error::Result,
    module::{
      module_graph::{self, ModuleGraph},
      Module, ModuleId, ScriptModuleMetaData,
    },
    resource::resource_pot::ResourcePotId,
  };

  use crate::resource_pot_to_bundle::{
    bundle::ModuleAnalyzerManager,
    modules_analyzer::module_analyzer::{
      ExportInfo, ExportSpecifierInfo, ModuleAnalyzer, Statement, Variable,
    },
    uniq_name::FindModuleExportResult,
  };

  use super::{BundleVariable, UniqName};

  #[test]

  fn uniq_name() {
    let mut uniq_name = UniqName::new();

    assert_eq!(uniq_name.uniq_name("name"), "name");

    uniq_name.insert("name");

    assert_eq!(uniq_name.uniq_name("name"), "name$1");

    uniq_name.insert("name");
    uniq_name.insert("name");
    uniq_name.insert("name");

    assert_eq!(uniq_name.uniq_name("name"), "name$4");

    uniq_name.insert("name");

    assert_eq!(uniq_name.uniq_name("name"), "name$5");

    uniq_name.insert("name$5");

    assert_eq!(uniq_name.uniq_name("name"), "name$6");
  }

  #[test]
  fn find_external() -> Result<()> {
    // b.js / external.js

    // external.js
    // ..
    //
    // b.js
    // export { a as b } from './external.js';
    //
    //
    // result: find_ident_by_index(b, b.js) -> External(_, external.js)

    let mut bundle_variable = BundleVariable::new();

    let b_module_id: ModuleId = "b.js".into();
    let external_module_id: ModuleId = "external.js".into();
    let context = Arc::new(CompilationContext::new(Config::default(), vec![])?);
    let resource_pot_id: ResourcePotId = "index".into();
    let mut b_module = Module::new(b_module_id.clone());

    b_module.meta = Box::new(farmfe_core::module::ModuleMetaData::Script(
      ScriptModuleMetaData {
        ..Default::default()
      },
    ));
    let mut external_module = Module::new(external_module_id.clone());
    external_module.external = true;
    external_module.meta = Box::new(farmfe_core::module::ModuleMetaData::Script(
      ScriptModuleMetaData {
        ..Default::default()
      },
    ));

    let mut module_analyzer_map = HashMap::new();

    let mut b_module_analyzer = ModuleAnalyzer::new(
      &b_module,
      &context,
      resource_pot_id.clone(),
      false,
      false,
      false,
    )?;
    let external_module_analyzer = ModuleAnalyzer::new(
      &external_module,
      &context,
      resource_pot_id.clone(),
      false,
      false,
      false,
    )?;
    let external_export = bundle_variable.register_var(&b_module_id, &"a".into(), false);
    let local_variable = bundle_variable.register_var(&b_module_id, &"b".into(), false);

    b_module_analyzer.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(external_module_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          external_export,
          Some(local_variable),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_map.insert(b_module_id.clone(), b_module_analyzer);
    module_analyzer_map.insert(external_module_id, external_module_analyzer);

    let mut module_graph = module_graph::ModuleGraph::new();

    module_graph.add_module(b_module);
    module_graph.add_module(external_module);

    let mut module_analyzer_manager =
      ModuleAnalyzerManager::new(module_analyzer_map, &module_graph);

    module_analyzer_manager.build_export_names(&b_module_id, &bundle_variable);

    let result: Option<FindModuleExportResult> = bundle_variable.find_ident_by_index(
      local_variable,
      &b_module_id,
      &module_analyzer_manager,
      resource_pot_id,
      false,
      false,
    );

    assert!(matches!(
      result,
      Some(FindModuleExportResult::External(_, _))
    ));

    if let FindModuleExportResult::External(index, ..) = result.unwrap() {
      assert_eq!(
        bundle_variable.name(index),
        bundle_variable.name(external_export)
      )
    };

    Ok(())
    // bundle_variable.regis
  }

  #[test]
  fn find_other_bundle() -> Result<()> {
    // index.js, bundle-b.js

    // bundle-b.js
    // export const bundleB = 100;
    //
    // index.js
    // export { bundleB as b } from './bundle-b.js';
    //
    //
    // result: find_ident_by_index(b, bundle-b.js) -> Bundle(bundleB, bundle-b.js)

    let mut bundle_variable = BundleVariable::new();

    let index_module_id: ModuleId = "index.js".into();
    let bundle_module_id: ModuleId = "bundle-b.js".into();
    let context = Arc::new(CompilationContext::new(Config::default(), vec![])?);
    let resource_pot_index: ResourcePotId = "index".into();
    let resource_pot_bundle: ResourcePotId = "bundle".into();

    let mut index_module = Module::new(index_module_id.clone());

    index_module.meta = Box::new(farmfe_core::module::ModuleMetaData::Script(
      ScriptModuleMetaData {
        ..Default::default()
      },
    ));
    let mut bundle_module = Module::new(bundle_module_id.clone());
    bundle_module.meta = Box::new(farmfe_core::module::ModuleMetaData::Script(
      ScriptModuleMetaData {
        ..Default::default()
      },
    ));

    let mut module_analyzer_map = HashMap::new();

    let mut index_module_analyzer = ModuleAnalyzer::new(
      &index_module,
      &context,
      resource_pot_index.clone(),
      false,
      false,
      false,
    )?;
    let mut bundle_module_analyzer = ModuleAnalyzer::new(
      &bundle_module,
      &context,
      resource_pot_bundle.clone(),
      false,
      false,
      false,
    )?;

    let bundle_export = bundle_variable.register_var(&bundle_module_id, &"bundleB".into(), false);
    let index_export_from =
      bundle_variable.register_var(&index_module_id, &"bundleB".into(), false);
    let export_as = bundle_variable.register_var(&index_module_id, &"b".into(), false);

    index_module_analyzer.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(bundle_module_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          index_export_from,
          Some(export_as),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    bundle_module_analyzer.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: None,
        specifiers: vec![ExportSpecifierInfo::Named(bundle_export.into())],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_map.insert(index_module_id.clone(), index_module_analyzer);
    module_analyzer_map.insert(bundle_module_id.clone(), bundle_module_analyzer);

    let mut module_graph = ModuleGraph::new();

    module_graph.add_module(index_module);
    module_graph.add_module(bundle_module);

    let mut module_analyzer_manager =
      ModuleAnalyzerManager::new(module_analyzer_map, &module_graph);

    module_analyzer_manager.build_export_names(&index_module_id, &bundle_variable);
    module_analyzer_manager.build_export_names(&bundle_module_id, &bundle_variable);

    let result = bundle_variable.find_ident_by_index(
      index_export_from,
      &bundle_module_id,
      &module_analyzer_manager,
      resource_pot_index,
      false,
      false,
    );

    assert!(matches!(
      result,
      Some(FindModuleExportResult::Bundle(_, _, _))
    ));

    if let FindModuleExportResult::Bundle(index, ..) = result.unwrap() {
      assert_eq!(
        bundle_variable.name(index),
        bundle_variable.name(index_export_from)
      )
    };

    Ok(())
  }
}
