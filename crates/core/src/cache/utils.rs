pub fn cache_panic(key: &str, cache_dir: &str) -> ! {
  panic!("Cache broken: {key} not found, please remove {cache_dir} and retry.")
}
