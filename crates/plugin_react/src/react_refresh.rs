use std::sync::Arc;

use farmfe_core::{
  swc_common::SourceMap,
  swc_ecma_ast::{Module as SwcModule, ModuleDecl, ModuleItem},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::script::parse_module;

const REFRESH_RUNTIME_IMPORT: &str = "import RefreshRuntime from 'react-refresh'";

const PRE_CODE: &str = r#"
var prevRefreshReg;
var prevRefreshSig;

prevRefreshReg = window.$RefreshReg$;
prevRefreshSig = window.$RefreshSig$;
window.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, module.id + id);
};
window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;
"#;

const POST_CODE: &str = r#"
window.$RefreshReg$ = prevRefreshReg;
window.$RefreshSig$ = prevRefreshSig;
module.meta.hot.accept();
RefreshRuntime.performReactRefresh();
"#;

fn inject_runtime_import(ast: &mut SwcModule) {
  let import_decl = {
    let mut module = parse_module(
      "refreshRuntimeImport",
      REFRESH_RUNTIME_IMPORT,
      Syntax::Es(Default::default()),
      Default::default(),
      Arc::new(SourceMap::default()),
    )
    .unwrap();

    assert_eq!(module.body.len(), 1);
    match module.body.remove(0) {
      ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => import_decl,
      _ => unreachable!(),
    }
  };

  ast
    .body
    .insert(0, ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
}

fn inject_pre_code(ast: &mut SwcModule) {
  let module = parse_module(
    "preCode",
    PRE_CODE,
    Syntax::Es(Default::default()),
    Default::default(),
    Arc::new(SourceMap::default()),
  )
  .unwrap();

  // insert pre code after last import
  ast.body.splice(1..1, module.body);
}

fn inject_post_code(ast: &mut SwcModule) {
  let module = parse_module(
    "postCode",
    POST_CODE,
    Syntax::Es(Default::default()),
    Default::default(),
    Arc::new(SourceMap::default()),
  )
  .unwrap();

  // insert post code at the end
  ast.body.extend(module.body);
}

pub fn inject_react_refresh(ast: &mut SwcModule) {
  inject_runtime_import(ast);
  inject_pre_code(ast);
  inject_post_code(ast);
}
