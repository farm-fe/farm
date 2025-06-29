use std::{any::Any, borrow::Cow, sync::Arc};

use downcast_rs::{impl_downcast, Downcast};
use farmfe_core::{
  cache_item, deserialize,
  error::Result,
  lazy_static::lazy_static,
  module::{meta_data::script::CommentsMetaData, CustomMetaDataMap, ModuleId, ModuleMetaData},
  petgraph::data::DataMap,
  plugin::PluginAnalyzeDepsHookResultEntry,
  resource::meta_data::ResourcePotMetaData,
  serde::{de::DeserializeOwned, Deserialize, Serialize},
  serde_json::{self, value},
  Cacheable, DashMap, HashMap,
};
use lightningcss::{css_modules::Config, printer::PrinterOptions, stylesheet::ToCssResult};

use crate::adapter::lightningcss::LightningCssParseResult;

///
/// adapter
///
/// apis:
/// 1. css modules
/// 2. parse
/// 3. source map
/// 4. prefixer
/// 5. codegen
/// 6. replace url
///

pub enum CssPluginAdapter {
  SwcCss,
  LightningCss(LightningCss),
}

enum CssPluginParseResult {
  SwcCss((farmfe_core::swc_css_ast::Stylesheet, CommentsMetaData)),
  LightningCss(LightningCssParseResult<'static>),
}

impl CssPluginParseResult {
  fn lightning_css_stylesheet(
    &self,
  ) -> Option<&lightningcss::stylesheet::StyleSheet<'static, 'static>> {
    match self {
      CssPluginParseResult::SwcCss(_) => None,
      CssPluginParseResult::LightningCss(ast) => Some(ast.ast()),
    }
  }

  fn ta_lightning_css(self) -> Option<LightningCssParseResult<'static>> {
    match self {
      CssPluginParseResult::SwcCss(_) => None,
      CssPluginParseResult::LightningCss(ast) => Some(ast),
    }
  }

  fn comments(&self) -> &CommentsMetaData {
    match self {
      CssPluginParseResult::SwcCss((_, comments)) => comments,
      CssPluginParseResult::LightningCss(ast) => ast.comments(),
    }
  }

  fn source_map(&self) -> Option<&String> {
    match self {
      CssPluginParseResult::SwcCss(_) => None,
      CssPluginParseResult::LightningCss(ast) => ast.source_map(),
    }
  }
}

struct CssPluginProcesser {
  adapter: CssPluginAdapter,
  ast_map: DashMap<String, CssPluginParseResult>,
}

#[derive(Debug, Clone, Default)]
struct ParseOption {
  module_id: String,
  data: HashMap<String, String>,
}

// impl Into<ParseOption> for ParseOption {
//   fn into(self) -> Cow<'static, ParseOption> {
//     Cow::Owned(self)
//   }
// }

impl From<ParseOption> for Cow<'static, ParseOption> {
  fn from(value: ParseOption) -> Self {
    Cow::Owned(value)
  }
}

impl<'a> From<&'a ParseOption> for Cow<'a, ParseOption> {
  fn from(value: &'a ParseOption) -> Self {
    Cow::Borrowed(value)
  }
}

