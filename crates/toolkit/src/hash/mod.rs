use sha2::{Sha256, Digest};

pub fn sha256(bytes: &[u8], len: usize) -> String {
  let mut hasher = Sha256::new();

  hasher.update(bytes);
  // Note that calling `finalize()` consumes hasher
  let hash = hasher.finalize();

  format!("{:x}", hash)[..len].to_string()
}
