use std::{mem, sync::Arc};

use farmfe_core::{
  deserialize, module::meta_data::script::CommentsMetaData, serialize, Cacheable, DashMap, HashMap,
};
use rkyv::{Archive, Archived, Deserialize, Fallible, Serialize};

impl<'a, __D: Fallible + ?Sized> Deserialize<LightningCssParseResult<'a>, __D>
  for Archived<LightningCssParseResult<'a>>
{
  #[inline]
  fn deserialize(
    &self,
    deserializer: &mut __D,
  ) -> ::core::result::Result<LightningCssParseResult<'a>, __D::Error> {
    let mut map =
      Deserialize::<HashMap<String, Vec<u8>>, __D>::deserialize(&self.map, deserializer)?;

    let source = map
      .remove("source")
      .map(|bytes| {
        let source = String::from_utf8(bytes).unwrap();
        Arc::new(source)
      })
      .unwrap();
    let comments = map
      .remove("comments")
      .map(|bytes| {
        CommentsMetaData::deserialize_bytes(&CommentsMetaData::default(), bytes)
          .unwrap()
          .downcast::<CommentsMetaData>()
          .unwrap_or_default()
      })
      .unwrap();

    let c = unsafe { Box::into_raw(Box::new(source.clone())) };
    let m = unsafe { &*c };
    let ast = farmfe_core::serde_json::from_str(&m).unwrap();

    let res = LightningCssParseResult {
      ast: ast,
      comments: *comments,
      source_map: None,
      source,
      bytes: Default::default(),
    };

    // res.bytes_map = map.into_iter().collect();
    Ok(res)
  }
}

impl<'a, __S: Fallible + ?Sized> Serialize<__S> for LightningCssParseResult<'a>
where
  __S: rkyv::ser::Serializer + rkyv::ser::ScratchSpace,
{
  #[inline]
  fn serialize(&self, serializer: &mut __S) -> ::core::result::Result<Self::Resolver, __S::Error> {
    let mut map = HashMap::<String, Vec<u8>>::default();

    let vec = farmfe_core::serde_json::to_vec(&self.ast).unwrap();

    map.insert("ast".to_string(), vec);
    map.insert(
      "comments".to_string(),
      self.comments.serialize_bytes().unwrap(),
    );
    if let Some(source_map) = &self.source_map {
      map.insert("source_map".to_string(), source_map.as_bytes().to_vec());
    }
    map.insert("source".to_string(), self.source.as_bytes().to_vec());

    let resolver_map = Serialize::<__S>::serialize(&map, serializer)?;

    // self.bytes = map;
    for (k, v) in map {
      self.bytes.insert(k, v);
    }

    Ok(CustomMetaDataMapResolver { map: resolver_map })
  }
}

pub struct ArchivedCustomMetaDataMap {
  ///The archived counterpart of [`LightningCssParseResult::map`]
  pub map: ::rkyv::Archived<HashMap<String, Vec<u8>>>,
}

pub struct CustomMetaDataMapResolver {
  map: ::rkyv::Resolver<HashMap<String, Vec<u8>>>,
}

impl<'a> Archive for LightningCssParseResult<'a> {
  type Archived = ArchivedCustomMetaDataMap;
  type Resolver = CustomMetaDataMapResolver;
  #[allow(clippy::unit_arg)]
  #[inline]
  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    let (fp, fo) = {
      #[allow(unused_unsafe)]
      unsafe {
        let fo = &raw mut (*out).map;
        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
      }
    };

    let mut map = HashMap::default();

    let mut keys = Vec::new();

    for item in self.bytes.iter() {
      keys.push(item.key().clone());
    }

    for key in keys {
      let (k, v) = self.bytes.remove(&key).unwrap();
      map.insert(k, v);
    }

    ::rkyv::Archive::resolve(&map, pos + fp, resolver.map, fo);
  }
}

#[derive(Debug)]
pub struct LightningCssParseResult<'a> {
  pub ast: Option<lightningcss::stylesheet::StyleSheet<'a, 'a>>,
  pub comments: CommentsMetaData,
  pub source_map: Option<String>,
  pub source: Arc<String>,
  pub bytes: DashMap<String, Vec<u8>>,
}

impl<'a: 'static> Cacheable for LightningCssParseResult<'a> {
  fn serialize_bytes(&self) -> Result<Vec<u8>, String> {
    Ok(serialize!(self))
  }

  fn deserialize_bytes(&self, bytes: Vec<u8>) -> Result<Box<dyn Cacheable>, String> {
    let v = deserialize!(&bytes, Box<LightningCssParseResult<'a>>);
    Ok(v)
  }
}

// impl<'a> farmfe_core::serde::Deserialize<'a> for LightningCssParseResult<'a> {
//   fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//   where
//     D: farmfe_core::serde::Deserializer<'a>,
//   {
//     let mut map = HashMap::<String, value::Value>::deserialize(deserializer)?;
//     // deserializer.deserialize_string(visitor)
//     let ast = map.remove("ast").and_then(|v| {
//       serde_json::from_str::<lightningcss::stylesheet::StyleSheet<'static, 'static>>(v.get()).ok()
//     });
//     let comments = map
//       .remove("comments")
//       .and_then(|v| {
//         CommentsMetaData::deserialize_bytes(&CommentsMetaData::default(), v.get().as_bytes()).ok()
//       })
//       .and_then(|v| v.downcast::<CommentsMetaData>().ok())
//       .unwrap_or_default();
//     let source_map = map.remove("source_map").map(|v| v.get().to_string());

//     Ok(LightningCssParseResult {
//       ast,
//       comments,
//       source_map,
//       source: Arc::new(String::new()),
//     })
//   }
// }

impl Default for LightningCssParseResult<'_> {
  fn default() -> Self {
    LightningCssParseResult {
      ast: None,
      comments: CommentsMetaData::default(),
      source_map: None,
      source: Arc::new(String::new()),
      bytes: Default::default(),
    }
  }
}
