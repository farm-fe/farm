use std::sync::Arc;

use farmfe_core::{
  swc_common::{GLOBALS, Globals, Mark, comments::SingleThreadedComments},
  swc_ecma_ast::{EsVersion, Module},
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  script::ParseScriptModuleResult, swc_ecma_transforms::resolver, swc_ecma_visit::VisitMutWith,
};

pub fn parse_module_internal(code: &str) -> (Module, SingleThreadedComments, Mark, Mark) {
  let ParseScriptModuleResult {
    ast: mut swc_module,
    comments,
    ..
  } = farmfe_toolkit::script::parse_module(
    &"any".into(),
    Arc::new(code.to_string()),
    Syntax::Es(EsSyntax {
      auto_accessors: true,
      decorators: true,
      ..Default::default()
    }),
    EsVersion::Es2022,
    // None,
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
          .map(|i| format!("{}{:?}", i.ident.sym, i.ident.ctxt()))
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
          .map(|i| format!("{}{:?}", i.ident.sym, i.ident.ctxt()))
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

#[test]
fn class_definition_side_effects() {
  GLOBALS.set(&Globals::new(), || {
    let bodies = [
      "static { console.log('static block'); }",
      "static field = console.log('static field');",
      "static #field = console.log('private static field');",
      "[console.log('computed method')]() {}",
      "static [console.log('computed static method')]() {}",
      "get [console.log('computed getter')]() { return 1; }",
      "[console.log('computed field')] = 1;",
      "static [console.log('computed static field')] = 1;",
    ];
    for body in bodies {
      for code in [
        format!("class Unused {{ {body} }}"),
        format!("const Unused = class {{ {body} }};"),
        format!("export const Unused = class {{ {body} }};"),
        format!("export class Unused {{ {body} }}"),
        format!("export default class {{ {body} }}"),
      ] {
        let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(&code);
        let effects = super::analyze_statement_side_effects(
          &module.body[0],
          unresolved_mark,
          top_level_mark,
          &comments,
        );
        assert!(effects.is_preserved(), "{code}: {effects:?}");
      }
    }
  });
}

#[test]
fn class_deferred_bodies_have_no_side_effects() {
  GLOBALS.set(&Globals::new(), || {
    for body in [
      "constructor() { console.log('constructor'); }",
      "method() { console.log('method'); }",
      "static method() { console.log('static method'); }",
      "get value() { return console.log('getter'); }",
      "field = console.log('instance field');",
      "#field = console.log('private instance field');",
      "static field = 1; static {} ['method']() {}",
    ] {
      for code in [
        format!("class Unused {{ {body} }}"),
        format!("const Unused = class {{ {body} }};"),
        format!("export const Unused = class {{ {body} }};"),
        format!("export class Unused {{ {body} }}"),
        format!("export default class {{ {body} }}"),
      ] {
        let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(&code);
        let effects = super::analyze_statement_side_effects(
          &module.body[0],
          unresolved_mark,
          top_level_mark,
          &comments,
        );
        assert_eq!(
          effects,
          super::StatementSideEffects::NoSideEffects,
          "{code}"
        );
      }
    }
  });
}

#[test]
fn class_definition_implicit_effects() {
  GLOBALS.set(&Globals::new(), || {
    for code in [
      "class Unused { static { throw 1; } }",
      "class Unused { static { if (true) { throw 1; } } }",
      "const key = { toString() { console.log('key'); return 'x'; } }; class Unused { [key]() {} }",
      "const key = { [Symbol.toPrimitive]() { throw 1; } }; class Unused { [key] = 1; }",
      "const key = { toString() { console.log('key'); return 'x'; } }; class Unused { static [key]() {} }",
      "class Unused { [{ toString() { console.log('key'); return 'x'; } }]() {} }",
      "class Unused { [/key/]() {} }",
      "class Unused extends 1 {}",
      "const Base = 1; class Unused extends Base {}",
      "const Base = new Proxy(class {}, { get() { throw 1; } }); class Unused extends Base {}",
    ] {
      let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);
      let effects = super::analyze_statement_side_effects(
        module.body.last().unwrap(), unresolved_mark, top_level_mark, &comments,
      );
      assert!(effects.is_preserved(), "{code}: {effects:?}");
    }
  });
}

#[test]
fn class_definition_safe_primitive_keys_and_null_base() {
  GLOBALS.set(&Globals::new(), || {
    for code in [
      "class Unused extends null {}",
      "class Unused { ['key']() {} [1]() {} [true]() {} [null]() {} [1n]() {} }",
      "class Unused { static {} }",
    ] {
      let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(code);
      let effects = super::analyze_statement_side_effects(
        &module.body[0],
        unresolved_mark,
        top_level_mark,
        &comments,
      );
      assert_eq!(
        effects,
        super::StatementSideEffects::NoSideEffects,
        "{code}"
      );
    }
  });
}

#[test]
fn class_static_initializer_implicit_effects() {
  GLOBALS.set(&Globals::new(), || {
    for body in [
      "static value = object.value;",
      "static #value = object.value;",
      "static accessor value = object.value;",
      "static accessor #value = object.value;",
      "static value = 1n + 1;",
      "static #value = 1n + 1;",
      "static value = class { static value = object.value; };",
    ] {
      let code =
        format!("const object = {{ get value() {{ throw 1; }} }}; class Unused {{ {body} }}");
      let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(&code);
      let effects = super::analyze_statement_side_effects(
        module.body.last().unwrap(),
        unresolved_mark,
        top_level_mark,
        &comments,
      );
      assert!(effects.is_preserved(), "{code}: {effects:?}");
    }
  });
}

#[test]
fn class_static_initializer_inert_values() {
  GLOBALS.set(&Globals::new(), || {
    for value in [
      "1",
      "'value'",
      "true",
      "null",
      "1n",
      "/value/",
      "function() { throw 1; }",
      "() => { throw 1; }",
      "class { value = object.value; }",
    ] {
      for field in ["value", "#value", "accessor value", "accessor #value"] {
        let code = format!("class Unused {{ static {field} = {value}; }}");
        let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(&code);
        let effects = super::analyze_statement_side_effects(
          &module.body[0],
          unresolved_mark,
          top_level_mark,
          &comments,
        );
        assert_eq!(
          effects,
          super::StatementSideEffects::NoSideEffects,
          "{code}"
        );
      }
    }
  });
}

#[test]
fn class_decorator_application_is_a_definition_time_effect() {
  GLOBALS.set(&Globals::new(), || {
    for decorator in ["decorate", "decorate()"] {
      for target in [
        format!("@{decorator} class Unused {{}}"),
        format!("class Unused {{ @{decorator} method() {{}} }}"),
        format!("class Unused {{ @{decorator} static method() {{}} }}"),
        format!("class Unused {{ @{decorator} #method() {{}} }}"),
        format!("class Unused {{ @{decorator} static #method() {{}} }}"),
        format!("class Unused {{ @{decorator} value = 1; }}"),
      ] {
        let code = format!("function decorate() {{}}; {target}");
        let (module, comments, unresolved_mark, top_level_mark) = parse_module_comments(&code);
        let effects = super::analyze_statement_side_effects(
          module.body.last().unwrap(),
          unresolved_mark,
          top_level_mark,
          &comments,
        );
        assert!(effects.is_preserved(), "{target}: {effects:?}");
      }
    }
  });
}

#[test]
fn update_and_delete_write_top_level_var() {
  GLOBALS.set(&Globals::new(), || {
    let code = r#"
    let count = 0;
    for (let i = 0; i < 3; i++) count++;
    const o = { a: 1, b: 2 };
    delete o.a;
    --count;
  "#;
    let (module, unresolved_mark, top_level_mark) = parse_module(code);

    for (index, expected) in [(1, "count#2"), (3, "o#2"), (4, "count#2")] {
      let side_effects = super::analyze_statement_side_effects(
        &module.body[index],
        unresolved_mark,
        top_level_mark,
        &SingleThreadedComments::default(),
      );
      let idents = match side_effects {
        super::StatementSideEffects::WriteTopLevelVar(var_name) => var_name
          .into_iter()
          .map(|i| format!("{}{:?}", i.ident.sym, i.ident.ctxt()))
          .collect::<Vec<_>>(),
        other => panic!("statement {index} should write a top level var, got {other:?}"),
      };
      assert!(
        idents.iter().any(|i| i == expected),
        "statement {index} should write {expected}, got {idents:?}"
      );
    }
  });
}

#[test]
fn update_delete_and_read_only_unary_classification() {
  GLOBALS.set(&Globals::new(), || {
    for expression in [
      "count++",
      "++count",
      "count--",
      "--count",
      "object.count++",
      "++object.count",
      "delete object.count",
      "delete object['count']",
    ] {
      let code = format!("let count = 0; const object = {{ count: 0 }}; {expression};");
      let (module, unresolved_mark, top_level_mark) = parse_module(&code);
      let effects = super::analyze_statement_side_effects(
        module.body.last().unwrap(),
        unresolved_mark,
        top_level_mark,
        &SingleThreadedComments::default(),
      );
      assert!(
        matches!(effects, super::StatementSideEffects::WriteTopLevelVar(_)),
        "{expression}: {effects:?}"
      );
    }
    for expression in ["!count", "typeof count", "-count", "+count", "void count"] {
      let code = format!("let count = 0; {expression};");
      let (module, unresolved_mark, top_level_mark) = parse_module(&code);
      let effects = super::analyze_statement_side_effects(
        module.body.last().unwrap(),
        unresolved_mark,
        top_level_mark,
        &SingleThreadedComments::default(),
      );
      assert!(
        matches!(effects, super::StatementSideEffects::ReadTopLevelVar(_)),
        "{expression}: {effects:?}"
      );
    }
  });
}
