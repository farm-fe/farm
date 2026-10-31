use super::*;
use farmfe_core::{
  context::CompilationContext,
  module::{ModuleId, ModuleMetaData, ModuleType, meta_data::script::ScriptModuleMetaData},
  plugin::{PluginProcessModuleHookParam, PluginTransformHookParam},
  swc_common::{GLOBALS, Globals, Mark},
  swc_ecma_ast::{EsVersion, Module},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  script::{codegen_module, parse_module},
  swc_ecma_transforms::resolver,
  swc_ecma_visit::VisitMutWith,
};
use std::sync::Arc;

fn browser_config() -> Config {
  let mut config = Config::default();
  config.define.insert(
    DEFAULT_DEFINE_PROCESS_ENV.into(),
    serde_json::json!({"NODE_ENV":"production","DEV":false}),
  );
  config.define.insert(
    format!("{REGEX_PREFIX}(global(This)?\\.)?process\\.env\\.NODE_ENV"),
    serde_json::json!("\"production\""),
  );
  config.define.insert(
    format!("{REGEX_PREFIX}(global(This)?\\.)?process\\.env\\.DEV"),
    serde_json::json!("false"),
  );
  config
}

fn transform(source: &str, config: Config) -> String {
  let mut config = config;
  let plugin = FarmPluginDefine::new(&config);
  plugin.config(&mut config).unwrap();
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  plugin
    .transform(
      &PluginTransformHookParam {
        module_id: "index.jsx".into(),
        resolved_path: "index.jsx",
        content: source.into(),
        module_type: ModuleType::Jsx,
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
      },
      &context,
    )
    .unwrap()
    .map_or_else(|| source.into(), |result| result.content)
}

fn process(source: &str, config: Config) -> Module {
  let mut config = config;
  let plugin = FarmPluginDefine::new(&config);
  plugin.config(&mut config).unwrap();
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  let id: ModuleId = "index.js".into();
  let parsed = parse_module(
    &id,
    Arc::new(source.into()),
    Syntax::Es(farmfe_core::swc_ecma_parser::EsSyntax {
      jsx: true,
      ..Default::default()
    }),
    EsVersion::EsNext,
  )
  .unwrap();
  let globals = Globals::new();
  let (mut ast, unresolved_mark) = GLOBALS.set(&globals, || {
    let unresolved_mark = Mark::new();
    let mut ast = parsed.ast;
    ast.visit_mut_with(&mut resolver(unresolved_mark, Mark::new(), false));
    (ast, unresolved_mark.as_u32())
  });
  context.meta.set_globals(&id, globals);
  let mut meta = ModuleMetaData::Script(Box::new(ScriptModuleMetaData {
    ast: std::mem::take(&mut ast),
    unresolved_mark,
    ..Default::default()
  }));
  plugin
    .process_module(
      &mut PluginProcessModuleHookParam {
        module_id: &id,
        module_type: &ModuleType::Js,
        meta: &mut meta,
        content: &mut Arc::new(source.into()),
        source_map_chain: &mut vec![],
      },
      &context,
    )
    .unwrap();
  meta.as_script().ast.clone()
}

fn print(ast: &Module) -> String {
  String::from_utf8(
    codegen_module(
      ast,
      Arc::new(Default::default()),
      None,
      Default::default(),
      None,
    )
    .unwrap(),
  )
  .unwrap()
}

fn exported<'a>(ast: &'a Module, name: &str) -> &'a farmfe_core::swc_ecma_ast::Expr {
  use farmfe_core::swc_ecma_ast::*;
  for item in &ast.body {
    if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
      decl: Decl::Var(declaration),
      ..
    })) = item
    {
      for value in &declaration.decls {
        if matches!(&value.name, Pat::Ident(ident) if ident.sym == name) {
          let mut expression = value.init.as_deref().unwrap();
          while let Expr::Paren(paren) = expression {
            expression = &paren.expr;
          }
          return expression;
        }
      }
    }
  }
  panic!("Missing export {name}");
}

