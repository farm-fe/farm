use farmfe_core::swc_common::DUMMY_SP;
use farmfe_core::swc_ecma_ast::*;
use farmfe_toolkit::swc_ecma_utils::{private_ident, quote_ident, quote_str};

pub(crate) fn build_slot_helper(helper_name: Ident, is_vnode: Ident) -> FnDecl {
    let arg = private_ident!("s");

    FnDecl {
        ident: helper_name,
        declare: false,
        function: Box::new(Function {
            params: vec![Param {
                span: DUMMY_SP,
                decorators: vec![],
                pat: Pat::Ident(BindingIdent {
                    id: arg.clone(),
                    type_ann: None,
                }),
            }],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(Expr::Bin(BinExpr {
                        span: DUMMY_SP,
                        op: op!("||"),
                        left: Box::new(Expr::Bin(BinExpr {
                            span: DUMMY_SP,
                            op: op!("==="),
                            left: Box::new(Expr::Unary(UnaryExpr {
                                span: DUMMY_SP,
                                op: op!("typeof"),
                                arg: Box::new(Expr::Ident(arg.clone())),
                            })),
                            right: Box::new(Expr::Lit(Lit::Str(quote_str!("function")))),
                        })),
                        right: Box::new(Expr::Bin(BinExpr {
                            span: DUMMY_SP,
                            op: op!("&&"),
                            left: Box::new(Expr::Bin(BinExpr {
                                span: DUMMY_SP,
                                op: op!("==="),
                                left: Box::new(Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                        span: DUMMY_SP,
                                        obj: Box::new(Expr::Member(MemberExpr {
                                            span: DUMMY_SP,
                                            obj: Box::new(Expr::Object(ObjectLit {
                                                span: DUMMY_SP,
                                                props: vec![],
                                            })),
                                            prop: MemberProp::Ident(quote_ident!("toString")),
                                        })),
                                        prop: MemberProp::Ident(quote_ident!("call")),
                                    }))),
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Ident(arg.clone())),
                                    }],
                                    ..Default::default()
                                })),
                                right: Box::new(Expr::Lit(Lit::Str(quote_str!("[object Object]")))),
                            })),
                            right: Box::new(Expr::Unary(UnaryExpr {
                                span: DUMMY_SP,
                                op: op!("!"),
                                arg: Box::new(Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Callee::Expr(Box::new(Expr::Ident(is_vnode))),
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Ident(arg)),
                                    }],
                                    ..Default::default()
                                })),
                            })),
                        })),
                    }))),
                })],
                ..Default::default()
            }),
            is_generator: false,
            is_async: false,
            ..Default::default()
        }),
    }
}

pub(crate) fn is_jsx_attr_value_constant(value: &JSXAttrValue) -> bool {
    match value {
        JSXAttrValue::Str(..) => true,
        JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        }) => is_constant(expr),
        _ => false,
    }
}

fn is_constant(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(ident) => &ident.sym == "undefined",
        Expr::Array(ArrayLit { elems, .. }) => elems.iter().all(|element| match element {
            Some(ExprOrSpread { spread: None, expr }) => is_constant(expr),
            _ => false,
        }),
        Expr::Object(ObjectLit { props, .. }) => props.iter().all(|prop| {
            if let PropOrSpread::Prop(prop) = prop {
                match &**prop {
                    Prop::KeyValue(KeyValueProp { value, .. }) => is_constant(value),
                    Prop::Shorthand(ident) => &ident.sym == "undefined",
                    _ => false,
                }
            } else {
                false
            }
        }),
        Expr::Lit(..) => true,
        _ => false,
    }
}

pub(crate) fn is_on(attr_name: &str) -> bool {
    match attr_name.as_bytes() {
        [b'o', b'n', c, ..] => !c.is_ascii_lowercase(),
        _ => false,
    }
}

