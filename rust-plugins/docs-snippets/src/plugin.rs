use crate::transform::{self, Options};
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  error::{CompilationError, Result},
  module::ModuleMetaData,
  plugin::{Plugin, PluginProcessModuleHookParam},
  swc_common::GLOBALS,
};
use farmfe_macro_plugin::farm_plugin;
use std::sync::Arc;

#[farm_plugin]
pub struct FarmPluginDocsSnippets {
  options: std::result::Result<Options, String>,
}

impl FarmPluginDocsSnippets {
  fn new(_config: &Config, options: String) -> Self {
    Self {
      options: serde_json::from_str::<Options>(&options)
        .map_err(|error| error.to_string())
        .and_then(|options| {
          options.validate()?;
          Ok(options)
        }),
    }
  }
  fn options(&self) -> Result<&Options> {
    self
      .options
      .as_ref()
      .map_err(|error| CompilationError::GenericError(format!("[DOCS_SNIPPET_OPTIONS] {error}")))
  }
}

impl Plugin for FarmPluginDocsSnippets {
  fn name(&self) -> &str {
    "FarmPluginDocsSnippets"
  }
  fn priority(&self) -> i32 {
    110
  }
  fn config(&self, _config: &mut Config) -> Result<Option<()>> {
    self.options()?;
    Ok(Some(()))
  }
  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let options = self.options()?;
    if !param.module_type.is_script() {
      return Ok(None);
    }
    let ModuleMetaData::Script(meta) = param.meta else {
      return Err(CompilationError::GenericError(format!(
        "[DOCS_SNIPPET_AST] {}: expected parsed script metadata",
        param.module_id.to_string()
      )));
    };
    let globals = context.meta.get_globals(param.module_id);
    let report = GLOBALS
      .set(globals.value(), || {
        transform::lower_diagnostic(&mut meta.ast, options)
      })
      .map_err(|error| {
        CompilationError::GenericError(crate::diagnostics::format(
          &error,
          param.module_id,
          param.content.clone(),
          param.source_map_chain,
        ))
      })?;
    for diagnostic in report.diagnostics {
      context
        .log_store
        .lock()
        .add_warning(crate::diagnostics::format(
          &diagnostic,
          param.module_id,
          param.content.clone(),
          param.source_map_chain,
        ));
    }
    Ok((report.transformed > 0).then_some(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{
    module::{ModuleId, ModuleMetaData, ModuleType, meta_data::script::ScriptModuleMetaData},
    swc_common::Globals,
    swc_ecma_ast::{EsVersion, ModuleDecl, ModuleItem},
    swc_ecma_parser::{EsSyntax, Syntax},
  };
  use farmfe_toolkit::script::{parse_module, swc_try_with::resolve_module_mark};

  #[test]
  fn snippets_run_before_script_and_react_lowering() {
    let plugin = FarmPluginDocsSnippets::new(&Config::default(), "{}".into());
    assert!(plugin.priority() > 99);
  }

  #[test]
  fn hook_lowers_resolved_jsx_without_changing_content_or_upstream_maps() {
    let id = ModuleId::from("fixture.jsx");
    let mut content = Arc::new("import {CodeBlock} from '@farmfe/docs/snippets';export const x=<CodeBlock>hello</CodeBlock>;".to_owned());
    let original_content = content.clone();
    let mut ast = parse_module(
      &id,
      content.clone(),
      Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
      }),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;
    let globals = Globals::new();
    resolve_module_mark(&mut ast, false, &globals);
    let context = Arc::new(CompilationContext::new(Config::default(), Vec::new()).unwrap());
    context.meta.set_globals(&id, globals);
    let mut meta = ModuleMetaData::Script(Box::new(ScriptModuleMetaData {
      ast,
      ..Default::default()
    }));
    let mut maps = vec![Arc::new("upstream-map-sentinel".to_owned())];
    let original_maps = maps.clone();
    let plugin = FarmPluginDocsSnippets::new(
      &context.config,
      r#"{"rendererModule":"custom/snippets"}"#.into(),
    );
    let result = plugin
      .process_module(
        &mut PluginProcessModuleHookParam {
          module_id: &id,
          module_type: &ModuleType::Jsx,
          content: &mut content,
          meta: &mut meta,
          source_map_chain: &mut maps,
        },
        &context,
      )
      .unwrap();
    assert_eq!(result, Some(()));
    assert_eq!(content, original_content);
    assert_eq!(maps, original_maps);
    assert!(meta.as_script().ast.body.iter().any(|item| matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(import)) if import.src.value == "custom/snippets")));
  }

  #[test]
  fn invalid_native_options_are_compilation_errors() {
    for options in ["{", r#"{"unknown":true}"#, r#"{"rendererModule":""}"#] {
      let mut config = Config::default();
      let plugin = FarmPluginDocsSnippets::new(&config, options.into());
      assert!(plugin.config(&mut config).is_err(), "{options}");
    }
  }

  #[test]
  fn invalid_snippet_reports_original_mdx_line_through_upstream_map() {
    let source = "---\ntitle: Error source\n---\n\nimport {CodeBlock} from '@farmfe/docs/snippets';\n\n# 中文\n\n<CodeBlock>{dynamic}</CodeBlock>\n";
    let compiled = farmfe_docs_content::compile_content_with_format(
      source,
      &mdxjs::Options {
        filepath: Some("docs/error.mdx".into()),
        jsx: true,
        ..Default::default()
      },
      true,
      None,
      farmfe_docs_content::ContentFormat::Auto,
    )
    .unwrap();
    let id = ModuleId::from("generated/error.jsx");
    let mut content = Arc::new(compiled.code);
    let mut ast = parse_module(
      &id,
      content.clone(),
      Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
      }),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;
    let globals = Globals::new();
    resolve_module_mark(&mut ast, false, &globals);
    let original_ast = ast.clone();
    let context = Arc::new(CompilationContext::new(Config::default(), Vec::new()).unwrap());
    context.meta.set_globals(&id, globals);
    let mut meta = ModuleMetaData::Script(Box::new(ScriptModuleMetaData {
      ast,
      ..Default::default()
    }));
    let mut maps = vec![Arc::new(compiled.source_map.unwrap())];
    let original_maps = maps.clone();
    let plugin = FarmPluginDocsSnippets::new(&context.config, "{}".into());
    let error = plugin
      .process_module(
        &mut PluginProcessModuleHookParam {
          module_id: &id,
          module_type: &ModuleType::Jsx,
          content: &mut content,
          meta: &mut meta,
          source_map_chain: &mut maps,
        },
        &context,
      )
      .unwrap_err()
      .to_string();
    assert!(error.contains("docs/error.mdx:9:1"), "{error}");
    assert!(error.contains("DOCS_SNIPPET_STATIC"), "{error}");
    assert_eq!(meta.as_script().ast, original_ast);
    assert_eq!(maps, original_maps);
  }
}