#[test]
fn automatic_environment_defines_do_not_rewrite_source_text() {
  let source = "// process.env.NODE_ENV\nconst text = \"process.env.NODE_ENV\"; const view = <code>process.env.NODE_ENV</code>; const template = `process.env.DEV`;";
  assert_eq!(transform(source, browser_config()), source);
}

#[test]
fn automatic_environment_defines_replace_only_unresolved_expression_reads() {
  use farmfe_core::swc_ecma_ast::{Expr, Lit};
  let ast = process(
    "export const actual = process.env.NODE_ENV; export const dev = process.env.DEV; export const literal = 'process.env.NODE_ENV'; function local(process) { return process.env.NODE_ENV; }",
    browser_config(),
  );
  assert!(
    matches!(exported(&ast, "actual"), Expr::Lit(Lit::Str(value)) if value.value == "production")
  );
  assert!(matches!(exported(&ast, "dev"), Expr::Lit(Lit::Bool(value)) if !value.value));
  assert!(
    matches!(exported(&ast, "literal"), Expr::Lit(Lit::Str(value)) if value.value == "process.env.NODE_ENV")
  );
  let output = print(&ast);
  assert!(output.contains("return process.env.NODE_ENV"), "{output}");
}

#[test]
fn explicit_text_and_regex_defines_keep_existing_substitution_contract() {
  let mut config = Config::default();
  config
    .define
    .insert("AUTHOR_TOKEN".into(), serde_json::json!("replaced"));
  config.define.insert(
    format!("{REGEX_PREFIX}prefix-(\\w+)"),
    serde_json::json!("value-$1"),
  );
  assert_eq!(
    transform("const text = 'AUTHOR_TOKEN prefix-demo';", config),
    "const text = 'replaced value-demo';"
  );
}

#[test]
fn template_expressions_change_but_template_text_does_not() {
  use farmfe_core::swc_ecma_ast::{Expr, Lit};
  let ast = process(
    "export const text = `process.env.NODE_ENV ${process.env.NODE_ENV} process.env.DEV`;",
    browser_config(),
  );
  let Expr::Tpl(template) = exported(&ast, "text") else {
    panic!("Not a template")
  };
  assert_eq!(template.quasis[0].raw, "process.env.NODE_ENV ");
  assert_eq!(template.quasis[1].raw, " process.env.DEV");
  let mut expression = template.exprs[0].as_ref();
  while let Expr::Paren(paren) = expression {
    expression = &paren.expr;
  }
  assert!(matches!(expression, Expr::Lit(Lit::Str(value)) if value.value == "production"));
}

#[test]
fn static_computed_global_environment_reads_and_prefix_boundaries() {
  use farmfe_core::swc_ecma_ast::{Expr, Lit};
  let ast = process(
    "export const a = process['env']['NODE_ENV']; export const b = globalThis.process.env.NODE_ENV; export const c = global.process.env.DEV; export const other = process.env.NODE_ENV_SUFFIX;",
    browser_config(),
  );
  for name in ["a", "b"] {
    assert!(
      matches!(exported(&ast, name), Expr::Lit(Lit::Str(value)) if value.value == "production")
    );
  }
  assert!(matches!(exported(&ast, "c"), Expr::Lit(Lit::Bool(value)) if !value.value));
  assert!(matches!(exported(&ast, "other"), Expr::Member(_)));
}

#[test]
fn local_global_and_global_this_bindings_are_preserved() {
  let output = print(&process(
    "function local(globalThis, global) { return [globalThis.process.env.NODE_ENV, global.process.env.DEV]; }",
    browser_config(),
  ));
  assert!(output.contains("globalThis.process.env.NODE_ENV"));
  assert!(output.contains("global.process.env.DEV"));
}

#[test]
fn writes_updates_and_delete_remain_references() {
  let output = print(&process(
    "process.env.NODE_ENV = 'test'; process.env.DEV++; delete process.env.NODE_ENV;",
    browser_config(),
  ));
  assert!(output.contains("process.env.NODE_ENV = 'test'"), "{output}");
  assert!(output.contains("process.env.DEV++"), "{output}");
  assert!(output.contains("delete process.env.NODE_ENV"), "{output}");
}

