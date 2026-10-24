use std::{
  collections::HashSet,
  sync::{Arc, LazyLock},
};

use farmfe_core::{
  config::script::ScriptConfig,
  context::create_swc_source_map,
  error::{CompilationError, Result as HookResult},
  module::ModuleType,
  plugin::{PluginTransformHookParam, PluginTransformHookResult},
  serde_json,
  swc_common::comments::SingleThreadedComments,
  swc_ecma_ast::{ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem},
  swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
};
use regex::Regex;

use crate::options::Options;

pub fn transform(
  options: &Options,
  param: &PluginTransformHookParam,
  config: &ScriptConfig,
) -> HookResult<Option<PluginTransformHookResult>> {
  Ok(
    rewrite_imports(
      options,
      &param.content,
      &param.module_type,
      &param.module_id,
      config,
    )?
    .map(|content| PluginTransformHookResult {
      content,
      module_type: Some(param.module_type.clone()),
      source_map: None,
      ignore_previous_source_map: true,
    }),
  )
}

fn rewrite_imports(
  options: &Options,
  content: &str,
  module_type: &ModuleType,
  id: &str,
  config: &ScriptConfig,
) -> HookResult<Option<String>> {
  let syntax = match module_type {
    ModuleType::Js | ModuleType::Jsx => Syntax::Es(EsSyntax {
      jsx: matches!(module_type, ModuleType::Jsx),
      import_attributes: true,
      ..config.parser.es_config
    }),
    ModuleType::Ts | ModuleType::Tsx => Syntax::Typescript(TsSyntax {
      tsx: matches!(module_type, ModuleType::Tsx),
      ..config.parser.ts_config
    }),
    _ => return Ok(None),
  };
  let name = options
    .library_name
    .as_deref()
    .filter(|name| !name.is_empty())
    .ok_or_else(|| CompilationError::GenericError("modular-import requires libraryName".into()))?;
  if !content.contains(name) && !content.contains('\\') {
    return Ok(None);
  }
  let (_, file) = create_swc_source_map(&id.into(), Arc::new(content.to_owned()));
  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    syntax,
    config.target,
    StringInput::from(&*file),
    Some(&comments),
  );
  let mut parser = Parser::new_from(lexer);
  let module = parser.parse_module().map_err(|error| {
    CompilationError::GenericError(format!("modular-import could not parse {id}: {error:?}"))
  })?;
  let prefix = content.len().saturating_sub(file.src.len());
  let relative = |position: u32| (position - file.start_pos.0) as usize + prefix;
  let mut edits = Vec::new();
  let mut styles = HashSet::new();
  for item in module.body {
    let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item else {
      continue;
    };
    if import.type_only || import.with.is_some() || import.src.value.to_string_lossy() != name {
      continue;
    }
    let mut generated = Vec::new();
    let mut retained = Vec::new();
    let mut retained_named = Vec::new();
    for specifier in &import.specifiers {
      match specifier {
        ImportSpecifier::Default(specifier) => retained.push(specifier.local.sym.to_string()),
        ImportSpecifier::Namespace(specifier) => {
          retained.push(format!("* as {}", specifier.local.sym))
        }
        ImportSpecifier::Named(specifier) => {
          let exported = match &specifier.imported {
            Some(ModuleExportName::Ident(name)) => name.sym.to_string(),
            Some(ModuleExportName::Str(name)) => name.value.to_string_lossy().into_owned(),
            None => specifier.local.sym.to_string(),
          };
          if specifier.is_type_only || exported == "default" {
            let original_name = match &specifier.imported {
              Some(ModuleExportName::Str(_)) => quoted(&exported),
              _ => exported.clone(),
            };
            let alias = if specifier.imported.is_some() {
              format!("{original_name} as {}", specifier.local.sym)
            } else {
              original_name
            };
            retained_named.push(format!(
              "{}{alias}",
              if specifier.is_type_only { "type " } else { "" }
            ));
            continue;
          }
          let component = component_path(&exported, options.camel2_dash.unwrap_or(true));
          let request = format!(
            "{}/{}/{component}",
            name,
            options.lib_dir.as_deref().unwrap_or_default()
          );
          generated.push(format!(
            "import {} from {};",
            specifier.local.sym,
            quoted(&request)
          ));
          let style = build_style_path(
            name,
            &options.style_lib_dir,
            &options.style_library_name,
            &component,
            &options.style_library_path,
          );
          if styles.insert(style.clone()) {
            generated.push(format!("import {};", quoted(&style)));
          }
        }
      }
    }
    if generated.is_empty() {
      continue;
    }
    if !retained_named.is_empty() {
      retained.push(format!("{{ {} }}", retained_named.join(", ")));
    }
    if !retained.is_empty() {
      generated.insert(
        0,
        format!("import {} from {};", retained.join(", "), quoted(name)),
      );
    }
    let start = relative(import.span.lo.0);
    let end = relative(import.span.hi.0);
    if !content
      .get(start..end)
      .is_some_and(|text| text.starts_with("import"))
    {
      return Err(CompilationError::GenericError(format!(
        "modular-import invalid source span in {id}"
      )));
    }
    // Keep comments within replaced declarations. Other source bytes, including
    // JSX and comments outside these spans, are left unchanged.
    let (leading, trailing) = comments.borrow_all();
    let mut contained: Vec<_> = leading
      .values()
      .chain(trailing.values())
      .flatten()
      .filter(|comment| comment.span.lo >= import.span.lo && comment.span.hi <= import.span.hi)
      .collect();
    contained.sort_by_key(|comment| comment.span.lo.0);
    contained.dedup_by_key(|comment| (comment.span.lo.0, comment.span.hi.0));
    let mut replacement = String::new();
    for comment in contained {
      if let Some(text) = content.get(relative(comment.span.lo.0)..relative(comment.span.hi.0)) {
        replacement.push_str(text);
        replacement.push('\n');
      }
    }
    replacement.push_str(&generated.join("\n"));
    edits.push((start, end, replacement));
  }
  if edits.is_empty() {
    return Ok(None);
  }
  let mut result = content.to_owned();
  for (start, end, replacement) in edits.into_iter().rev() {
    result.replace_range(start..end, &replacement);
  }
  Ok(Some(result))
}

