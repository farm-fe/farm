use std::sync::{Arc, RwLock};

use hashbrown::{HashMap, HashSet};

use crate::{module::ModuleId, plugin::PluginAnalyzeDepsHookResultEntry};

/// All hook operation record are write down by [RecordManager]
pub struct RecordManager {
  resolve_id_map: Arc<RwLock<HashMap<String, Vec<ResolveRecord>>>>,
  transform_map: Arc<RwLock<HashMap<String, Vec<TransformRecord>>>>,
  process_map: Arc<RwLock<HashMap<String, Vec<ModuleRecord>>>>,
  analyze_deps_map: Arc<RwLock<HashMap<String, Vec<AnalyzeDepsRecord>>>>,
  resource_pot_map: Arc<RwLock<HashMap<String, Vec<ResourcePotRecord>>>>,
}

impl RecordManager {
  pub fn new() -> Self {
    Self {
      resolve_id_map: Arc::new(RwLock::new(HashMap::new())),
      transform_map: Arc::new(RwLock::new(HashMap::new())),
      process_map: Arc::new(RwLock::new(HashMap::new())),
      analyze_deps_map: Arc::new(RwLock::new(HashMap::new())),
      resource_pot_map: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn add_resolve_record(&self, source: String, record: ResolveRecord) {
    let mut resolve_id_map = self.resolve_id_map.write().unwrap();
    if let Some(records) = resolve_id_map.get_mut(&source) {
      records.push(record);
    } else {
      resolve_id_map.insert(source, vec![record]);
    }
  }

  pub fn add_load_record(&self, id: String, record: TransformRecord) {
    let mut transform_map = self.transform_map.write().unwrap();
    if transform_map.get(&id).is_none() {
      transform_map.insert(id, vec![record]);
    }
  }

  pub fn add_transform_record(&self, id: String, record: TransformRecord) {
    let mut transform_map = self.transform_map.write().unwrap();
    if let Some(records) = transform_map.get_mut(&id) {
      records.push(record);
    }
  }

  pub fn add_parse_record(&self, id: String, record: ModuleRecord) {
    let mut process_map = self.process_map.write().unwrap();
    if process_map.get(&id).is_none() {
      process_map.insert(id, vec![record]);
    }
  }

  pub fn add_process_record(&self, id: String, record: ModuleRecord) {
    let mut process_map = self.process_map.write().unwrap();
    if let Some(records) = process_map.get_mut(&id) {
      records.push(record);
    }
  }

  pub fn add_analyze_deps_record(&self, id: String, record: AnalyzeDepsRecord) {
    let mut analyze_deps_map = self.analyze_deps_map.write().unwrap();
    if let Some(records) = analyze_deps_map.get_mut(&id) {
      records.push(record);
    } else {
      analyze_deps_map.insert(id, vec![record]);
    }
  }

  pub fn add_resource_pot_record(&self, id: String, record: ResourcePotRecord) {
    let mut resource_pot_map = self.resource_pot_map.write().unwrap();
    if let Some(records) = resource_pot_map.get_mut(&id) {
      records.push(record);
    } else {
      resource_pot_map.insert(id, vec![record]);
    }
  }

  pub fn get_resolve_records(&self) -> Vec<String> {
    let resolve_id_map = self.resolve_id_map.read().unwrap();
    let mut resolve_id_set = HashSet::new();
    for records in resolve_id_map.values() {
      for record in records {
        resolve_id_set.insert(record.result.clone());
      }
    }

    let resolve_ids: Vec<String> = resolve_id_set.into_iter().collect();
    resolve_ids
  }

  pub fn get_transform_records_by_id(&self, id: &str) -> Vec<TransformRecord> {
    let transform_map = self.transform_map.read().unwrap();
    match transform_map.get(id) {
      Some(records) => records.clone(),
      None => Vec::new(),
    }
  }

  pub fn get_process_records_by_id(&self, id: &str) -> Vec<ModuleRecord> {
    let process_map = self.process_map.read().unwrap();
    match process_map.get(id) {
      Some(records) => records.clone(),
      None => Vec::new(),
    }
  }

  pub fn get_analyze_deps_records_by_id(&self, id: &str) -> Vec<AnalyzeDepsRecord> {
    let analyze_deps_map: std::sync::RwLockReadGuard<'_, HashMap<String, Vec<AnalyzeDepsRecord>>> =
      self.analyze_deps_map.read().unwrap();
    match analyze_deps_map.get(id) {
      Some(records) => records.clone(),
      None => Vec::new(),
    }
  }

  pub fn get_resource_pot_records_by_id(&self, id: &str) -> Vec<ResourcePotRecord> {
    let resource_pot_map = self.resource_pot_map.read().unwrap();
    match resource_pot_map.get(id) {
      Some(records) => records.clone(),
      None => Vec::new(),
    }
  }
}

impl Default for RecordManager {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
pub struct ResolveRecord {
  pub name: String,
  pub result: String,
}

#[derive(Debug, Clone)]
pub struct TransformRecord {
  pub name: String,
  pub result: String,
  pub source_maps: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleRecord {
  pub name: String,
}

#[derive(Debug, Clone)]
pub struct AnalyzeDepsRecord {
  pub name: String,
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Debug, Clone)]
pub struct ResourcePotRecord {
  pub name: String,
  pub hook: String,
  pub modules: Vec<ModuleId>,
  pub resources: Vec<String>,
}