#[test]
fn parenthesized_and_destructuring_write_targets_remain_references() {
  let output = print(&process(
    "++(process.env.DEV); delete ((process.env.NODE_ENV)); (process.env.NODE_ENV) = 'test'; ({value: process.env.NODE_ENV} = source); [process.env.DEV] = source;",
    browser_config(),
  ));
  assert!(output.contains("++(process.env.DEV)"), "{output}");
  assert!(
    output.contains("delete ((process.env.NODE_ENV))"),
    "{output}"
  );
  assert!(
    output.contains("(process.env.NODE_ENV) = 'test'"),
    "{output}"
  );
  assert!(output.contains("value: process.env.NODE_ENV"), "{output}");
  assert!(output.contains("[process.env.DEV] = source"), "{output}");
}

#[test]
fn computed_reads_inside_parenthesized_write_targets_still_replace() {
  let output = print(&process(
    "delete (process.env[process.env.NODE_ENV]); ++(process.env[process.env.NODE_ENV]);",
    browser_config(),
  ));
  assert!(!output.contains("process.env.NODE_ENV"), "{output}");
  assert!(output.contains("process.env[(\"production\")]"), "{output}");
}

#[test]
fn jsx_static_text_attributes_comments_and_regex_remain_unchanged() {
  let source = "// process.env.NODE_ENV\nexport const view = <code title='process.env.NODE_ENV'>process.env.NODE_ENV{process.env.NODE_ENV}</code>; export const regex = /process.env.NODE_ENV/;";
  assert_eq!(transform(source, browser_config()), source);
  let output = print(&process(source, browser_config()));
  assert!(output.contains("title='process.env.NODE_ENV'"), "{output}");
  assert!(output.contains(">process.env.NODE_ENV{"), "{output}");
  assert!(output.contains("production"), "{output}");
  assert!(output.contains("/process.env.NODE_ENV/"), "{output}");
}

#[test]
fn node_environment_without_automatic_pairs_is_not_replaced() {
  let mut config = browser_config();
  config
    .define
    .retain(|key, _| key == DEFAULT_DEFINE_PROCESS_ENV);
  let source = "export const actual = process.env.NODE_ENV;";
  assert_eq!(transform(source, config.clone()), source);
  assert!(print(&process(source, config)).contains("process.env.NODE_ENV"));
}

#[test]
fn automatic_json_values_retain_types_and_original_expression_span() {
  use farmfe_core::swc_common::Spanned;
  use farmfe_core::swc_ecma_ast::Expr;
  let mut config = Config::default();
  let values = serde_json::json!({"COUNT":42,"EMPTY":null,"LIST":[1,"two"],"OBJECT":{"enabled":true},"TEXT":"quote\"\n中文"});
  config
    .define
    .insert(DEFAULT_DEFINE_PROCESS_ENV.into(), values.clone());
  for (name, value) in values.as_object().unwrap() {
    config.define.insert(
      format!("{REGEX_PREFIX}(global(This)?\\.)?process\\.env\\.{name}"),
      serde_json::json!(value.to_string()),
    );
  }
  let source = "const unicode = '中文'; export const a = process.env.OBJECT; export const b = process.env.LIST; export const c = process.env.COUNT; export const d = process.env.EMPTY; export const e = process.env.TEXT;";
  let original = process(source, Config::default());
  let transformed = process(source, config);
  assert_eq!(
    exported(&original, "a").span(),
    exported(&transformed, "a").span()
  );
  assert!(matches!(exported(&transformed, "a"), Expr::Object(_)));
  assert!(matches!(exported(&transformed, "b"), Expr::Array(_)));
  assert!(
    matches!(exported(&transformed, "c"), Expr::Lit(farmfe_core::swc_ecma_ast::Lit::Num(value)) if value.value == 42.0)
  );
  assert!(matches!(
    exported(&transformed, "d"),
    Expr::Lit(farmfe_core::swc_ecma_ast::Lit::Null(_))
  ));
  assert!(
    matches!(exported(&transformed, "e"), Expr::Lit(farmfe_core::swc_ecma_ast::Lit::Str(value)) if value.value == "quote\"\n中文")
  );
}