fn quoted(value: &str) -> String {
  serde_json::to_string(value).expect("a string always serializes")
}

fn component_path(name: &str, camel2_dash: bool) -> String {
  if !camel2_dash {
    return name.to_owned();
  }
  static ACRONYM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([A-Z]+)([A-Z][a-z])").expect("constant regex"));
  static WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").expect("constant regex"));
  WORD
    .replace_all(&ACRONYM.replace_all(name, "$1-$2"), "$1-$2")
    .to_lowercase()
}

fn build_style_path(
  name: &str,
  directory: &Option<String>,
  library: &Option<String>,
  component: &str,
  suffix: &Option<String>,
) -> String {
  let base = if let Some(library) = library {
    format!(
      "{}/{}/{library}/{component}",
      name,
      directory.as_deref().unwrap_or_default()
    )
  } else {
    format!(
      "{}/{}/{component}",
      name,
      directory.as_deref().unwrap_or_default()
    )
  };
  let suffix = suffix.as_deref().unwrap_or_default();
  format!(
    "{base}{}{suffix}",
    if suffix.starts_with('.') { "" } else { "/" }
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{serde_json::json, swc_ecma_ast::EsVersion};

  fn rewrite(source: &str, extra: serde_json::Value) -> HookResult<Option<String>> {
    let mut options = json!({"libraryName":"ui","libDir":"es","styleLibDir":"es","styleLibraryPath":"style/index.css","camel2Dash":false});
    options
      .as_object_mut()
      .unwrap()
      .extend(extra.as_object().unwrap().clone());
    let options = serde_json::from_value(options).unwrap();
    rewrite_imports(
      &options,
      source,
      &ModuleType::Tsx,
      "fixture.tsx",
      &ScriptConfig {
        target: EsVersion::EsNext,
        ..Default::default()
      },
    )
  }

  #[test]
  fn aliased_multiple_components_keep_export_case_and_styles() {
    let output = rewrite("import { Button as BulkButton, Tooltip as Hint } from 'ui';\nexport const v = <BulkButton/>;", json!({})).unwrap().unwrap();
    assert!(output.contains("import BulkButton from \"ui/es/Button\";"));
    assert!(output.contains("import Hint from \"ui/es/Tooltip\";"));
    assert!(output.contains("import \"ui/es/Button/style/index.css\";"));
    assert!(output.contains("import \"ui/es/Tooltip/style/index.css\";"));
    assert!(output.ends_with("export const v = <BulkButton/>;"));
  }

  #[test]
  fn retained_default_types_and_namespace_are_not_rewritten() {
    let output = rewrite("import UI, { Button, type Props } from 'ui';", json!({}))
      .unwrap()
      .unwrap();
    assert!(output.contains("import UI, { type Props } from \"ui\";"));
    assert!(
      rewrite("import type { Props } from 'ui';", json!({}))
        .unwrap()
        .is_none()
    );
    assert!(
      rewrite("import UI, * as all from 'ui';", json!({}))
        .unwrap()
        .is_none()
    );
    assert!(rewrite(&output, json!({})).unwrap().is_none());
  }

  #[test]
  fn comments_and_string_literals_do_not_inject_imports() {
    assert!(
      rewrite(
        "// import { Button } from 'ui';\nconst text = \"import { Button } from 'ui';\";",
        json!({})
      )
      .unwrap()
      .is_none()
    );
  }

  #[test]
  fn utf8_bom_comments_and_non_import_text_survive() {
    let source = "\u{feff}const title = '商品🌾';\nimport { /* attribution */ Button as B } from 'ui';\nconst after = '尾';";
    let output = rewrite(source, json!({})).unwrap().unwrap();
    assert!(output.starts_with("\u{feff}const title = '商品🌾';\n"));
    assert!(output.contains("/* attribution */"));
    assert!(output.ends_with("\nconst after = '尾';"));
  }

  #[test]
  fn dash_case_handles_acronyms_and_style_paths() {
    let output = rewrite(
      "import { DatePicker, URLInput } from 'ui';",
      json!({"camel2Dash":true}),
    )
    .unwrap()
    .unwrap();
    assert!(output.contains("ui/es/date-picker"));
    assert!(output.contains("ui/es/url-input"));
    let custom = rewrite(
      "import { Button } from 'ui';",
      json!({"styleLibraryName":"theme","styleLibraryPath":".css"}),
    )
    .unwrap()
    .unwrap();
    assert!(custom.contains("ui/es/theme/Button.css"));
  }

  #[test]
  fn aliases_share_one_style_and_escaped_requests_work() {
    let output = rewrite(
      r"import { Button as A, Button as B } from '\x75i';",
      json!({}),
    )
    .unwrap()
    .unwrap();
    assert_eq!(output.matches("ui/es/Button/style/index.css").count(), 1);
    assert!(output.contains("import A from"));
    assert!(output.contains("import B from"));
  }

  #[test]
  fn malformed_target_import_returns_an_error() {
    assert!(rewrite("import { Button from 'ui';", json!({})).is_err());
  }
}