impl CssPluginProcesser {
  pub fn parse<'a, O>(&self, content: Arc<String>, options: O) -> Result<()>
  where
    O: Into<Cow<'a, ParseOption>>,
  {
    let options: Cow<'a, ParseOption> = options.into();
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        // Implement SwcCss parsing logic here
        unimplemented!()
      }
      CssPluginAdapter::LightningCss(adapter) => {
        let content = adapter.parse(content)?;

        self.ast_map.insert(
          options.module_id.to_string(),
          CssPluginParseResult::LightningCss(content),
        );
      }
    }

    Ok(())
  }

  pub fn css_modules(
    &self,
    content: Arc<String>,
    options: ParseOption,
  ) -> Result<Option<CssModuleExports>> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        // Implement SwcCss css_modules logic here
        unimplemented!()
      }

      CssPluginAdapter::LightningCss(_) => {
        self.parse(content, &options)?;
        let ast = self.ast_map.get(&options.module_id).unwrap();
        let m = ast.value().lightning_css_stylesheet().unwrap();

        let ToCssResult { exports, .. } = m
          .to_css(PrinterOptions {
            ..Default::default()
          })
          .unwrap();

        return Ok(exports.map(|map| {
          map
            .into_iter()
            .fold(HashMap::default(), |mut acc, (name, export)| {
              acc.insert(
                name,
                CssModuleExport {
                  name: export.name,
                  reference: export
                    .composes
                    .into_iter()
                    .map(|reference| match reference {
                      lightningcss::css_modules::CssModuleReference::Local { name } => {
                        CssModuleReference::Local { name }
                      }
                      lightningcss::css_modules::CssModuleReference::Global { name } => {
                        CssModuleReference::Global { name }
                      }
                      lightningcss::css_modules::CssModuleReference::Dependency {
                        name,
                        specifier,
                      } => CssModuleReference::Dependency { name, specifier },
                    })
                    .collect(),
                },
              );

              acc
            })
        }));

        // self
        //   .ast_map
        //   .get(&options.module_id.to_string())
        //   .and_then(|v| {
        //     // if let CssPluginParseResult::LightningCss(parse_result) = v.value() {
        //     //   adapter.css_modules(parse_result.ast())
        //     // } else {
        //     //   None
        //     // }
        //   })
        //   .transpose()
      }
    }

    Ok(None)
  }

  pub fn create_module_data(
    &self,
    module_id: &String,
    content: Arc<String>,
  ) -> Result<ModuleMetaData> {
    let ast = if let Some((_, v)) = self.ast_map.remove(module_id) {
      v
    } else {
      self.parse(
        content,
        ParseOption {
          module_id: module_id.clone(),
          ..Default::default()
        },
      )?;

      self.ast_map.remove(module_id).unwrap().1
    };

    match &self.adapter {
      CssPluginAdapter::SwcCss => todo!(),
      CssPluginAdapter::LightningCss(_) => {
        let mut map = CustomMetaDataMap::default();
        map.insert(
          "lightning_css".to_string(),
          Box::new(ast.ta_lightning_css().unwrap()),
        );

        Ok(ModuleMetaData::Custom(map))
      }
    }
  }

  pub fn prefixer(&self, metadata: &mut ModuleMetaData) -> Result<()> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        // Implement SwcCss prefixer logic here
        unimplemented!()
      }
      CssPluginAdapter::LightningCss(_) => {
        // Implement LightningCss prefixer logic here
        unimplemented!()
      }
    }
  }

  ///
  /// analyze css dependencies
  ///
  pub fn analyze_deps(&self) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        // Implement SwcCss analyze_deps logic here
        unimplemented!()
      }
      CssPluginAdapter::LightningCss(_) => {
        // self.adapter.analyze_deps()
        unimplemented!()
      }
    }
  }

  ///
  /// after build:
  /// 1. if reference another css file, replace the url with the real path
  /// 2. if combine reference file content, remove the @import
  ///
  pub fn source_replace(&self, metadata: &mut ModuleMetaData) -> Result<()> {
    Ok(())
  }

  pub fn merge_sourcemap(&self) -> Result<()> {
    Ok(())
  }

  pub fn create_resource_pot_metadata(&self, module_id: &ModuleId) -> Result<ModuleMetaData> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => todo!(),
      CssPluginAdapter::LightningCss(lightning_css) => todo!(),
    }
    // let mut map = CustomMetaDataMap::default();
    // map.insert(
    //   "lightning_css".to_string(),
    //   Box::new(LightningCssParseResult::default()),
    // );

    // Ok(ModuleMetaData::Custom(map))
  }

  pub fn codegen(&self, metadata: &mut ResourcePotMetaData) -> Result<(String, Option<String>)> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        // Implement SwcCss codegen logic here
        unimplemented!()
      }
      CssPluginAdapter::LightningCss(_) => {
        // Implement LightningCss codegen logic here
        unimplemented!()
      }
    }
  }
}

