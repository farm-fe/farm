use std::sync::Arc;

use farmfe_core::{
  deserialize,
  error::{CompilationError, Result},
  module::{meta_data::script::CommentsMetaData, CustomMetaDataMap},
  parking_lot::Mutex,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::meta_data::ResourcePotMetaData,
  serialize, Cacheable, DashMap, HashMap,
};
use lightningcss::{
  printer::PrinterOptions,
  stylesheet::{StyleSheet, ToCssResult},
  visitor::Visit,
};
use parcel_sourcemap::SourceMap;
use rkyv::{Archive, Archived, Deserialize, Fallible, Serialize};

use crate::{
  adapter::adapter_trait::{
    CodegenContext, CreateResourcePotMetadataContext, CssToScriptContext, CssToScriptResult,
    SourceReplaceContext,
  },
  source_replacer::SourceReplacer,
};

fn deserialize_stylesheet<'a>(
  source: Arc<String>,
) -> std::result::Result<lightningcss::stylesheet::StyleSheet<'a, 'a>, String> {
  let c = { Box::into_raw(Box::new(source)) };
  let m = unsafe { &*c };
  let ast = farmfe_core::serde_json::from_str(&m).map_err(|e| e.to_string())?;
  Ok(ast)
}

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

    let comments = map
      .remove("comments")
      .map(|bytes| {
        CommentsMetaData::deserialize_bytes(&CommentsMetaData::default(), bytes)
          .unwrap()
          .downcast::<CommentsMetaData>()
          .unwrap_or_default()
      })
      .unwrap();
    let (ast, source) = map
      .remove("ast")
      .map(|bytes| {
        // let source = Arc::new(String::from_utf8(bytes).unwrap());
        let result = Arc::new(String::from_utf8_lossy(&bytes).to_string());
        deserialize_stylesheet(result.clone())
          .ok()
          .map(|v| (v, result))
      })
      .flatten()
      .unwrap();
    let source_map = map
      .remove("source_map")
      .map(|v| String::from_utf8(v).unwrap());

    let res = LightningCssParseResult {
      ast: Some(ast),
      comments: *comments,
      source_map,
      source: vec![source],
      bytes: Default::default(),
    };

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
    map.insert(
      "source".to_string(),
      farmfe_core::serde_json::to_vec(&self.source).unwrap(),
    );

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
  pub source: Vec<Arc<String>>,
  pub bytes: DashMap<String, Vec<u8>>,
}

impl<'a: 'static> Cacheable for LightningCssParseResult<'a> {
  fn serialize_bytes(&self) -> std::result::Result<Vec<u8>, String> {
    Ok(serialize!(self))
  }

  fn deserialize_bytes(&self, bytes: Vec<u8>) -> std::result::Result<Box<dyn Cacheable>, String> {
    let v = deserialize!(&bytes, LightningCssParseResult<'a>);
    Ok(Box::new(v))
  }
}

pub fn source_replace(
  SourceReplaceContext {
    module,
    module_graph,
    resources_map,
    context,
    ..
  }: SourceReplaceContext,
) -> Result<Option<(StyleSheet<'static, 'static>, Arc<String>)>> {
  let mut replacer = SourceReplacer::new(
    module.id.clone(),
    module_graph,
    resources_map,
    context.config.output.public_path.clone(),
    context.config.resolve.alias.clone(),
  );

  let result = module
    .meta
    .get_custom::<LightningCssParseResult<'static>>("lightning_css");

  if let Some(ast) = result.ast.as_ref() {
    let v = Arc::new(farmfe_core::serde_json::to_string(ast).unwrap());
    let mut ast = deserialize_stylesheet(v.clone()).unwrap();
    ast.visit(&mut replacer)?;

    return Ok(Some((ast, v)));
  }

  Ok(None)
}

// pub fn codegen(context: CodegenContext) -> Result<(String, Option<String>)> {

// }

