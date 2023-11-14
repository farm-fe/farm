//! Cache store of the persistent cache, responsible for reading and writing the cache from the disk.
use rkyv::Deserialize;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use farmfe_macro_cache_item::cache_item;

use crate::{config::Mode, deserialize, serialize};

const FARM_CACHE_VERSION: &str = "0.0.1";
// TODO make CacheStore a trait and implement DiskCacheStore or RemoteCacheStore or more.
#[derive(Default)]
pub struct CacheStore {
  cache_dir: PathBuf,
  namespace: String,
  /// The maximum number of items per cache file.
  items_per_cache_file: usize,
}

impl CacheStore {
  pub fn new(cache_dir_str: &str, namespace: &str, mode: Mode) -> Self {
    let mut cache_dir = Path::new(&format!("{cache_dir_str}-{FARM_CACHE_VERSION}")).to_path_buf();

    if matches!(mode, Mode::Development) {
      cache_dir.push("development");
    } else {
      cache_dir.push("production");
    }

    Self {
      cache_dir,
      namespace: namespace.to_string(),
      // only generate one cache file for now.
      items_per_cache_file: usize::MAX,
    }
  }

  pub fn set_items_per_cache_file(&mut self, size: usize) {
    self.items_per_cache_file = size;
  }

  /// Write the cache map to the disk.
  /// A cache file will be created for every 1000 items.
  pub fn write_cache(&self, cache_map: HashMap<String, Vec<u8>>, cache_type: &str) {
    if self.namespace.is_empty() && self.items_per_cache_file == usize::default() {
      return;
    }

    let start = std::time::Instant::now();
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
        let file_name = cache_content.file_name(&self.namespace, cache_file_index);
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
      let file_name = cache_content.file_name(&self.namespace, cache_file_index);
      let file_path = cache_file_dir.join(file_name);

      let bytes = serialize!(&cache_content);
      std::fs::write(file_path, bytes).unwrap();
    }
    println!("[store] write cache time: {:?}", start.elapsed());
  }

  pub fn read_cache(&self, cache_type: &str) -> HashMap<String, Vec<u8>> {
    if self.namespace.is_empty() && self.items_per_cache_file == usize::default() {
      return HashMap::new();
    }

    let start = std::time::Instant::now();
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
              .starts_with(self.namespace.as_str())
          {
            let start = std::time::Instant::now();
            let bytes = std::fs::read(file_path).unwrap();
            println!(
              "[store] read {cache_type} cache file time: {:?}",
              start.elapsed()
            );
            let cache_content: CacheContentFile = deserialize!(&bytes, CacheContentFile);
            cache_map.extend(cache_content.list);
          }
        } else {
          println!("[warn] Failed to read cache file: {:?}", file);
        }
      }
    }

    println!(
      "[store] read {cache_type} cache time: {:?}",
      start.elapsed()
    );

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

  pub fn file_name(&self, namespace: &str, index: usize) -> String {
    format!("{namespace}-{}", index)
  }
}
