use std::fmt::{Debug, Formatter};

use downcast_rs::{impl_downcast, Downcast};
use rkyv::{ser::serializers::AllocSerializer, Archive, Deserialize, Serialize};

pub trait Cacheable: std::any::Any + Send + Sync + Downcast {
  /// Serialize the data to bytes
  fn serialize_bytes(&self) -> Result<Vec<u8>, String>;
  /// Deserialize the bytes to data
  fn deserialize_bytes(bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>
  where
    Self: Sized;
}

impl_downcast!(Cacheable);

pub struct CacheableContainer<T> {
  inner: T,
}

impl<T> Cacheable for CacheableContainer<T>
where
  T: Serialize<AllocSerializer<256>> + Archive + Send + Sync + 'static,
  T::Archived: Deserialize<T, rkyv::Infallible>,
{
  fn serialize_bytes(&self) -> Result<Vec<u8>, String> {
    let bytes = rkyv::to_bytes::<_, 256>(&self.inner).unwrap();
    Ok(bytes.into_vec())
  }

  fn deserialize_bytes(bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>
  where
    Self: Sized,
  {
    let archived = unsafe { rkyv::archived_root::<T>(&bytes[..]) };
    let deserialized: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
    Ok(Box::new(CacheableContainer {
      inner: deserialized,
    }))
  }
}

impl<T> From<T> for CacheableContainer<T>
where
  T: Serialize<AllocSerializer<256>> + Archive + Sync + Send + 'static,
  T::Archived: Deserialize<T, rkyv::Infallible>,
{
  fn from(value: T) -> Self {
    CacheableContainer { inner: value }
  }
}

impl<T> CacheableContainer<T> {
  pub fn new(value: T) -> Self {
    CacheableContainer { inner: value }
  }

  pub fn take(self) -> T {
    self.inner
  }

  pub fn value(&self) -> &T {
    &self.inner
  }

  pub fn value_mut(&mut self) -> &mut T {
    &mut self.inner
  }
}

impl<T> Clone for CacheableContainer<T>
where
  T: Clone,
{
  fn clone(&self) -> Self {
    CacheableContainer {
      inner: self.inner.clone(),
    }
  }
}

impl<T> Debug for CacheableContainer<T>
where
  T: Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(format!("CacheableContainer({:?})", &self.inner).as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn t1() {
    let value = 1;

    let v: CacheableContainer<_> = value.into();

    let bytes = v.serialize_bytes().expect("expect serialize success");
    let deserialize_value = CacheableContainer::<i32>::deserialize_bytes(bytes).unwrap();

    assert_eq!(
      deserialize_value
        .downcast_ref::<CacheableContainer<i32>>()
        .unwrap()
        .value(),
      &1
    );
  }
}