// impl<A, M> CssAdapter<A, M> for CssPluginAdapter {
//   fn parse(&self, content: Arc<String>) -> Result<Box<(dyn ParseResult<A, M> + 'static)>> {
//     match self {
//       CssPluginAdapter::SwcCss => {
//         // Implement SwcCss parsing logic here
//         unimplemented!()
//       }
//       CssPluginAdapter::LightningCss(adapter) => adapter.parse(content),
//     }
//   }
// }

pub trait ParseResult<A, M>: std::any::Any + Downcast + Cacheable {
  fn ast(&self) -> &A;
  fn ast_mut(&mut self) -> &mut A;
  fn take_ast(self) -> A;

  fn comments(&self) -> &M;
  fn comments_mut(&mut self) -> &mut M;
  fn take_comments(self) -> M;

  fn source_map(&self) -> Option<&String>;
  fn source_map_mut(&mut self) -> Option<&mut String>;
  fn take_source_map(self) -> Option<String>;
}

impl_downcast!(ParseResult<A, M>);

#[derive(Debug)]
pub enum CssModuleReference {
  Local { name: String },
  Global { name: String },
  Dependency { name: String, specifier: String },
}

#[derive(Debug)]
pub struct CssModuleExport {
  pub name: String,
  pub reference: Vec<CssModuleReference>,
}

///
///
/// ```css
/// .foo {
///   color: red;
///   compose: action from './bar.css';
/// }
/// ```
///
/// ```json
/// {
///   "foo": {
///       "name": "random_ident",
///       "reference": [
///          Local {
///             name: "foo",
///          }
///       ]
///   }
/// }
/// ```
///
pub type CssModuleExports = HashMap<String, CssModuleExport>;

pub trait CssAdapter<A, M> {
  fn parse(&self, content: Arc<String>) -> Result<Box<(dyn ParseResult<A, M> + 'static)>>;

  #[allow(unused_variables)]
  fn css_modules(&self, stylesheet: &A) -> Result<Option<CssModuleExports>> {
    Ok(None)
  }

  fn prefixer() {}

  #[allow(unused_variables)]
  fn codegen(&self, stylesheet: &A, options: CodegenOptions) -> Result<Option<CodegenResult>> {
    Ok(None)
  }

  fn replace_url() {}

  fn analyze_deps() -> Vec<PluginAnalyzeDepsHookResultEntry> {
    vec![]
  }

  fn serilized(content: &Box<(dyn ParseResult<A, M>)>) -> Option<HashMap<String, Vec<u8>>> {
    None
  }

  fn deserialized(
    content: HashMap<String, Vec<u8>>,
  ) -> Result<Option<Box<(dyn ParseResult<A, M>)>>> {
    Ok(None)
  }
}

// pub struct SwcCss {}

// impl SwcCss {
//   pub fn new() -> Self {
//     SwcCss {}
//   }

//   pub fn create_parse<'a>(&self, content: &'a String) {}
// }

// struct SwcCssParse<'a> {
//   content: &'a String,
// }

// impl<'a> SwcCssParse<'a> {
//   pub fn new(content: &'a String) -> Self {
//     SwcCssParse { content }
//   }

//   fn parse(
//     content: &'a str,
//   ) -> *mut (dyn ParseResult<lightningcss::stylesheet::StyleSheet<'a, '_>, CommentsMetaData> + 'a)
//   {
//     let options = lightningcss::stylesheet::ParserOptions::<'a, '_> {
//       css_modules: Some(Config::default()),
//       ..Default::default()
//     };
//     let stylesheet = lightningcss::stylesheet::StyleSheet::parse(&content, options).unwrap();

//     Box::into_raw(Box::new(LightningCssParseResult::<'a> {
//       ast: stylesheet,
//       comments: CommentsMetaData::default(),
//       source_map: None,
//       // source: content,
//     }))
//   }
// }

// impl CssAdapter<Stylesheet, CommentsMetaData> for SwcCss {
//   fn parse(content: String) -> Result<ParseResult<Stylesheet, CommentsMetaData>> {
//     let ParseCssModuleResult {
//       ast,
//       comments,
//       source_map,
//     } = parse_css_stylesheet("unknown", Arc::new(content))?;
//     Ok(Box::new((
//       ast,
//       CommentsMetaData::from(comments),
//       source_map,
//     )))
//   }
// }

pub struct LightningCss {}

impl LightningCss {
  fn parse(&self, content: Arc<String>) -> Result<LightningCssParseResult<'static>> {
    let options = lightningcss::stylesheet::ParserOptions::<'static, 'static> {
      css_modules: Some(Default::default()),
      ..Default::default()
    };

