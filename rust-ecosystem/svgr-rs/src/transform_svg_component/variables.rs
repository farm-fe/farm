use std::rc::Rc;

use swc_common::{FileName, SourceMap, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_parser as parser;

use super::core;
use crate::{ExpandProps, ExportType, SvgrError};

pub struct TemplateVariables {
  #[allow(dead_code)]
  pub component_name: String,
  pub interfaces: Vec<ModuleItem>,
  pub props: Vec<Pat>,
  pub imports: Vec<ModuleItem>,
  pub exports: Vec<ModuleItem>,
  pub jsx: JSXElement,
}

#[derive(Default)]
pub enum JSXRuntime {
  Automatic,
  #[default]
  Classic,
}

#[derive(Default)]
pub struct Options {
  pub typescript: bool,
  pub title_prop: bool,
  pub desc_prop: bool,
  pub expand_props: ExpandProps,
  pub r#ref: bool,
  // pub template: Option<Box<dyn Template>>,
  pub native: bool,
  pub memo: bool,
  pub export_type: ExportType,
  pub named_export: Option<String>,
  pub jsx_runtime: JSXRuntime,
  pub jsx_runtime_import: Option<core::config::JSXRuntimeImport>,
  pub import_source: Option<String>,
}

pub fn get_variables(
  opts: Options,
  state: &core::state::InternalConfig,
  jsx: JSXElement,
) -> Result<TemplateVariables, SvgrError> {
  let mut interfaces = vec![];
  let mut props = vec![];
  let mut imports = vec![];
  let mut exports = vec![];

  let import_source = opts.import_source.unwrap_or("react".to_string());

  let mut export_identifier = state.component_name.clone();

  let is_automatic = matches!(opts.jsx_runtime, JSXRuntime::Automatic);
  if !is_automatic {
    match opts.jsx_runtime_import {
      Some(jsx_runtime_import) => {
        let jsx_runtime_import = get_jsx_runtime_import(&jsx_runtime_import)?;
        imports.push(jsx_runtime_import);
      }
      None => {
        let default_jsx_runtime_import = core::config::JSXRuntimeImport {
          source: "react".to_string(),
          namespace: Some("React".to_string()),
          ..Default::default()
        };
        let jsx_runtime_import = get_jsx_runtime_import(&default_jsx_runtime_import)?;
        imports.push(jsx_runtime_import);
      }
    }
  }

  if opts.native {
    let specifier = ImportSpecifier::Default(ImportDefaultSpecifier {
      span: DUMMY_SP,
      local: Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: "Svg".into(),
        optional: false,
      },
    });
    get_or_create_import(&mut imports, "react-native-svg", specifier);
  }

  if opts.title_prop || opts.desc_prop {
    let mut properties = vec![];
    let mut property_signatures = vec![];

    if opts.title_prop {
      properties.push(create_property("title"));
      properties.push(create_property("titleId"));

      if opts.typescript {
        property_signatures.push(create_signature("title"));
        property_signatures.push(create_signature("titleId"));
      }
    }

    if opts.desc_prop {
      properties.push(create_property("desc"));
      properties.push(create_property("descId"));

      if opts.typescript {
        property_signatures.push(create_signature("desc"));
        property_signatures.push(create_signature("descId"));
      }
    }

    let mut prop = ObjectPat {
      span: DUMMY_SP,
      props: properties,
      optional: false,
      type_ann: None,
    };

    if opts.typescript {
      let interface = ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(Box::new(TsInterfaceDecl {
        id: Ident::new("SVGRProps".into(), DUMMY_SP, SyntaxContext::empty()),
        span: DUMMY_SP,
        declare: false,
        type_params: None,
        extends: vec![],
        body: TsInterfaceBody {
          span: DUMMY_SP,
          body: property_signatures,
        },
      }))));
      interfaces.push(interface);

      prop.type_ann = Some(Box::new(TsTypeAnn {
        span: DUMMY_SP,
        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
          span: DUMMY_SP,
          type_name: TsEntityName::Ident(Ident::new(
            "SVGRProps".into(),
            DUMMY_SP,
            SyntaxContext::empty(),
          )),
          type_params: None,
        })),
      }));
    }

    props.push(Pat::Object(prop));
  }

  let need_expand_props = !matches!(opts.expand_props, ExpandProps::None);
  if need_expand_props {
    let existing = if !props.is_empty() {
      if let Pat::Object(ref mut object_pat) = props[0] {
        let identifier = Pat::Ident(BindingIdent::from(Ident::new(
          "props".into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        )));
        object_pat.props.push(ObjectPatProp::Rest(RestPat {
          span: DUMMY_SP,
          dot3_token: DUMMY_SP,
          arg: Box::new(identifier),
          type_ann: None,
        }));

        if opts.typescript {
          let svg_props_type =
            ts_type_reference_svg_props(&mut imports, opts.native, &import_source);
          let type_ann = Box::new(TsType::TsUnionOrIntersectionType(
            TsUnionOrIntersectionType::TsIntersectionType(TsIntersectionType {
              span: DUMMY_SP,
              types: vec![
                svg_props_type,
                Box::new(TsType::TsTypeRef(TsTypeRef {
                  span: DUMMY_SP,
                  type_name: TsEntityName::Ident(Ident::new(
                    "SVGRProps".into(),
                    DUMMY_SP,
                    SyntaxContext::empty(),
                  )),
                  type_params: None,
                })),
              ],
            }),
          ));
          object_pat.type_ann = Some(Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann,
          }));
        }

        true
      } else {
        false
      }
    } else {
      false
    };

    if !existing {
      let mut prop =
        BindingIdent::from(Ident::new("props".into(), DUMMY_SP, SyntaxContext::empty()));

      if opts.typescript {
        let type_ann = ts_type_reference_svg_props(&mut imports, opts.native, &import_source);
        prop.type_ann = Some(Box::new(TsTypeAnn {
          span: DUMMY_SP,
          type_ann,
        }));
      }

      props.push(Pat::Ident(prop));
    }
  }

  if opts.r#ref {
    if props.is_empty() {
      props.push(Pat::Ident(BindingIdent::from(Ident::new(
        "_".into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      ))));
    }
    let mut prop = BindingIdent::from(Ident::new("ref".into(), DUMMY_SP, SyntaxContext::empty()));

    if opts.typescript {
      get_or_create_named_import(&mut imports, "react", "Ref");

      prop.type_ann = Some(Box::new(TsTypeAnn {
        span: DUMMY_SP,
        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
          span: DUMMY_SP,
          type_name: TsEntityName::Ident(Ident::new(
            "Ref".into(),
            DUMMY_SP,
            SyntaxContext::empty(),
          )),
          type_params: Some(Box::new(TsTypeParamInstantiation {
            span: DUMMY_SP,
            params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
              span: DUMMY_SP,
              type_name: TsEntityName::Ident(Ident::new(
                "SVGSVGElement".into(),
                DUMMY_SP,
                SyntaxContext::empty(),
              )),
              type_params: None,
            }))],
          })),
        })),
      }));
    }

    props.push(Pat::Ident(prop));

    get_or_create_named_import(&mut imports, &import_source, "forwardRef");
    let hoc = create_var_decl_init_hoc("ForwardRef", "forwardRef", &export_identifier);
    exports.push(hoc);
    export_identifier = "ForwardRef".to_string();
  }

  if opts.memo {
    get_or_create_named_import(&mut imports, &import_source, "memo");
    let hoc = create_var_decl_init_hoc("Memo", "memo", &export_identifier);
    exports.push(hoc);
    export_identifier = "Memo".to_string();
  }

  let need_named_export = if state.caller.is_some() {
    true
  } else {
    matches!(opts.export_type, ExportType::Named)
  };
  if need_named_export {
    if let Some(named_export) = opts.named_export {
      let specifier = ExportSpecifier::Named(ExportNamedSpecifier {
        span: DUMMY_SP,
        orig: ModuleExportName::Ident(Ident::new(
          export_identifier.clone().into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        )),
        exported: Some(ModuleExportName::Ident(Ident {
          span: DUMMY_SP,
          ctxt: SyntaxContext::empty(),
          sym: named_export.into(),
          optional: false,
        })),
        is_type_only: false,
      });

      exports.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
        NamedExport {
          span: DUMMY_SP,
          specifiers: vec![specifier],
          src: None,
          type_only: false,
          with: None,
        },
      )));

      if let Some(caller) = &state.caller {
        if let Some(previous_export) = caller.previous_export.clone() {
          let cm = Rc::<SourceMap>::default();
          let fm = cm.new_source_file(FileName::Anon.into(), previous_export);

          let mut recovered_errors = vec![];
          let module = parser::parse_file_as_module(
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
          for module_item in module.body {
            exports.push(module_item)
          }
        }
      }
    } else {
      return Err(SvgrError::Configuration(
        r#""namedExport" not specified"#.to_string(),
      ));
    }
  }

  if !need_named_export {
    exports.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
      ExportDefaultExpr {
        span: DUMMY_SP,
        expr: Box::new(Expr::Ident(Ident::new(
          export_identifier.into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        ))),
      },
    )));
  }

  Ok(TemplateVariables {
    component_name: state.component_name.clone(),
    interfaces,
    props,
    imports,
    exports,
    jsx,
  })
}

