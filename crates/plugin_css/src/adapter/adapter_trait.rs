use std::{borrow::Cow, sync::Arc};

use downcast_rs::{impl_downcast, Downcast};
use farmfe_core::{
  config::CssConfig,
  context::CompilationContext,
  error::Result,
  lazy_static::lazy_static,
  module::{
    meta_data::{css::CssModuleMetaData, script::CommentsMetaData},
    module_graph::ModuleGraph,
    CustomMetaDataMap, Module, ModuleId, ModuleMetaData,
  },
  plugin::PluginAnalyzeDepsHookResultEntry,
  resource::{meta_data::ResourcePotMetaData, resource_pot::ResourcePot, Resource},
  swc_common::{comments::SingleThreadedComments, Globals, SourceMap},
  Cacheable, DashMap, HashMap,
};
use farmfe_toolkit::{
  css::{parse_css_stylesheet, ParseCssModuleResult},
  itertools::Itertools,
  script::swc_try_with::try_with,
  swc_css_modules::compile,
  swc_css_visit::{Visit, VisitMutWith},
};
use farmfe_utils::hash::sha256;
use lightningcss::{
  printer::PrinterOptions,
  stylesheet::{StyleSheet, ToCssResult},
  visitor::Visit as _,
};

use crate::{
  adapter::{
    lightningcss_adapter::{self, LightningCssParseResult},
    swc_adapter::{self, CssModuleRename},
  },
  dep_analyzer::DepAnalyzer,
};

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

pub enum CssPluginParseResult {
  SwcCss(
    (
      farmfe_core::swc_css_ast::Stylesheet,
      CommentsMetaData,
      Arc<SourceMap>,
    ),
  ),
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

  fn to_swc_css(
    self,
  ) -> Option<(
    farmfe_core::swc_css_ast::Stylesheet,
    CommentsMetaData,
    Arc<SourceMap>,
  )> {
    match self {
      CssPluginParseResult::SwcCss(ast) => Some(ast),
      CssPluginParseResult::LightningCss(_) => None,
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
      CssPluginParseResult::SwcCss(s) => &s.1,
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

pub struct CssPluginProcesser {
  pub adapter: CssPluginAdapter,
  pub ast_map: DashMap<String, CssPluginParseResult>,
}

#[derive(Debug, Clone, Default)]
pub struct ParseOption {
  pub module_id: String,
  pub data: HashMap<String, String>,
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

pub struct CssModulesContext<'a> {
  // pub meta: CompilationContext,
  pub module_id: &'a ModuleId,
  pub content: Arc<String>,
  pub context: &'a Arc<CompilationContext>,
  pub content_map: &'a DashMap<String, Arc<String>>,
}

impl CssPluginProcesser {
  pub fn parse<'a, O>(&self, content: Arc<String>, options: O) -> Result<()>
  where
    O: Into<Cow<'a, ParseOption>>,
  {
    let options: Cow<'a, ParseOption> = options.into();
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        let ParseCssModuleResult {
          ast,
          comments,
          source_map,
        } = swc_adapter::parse(content, &options)?;

        self.ast_map.insert(
          options.module_id.to_string(),
          CssPluginParseResult::SwcCss((ast, comments.into(), source_map)),
        );
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

  pub fn css_modules(&self, context: CssModulesContext) -> Result<Option<CssModuleExports>> {
    // let options: Cow<'a, ParseOption> = options.into();
    println!("css_modules execute: {}", match &self.adapter {
        CssPluginAdapter::SwcCss => "SwcCss",
        CssPluginAdapter::LightningCss(_) => "LightningCss",
    });
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        let module_string_id = context.module_id.to_string();
        println!("set source map for css modules: {}", module_string_id);
        let ParseCssModuleResult {
          ast: mut css_stylesheet,
          comments,
          source_map,
        } = parse_css_stylesheet(&module_string_id, context.content.clone())?;
        // set source map and globals for css modules ast
        context
          .context
          .meta
          .set_module_source_map(&context.module_id, source_map);
        context
          .context
          .meta
          .set_globals(&context.module_id, Globals::default());

        // we can not use css_modules_resolved_path here because of the compatibility of windows. eg: \\ vs \\\\

        let stylesheet = compile(
          &mut css_stylesheet,
          CssModuleRename {
            indent_name: context
              .context
              .config
              .css
              .modules
              .as_ref()
              .unwrap()
              .indent_name
              .clone(),
            hash: sha256(module_string_id.to_string().as_bytes(), 8),
          },
        );

        self.ast_map.insert(
          module_string_id.clone(),
          CssPluginParseResult::SwcCss((
            css_stylesheet,
            CommentsMetaData::from(comments),
            Arc::new(SourceMap::default()),
          )),
        );

        context
          .content_map
          .insert(module_string_id, context.content.clone());

        return Ok(Some(stylesheet.renamed.into_iter().fold(
          HashMap::default(),
          |mut res, (name, export)| {
            // res.insert(name.to_string(), export);
            res.insert(
              name.to_string(),
              export
                .into_iter()
                .map(|v| match v {
                  farmfe_toolkit::swc_css_modules::CssClassName::Local { name } => {
                    CssModuleReference::Local {
                      name: name.value.to_string(),
                    }
                  }
                  farmfe_toolkit::swc_css_modules::CssClassName::Global { name } => {
                    CssModuleReference::Global {
                      name: name.value.to_string(),
                    }
                  }
                  farmfe_toolkit::swc_css_modules::CssClassName::Import { name, from } => {
                    CssModuleReference::Dependency {
                      name: name.value.to_string(),
                      specifier: from.to_string(),
                    }
                  }
                })
                .collect(),
            );
            res
          },
        )));
      }

      CssPluginAdapter::LightningCss(_) => {
        self.parse(
          context.content.clone(),
          ParseOption {
            module_id: context.module_id.to_string(),
            data: Default::default(),
          },
        )?;
        context.content_map.insert(context.module_id.to_string(), context.content.clone());
        let ast = self.ast_map.get(&context.module_id.to_string()).unwrap();
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
                export
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
                  .chain([CssModuleReference::Local { name: export.name }].into_iter())
                  .collect(),
              );

              acc
            })
        }));
      }
    }
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
      CssPluginAdapter::SwcCss => {
        let (css_stylesheet, comments, ..) = ast.to_swc_css().unwrap();

        let meta = ModuleMetaData::Css(Box::new(CssModuleMetaData {
          ast: css_stylesheet,
          comments,
          custom: Default::default(),
        }));

        Ok(meta)
      }
      CssPluginAdapter::LightningCss(_) => {
        let map = CustomMetaDataMap::default();

        map.insert(
          "lightning_css".to_string(),
          Box::new(ast.ta_lightning_css().unwrap()),
        );

        Ok(ModuleMetaData::Custom(map))
      }
    }
  }

  pub fn prefixer(
    &self,
    metadata: &mut ModuleMetaData,
    css_config: &Box<CssConfig>,
  ) -> Result<Option<()>> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        if let ModuleMetaData::Css(meta) = metadata {
          swc_adapter::prefixer(&mut meta.ast, css_config.prefixer.as_ref().unwrap());
          return Ok(Some(()));
        }

        Ok(None)
      }
      CssPluginAdapter::LightningCss(_) => {
        // do noting
        Ok(None)
      }
    }
  }

  ///
  /// analyze css dependencies
  ///
  pub fn analyze_deps(
    &self,
    metadata: &ModuleMetaData,
    context: &Arc<CompilationContext>,
  ) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    let mut deps_analyzer = DepAnalyzer::new(context.config.resolve.alias.clone());
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        if let ModuleMetaData::Css(css) = metadata {
          deps_analyzer.visit_stylesheet(&css.ast);
        }
      }

      CssPluginAdapter::LightningCss(_) => {
        if let ModuleMetaData::Custom(meta) = metadata {
          if let Some(mut lightning_css) =
            meta.get_mut::<LightningCssParseResult<'static>>("lightning_css")
          {
            lightning_css.ast_mut().visit(&mut deps_analyzer).unwrap();
          }
        }
      }
    }

    deps_analyzer.deps
  }

  ///
  /// after build:
  /// 1. if reference another css file, replace the url with the real path
  /// 2. if combine reference file content, remove the @import
  ///
  pub fn source_replace<'a>(&self, contenxt: SourceReplaceContext<'a>) -> Result<()> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        swc_adapter::module_source_replace(contenxt)?;
      }
      CssPluginAdapter::LightningCss(_) => {
        unreachable!("lightningcss source replace not implemented")
      }
    }
    Ok(())
  }

  pub fn css_to_script(&self, context: CssToScriptContext<'_>) -> Result<CssToScriptResult> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => swc_adapter::css_to_script(context),
      CssPluginAdapter::LightningCss(_) => lightningcss_adapter::css_to_script(context),
    }
  }

  pub fn merge_sourcemap(&self) -> Result<()> {
    Ok(())
  }

  pub fn create_resource_pot_metadata(
    &self,
    context: CreateResourcePotMetadataContext,
  ) -> Result<ResourcePotMetaData> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => swc_adapter::create_resource_pot_metadata(context),
      CssPluginAdapter::LightningCss(_) => {
        lightningcss_adapter::create_resource_pot_metadata(context)
      }
    }
  }

  pub fn codegen(&self, context: CodegenContext) -> Result<(String, Option<String>)> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => swc_adapter::codegen(context),
      CssPluginAdapter::LightningCss(_) => lightningcss_adapter::codegen_for_resource_pot(context),
    }
  }
}

