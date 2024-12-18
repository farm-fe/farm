use downcast_rs::{impl_downcast, Downcast};

pub trait Cacheable: std::any::Any + Send + Sync + Downcast {
  /// Serialize the data to bytes
  fn serialize_bytes(&self) -> Result<Vec<u8>, String>;
  /// Deserialize the bytes to data
  fn deserialize_bytes(&self, bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>;
}

impl_downcast!(Cacheable);