fn get_jsx_runtime_import(cfg: &core::config::JSXRuntimeImport) -> Result<ModuleItem, SvgrError> {
  let specifiers = get_jsx_runtime_import_specifiers(cfg)?;

  Ok(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers,
    src: Box::new(Str {
      span: DUMMY_SP,
      value: cfg.source.clone().into(),
      raw: None,
    }),
    type_only: false,
    with: None,
    phase: Default::default(),
  })))
}

fn get_jsx_runtime_import_specifiers(
  cfg: &core::config::JSXRuntimeImport,
) -> Result<Vec<ImportSpecifier>, SvgrError> {
  if let Some(namespace) = cfg.namespace.clone() {
    let specifier = ImportSpecifier::Namespace(ImportStarAsSpecifier {
      span: DUMMY_SP,
      local: Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: namespace.into(),
        optional: false,
      },
    });
    return Ok(vec![specifier]);
  }

  if let Some(default_specifier) = cfg.default_specifier.clone() {
    let specifier = ImportSpecifier::Default(ImportDefaultSpecifier {
      span: DUMMY_SP,
      local: Ident {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        sym: default_specifier.into(),
        optional: false,
      },
    });
    return Ok(vec![specifier]);
  }

  if let Some(specifiers) = cfg.specifiers.clone() {
    let mut import_specifiers = vec![];
    for specifier in specifiers {
      import_specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
        span: DUMMY_SP,
        local: Ident {
          span: DUMMY_SP,
          ctxt: SyntaxContext::empty(),
          sym: specifier.into(),
          optional: false,
        },
        imported: None,
        is_type_only: false,
      }));
    }
    return Ok(import_specifiers);
  }

  Err(SvgrError::Configuration(
    r#"Specify "namespace", "defaultSpecifier", or "specifiers" in "jsxRuntimeImport" option"#
      .to_string(),
  ))
}