    // let v = unsafe { content.as_ref() };
    let ptr = Box::into_raw(Box::new(content));
    let v = unsafe { &*ptr };
    let stylesheet = lightningcss::stylesheet::StyleSheet::parse(&v, options).unwrap();

    Ok(LightningCssParseResult {
      ast: Some(stylesheet),
      comments: CommentsMetaData::default(),
      source_map: None,
      source: v.clone(),
      ..Default::default()
    })
  }
}

// type StyleSheet<'a, 'b> = lightningcss::stylesheet::StyleSheet<'a, 'b>;

// use farmfe_core::Cacheable;

// impl Cacheable for LightningCssParseResult<'_> {
//   fn serialize_bytes(&self) -> std::result::Result<Vec<u8>, String> {
//     // let mut data = Vec::new();
//     // if let Some(ast) = &self.ast {
//     //   data.extend_from_slice(ast.to_string().as_bytes());
//     // }
//     // data.extend_from_slice(self.comments.serialize_bytes()?.as_slice());
//     // if let Some(source_map) = &self.source_map {
//     //   data.extend_from_slice(source_map.as_bytes());
//     // }
//     // Ok(data)
//   }

//   fn deserialize_bytes(&self, bytes: Vec<u8>) -> std::result::Result<Box<dyn Cacheable>, String> {
//     todo!()
//   }
// }

impl<'a: 'static> ParseResult<lightningcss::stylesheet::StyleSheet<'a, 'a>, CommentsMetaData>
  for LightningCssParseResult<'a>
{
  fn ast(&self) -> &lightningcss::stylesheet::StyleSheet<'a, 'a> {
    self.ast.as_ref().expect("AST is not set")
  }

  fn ast_mut(&mut self) -> &mut lightningcss::stylesheet::StyleSheet<'a, 'a> {
    self.ast.as_mut().expect("AST is not set")
  }

  fn take_ast(mut self) -> lightningcss::stylesheet::StyleSheet<'a, 'a> {
    self.ast.take().expect("AST is not set")
  }

  fn comments(&self) -> &CommentsMetaData {
    &self.comments
  }

  fn comments_mut(&mut self) -> &mut CommentsMetaData {
    &mut self.comments
  }

  fn take_comments(self) -> CommentsMetaData {
    self.comments
  }

  fn source_map(&self) -> Option<&String> {
    self.source_map.as_ref()
  }

  fn source_map_mut(&mut self) -> Option<&mut String> {
    self.source_map.as_mut()
  }

  fn take_source_map(self) -> Option<String> {
    self.source_map
  }
}

