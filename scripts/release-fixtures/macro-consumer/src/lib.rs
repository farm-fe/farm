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


#[farmfe_macro_cache_item::cache_item]
#[derive(Debug, PartialEq)]
struct Record { name: String, values: Vec<Option<u32>> }
#[farmfe_macro_cache_item::cache_item]
#[derive(Debug, PartialEq)]
enum Entry { Empty, Named(String), Pair(u32, bool) }
#[cfg(test)]
mod tests { use super::*;
#[test] fn struct_round_trip() { let expected=Record{name:"candidate".into(),values:vec![Some(7),None]}; let bytes=expected.serialize_bytes().unwrap(); let actual=Record::deserialize_bytes(bytes).unwrap(); assert_eq!(actual.downcast_ref::<Record>().unwrap(), &expected); }
#[test] fn enum_round_trip() { for expected in [Entry::Empty,Entry::Named("V2".into()),Entry::Pair(42,true)] { let bytes=expected.serialize_bytes().unwrap(); let actual=Entry::deserialize_bytes(bytes).unwrap(); assert_eq!(actual.downcast_ref::<Entry>().unwrap(), &expected); } }
}