fn get_or_create_import(
  imports: &mut Vec<ModuleItem>,
  source_value: &str,
  specifier: ImportSpecifier,
) {
  let mut existing = None;
  for import in imports.iter_mut() {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = import {
      let is_namespace_import = import_decl
        .specifiers
        .iter()
        .any(|specifier| matches!(specifier, ImportSpecifier::Namespace(_)));
      if !is_namespace_import && import_decl.src.value.as_str() == Some(source_value) {
        existing = Some(import_decl);
        break;
      }
    }
  }

  if let Some(import_decl) = existing {
    import_decl.specifiers.push(specifier);
    return;
  }

  let module_item = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers: vec![specifier],
    src: Box::new(Str {
      span: DUMMY_SP,
      value: source_value.into(),
      raw: None,
    }),
    type_only: false,
    with: None,
    phase: Default::default(),
  }));
  imports.push(module_item);
}

fn get_or_create_named_import(imports: &mut Vec<ModuleItem>, source_value: &str, name: &str) {
  let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
    span: DUMMY_SP,
    local: Ident {
      span: DUMMY_SP,
      ctxt: SyntaxContext::empty(),
      sym: name.into(),
      optional: false,
    },
    imported: None,
    is_type_only: false,
  });
  get_or_create_import(imports, source_value, specifier)
}

fn create_var_decl_init_hoc(var_name: &str, callee: &str, component_name: &str) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    kind: VarDeclKind::Const,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent::from(Ident::new(
        var_name.into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      ))),
      definite: false,
      init: Some(Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
          callee.into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        )))),
        args: vec![ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(Ident::new(
            component_name.into(),
            DUMMY_SP,
            SyntaxContext::empty(),
          ))),
        }],
        type_args: None,
      }))),
    }],
  }))))
}

fn create_property(key: &str) -> ObjectPatProp {
  ObjectPatProp::Assign(AssignPatProp {
    span: DUMMY_SP,
    key: Ident::new(key.into(), DUMMY_SP, SyntaxContext::empty()).into(),
    value: None,
  })
}

fn create_signature(key: &str) -> TsTypeElement {
  TsTypeElement::TsPropertySignature(TsPropertySignature {
    span: DUMMY_SP,
    readonly: false,
    key: Box::new(Expr::Ident(Ident::new(
      key.into(),
      DUMMY_SP,
      SyntaxContext::empty(),
    ))),
    computed: false,
    optional: true,
    type_ann: Some(Box::new(TsTypeAnn {
      span: DUMMY_SP,
      type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
        span: DUMMY_SP,
        kind: TsKeywordTypeKind::TsStringKeyword,
      })),
    })),
  })
}

fn ts_type_reference_svg_props(
  imports: &mut Vec<ModuleItem>,
  native: bool,
  import_source: &str,
) -> Box<TsType> {
  if native {
    get_or_create_named_import(imports, "react-native-svg", "SvgProps");

    return Box::new(TsType::TsTypeRef(TsTypeRef {
      span: DUMMY_SP,
      type_name: TsEntityName::Ident(Ident::new(
        "SvgProps".into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      )),
      type_params: None,
    }));
  }

  get_or_create_named_import(imports, import_source, "SVGProps");

  Box::new(TsType::TsTypeRef(TsTypeRef {
    span: DUMMY_SP,
    type_name: TsEntityName::Ident(Ident::new(
      "SVGProps".into(),
      DUMMY_SP,
      SyntaxContext::empty(),
    )),
    type_params: Some(Box::new(TsTypeParamInstantiation {
      span: DUMMY_SP,
      params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
        span: DUMMY_SP,
        type_name: TsEntityName::Ident(Ident::new(
          "SVGSVGElement".into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        )),
        type_params: None,
      }))],
    })),
  }))
}