pub struct CodegenOptions {
  pub minify: bool,
  pub source_map: Option<String>,
  pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct DynVec {
  pub data: Arc<Box<Vec<u8>>>,
}

pub struct CodegenResult {
  pub code: String,
  pub source_map: Option<String>,
}

lazy_static! {
  pub static ref LIGHTNING_SOURCE_CODE_MAP: HashMap<String, Arc<String>> = {
    let map = HashMap::default();

    map
  };
}

// impl CssAdapter<lightningcss::stylesheet::StyleSheet<'static, 'static>, CommentsMetaData>
//   for LightningCss
// {
//   fn parse(
//     &self,
//     content: Arc<String>,
//   ) -> Result<
//     Box<
//       (dyn ParseResult<lightningcss::stylesheet::StyleSheet<'static, 'static>, CommentsMetaData>),
//     >,
//   > {
//     let options = lightningcss::stylesheet::ParserOptions::<'static, 'static> {
//       css_modules: Some(Default::default()),
//       ..Default::default()
//     };

//     // let v = unsafe { content.as_ref() };
//     let ptr = Box::into_raw(Box::new(content));
//     let v = unsafe { &*ptr };
//     let stylesheet = lightningcss::stylesheet::StyleSheet::parse(&v, options).unwrap();

//     Ok(Box::new(LightningCssParseResult {
//       ast: Some(stylesheet),
//       comments: CommentsMetaData::default(),
//       source_map: None,
//       source: v.clone(),
//       ..Default::default()
//     }))
//   }

//   fn css_modules(
//     &self,
//     stylesheet: &lightningcss::stylesheet::StyleSheet<'static, 'static>,
//   ) -> Result<Option<CssModuleExports>> {
//     let mut sm = parcel_sourcemap::SourceMap::new("hello.css");
//     let result = stylesheet
//       .to_css(PrinterOptions {
//         source_map: Some(&mut sm),
//         ..Default::default()
//       })
//       .unwrap();

//     Ok(result.exports.map(|map| {
//       map
//         .into_iter()
//         .fold(HashMap::default(), |mut acc, (name, export)| {
//           acc.insert(
//             name,
//             CssModuleExport {
//               name: export.name,
//               reference: export
//                 .composes
//                 .into_iter()
//                 .map(|reference| match reference {
//                   lightningcss::css_modules::CssModuleReference::Local { name } => {
//                     CssModuleReference::Local { name }
//                   }
//                   lightningcss::css_modules::CssModuleReference::Global { name } => {
//                     CssModuleReference::Global { name }
//                   }
//                   lightningcss::css_modules::CssModuleReference::Dependency { name, specifier } => {
//                     CssModuleReference::Dependency { name, specifier }
//                   }
//                 })
//                 .collect(),
//             },
//           );

//           acc
//         })
//     }))
//   }

//   fn codegen(
//     &self,
//     stylesheet: &lightningcss::stylesheet::StyleSheet<'static, 'static>,
//     options: CodegenOptions,
//   ) -> Result<Option<CodegenResult>> {
//     let mut source_map = options
//       .source_map
//       .and_then(|_| Some(parcel_sourcemap::SourceMap::new("unknown")));

//     let printer_options = PrinterOptions {
//       minify: options.minify,
//       source_map: source_map.as_mut(),
//       ..Default::default()
//     };

//     let result = stylesheet.to_css(printer_options).unwrap();

//     Ok(Some(CodegenResult {
//       code: result.code,
//       source_map: source_map.map(|mut smap| smap.to_json(None).unwrap()),
//     }))
//   }

//   fn serilized<'i>(
//     content: &Box<
//       (dyn ParseResult<lightningcss::stylesheet::StyleSheet<'static, 'static>, CommentsMetaData>),
//     >,
//   ) -> Option<HashMap<String, Vec<u8>>> {
//     // let content_any = unsafe { *&content as *const (dyn Any + 'a) };

//     // let content = content_any.as_ref().unwrap()
//     //   .downcast_ref::<LightningCssParseResult>()
//     //   .unwrap();

//     let mut map = HashMap::default();

//     let ast = content.ast();

//     if let Ok(v) = serde_json::to_vec(ast) {
//       map.insert("ast".to_string(), v);
//     };

//     if let Ok(v) = content.comments().serialize_bytes() {
//       map.insert("comments".to_string(), v);
//     };

//     if let Some(source_map) = content.source_map() {
//       map.insert("source_map".to_string(), source_map.as_bytes().to_vec());
//     }

//     Some(map)
//   }

//   fn deserialized(
//     mut content: HashMap<String, Vec<u8>>,
//   ) -> Result<
//     Option<
//       Box<
//         (dyn ParseResult<lightningcss::stylesheet::StyleSheet<'static, 'static>, CommentsMetaData>),
//       >,
//     >,
//   > {
//     let comments = content
//       .remove("comments")
//       .and_then(|v| CommentsMetaData::deserialize_bytes(&CommentsMetaData::default(), v).ok())
//       .and_then(|v| v.downcast::<CommentsMetaData>().ok())
//       .unwrap_or_default();

//     let source_map = content
//       .remove("source_map")
//       .map(|v| String::from_utf8_lossy(&v).to_string());

//     let v = LIGHTNING_SOURCE_CODE_MAP.get("xx");

//     let ast = v.map(|ast_str| {
//       // let ptr = Box::into_raw(ast_str);
//       let text = ast_str.as_ref().as_str();

//       let m = serde_json::from_slice::<lightningcss::stylesheet::StyleSheet<'static, 'static>>(
//         text.as_bytes(),
//       )
//       .ok()
//       .unwrap();

//       // let xxx = unsafe { Box::from_raw(ptr) };

//       // data.insert("source".to_string(), xxx as Box<dyn Any>);
//       m
//     });

//     // let a1 = ast_str.as_ref().map(|v| {
//     //   Box::into_raw(v)
//     // });
//     // let ast = a1
//     //   .as_ref()
//     //   .and_then(|v| {
//     //     let x = unsafe { *v };
//     //     // let x1 = *x;
//     //     let m = serde_json::from_slice::<lightningcss::stylesheet::StyleSheet<'static, 'static>>(x.as_ref())
//     //       .ok()
//     //       .unwrap();

//     //     Some(m)
//     //   })
//     //   .map(|ast| lightningcss::stylesheet::StyleSheet::<'static, 'static>::from(ast));

//     // let ast_str = a1.unwrap();

//     // let dyn_dev: dyn Any + 'static = DynVec {
//     //   data: Arc::new(Box::new(vec![])),
//     // };
//     // let v = Box::new(ast_str) as Box<dyn Any>;

//     Ok(Some(Box::new(LightningCssParseResult {
//       ast,
//       comments: *comments,
//       source_map,
//       source: Arc::new(String::new()),
//       ..Default::default()
//     })))
//   }
// }

#[cfg(test)]
mod tests {

