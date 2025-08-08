#[derive(Debug)]
pub enum CacheError {
  IoError(std::io::Error),
  GenericError(String),
}

impl From<std::io::Error> for CacheError {
  fn from(e: std::io::Error) -> Self {
    CacheError::IoError(e)
  }
}

impl From<String> for CacheError {
  fn from(e: String) -> Self {
    CacheError::GenericError(e)
  }
}

impl From<&str> for CacheError {
  fn from(e: &str) -> Self {
    CacheError::GenericError(e.to_string())
  }
}
