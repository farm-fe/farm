#[macro_export]
macro_rules! serialize {
  ($t:expr) => {{
    let bytes = rkyv::to_bytes::<_, 1024>($t).unwrap();
    bytes.to_vec()
  }};
}

#[macro_export]
macro_rules! deserialize {
  ($bytes:expr, $ty:ty) => {{
    let archived = unsafe { rkyv::archived_root::<$ty>($bytes) };
    let deserialized: $ty = archived
      .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
      .unwrap();

    deserialized
  }};
}
