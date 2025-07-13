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
  swc_common::{Globals, SourceMap},
  Cacheable, DashMap, HashMap,
};
use farmfe_toolkit::{
  css::{parse_css_stylesheet, ParseCssModuleResult},
  swc_css_modules::compile,
  swc_css_visit::Visit,
};
use farmfe_utils::hash::sha256;
use lightningcss::{printer::PrinterOptions, stylesheet::ToCssResult, visitor::Visit as _};

use crate::{
  adapter::{
    lightningcss_adapter::{self, LightningCssParseResult},
    swc_adapter::{self, CssModuleRename},
  },
  dep_analyzer::DepAnalyzer,
};

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

  // fn comments(&self) -> &CommentsMetaData {
  //   match self {
  //     CssPluginParseResult::SwcCss(s) => &s.1,
  //     CssPluginParseResult::LightningCss(ast) => ast.comments(),
  //   }
  // }

  // fn source_map(&self) -> Option<&String> {
  //   match self {
  //     CssPluginParseResult::SwcCss(_) => None,
  //     CssPluginParseResult::LightningCss(ast) => ast.source_map(),
  //   }
  // }
}

pub struct CssPluginProcesser {
  pub adapter: CssPluginAdapter,
  pub ast_map: DashMap<String, CssPluginParseResult>,
}

#[derive(Clone, Default)]
pub struct ParseOption {
  pub module_id: String,
  pub data: HashMap<String, String>,
  pub context: Arc<CompilationContext>,
  pub enable_css_modules: bool,
}

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
        let content =
          adapter.parse(content, options.context.clone(), options.enable_css_modules)?;

        self.ast_map.insert(
          options.module_id.to_string(),
          CssPluginParseResult::LightningCss(content),
        );
      }
    }

    Ok(())
  }

  pub fn css_modules(&self, context: CssModulesContext) -> Result<Option<CssModuleExports>> {
    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        let module_string_id = context.module_id.to_string();
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
            context: context.context.clone(),
            enable_css_modules: true,
          },
        )?;
        context
          .content_map
          .insert(context.module_id.to_string(), context.content.clone());
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
    CreateModuleDataMetadataContext {
      module_id,
      content,
      context,
    }: CreateModuleDataMetadataContext,
  ) -> Result<ModuleMetaData> {
    let module_id_str = module_id.to_string();
    let ast = if let Some((_, v)) = self.ast_map.remove(&module_id_str) {
      v
    } else {
      self.parse(
        content.clone(),
        ParseOption {
          module_id: module_id_str.clone(),
          enable_css_modules: false,
          ..Default::default()
        },
      )?;

      self.ast_map.remove(&module_id_str).unwrap().1
    };

    match &self.adapter {
      CssPluginAdapter::SwcCss => {
        let (css_stylesheet, comments, source_map) = ast.to_swc_css().unwrap();

        context.meta.set_module_source_map(module_id, source_map);
        context.meta.set_globals(module_id, Default::default());

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

pub struct CreateModuleDataMetadataContext<'a> {
  pub module_id: &'a ModuleId,
  pub content: &'a Arc<String>,
  pub context: &'a Arc<CompilationContext>,
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

pub trait ParseResult<A, M>: std::any::Any + Downcast + Cacheable {
  fn ast(&self) -> &A;
  fn ast_mut(&mut self) -> &mut A;
  // fn take_ast(self) -> A;

  // fn comments(&self) -> &M;
  // fn comments_mut(&mut self) -> &mut M;
  // fn take_comments(self) -> M;

  // fn source_map(&self) -> Option<&String>;
  // fn source_map_mut(&mut self) -> Option<&mut String>;
  // fn take_source_map(self) -> Option<String>;
}

impl_downcast!(ParseResult<A, M>);

#[derive(Debug)]
pub enum CssModuleReference {
  Local { name: String },
  Global { name: String },
  Dependency { name: String, specifier: String },
}

// #[derive(Debug)]
// pub struct CssModuleExport {
//   pub name: String,
//   pub reference: Vec<CssModuleReference>,
// }

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

pub struct LightningCss {}

impl LightningCss {
  fn parse(
    &self,
    content: Arc<String>,
    context: Arc<CompilationContext>,
    enable_css_modules: bool,
  ) -> Result<LightningCssParseResult<'static>> {
    let options = lightningcss::stylesheet::ParserOptions::<'static, 'static> {
      css_modules: if enable_css_modules && let Some(_) = context.config.css.modules.as_ref() {
        Some(Default::default())
      } else {
        None
      },
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

impl<'a: 'static> ParseResult<lightningcss::stylesheet::StyleSheet<'a, 'a>, CommentsMetaData>
  for LightningCssParseResult<'a>
{
  fn ast(&self) -> &lightningcss::stylesheet::StyleSheet<'a, 'a> {
    self.ast.as_ref().expect("AST is not set")
  }

  fn ast_mut(&mut self) -> &mut lightningcss::stylesheet::StyleSheet<'a, 'a> {
    self.ast.as_mut().expect("AST is not set")
  }

  // fn take_ast(mut self) -> lightningcss::stylesheet::StyleSheet<'a, 'a> {
  //   self.ast.take().expect("AST is not set")
  // }

  // fn comments(&self) -> &CommentsMetaData {
  //   &self.comments
  // }

  // fn comments_mut(&mut self) -> &mut CommentsMetaData {
  //   &mut self.comments
  // }

  // fn take_comments(self) -> CommentsMetaData {
  //   self.comments
  // }

  // fn source_map(&self) -> Option<&String> {
  //   self.source_map.as_ref()
  // }

  // fn source_map_mut(&mut self) -> Option<&mut String> {
  //   self.source_map.as_mut()
  // }

  // fn take_source_map(self) -> Option<String> {
  //   self.source_map
  // }
}