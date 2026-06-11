use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

use crate::{core, SvgrError};

mod variables;

fn get_variables_options(config: &core::config::Config) -> variables::Options {
  let mut opts = variables::Options {
    typescript: config.typescript,
    title_prop: config.title_prop,
    desc_prop: config.desc_prop,
    expand_props: config.expand_props.clone(),
    r#ref: config.r#ref,
    native: config.native,
    memo: config.memo,
    named_export: Some(config.named_export.clone()),
    export_type: config.export_type.clone(),
    ..Default::default()
  };

  if let Some(jsx_runtime_import) = &config.jsx_runtime_import {
    opts.import_source = Some(jsx_runtime_import.source.clone());
    opts.jsx_runtime_import = Some(jsx_runtime_import.clone());
    return opts;
  }

  match &config.jsx_runtime {
    core::config::JSXRuntime::Classic => {
      opts.jsx_runtime = variables::JSXRuntime::Classic;
      opts.import_source = Some("react".to_string());
      opts.jsx_runtime_import = Some(core::config::JSXRuntimeImport {
        source: "react".to_string(),
        namespace: Some("React".to_string()),
        ..Default::default()
      });
    }
    core::config::JSXRuntime::ClassicPreact => {
      opts.jsx_runtime = variables::JSXRuntime::Classic;
      opts.import_source = Some("preact".to_string());
      opts.jsx_runtime_import = Some(core::config::JSXRuntimeImport {
        source: "preact".to_string(),
        specifiers: Some(vec!["h".to_string()]),
        ..Default::default()
      });
    }
    core::config::JSXRuntime::Automatic => {
      opts.jsx_runtime = variables::JSXRuntime::Automatic;
    }
  }

  opts
}

pub fn transform(
  jsx_element: JSXElement,
  config: &core::config::Config,
  state: &core::state::InternalConfig,
) -> Result<Module, SvgrError> {
  let variables_options = get_variables_options(config);

  let variables = variables::get_variables(variables_options, state, jsx_element)?;

  let mut body = vec![];

  for import in variables.imports {
    body.push(import);
  }

  for interface in variables.interfaces {
    body.push(interface);
  }

  body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    kind: VarDeclKind::Const,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent::from(Ident::new(
        state.component_name.clone().into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      ))),
      definite: false,
      init: Some(Box::new(Expr::Arrow(ArrowExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        params: variables.props,
        body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::JSXElement(Box::new(
          variables.jsx,
        ))))),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
      }))),
    }],
  })))));

  for export in variables.exports {
    body.push(export);
  }

  Ok(Module {
    span: DUMMY_SP,
    body,
    shebang: None,
  })
}

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use swc_common::{FileName, SourceMap};
  use swc_ecma_ast::EsVersion;
  use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
  use swc_ecma_parser as parser;

  use super::*;
  use crate::core;

  fn test_code(
    input: &str,
    config: &core::config::Config,
    state: &core::state::InternalConfig,
    expected: &str,
  ) {
    let cm = Rc::<SourceMap>::default();
    let fm = cm.new_source_file(FileName::Anon.into(), input.to_string());

    let mut recovered_errors = vec![];
    let expr = parser::parse_file_as_expr(
      fm.as_ref(),
      parser::Syntax::Es(parser::EsSyntax {
        jsx: true,
        ..Default::default()
      }),
      EsVersion::Es2020,
      None,
      &mut recovered_errors,
    )
    .unwrap();

    let jsx_element = expr.as_jsx_element().unwrap();

    let m = transform(*jsx_element.clone(), config, state).unwrap();

    let mut buf = vec![];
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: cm.clone(),
      comments: None,
      wr: JsWriter::new(cm, "\n", &mut buf, None),
    };
    emitter.emit_module(&m).unwrap();
    let result = String::from_utf8_lossy(&buf).to_string();

    assert_eq!(result, expected);
  }

  fn test_js_n_ts(
    input: &str,
    config: &core::config::Config,
    state: &core::state::InternalConfig,
    js: &str,
    ts: &str,
  ) {
    test_code(input, config, state, js);

    let mut config = config.clone();
    config.typescript = true;
    test_code(input, &config, state, ts);
  }

  #[test]
  fn transforms_whole_program() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_native_option_adds_import_from_react_native_svg() {
    test_js_n_ts(
      r#"<Svg><g/></Svg>"#,
      &core::config::Config {
        native: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
import Svg from "react-native-svg";
const SvgComponent = ()=><Svg><g/></Svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import Svg from "react-native-svg";
const SvgComponent = ()=><Svg><g/></Svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_ref_option_adds_forward_ref_component() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        r#ref: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import { forwardRef } from "react";
const SvgComponent = (_, ref)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
      r#"import * as React from "react";
import { Ref, forwardRef } from "react";
const SvgComponent = (_, ref: Ref<SVGSVGElement>)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
    );
  }

  #[test]
  fn with_title_prop_adds_title_and_title_id_prop() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        title_prop: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ title, titleId })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