  use std::sync::Arc;

  use farmfe_core::module::Module;
  use farmfe_core::module::ModuleMetaData;
  use farmfe_core::serialize;

  use crate::adapter::adapter_trait::CssAdapter;
  use crate::adapter::adapter_trait::CssModuleReference;
  use crate::adapter::adapter_trait::CssPluginAdapter;
  use crate::adapter::adapter_trait::LightningCss;
  use crate::adapter::adapter_trait::LightningCssParseResult;
  use crate::adapter::adapter_trait::ParseResult;

  #[test]
  fn test_lightning_css_adapter() {
    let adapter = LightningCss {};
    let content = Arc::new(".foo { color: red; }".to_string());
    let parse_result = adapter.parse(content.clone()).unwrap();

    // let v = parse_result
    //   .downcast_ref::<LightningCssParseResult>()
    //   .unwrap();

    // let mut m = Module::new("xxx".into());

    // m.meta = Box::new(ModuleMetaData::Custom(
    //   farmfe_core::module::meta_data::custom::CustomMetaDataMap::default(),
    // ));

    // m.meta
    //   .as_custom_mut()
    //   .insert("lightningcss".to_string(), parse_result);

    // let v = m.meta.get_custom_mut::<Box<dyn ParseResult>>("lightingcss");

    // parse_result

    // println!("v: {:#?}", v);
    // parse_result
    // parse_result

    drop(content);

    // println!("result: {:#?}", parse_result.ast());
  }

  #[test]
  fn t() {
    // let adapter = CssPluginAdapter::LightningCss(LightningCss {});
    // adapter
    //   .parse(Arc::new(".foo { color: red; }".to_string()))
    //   .unwrap();
  }

  #[test]
  fn test_lightning_css_modules() {
    let adapter = LightningCss {};
    let content = Arc::new(
      r#"
.foo {
  color: red;
  composes: action;
}

.action {
  background: blue;
}
"#
      .to_string(),
    );
    // let parse_result = adapter.parse(content.clone()).unwrap();

    // drop(content);

    // let css_modules = adapter.css_modules(parse_result.ast()).unwrap();
    // assert!(css_modules.is_some());
    // let css_modules = css_modules.unwrap();
    // assert!(!css_modules.is_empty());

    // // assert!(&css_modules[0], );
    // assert_eq!(css_modules.len(), 2);
    // assert!(css_modules.get("foo").is_some());
    // css_modules.get("foo").map(|export| {
    //   assert_eq!(export.reference.len(), 1);
    //   assert!(matches!(
    //     &export.reference[0],
    //     CssModuleReference::Local { .. }
    //   ));
    // });
  }
}
