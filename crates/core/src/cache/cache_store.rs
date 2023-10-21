//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.
use rkyv::Deserialize;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use farmfe_macro_cache_item::cache_item;

use crate::{config::Mode, deserialize, serialize};

const FARM_CACHE_VERSION: &str = "0.0.1";

pub struct CacheStore {
  cache_dir: PathBuf,
  /// The maximum number of items per cache file. default: 1000
  items_per_cache_file: usize,
}

impl CacheStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let mut cache_dir = Path::new(cache_dir_str).to_path_buf();
    cache_dir.push(namespace.to_string() + "-" + FARM_CACHE_VERSION);

    if matches!(mode, Mode::Development) {
      cache_dir.push("development");
    } else {
      cache_dir.push("production");
    }

    if cache_dir_str.len() > 0 && !cache_dir.exists() {
      std::fs::create_dir_all(&cache_dir).unwrap();
    }

    Self {
      cache_dir,
      items_per_cache_file: 1000,
    }
  }

  pub fn set_items_per_cache_file(&mut self, size: usize) {
    self.items_per_cache_file = size;
  }

  /// Write the cache map to the disk.
  /// A cache file will be created for every 1000 items.
  pub fn write_cache(&self, cache_map: HashMap<String, Vec<u8>>, cache_type: &str) {
    let cache_file_dir = self.cache_dir.join(cache_type);
    // clear the cache file dir
    if cache_file_dir.exists() {
      std::fs::remove_dir_all(&cache_file_dir).unwrap();
    }

    std::fs::create_dir_all(&cache_file_dir).unwrap();

    let mut cache_map_vec = cache_map.into_iter().collect::<Vec<(String, Vec<u8>)>>();
    let mut cache_file_index = 0;

    cache_map_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut cache_content = CacheContentFile::new();

    for (i, item) in cache_map_vec.into_iter().enumerate() {
      cache_content.push(item);

      if (i + 1) % self.items_per_cache_file == 0 {
        // write cache file
        let file_name = cache_content.file_name(cache_file_index);
        let file_path = cache_file_dir.join(file_name);

        let bytes = serialize!(&cache_content);
        std::fs::write(file_path, bytes).unwrap();
        // increase cache file index and reset cache content
        cache_content.clear();
        cache_file_index += 1;
      }
    }
    // write the last cache file
    if cache_content.list.len() > 0 {
      // write cache file
      let file_name = cache_content.file_name(cache_file_index);
      let file_path = cache_file_dir.join(file_name);

      let bytes = serialize!(&cache_content);
      std::fs::write(file_path, bytes).unwrap();
    }
  }

  pub fn read_cache(&self, cache_type: &str) -> HashMap<String, Vec<u8>> {
    let cache_file_dir = self.cache_dir.join(cache_type);
    // read all cache files from the cache file dir
    let mut cache_map = HashMap::new();

    if cache_file_dir.exists() && cache_file_dir.is_dir() {
      for file in std::fs::read_dir(cache_file_dir).unwrap() {
        if let Ok(entry) = file {
          let file_path = entry.path();

          if file_path.is_file()
            && entry
              .file_name()
              .to_string_lossy()
              .to_string()
              .starts_with("farm-cache-part-")
          {
            let bytes = std::fs::read(file_path).unwrap();
            let cache_content: CacheContentFile = deserialize!(&bytes, CacheContentFile);
            cache_map.extend(cache_content.list);
          }
        } else {
          println!("[warn] Failed to read cache file: {:?}", file);
        }
      }
    }

    cache_map
  }
}

#[cache_item]
pub struct CacheContentFile {
  list: HashMap<String, Vec<u8>>,
}

impl CacheContentFile {
  pub fn new() -> Self {
    Self {
      list: HashMap::new(),
    }
  }

  pub fn push(&mut self, item: (String, Vec<u8>)) {
    self.list.insert(item.0, item.1);
  }

  pub fn clear(&mut self) {
    self.list.clear();
  }

  pub fn file_name(&self, index: usize) -> String {
    format!("farm-cache-part-{}", index)
  }
}