interface SVGRProps {
    title?: string;
    titleId?: string;
}
const SvgComponent = ({ title, titleId }: SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_title_prop_and_expand_props_adds_title_title_id_props_and_expands_props() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        title_prop: true,
        expand_props: core::config::ExpandProps::End,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ title, titleId, ...props })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import { SVGProps } from "react";
interface SVGRProps {
    title?: string;
    titleId?: string;
}
const SvgComponent = ({ title, titleId, ...props }: SVGProps<SVGSVGElement> & SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_desc_prop_adds_desc_and_desc_id_prop() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        desc_prop: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ desc, descId })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
interface SVGRProps {
    desc?: string;
    descId?: string;
}
const SvgComponent = ({ desc, descId }: SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_desc_prop_and_expand_props_adds_desc_desc_id_props_and_expands_prop() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        expand_props: core::config::ExpandProps::End,
        desc_prop: true,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ desc, descId, ...props })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import { SVGProps } from "react";
interface SVGRProps {
    desc?: string;
    descId?: string;
}
const SvgComponent = ({ desc, descId, ...props }: SVGProps<SVGSVGElement> & SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_title_prop_and_desc_prop_adds_title_title_id_desc_and_desc_id_prop() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        title_prop: true,
        desc_prop: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ title, titleId, desc, descId })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
