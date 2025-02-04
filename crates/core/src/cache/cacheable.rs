use downcast_rs::{impl_downcast, Downcast};

pub trait Cacheable: std::any::Any + Send + Sync + Downcast {
  /// Serialize the data to bytes
  fn serialize_bytes(&self) -> Result<Vec<u8>, String>;
  /// Deserialize the bytes to data
  fn deserialize_bytes(bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>
  where
    Self: Sized;
}

impl_downcast!(Cacheable);