pub(crate) fn dedupe_props(props: Vec<PropOrSpread>) -> Vec<PropOrSpread> {
    let capacity = props.len();
    props.into_iter().fold(
        Vec::with_capacity(capacity),
        |mut defined, prop_or_spread| {
            if let PropOrSpread::Prop(prop) = prop_or_spread {
                if let Prop::KeyValue(KeyValueProp {
                    key:
                        PropName::Str(Str {
                            value: ref name,
                            raw,
                            span,
                        }),
                    value,
                }) = *prop
                {
                    match defined.iter_mut().find_map(|item| match item {
                        PropOrSpread::Prop(prop) => match &mut **prop {
                            Prop::KeyValue(KeyValueProp {
                                key:
                                    PropName::Str(Str {
                                        value: defined_name,
                                        ..
                                    }),
                                value,
                                ..
                            }) if defined_name == name => Some(value),
                            _ => None,
                        },
                        _ => None,
                    }) {
                        Some(defined_value)
                            if name == "class" || name == "style" || name.starts_with("on") =>
                        {
                            if let Expr::Array(ArrayLit { elems, .. }) = &mut **defined_value {
                                elems.push(Some(ExprOrSpread {
                                    spread: None,
                                    expr: value,
                                }));
                            } else {
                                *defined_value = Box::new(Expr::Array(ArrayLit {
                                    span: DUMMY_SP,
                                    elems: vec![
                                        Some(ExprOrSpread {
                                            spread: None,
                                            expr: defined_value.clone(),
                                        }),
                                        Some(ExprOrSpread {
                                            spread: None,
                                            expr: value,
                                        }),
                                    ],
                                }));
                            }
                        }
                        Some(..) => {}
                        None => {
                            defined.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
                                    key: PropName::Str(Str {
                                        span,
                                        value: name.clone(),
                                        raw,
                                    }),
                                    value,
                                },
                            ))));
                        }
                    }
                } else {
                    defined.push(PropOrSpread::Prop(prop));
                }
            } else {
                defined.push(prop_or_spread);
            }
            defined
        },
    )
}

pub(crate) fn decouple_v_models(
    elems: Vec<Option<ExprOrSpread>>,
) -> impl Iterator<Item = JSXAttrOrSpread> {
    elems
        .into_iter()
        .filter_map(|elem| match elem {
            Some(ExprOrSpread { spread: None, expr }) => expr.array(),
            _ => None,
        })
        .map(|ArrayLit { mut elems, .. }| {
            let argument = elems
                .get(1)
                .and_then(|elem| {
                    if let Some(ExprOrSpread { spread: None, expr }) = elem {
                        expr.as_lit()
                    } else {
                        None
                    }
                })
                .and_then(|lit| {
                    if let Lit::Str(Str { value, .. }) = lit {
                        Some(value.clone())
                    } else {
                        None
                    }
                });
            if argument.is_some() {
                elems.remove(1);
            }
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: if let Some(argument) = argument {
                    JSXAttrName::JSXNamespacedName(JSXNamespacedName {
                        span: DUMMY_SP,
                        ns: quote_ident!("v-model"),
                        name: IdentName {
                            span: DUMMY_SP,
                            sym: argument.to_atom_lossy().into_owned(),
                        },
                    })
                } else {
                    JSXAttrName::Ident(quote_ident!("v-model"))
                },
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Array(ArrayLit {
                        span: DUMMY_SP,
                        elems,
                    }))),
                })),
            })
        })
}

pub(crate) fn transform_text(text: &str) -> String {
    let jsx_text_value = text.replace('\t', " ");
    let mut jsx_text_lines = jsx_text_value.lines().enumerate().peekable();

    let mut lines = vec![];
    while let Some((index, line)) = jsx_text_lines.next() {
        let line = if index == 0 {
            // first line
            line.trim_end()
        } else if jsx_text_lines.peek().is_none() {
            // last line
            line.trim_start()
        } else {
            line.trim()
        };
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines.join(" ")
}
