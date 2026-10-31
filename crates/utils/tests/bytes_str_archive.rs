use farmfe_utils::bytes_str::FarmBytesStr;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

fn cases() -> Vec<FarmBytesStr> {
  vec![
    FarmBytesStr::default(),
    FarmBytesStr::from_string(String::new()),
    FarmBytesStr::from_string("SKU-2048".into()),
    FarmBytesStr::from_string("src/products.tsx?raw".into()),
    FarmBytesStr::from_string("商品/库存.ts?lang=中文".into()),
    FarmBytesStr::from_string("catalog/".repeat(256)),
  ]
}

fn hash(value: &impl Hash) -> u64 {
  let mut hasher = DefaultHasher::new();
  value.hash(&mut hasher);
  hasher.finish()
}

#[test]
fn archived_hash_matches_original_for_each_hash_state() {
  for (index, value) in cases().iter().enumerate() {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value).expect("serialize string");
    let archived = rkyv::access::<rkyv::Archived<FarmBytesStr>, rkyv::rancor::Error>(&bytes)
      .expect("validate string");
    assert_eq!(hash(value), hash(archived), "input {index}");
  }
}

#[test]
fn archived_map_finds_every_original_entry() {
  let values: HashMap<_, _> = cases()
    .into_iter()
    .enumerate()
    .map(|(i, key)| (key, i as u32))
    .collect();
  let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&values).expect("serialize map");
  let archived =
    rkyv::access::<rkyv::Archived<HashMap<FarmBytesStr, u32>>, rkyv::rancor::Error>(&bytes)
      .expect("validate map");
  for (key, value) in archived.iter() {
    assert_eq!(archived.get(key), Some(value), "missing archived entry");
  }
}

#[test]
fn archived_singleton_map_finds_each_hash_state() {
  for (index, key) in cases().into_iter().enumerate() {
    let values = HashMap::from([(key, 42_u32)]);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&values).expect("serialize singleton");
    let archived =
      rkyv::access::<rkyv::Archived<HashMap<FarmBytesStr, u32>>, rkyv::rancor::Error>(&bytes)
        .expect("validate singleton");
    let (key, value) = archived.iter().next().expect("entry exists");
    assert_eq!(archived.get(key), Some(value), "input {index}");
  }
}
