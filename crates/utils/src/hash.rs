use base64::{engine::general_purpose, Engine};
use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8], len: usize) -> String {
  let mut hasher = Sha256::new();

  hasher.update(bytes);
  // Note that calling `finalize()` consumes hasher
  let hash = hasher.finalize();

  format!("{hash:x}")[..len].to_string()
}

pub fn base64_encode(bytes: &[u8]) -> String {
  general_purpose::STANDARD.encode(bytes)
}

pub fn base64_decode(bytes: &[u8]) -> String {
  // Handle Windows long path issue: return safe representation if decode fails
  match general_purpose::STANDARD.decode(bytes) {
    Ok(decoded) => {
      match String::from_utf8(decoded.clone()) {
        Ok(s) => s,
        Err(_) => {
          // Decoded bytes are not valid UTF-8 - return a lossy UTF-8 view of decoded bytes
          String::from_utf8_lossy(&decoded).to_string()
        }
      }
    },
    Err(_) => {
      // If base64 decode fails (usually due to long paths), return safe representation of original input
      String::from_utf8_lossy(bytes).to_string()
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_sha256() {
    assert_eq!(super::sha256(b"hello world", 8), "b94d27b9".to_string());
  }

  #[test]
  fn test_base64_encode() {
    assert_eq!(super::base64_encode(b"hello world"), "aGVsbG8gd29ybGQ=");
  }

  #[test]
  fn test_base64_decode() {
    assert_eq!(
      super::base64_decode(b"aGVsbG8gd29ybGQ="),
      "hello world".to_string()
    );
  }
}