pub fn css_to_script(
  CssToScriptContext { module_id, context }: CssToScriptContext<'_>,
) -> Result<CssToScriptResult> {
  let module_graph = context.module_graph.read();
  let resources_map = context.resources_map.lock();
  let module = module_graph.module(module_id).unwrap();
  let (stylesheet, _) = source_replace(SourceReplaceContext {
    module,
    module_graph: &module_graph,
    resources_map: &resources_map,
    context,
    without_context: true,
  })?
  .unwrap();

  let mut src: Option<SourceMap> = if context.config.sourcemap.enabled(module.immutable) {
    Some(SourceMap::new(""))
  } else {
    None
  };
  let ToCssResult { code, .. } = stylesheet
    .to_css(PrinterOptions {
      minify: context.config.minify.enabled(),
      source_map: src.as_mut(),
      targets: Default::default(),
      ..Default::default()
    })
    .unwrap();

  Ok(CssToScriptResult {
    code,
    source_map: src.as_mut().map(|v| v.to_json(None).ok()).flatten(),
  })
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

pub fn create_resource_pot_metadata(
  CreateResourcePotMetadataContext {
    context,
    modules,
    module_execution_order,
    module_graph,
    ..
  }: CreateResourcePotMetadataContext<'_>,
) -> Result<ResourcePotMetaData> {
  let resources_map = context.resources_map.lock();

  let rendered_modules = Mutex::new(Vec::with_capacity(modules.len()));
  modules.into_par_iter().try_for_each(|module| {
    println!(
      "{:#?}",
      match &module.meta {
        box farmfe_core::module::ModuleMetaData::Script(_) => "Script",
        box farmfe_core::module::ModuleMetaData::Css(_) => "Css",
        box farmfe_core::module::ModuleMetaData::Html(_) => "Html",
        box farmfe_core::module::ModuleMetaData::Custom(_) => "Custom",
      }
    );

    let css_stylesheet = source_replace(SourceReplaceContext {
      module,
      module_graph,
      resources_map: &resources_map,
      context,
      without_context: true,
    })?
    .unwrap();

    rendered_modules
      .lock()
      .push((module.id.clone(), css_stylesheet));

    Ok::<(), CompilationError>(())
  })?;

  let mut rendered_modules = rendered_modules.into_inner();

  rendered_modules.sort_by_key(|module| module_execution_order[&module.0]);

  let mut stylesheet: Option<StyleSheet> = None;

  // let source_map = merge_css_sourcemap(&mut rendered_modules, context);
  // context
  //   .meta
  //   .set_resource_pot_source_map(&resource_pot.id, source_map);

  let mut r = LightningCssParseResult {
    ast: None,
    ..Default::default()
  };
  for (_, (rendered_module_ast, source_code)) in rendered_modules {
    // rendered_module_ast
    if let Some(ref mut stylesheet_ast) = stylesheet {
      let StyleSheet { rules, sources, .. } = rendered_module_ast;
      stylesheet_ast.rules.0.extend(rules.0);
      stylesheet_ast.sources.extend(sources);
    } else {
      stylesheet = Some(rendered_module_ast);
    }

    r.source.push(source_code);
  }

  r.ast = stylesheet;

  let map = CustomMetaDataMap::new();

  map.insert("lightning_css".to_string(), Box::new(r));

  Ok(ResourcePotMetaData::Custom(map))
}

pub fn codegen_for_resource_pot(
  CodegenContext {
    context,
    resource_pot,
  }: CodegenContext,
) -> Result<(String, Option<String>)> {
  let parse_result = resource_pot
    .meta
    .get_custom::<LightningCssParseResult>("lightning_css");

  let mut source_map = if context.config.sourcemap.enabled(resource_pot.immutable) {
    Some(SourceMap::new(""))
  } else {
    None
  };
  let ToCssResult { code, .. } = parse_result
    .ast
    .as_ref()
    .unwrap()
    .to_css(PrinterOptions {
      minify: context.config.minify.enabled(),
      source_map: source_map.as_mut(),
      targets: Default::default(),
      ..Default::default()
    })
    .unwrap();

  Ok((
    code,
    source_map.as_mut().map(|v| v.to_json(None).ok()).flatten(),
  ))
}

impl Default for LightningCssParseResult<'_> {
  fn default() -> Self {
    LightningCssParseResult {
      ast: None,
      comments: CommentsMetaData::default(),
      source_map: None,
      source: vec![],
      bytes: Default::default(),
    }
  }
}
