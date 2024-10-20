use farmfe_core::{
  swc_common::{comments::SingleThreadedComments, Globals, Mark, GLOBALS},
  swc_ecma_ast::{EsVersion, Module},
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  script::ParseScriptModuleResult, swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith,
};

pub fn parse_module_internal(code: &str) -> (Module, SingleThreadedComments, Mark, Mark) {
  let ParseScriptModuleResult {
    ast: mut swc_module,
    comments,
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

  (swc_module, comments, unresolved_mark, top_level_mark)
}

pub fn parse_module(code: &str) -> (Module, Mark, Mark) {
  let res = parse_module_internal(code);
  (res.0, res.2, res.3)
}

pub fn parse_module_comments(code: &str) -> (Module, SingleThreadedComments, Mark, Mark) {
  parse_module_internal(code)
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

    for (i, item) in module.body.iter().enumerate() {
      let side_effects = super::analyze_statement_side_effects(
        item,
        unresolved_mark,
        top_level_mark,
        &SingleThreadedComments::default(),
      );

      if i == 2 || i == 6 {
        assert!(matches!(
          side_effects,
          super::StatementSideEffects::ReadTopLevelVar(_)
        ));
      } else {
        assert_eq!(side_effects, super::StatementSideEffects::NoSideEffects);
      }
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
    let side_effects = super::analyze_statement_side_effects(
      item_2,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );
    println!("{side_effects:?}");
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
    let side_effects = super::analyze_statement_side_effects(
      item_4,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );
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
    let side_effects = super::analyze_statement_side_effects(
      item_2,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));

    let item_4 = &module.body[3];
    let side_effects = super::analyze_statement_side_effects(
      item_4,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteOrCallGlobalVar
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
    let side_effects = super::analyze_statement_side_effects(
      item_1,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteOrCallGlobalVar
    ));
  });
}

#[test]
fn access_global_var_and_func_call() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    const a = 10;

const b = 20;

const c = 30;

console.log(a, b);

  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    let item_4 = &module.body[3];
    let side_effects = super::analyze_statement_side_effects(
      item_4,
      unresolved_mark,
      top_level_mark,
      &SingleThreadedComments::default(),
    );

    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteOrCallGlobalVar
    ));
  });
}

#[test]
fn top_level_if_and_for_while_statement() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    const Steps = () => {};
    Steps.Step = RcSteps.Step;
if (process.env.NODE_ENV !== 'production') {
  Steps.displayName = 'Steps';
}
for (let i = 0; i < 10; i++) {
  Steps[i] = i;
}
while (getCount() < 100) {
  setCount(getCount() + 1);
}
export default Steps;
  "#;
    let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark, &comments);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));

    let item_3 = &module.body[2];
    let side_effects =
      super::analyze_statement_side_effects(item_3, unresolved_mark, top_level_mark, &comments);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));

    let item_4 = &module.body[3];
    let side_effects =
      super::analyze_statement_side_effects(item_4, unresolved_mark, top_level_mark, &comments);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));

    let item_5 = &module.body[4];
    let side_effects =
      super::analyze_statement_side_effects(item_5, unresolved_mark, top_level_mark, &comments);
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteOrCallGlobalVar
    ));
  });
}

#[test]
fn pure_comments() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
  import React from 'react';
const comp = /*#__PURE__*/React.createElement();
comp();
"#;
    let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::NoSideEffects
    ));

    let item_3 = &module.body[2];
    let side_effects =
      super::analyze_statement_side_effects(item_3, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::UnclassifiedSelfExecuted
    ));
  })
}

#[test]
fn assign_local_var() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
  import React from 'react';
const comp = /*#__PURE__*/React.createElement();
_c = comp;
var _c;
"#;
    let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::NoSideEffects
    ));

    let item_3 = &module.body[2];
    let side_effects =
      super::analyze_statement_side_effects(item_3, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));
  })
}

#[test]
fn object_lit_assign() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    const a2 = {};
    const b2 = { a2 };
    
    b2.a2.aaa = 2;
    
    export { a2 };
"#;
    let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);

    let item_2 = &module.body[1];
    let side_effects =
      super::analyze_statement_side_effects(item_2, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::ReadTopLevelVar(_)
    ));

    let item_3 = &module.body[2];
    let side_effects =
      super::analyze_statement_side_effects(item_3, unresolved_mark, top_level_mark, &comments);
    println!("{side_effects:?}");
    assert!(matches!(
      side_effects,
      super::StatementSideEffects::WriteTopLevelVar(_)
    ));
  })
}
