use std::sync::{Arc, RwLock};

use hashbrown::{HashMap, HashSet};

/// All hook operation record are write down by [RecordManager]
pub struct RecordManager {
  resolve_id_map: Arc<RwLock<HashMap<String, Vec<ResolveRecord>>>>,
  transform_map: Arc<RwLock<HashMap<String, Vec<TransformRecord>>>>,
}

impl RecordManager {
  pub fn new() -> Self {
    Self {
      resolve_id_map: Arc::new(RwLock::new(HashMap::new())),
      transform_map: Arc::new(RwLock::new(HashMap::new())),
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
