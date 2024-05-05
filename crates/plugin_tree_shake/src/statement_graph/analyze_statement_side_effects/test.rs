use farmfe_core::{
  swc_common::{Globals, Mark, GLOBALS},
  swc_ecma_ast::{EsVersion, Module},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  script::ParseScriptModuleResult, swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith,
};

pub fn parse_module(code: &str) -> (Module, Mark, Mark) {
  let ParseScriptModuleResult {
    ast: mut swc_module,
    ..
  } = farmfe_toolkit::script::parse_module(
    "any",
    code,
    Syntax::Es(Default::default()),
    EsVersion::Es2022,
  )
  .unwrap();
  let top_level_mark = Mark::new();
  let unresolved_mark = Mark::new();

  swc_module.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

  (swc_module, unresolved_mark, top_level_mark)
}

#[test]
fn no_side_effects() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    import { a } from 'b';
    export const c = 1;
    export const d = c;

    function f() {
      const e = 1;
      return e;
    }

    var g = '123';

    export { g, f };
    export default a;
  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    for item in module.body.iter() {
      let side_effects =
        super::analyze_statement_side_effects(item, unresolved_mark, top_level_mark);
      assert_eq!(side_effects, super::StatementSideEffects::NoSideEffects);
    }
  });
}

#[test]
fn write_top_level_var() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    import { a, b } from 'c';
    a.prototype.aa = b;

    const c = 1;
    c = 2;
  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark);
    println!("{:?}", side_effects);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));
    let idents = match side_effects {
      super::StatementSideEffects::WriteTopLevelVar(var_name) => {
        let mut idents = var_name
          .into_iter()
          .map(|i| format!("{}{:?}", i.0, i.1))
          .collect::<Vec<_>>();
        idents.sort();
        idents
      }
      _ => unreachable!(),
    };
    assert_eq!(idents, vec!["a#2"]);

    let item_4 = &module.body[3];
    let side_effects =
      super::analyze_statement_side_effects(item_4, unresolved_mark, top_level_mark);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));
    let idents = match side_effects {
      super::StatementSideEffects::WriteTopLevelVar(var_name) => {
        let mut idents = var_name
          .into_iter()
          .map(|i| format!("{}{:?}", i.0, i.1))
          .collect::<Vec<_>>();
        idents.sort();
        idents
      }
      _ => unreachable!(),
    };
    assert_eq!(idents, vec!["c#2"]);
  });
}

#[test]
fn access_global_var() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    import { a, b } from 'c';
    a.prototype.aa = window.aa;

    const c = 1;
    console.log(c);
  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark);
    println!("{:?}", side_effects);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::AccessGlobalVar
    ));

    let item_4 = &module.body[3];
    let side_effects =
      super::analyze_statement_side_effects(item_4, unresolved_mark, top_level_mark);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::UnclassifiedSelfExecuted
    ));
  });
}

#[test]
fn commonjs_module_exports() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    module.exports = {
      program: function() {}
    }
  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    let item_1 = &module.body[0];
    let side_effects =
      super::analyze_statement_side_effects(item_1, unresolved_mark, top_level_mark);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::AccessGlobalVar
    ));
  });
}
