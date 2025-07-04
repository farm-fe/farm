use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::Arc,
};

use common::create_module;
use farmfe_core::{
  module::module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
  plugin::ResolveKind,
  swc_common::{Globals, GLOBALS},
};
use farmfe_plugin_tree_shake::{
  module::{TreeShakeModule, UsedExports, UsedExportsIdent},
  tree_shake_modules::remove_useless_stmts::remove_useless_stmts,
};
use farmfe_testing_helpers::fixture;
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  fs::read_file_utf8,
  script::codegen_module,
};

use crate::common::create_module_with_comments;

mod common;

#[test]
fn remove_useless_stmts_basic() {
  let code = r#"
import { aValue, bar } from './foo';
const a = aValue;
const b = 2;
const c = 3;
export { a, b, c as d };
export { e, default as f, g } from './src/foo';
export * as any from './src/bar';
export const h = 1;
export function i() {
  return h;
}
export class j {
  constructor() {
    this.i = i();
  }
}
export default 'default';
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from([
      UsedExportsIdent::Default,
      UsedExportsIdent::SwcIdent("j".to_string()),
      UsedExportsIdent::SwcIdent("d".to_string()),
      UsedExportsIdent::SwcIdent("f".to_string()),
      UsedExportsIdent::SwcIdent("a".to_string()),
    ]));
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let mut tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);
    let mut module_src_bar = create_module("").0;
    module_src_bar.id = "src/bar".into();
    tree_shake_module_map.insert("src/bar".into(), TreeShakeModule::new(&mut module_src_bar));
    module_graph.add_module(module_src_bar);
    module_graph
      .add_edge(
        &module_id,
        &"src/bar".into(),
        ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
          source: "./src/bar".to_string(),
          kind: ResolveKind::ExportFrom,
          order: 0,
        }]),
      )
      .unwrap();

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);

    let module = module_graph.module(&module_id).unwrap();
    let swc_module = &module.meta.as_script().ast;

    let bytes = codegen_module(swc_module, cm, None, Default::default(), None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    println!("{result}");
    let expect = r#"import { aValue } from './foo';
const a = aValue;
const c = 3;
export { a, c as d };
export { default as f } from './src/foo';
export const h = 1;
export function i() {
    return h;
}
export class j {
    constructor(){
        this.i = i();
    }
}
export default 'default';
    "#
    .trim();
    // asset result and expect line by line
    let result_lines = result.trim().lines();
    let expect_lines = expect.lines();
    for (result_line, expect_line) in result_lines.zip(expect_lines) {
      assert_eq!(result_line, expect_line);
    }
  });
}

#[test]
fn remove_useless_stmts_export_all() {
  let code = r#"
import { aValue, bar } from './foo';
export const a = aValue;
const b = 2;
export * from './src/foo';
"#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    // tree_shake_module.used_exports = UsedExports::Partial(HashMap::from([(
    //   "index.ts".into(),
    //   vec!["a".to_string(), "c".to_string(), "d".to_string()],
    // )]));
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from([
      UsedExportsIdent::SwcIdent("a".to_string()),
      UsedExportsIdent::SwcIdent("c".to_string()),
      UsedExportsIdent::SwcIdent("d".to_string()),
    ]));
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);
    let module = module_graph.module(&module_id).unwrap();
    let swc_module = &module.meta.as_script().ast;

    let bytes = codegen_module(swc_module, cm, None, Default::default(), None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    assert_eq!(
      result,
      r#"import { aValue } from './foo';
export const a = aValue;
export * from './src/foo';
"#
    );
  });
}

#[test]
fn remove_useless_stmts_export_all_multiple() {
  let code = r#"
import { aValue, bar } from './foo';
export const a = aValue;
export * from './src/foo';
export * from './src/bar';
"#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from([
      UsedExportsIdent::SwcIdent("c".to_string()),
      UsedExportsIdent::SwcIdent("d".to_string()),
    ]));
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let mut tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);
    let mut module_foo = create_module("").0;
    module_foo.id = "src/foo".into();
    tree_shake_module_map.insert("src/foo".into(), TreeShakeModule::new(&mut module_foo));
    module_graph.add_module(module_foo);
    module_graph
      .add_edge(
        &module_id,
        &"src/foo".into(),
        ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
          source: "./foo".to_string(),
          kind: ResolveKind::Import,
          order: 0,
        }]),
      )
      .unwrap();

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);
    let module = module_graph.module(&module_id).unwrap();
    let swc_module = &module.meta.as_script().ast;

    let bytes = codegen_module(swc_module, cm, None, Default::default(), None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    assert_eq!(
      result,
      r#"export * from './src/foo';
export * from './src/bar';
"#
    );
  });
}

