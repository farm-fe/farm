use farmfe_core::{
  config::FARM_MODULE,
  swc_ecma_ast::{Module as SwcModule, ModuleDecl, ModuleItem},
  swc_ecma_parser::Syntax,
};

use farmfe_toolkit_plugin_types::{
  libloading::Library,
  swc_ast::{parse_module, ParseScriptModuleResult},
};
use lazy_static::lazy_static;

const REFRESH_RUNTIME_IMPORT: &str = "import RefreshRuntime from 'react-refresh'";
pub const IS_REACT_REFRESH_BOUNDARY: &str = "farmfe_plugin_react_is_react_refresh_boundary";

// TODO namespace window.$RefreshReg$ and window.$RefreshSig$
lazy_static! {
  pub static ref PRE_CODE: String = {
    format!(
      r#"
    var prevRefreshReg;
    var prevRefreshSig;

    prevRefreshReg = window.$RefreshReg$;
    prevRefreshSig = window.$RefreshSig$;
    window.$RefreshReg$ = (type, id) => {{
      RefreshRuntime.register(type, {FARM_MODULE}.id + " " + id);
    }};
    window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;
    "#
    )
  };
}
// polyfill like https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react/src/refreshUtils.js
//
//
//
// ```js
// Object.defineProperty(exports, "Foo", {
//  get() {
//    return Foo
//  }
// });
//
// Object.defineProperty(exports, "Bar", {
//  get() {
//    return Bar
//  }
// });
//
//
//
// // moduleA
// const Foo = () => {}
//
// // ...
// ReactRefreshUtil.registerExportsForReactRefresh(module.exports, module.id);
// // Until here, `Bar` is still not initialized
// // ...
//
// // moduleB
// const Bar = () => {}
// ```
//

// When merging multiple modules, the exports will be promoted and contain side effects
// so the getter cannot be triggered immediately when `registerExportsForReactRefresh`.
// maybe there are other better ways.
const POST_CODE: &str = r#"
window.$RefreshReg$ = prevRefreshReg;
window.$RefreshSig$ = prevRefreshSig;
if (import.meta.hot) {
  setTimeout(() => {
    ReactRefreshUtil.registerExportsForReactRefresh(module.exports, module.id);

    import.meta.hot.accept(nextExport => {
      const message = ReactRefreshUtil.validateRefreshBoundaryAndEnqueueUpdate(module.exports, nextExport);

      if (message) {
        import.meta.hot.invalidate(message);
      }
    });

    ReactRefreshUtil.enqueueUpdate();
  })
}
"#;

fn inject_runtime_import(lib: &Library, ast: &mut SwcModule) {
  let parse_import_decl = |file_name: &str, code: &str| {
    let ParseScriptModuleResult {
      ast: mut module, ..
    } = parse_module(
      lib,
      file_name,
      code,
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    assert_eq!(module.body.len(), 1);
    match module.body.remove(0) {
      ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => import_decl,
      _ => unreachable!(),
    }
  };

  let import_decl = parse_import_decl("refreshRuntimeImport", REFRESH_RUNTIME_IMPORT);

  // inject react boundary detection
  let react_boundary_import =
    format!("import * as ReactRefreshUtil from '{IS_REACT_REFRESH_BOUNDARY}'");
  let react_boundary_import_decl = parse_import_decl("ReactRefreshUtil", &react_boundary_import);

  ast.body.insert(
    0,
    ModuleItem::ModuleDecl(ModuleDecl::Import(react_boundary_import_decl)),
  );
  ast
    .body
    .insert(0, ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
}

fn inject_pre_code(lib: &Library, ast: &mut SwcModule) {
  let ParseScriptModuleResult { ast: module, .. } = parse_module(
    lib,
    "preCode",
    &PRE_CODE,
    Syntax::Es(Default::default()),
    Default::default(),
  )
  .unwrap();

  // insert pre code after last import
  ast.body.splice(1..1, module.body);
}

fn inject_post_code(lib: &Library, ast: &mut SwcModule) {
  let ParseScriptModuleResult { ast: module, .. } = parse_module(
    lib,
    "postCode",
    POST_CODE,
    Syntax::Es(Default::default()),
    Default::default(),
  )
  .unwrap();

  // insert post code at the end
  ast.body.extend(module.body);
}

// [react fast refresh](https://github.com/facebook/react/issues/16604#issuecomment-528663101)
pub fn inject_react_refresh(lib: &Library, ast: &mut SwcModule) {
  inject_runtime_import(lib, ast);
  inject_pre_code(lib, ast);
  inject_post_code(lib, ast);
}
