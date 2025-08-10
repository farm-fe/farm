use downcast_rs::{impl_downcast, Downcast};

use crate::{deserialize, serialize};

pub trait Cacheable: std::any::Any + Send + Sync + Downcast {
  /// Serialize the data to bytes
  fn serialize_bytes(&self) -> Result<Vec<u8>, String>;
  /// Deserialize the bytes to data
  fn deserialize_bytes(bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>
  where
    Self: Sized;
}

impl_downcast!(Cacheable);

macro_rules! impl_primitive_cacheable {
    ($($t:ty),*) => {
        $(
            impl Cacheable for $t {
                fn serialize_bytes(&self) -> Result<Vec<u8>, String> {
                    Ok(serialize!(self))
                }

                fn deserialize_bytes(bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String>
                where
                    Self: Sized,
                {
                    Ok(Box::new(deserialize!(&bytes, $t, rkyv::Archived<$t>)))
                }
            }
        )*
    };
    () => {};
}

impl_primitive_cacheable!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char, String);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn t1() {
    let value = 1;

    let bytes = value.serialize_bytes().expect("expect serialize success");
    let deserialize_value = i32::deserialize_bytes(bytes).unwrap();

    assert_eq!(deserialize_value.downcast_ref::<i32>().unwrap(), &1);
  }
}