#[test]
fn remove_useless_stmts_nested_defined_idents() {
  let code = r#"
  import { a, invalidate } from './dep';

  console.log(a);

  const id = 'InvalidateParent';

  export function InvalidateParent() {
    return {
      render: () => {
        const renderData = invalidate();

        const div = document.createElement('div', {});
        div.id = id;
        div.innerText = renderData;
        div.className = 'box';
        return div;
      }
    };
  }

  if (import.meta.hot) {
    // self accept without reload the page
    import.meta.hot.accept();
    const div = document.getElementById(id);

    if (div) {
      const comp = InvalidateParent().render();
      console.log(div, comp);
      div.replaceWith(comp);
    }
  }
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports = UsedExports::All;
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);
    let module = module_graph.module(&module_id).unwrap();
    let swc_module = &module.meta.as_script().ast;

    let bytes = codegen_module(swc_module, cm, None, Default::default(), None).unwrap();
    let result = String::from_utf8(bytes).unwrap();

    let expect = r#"import { a, invalidate } from './dep';
console.log(a);
const id = 'InvalidateParent';
export function InvalidateParent() {
    return {
        render: ()=>{
            const renderData = invalidate();
            const div = document.createElement('div', {});
            div.id = id;
            div.innerText = renderData;
            div.className = 'box';
            return div;
        }
    };
}
if (import.meta.hot) {
    import.meta.hot.accept();
    const div = document.getElementById(id);
    if (div) {
        const comp = InvalidateParent().render();
        console.log(div, comp);
        div.replaceWith(comp);
    }
}
"#
    .trim();
    // asset result and expect line by line
    let result_lines = result.trim().lines();
    let expect_lines = expect.lines();
    for (result_line, expect_line) in result_lines.zip(expect_lines) {
      assert_eq!(result_line, expect_line);
    }
  });
}

#[test]
fn trace_loadable_esm() {
  let code = include_str!("./fixtures/remove_useless_stmts/loadable.esm.js");

  GLOBALS.set(&Globals::new(), || {
    let mut module = create_module_with_comments(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports =
      UsedExports::Partial(HashSet::from([UsedExportsIdent::Default]));
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);

    fixture!("tests/fixtures/remove_useless_stmts/*.js", |file, _| {
      let (cm, _) = create_swc_source_map(Source {
        path: PathBuf::from("any"),
        content: Arc::new(code.to_string()),
      });
      let ast = &module_graph
        .module_mut(&module_id)
        .unwrap()
        .meta
        .as_script()
        .ast;
      let code_bytes = codegen_module(ast, cm, None, Default::default(), None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      let output_path = PathBuf::from(file).parent().unwrap().join("output.js");

      if output_path.exists() {
        let output_code =
          read_file_utf8(output_path.to_string_lossy().to_string().as_str()).unwrap();
        assert_eq!(
          output_code.replace("\r\n", "\n"),
          code.replace("\r\n", "\n")
        );
      } else {
        std::fs::write(output_path, code).unwrap();
      }
    });
  });
}

#[test]
fn trace_complex_decl_stmt() {
  let code = r#"
  import { h, BaseTransition, BaseTransitionPropsValidators } from '@vue/runtime-core';

  const Transition = (props, { slots }) => h(BaseTransition, resolveTransitionProps(props), slots);
  Transition.displayName = "Transition";

  const TransitionPropsValidators = Transition.props = /* @__PURE__ */ extend(
    {},
    BaseTransitionPropsValidators,
    DOMTransitionPropsValidators
  );

  export default Transition;
  "#;

  GLOBALS.set(&Globals::new(), || {
    let mut module = create_module_with_comments(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports =
      UsedExports::Partial(HashSet::from([UsedExportsIdent::Default]));
    tree_shake_module.trace_and_mark_used_statements();

    let module_id = module.id.clone();
    let mut module_graph = ModuleGraph::new();
    let tree_shake_module_map = HashMap::from([(module.id.clone(), tree_shake_module)]);
    module_graph.add_module(module);

    remove_useless_stmts(&module_id, &mut module_graph, &tree_shake_module_map);

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from("any"),
      content: Arc::new(code.to_string()),
    });
    let ast = &module_graph
      .module_mut(&module_id)
      .unwrap()
      .meta
      .as_script()
      .ast;
    let code_bytes = codegen_module(ast, cm, None, Default::default(), None).unwrap();
    let code = String::from_utf8(code_bytes).unwrap();

    assert_eq!(code.replace("\r\n", "\n"), r#"import { h, BaseTransition, BaseTransitionPropsValidators } from '@vue/runtime-core';
const Transition = (props, { slots })=>h(BaseTransition, resolveTransitionProps(props), slots);
Transition.displayName = "Transition";
const TransitionPropsValidators = Transition.props = extend({}, BaseTransitionPropsValidators, DOMTransitionPropsValidators);
export default Transition;
"#.replace("\r\n", "\n"))
  });
}