interface SVGRProps {
    title?: string;
    titleId?: string;
    desc?: string;
    descId?: string;
}
const SvgComponent = ({ title, titleId, desc, descId }: SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_title_prop_desc_prop_and_expand_props_adds_title_title_id_desc_desc_id_props_and_expands_props(
  ) {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        expand_props: core::config::ExpandProps::End,
        title_prop: true,
        desc_prop: true,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ({ title, titleId, desc, descId, ...props })=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import { SVGProps } from "react";
interface SVGRProps {
    title?: string;
    titleId?: string;
    desc?: string;
    descId?: string;
}
const SvgComponent = ({ title, titleId, desc, descId, ...props }: SVGProps<SVGSVGElement> & SVGRProps)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_expand_props_add_props() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        expand_props: core::config::ExpandProps::End,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = (props)=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import { SVGProps } from "react";
const SvgComponent = (props: SVGProps<SVGSVGElement>)=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_ref_and_expand_props_option_expands_props() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        expand_props: core::config::ExpandProps::End,
        r#ref: true,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import { forwardRef } from "react";
const SvgComponent = (props, ref)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
      r#"import * as React from "react";
import { SVGProps, Ref, forwardRef } from "react";
const SvgComponent = (props: SVGProps<SVGSVGElement>, ref: Ref<SVGSVGElement>)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
    );
  }

  #[test]
  fn with_native_ref_option_adds_import_from_react_native_svg_and_adds_forward_ref_component() {
    test_js_n_ts(
      r#"<Svg><g/></Svg>"#,
      &core::config::Config {
        native: true,
        r#ref: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import Svg from "react-native-svg";
import { forwardRef } from "react";
const SvgComponent = (_, ref)=><Svg><g/></Svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
      r#"import * as React from "react";
import Svg from "react-native-svg";
import { Ref, forwardRef } from "react";
const SvgComponent = (_, ref: Ref<SVGSVGElement>)=><Svg><g/></Svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
    );
  }

  #[test]
  fn with_native_and_expand_props_option() {
    test_js_n_ts(
      r#"<Svg><g/></Svg>"#,
      &core::config::Config {
        native: true,
        expand_props: core::config::ExpandProps::End,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import Svg from "react-native-svg";
const SvgComponent = (props)=><Svg><g/></Svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
import Svg, { SvgProps } from "react-native-svg";
const SvgComponent = (props: SvgProps)=><Svg><g/></Svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn with_native_ref_and_expand_props_option_adds_import_from_react_native_svg_and_adds_props_and_adds_forward_ref_component(
  ) {
    test_js_n_ts(
      r#"<Svg><g/></Svg>"#,
      &core::config::Config {
        native: true,
        expand_props: core::config::ExpandProps::End,
        r#ref: true,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import Svg from "react-native-svg";
import { forwardRef } from "react";
const SvgComponent = (props, ref)=><Svg><g/></Svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
      r#"import * as React from "react";
import Svg, { SvgProps } from "react-native-svg";
import { Ref, forwardRef } from "react";
const SvgComponent = (props: SvgProps, ref: Ref<SVGSVGElement>)=><Svg><g/></Svg>;
const ForwardRef = forwardRef(SvgComponent);
export default ForwardRef;
"#,
    );
  }

  #[test]
  fn with_memo_option_wrap_component_in_react_memo() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        memo: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import { memo } from "react";
const SvgComponent = ()=><svg><g/></svg>;
const Memo = memo(SvgComponent);
export default Memo;
"#,
      r#"import * as React from "react";
import { memo } from "react";
const SvgComponent = ()=><svg><g/></svg>;
const Memo = memo(SvgComponent);
export default Memo;
"#,
    );
  }

  #[test]
  fn with_both_memo_and_ref_option_wrap_component_in_react_memo_and_react_forward_ref() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        memo: true,
        r#ref: true,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
import { forwardRef, memo } from "react";
const SvgComponent = (_, ref)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
const Memo = memo(ForwardRef);
export default Memo;
"#,
      r#"import * as React from "react";
import { Ref, forwardRef, memo } from "react";
const SvgComponent = (_, ref: Ref<SVGSVGElement>)=><svg><g/></svg>;
const ForwardRef = forwardRef(SvgComponent);
const Memo = memo(ForwardRef);
export default Memo;
"#,
    );
  }

  #[test]
  fn with_named_export_option_and_previous_export_state_has_custom_named_export() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        named_export: "Component".to_string(),
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        caller: Some(core::state::Caller {
          previous_export: Some(
            "var img = new Image(); img.src = '...'; export default img;".to_string(),
          ),
          ..Default::default()
        }),
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export { SvgComponent as Component };
var img = new Image();
img.src = '...';
export default img;
"#,
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export { SvgComponent as Component };
var img = new Image();
img.src = '...';
export default img;
"#,
    );
  }

  #[test]
  fn with_named_export_and_export_type_option_and_without_previous_export_state_exports_via_named_export(
  ) {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        named_export: "ReactComponent".to_string(),
        export_type: core::config::ExportType::Named,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        component_name: "SvgComponent".to_string(),
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export { SvgComponent as ReactComponent };
"#,
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export { SvgComponent as ReactComponent };
"#,
    );
  }

  // TODO: custom templates

  #[test]
  fn jsx_runtime_supports_automatic_jsx_runtime() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Automatic,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn jsx_runtime_supports_classic_jsx_runtime() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Classic,
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as React from "react";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn allows_to_specify_a_custom_classic_jsx_runtime_using_specifiers() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Classic,
        jsx_runtime_import: Some(core::config::JSXRuntimeImport {
          specifiers: Some(vec!["h".to_string()]),
          source: "preact".to_string(),
          ..Default::default()
        }),
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import { h } from "preact";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import { h } from "preact";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn allows_to_specify_a_custom_classic_jsx_runtime_using_namespace() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Classic,
        jsx_runtime_import: Some(core::config::JSXRuntimeImport {
          namespace: Some("Preact".to_string()),
          source: "preact".to_string(),
          ..Default::default()
        }),
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import * as Preact from "preact";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import * as Preact from "preact";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  fn allows_to_specify_a_custom_classic_jsx_runtime_using_default_specifier() {
    test_js_n_ts(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Classic,
        jsx_runtime_import: Some(core::config::JSXRuntimeImport {
          default_specifier: Some("h".to_string()),
          source: "hyperapp-jsx-pragma".to_string(),
          ..Default::default()
        }),
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#"import h from "hyperapp-jsx-pragma";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
      r#"import h from "hyperapp-jsx-pragma";
const SvgComponent = ()=><svg><g/></svg>;
export default SvgComponent;
"#,
    );
  }

  #[test]
  #[should_panic(
    expected = r#"called `Result::unwrap()` on an `Err` value: Configuration("Specify \"namespace\", \"defaultSpecifier\", or \"specifiers\" in \"jsxRuntimeImport\" option")"#
  )]
  fn throws_with_invalid_configuration() {
    test_code(
      r#"<svg><g/></svg>"#,
      &core::config::Config {
        jsx_runtime: core::config::JSXRuntime::Classic,
        jsx_runtime_import: Some(core::config::JSXRuntimeImport {
          source: "preact".to_string(),
          ..Default::default()
        }),
        expand_props: core::config::ExpandProps::None,
        ..Default::default()
      },
      &core::state::InternalConfig {
        ..Default::default()
      },
      r#""#,
    );
  }
}
