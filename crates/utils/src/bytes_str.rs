use std::{
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
};

use bytes::Bytes;
use rkyv::hash::FxHasher64;

/// Zero copy byte str inspired by https://github.com/dudykr/ddbase/blob/main/crates/bytes-str/src/byte_str.rs
#[derive(Clone, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Hash, PartialEq, Eq))]
pub struct FarmBytesStr {
  pub(crate) bytes: Bytes,
  precomputed_hash: Option<u64>,
}

impl FarmBytesStr {
  pub fn new() -> Self {
    Self {
      bytes: Bytes::new(),
      precomputed_hash: None,
    }
  }

  pub fn from_string(s: String) -> Self {
    s.into()
  }

  pub fn as_str(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.bytes) }
  }

  pub fn into_bytes(self) -> Bytes {
    self.bytes
  }
}

impl From<String> for FarmBytesStr {
  fn from(s: String) -> Self {
    let bytes = Bytes::from(s.into_bytes());
    let mut hasher = FxHasher64::default();
    bytes.hash(&mut hasher);

    Self {
      bytes,
      precomputed_hash: Some(hasher.finish()),
    }
  }
}

impl AsRef<str> for FarmBytesStr {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl fmt::Debug for FarmBytesStr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(self.as_str(), f)
  }
}

impl fmt::Display for FarmBytesStr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self.as_str(), f)
  }
}

impl Hash for FarmBytesStr {
  fn hash<H: Hasher>(&self, state: &mut H) {
    if let Some(hash) = self.precomputed_hash {
      hash.hash(state);
    } else {
      self.bytes.hash(state);
    }
  }
}

impl PartialEq for FarmBytesStr {
  fn eq(&self, other: &Self) -> bool {
    if self.precomputed_hash != other.precomputed_hash {
      return false;
    }

    self.bytes == other.bytes
  }
}

impl Eq for FarmBytesStr {}

impl Ord for FarmBytesStr {
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl PartialOrd for FarmBytesStr {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