pub struct CssToScriptContext<'a> {
  pub module_id: &'a ModuleId,
  pub context: &'a Arc<CompilationContext>,
}

pub struct CssToScriptResult {
  pub code: String,
  pub source_map: Option<String>,
}

pub struct CodegenContext<'a> {
  pub context: &'a Arc<CompilationContext>,
  pub resource_pot: &'a ResourcePot,
}

pub struct CreateResourcePotMetadataContext<'a> {
  pub resource_pot: &'a ResourcePot,
  pub context: &'a Arc<CompilationContext>,
  pub modules: Vec<&'a farmfe_core::module::Module>,
  pub module_execution_order: &'a HashMap<&'a ModuleId, usize>,
  pub module_graph: &'a ModuleGraph,
}

pub struct SourceReplaceContext<'a> {
  pub module: &'a Module,
  pub module_graph: &'a ModuleGraph,
  pub resources_map: &'a HashMap<String, Resource>,
  pub context: &'a Arc<CompilationContext>,
  pub without_context: bool,
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
pub type CssModuleExports = HashMap<String, Vec<CssModuleReference>>;

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
    let stylesheet = lightningcss::stylesheet::StyleSheet::parse(&v, options);

    Ok(LightningCssParseResult {
      ast: stylesheet.ok(),
      comments: CommentsMetaData::default(),
      source_map: None,
      source: vec![v.clone()],
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
//     panic!("impl prefixer todo")
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
  use crate::adapter::adapter_trait::CssPluginProcesser;
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

  const CODE: &'static str = r#"
.foo {
  color: red;
  composes: action;
}

.action {
  background: blue;
}
"#;

  // fn t1() {
  // let processer = CssPluginProcesser {
  //   adapter: CssPluginAdapter::LightningCss(LightningCss {}),
  //   ast_map: Default::default(),
  // };
  // }

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
