use farmfe_core::{module::module_graph::ModuleGraph, HashMap, HashSet};

/// expand the exports of the module graph
/// for example, if the input files are:
/// ```js
/// // a.js
/// export * from './b.js';
/// export { bar } from './c.js';
/// export { default as baz } from './d.js';
/// export * as ns from './e.js';
/// export * from './f.js';
///
/// // b.js
/// export * as a from './a.js';
/// export * from './a.js';
/// export var foo = 1;
///
/// // c.js
/// export * as d from './d.js';
/// export var bar = 2;
///
/// // d.js
/// export * from './e.js';
/// export default 3;
///
/// // e.js
/// export var e = 4;
/// export default function() { console.log('e'); }
///
/// // f.js
/// var f = 5;
/// var f_d = 'f_d';
/// export { f, f_d as default };
/// ```
/// The output should be:
/// ```js
/// // exports of a.js
/// { foo: 1, bar: 2, baz: 3, ns: e_ns, f: 5  }
///
/// // exports of b.js
/// { a: a_ns， foo: 1, bar: 2, baz: 3, ns: e_ns, f: 5, default: 'f_d' }
///
/// // exports of c.js
/// { d: d_ns, bar: 2 }
///
/// // exports of d.js
/// { e: 4, default: 3 }
///
/// // exports of e.js
/// { e: 4, default: function() { console.log('e'); } }
///
/// // exports of f.js
/// { f: 5, default: 'f_d' }
/// ```
pub fn expand_exports(module_graph: &ModuleGraph) -> HashMap<ModuleId, ReferenceExport> {
  // 1. collect all exports by breadth-first search in the module graph(a common utility function that is used in concatenating modules and mangle exports)
  //  1.1. the export type can be: namespace object, named export(including default), reexport
  //  1.2. change the internal export of each module from the importer similar to the tree shaking
  // 2. concatenate import/export statements based on the collected exports above
  let mut reference_exports = HashMap::new();
  let mut visited = HashSet::new();

  for module_id in module_graph.module_ids() {
    let module = module_graph.module(&module_id).unwrap();
    if visited.contains(&module_id) {
      continue;
    }

    let reference_export = expand_exports_of_module(module_id, module_graph, &mut visited)?;
    reference_exports.insert(module_id, reference_export);
  }

  reference_exports
}
