use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::{
  module::{ModuleId, ModuleType},
  plugin::PluginAnalyzeDepsHookResultEntry,
};

#[derive(Debug, Clone)]
pub enum Trigger {
  Compiler,
  Update,
}

/// All hook operation record are write down by [RecordManager]
pub struct RecordManager {
  resolve_id_map: Arc<RwLock<HashMap<String, Vec<ResolveRecord>>>>,
  transform_map: Arc<RwLock<HashMap<String, Vec<TransformRecord>>>>,
  process_map: Arc<RwLock<HashMap<String, Vec<ModuleRecord>>>>,
  analyze_deps_map: Arc<RwLock<HashMap<String, Vec<AnalyzeDepsRecord>>>>,
  resource_pot_map: Arc<RwLock<HashMap<String, Vec<ResourcePotRecord>>>>,
  pub plugin_stats: Arc<RwLock<HashMap<String, HashMap<String, PluginStats>>>>,
  trigger: Arc<RwLock<Trigger>>,
}

impl RecordManager {
  pub fn new() -> Self {
    Self {
      resolve_id_map: Arc::new(RwLock::new(HashMap::new())),
      transform_map: Arc::new(RwLock::new(HashMap::new())),
      process_map: Arc::new(RwLock::new(HashMap::new())),
      analyze_deps_map: Arc::new(RwLock::new(HashMap::new())),
      resource_pot_map: Arc::new(RwLock::new(HashMap::new())),
      plugin_stats: Arc::new(RwLock::new(HashMap::new())),
      trigger: Arc::new(RwLock::new(Trigger::Compiler)),
    }
  }

  pub fn set_trigger(&self, trigger: Trigger) {
    *self.trigger.write().unwrap() = trigger;
  }

  pub fn add_resolve_record(&self, source: String, mut record: ResolveRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook, record.duration);
    // clone is quicker than to_owned
    record.trigger = self.trigger.read().unwrap().clone();

    let mut resolve_id_map = self.resolve_id_map.write().unwrap();
    if let Some(records) = resolve_id_map.get_mut(&source) {
      records.push(record);
    } else {
      resolve_id_map.insert(source, vec![record]);
    }
  }

  pub fn add_load_record(&self, id: String, mut record: TransformRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook, record.duration);
    record.trigger = self.trigger.read().unwrap().clone();

    let mut transform_map = self.transform_map.write().unwrap();
    if transform_map.get(&id).is_none() {
      transform_map.insert(id, vec![record]);
    }
  }

  pub fn add_transform_record(&self, id: String, mut record: TransformRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook, record.duration);
    record.trigger = self.trigger.read().unwrap().clone();

    // reduce lock holding time
    let mut transform_map = self.transform_map.write().unwrap();
    if let Some(records) = transform_map.get_mut(&id) {
      records.push(record);
    }
  }

  pub fn add_parse_record(&self, id: String, mut record: ModuleRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook, record.duration);
    record.trigger = self.trigger.read().unwrap().clone();

    let mut process_map = self.process_map.write().unwrap();
    if process_map.get(&id).is_none() {
      process_map.insert(id, vec![record]);
    }
  }

  pub fn add_process_record(&self, id: String, mut record: ModuleRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook.clone(), record.duration);
    record.trigger = self.trigger.read().unwrap().clone();

    let mut process_map = self.process_map.write().unwrap();
    if let Some(records) = process_map.get_mut(&id) {
      records.push(record);
    }
  }

  pub fn add_analyze_deps_record(&self, id: String, mut record: AnalyzeDepsRecord) {
    self.update_plugin_stats(record.plugin.clone(), &record.hook.clone(), record.duration);
    record.trigger = self.trigger.read().unwrap().clone();

    let mut analyze_deps_map = self.analyze_deps_map.write().unwrap();
    if let Some(records) = analyze_deps_map.get_mut(&id) {
      records.push(record);
    } else {
      analyze_deps_map.insert(id, vec![record]);
    }
  }

  pub fn add_resource_pot_record(&self, id: String, record: ResourcePotRecord) {
    let mut resource_pot_map = self.resource_pot_map.write().unwrap();
    // reduce the number of accesses to resource_pot_map
    resource_pot_map
      .entry(id)
      .or_default()
      .push(record);
  }

  pub fn get_resolve_records_by_id(&self, id: &str) -> Vec<ResolveRecord> {
    let resolve_map = self.resolve_id_map.read().unwrap();
    match resolve_map.get(id) {
      Some(records) => records.clone(),
      None => Vec::new(),
    }
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

  pub fn update_plugin_stats(&self, plugin_name: String, hook_name: &str, duration: i64) {
    let mut plugin_stats = self.plugin_stats.write().unwrap();

    let plugin_entry = plugin_stats.entry(plugin_name).or_default();

    let stats = plugin_entry
      .entry(hook_name.to_string())
      .or_insert(PluginStats {
        total_duration: 0,
        call_count: 0,
      });

    stats.total_duration += duration;
    stats.call_count += 1;
  }
}

impl Default for RecordManager {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ResolveRecord {
  pub plugin: String,
  pub hook: String,
  pub source: String,
  pub importer: Option<String>,
  pub kind: String,
  pub trigger: Trigger,
  pub start_time: i64,
  pub end_time: i64,
  pub duration: i64,
}

#[derive(Debug, Clone)]
pub struct TransformRecord {
  pub plugin: String,
  pub hook: String,
  pub content: String,
  pub source_maps: Option<String>,
  pub module_type: ModuleType,
  pub trigger: Trigger,
  pub start_time: i64,
  pub end_time: i64,
  pub duration: i64,
}

#[derive(Debug, Clone)]
pub struct ModuleRecord {
  pub plugin: String,
  pub hook: String,
  pub module_type: ModuleType,
  pub trigger: Trigger,
  pub start_time: i64,
  pub end_time: i64,
  pub duration: i64,
}

#[derive(Debug, Clone)]
pub struct AnalyzeDepsRecord {
  pub plugin: String,
  pub hook: String,
  pub module_type: ModuleType,
  pub trigger: Trigger,
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
  pub start_time: i64,
  pub end_time: i64,
  pub duration: i64,
}

#[derive(Debug, Clone)]
pub struct ResourcePotRecord {
  pub name: String,
  pub hook: String,
  pub modules: Vec<ModuleId>,
  pub resources: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStats {
  pub total_duration: i64,
  pub call_count: usize,
}
