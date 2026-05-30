use std::{collections::BTreeSet, str::Split};
use farmfe_core::swc_common::DUMMY_SP;
use farmfe_core::swc_ecma_ast::*;
use farmfe_toolkit::swc_atoms::Atom;
use farmfe_toolkit::swc_ecma_utils::{quote_ident, quote_str};

pub(crate) fn is_directive(jsx_attr: &JSXAttr) -> bool {
    let name = match &jsx_attr.name {
        JSXAttrName::Ident(ident) => &ident.sym,
        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, .. }) => &ns.sym,
    };
    matches!(name.as_bytes(), [b'v', b'-' | b'A'..=b'Z', ..])
}

pub(crate) struct NormalDirective {
    pub(crate) name: Atom,
    pub(crate) argument: Option<Expr>,
    pub(crate) modifiers: Option<Expr>,
    pub(crate) value: Expr,
}

pub(crate) struct VModelDirective {
    pub(crate) argument: Option<Expr>,
    pub(crate) transformed_argument: Option<Expr>,
    pub(crate) modifiers: Option<Expr>,
    pub(crate) value: Expr,
}

pub(crate) enum Directive {
    Normal(NormalDirective),
    Text(Expr),
    Html(Expr),
    VModel(VModelDirective),
    Slots(Option<Box<Expr>>),
}

pub(crate) fn parse_directive(
    jsx_attr: &JSXAttr,
    is_component: bool,
) -> Result<Directive, String> {
    let (name, argument, splitted) = match &jsx_attr.name {
        JSXAttrName::Ident(ident) => {
            let mut splitted = ident
                .sym
                .trim_start_matches('v')
                .trim_start_matches('-')
                .split('_');
            (
                splitted.next().unwrap_or(&*ident.sym).to_ascii_lowercase(),
                splitted.next(),
                splitted,
            )
        }
        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name, .. }) => {
            let mut splitted = name.sym.split('_');
            (
                ns.sym
                    .trim_start_matches('v')
                    .trim_start_matches('-')
                    .to_ascii_lowercase(),
                Some(splitted.next().unwrap_or(&*name.sym)),
                splitted,
            )
        }
    };

    let mut argument = argument.map(|argument| Expr::Lit(Lit::Str(quote_str!(argument))));

    match &*name {
        "html" => return parse_v_html_directive(jsx_attr),
        "text" => return parse_v_text_directive(jsx_attr),
        "model" => return parse_v_model_directive(jsx_attr, is_component, argument, splitted),
        "slots" => return parse_v_slots_directive(jsx_attr),
        _ => {}
    }

    let mut modifiers = None;
    let value;

    if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
        expr: JSXExpr::Expr(expr),
        ..
    })) = &jsx_attr.value
    {
        if let Expr::Array(ArrayLit { elems, .. }) = &**expr {
            value = match elems.first() {
                Some(Some(ExprOrSpread { spread: None, expr })) => (**expr).clone(),
                _ => Expr::Ident(quote_ident!("").into()),
            };
            if let Some(Some(ExprOrSpread { spread: None, expr })) = elems.get(1) {
                match &**expr {
                    Expr::Array(ArrayLit { elems, .. }) => {
                        modifiers = Some(parse_modifiers(elems));
                    }
                    expr => {
                        if argument.is_none() {
                            argument = Some(expr.clone());
                        }
                        if let Some(Some(ExprOrSpread { spread: None, expr })) = elems.get(2) {
                            if let Expr::Array(ArrayLit { elems, .. }) = &**expr {
                                modifiers = Some(parse_modifiers(elems));
                            }
                        }
                    }
                }
            } else {
                modifiers = Some(splitted.map(Atom::from).collect());
            }
        } else {
            modifiers = Some(splitted.map(Atom::from).collect());
            value = (**expr).clone();
        }
    } else {
        modifiers = Some(splitted.map(Atom::from).collect());
        value = Expr::Ident(quote_ident!("").into());
    }

    Ok(Directive::Normal(NormalDirective {
        name: Atom::from(name),
        argument: if modifiers
            .as_ref()
            .map(|modifiers| !modifiers.is_empty())
            .unwrap_or_default()
        {
            argument.or_else(|| {
                Some(Expr::Unary(UnaryExpr {
                    span: DUMMY_SP,
                    op: op!("void"),
                    arg: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: 0.0,
                        raw: None,
                    }))),
                }))
            })
        } else {
            argument
        },
        modifiers: modifiers.and_then(|modifiers| transform_modifiers(modifiers, false)),
        value,
    }))
}

fn parse_modifiers(exprs: &[Option<ExprOrSpread>]) -> BTreeSet<Atom> {
    exprs
        .iter()
        .filter_map(|expr| match expr {
            Some(ExprOrSpread { spread: None, expr }) => match &**expr {
                Expr::Lit(Lit::Str(Str { value, .. })) => Some(value.to_atom_lossy().into_owned()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn parse_v_text_directive(jsx_attr: &JSXAttr) -> Result<Directive, String> {
    let expr = match &jsx_attr.value {
        Some(JSXAttrValue::Str(str_lit)) => Expr::Lit(Lit::Str(str_lit.clone())),
        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        })) => {
            if let Some(Some(ExprOrSpread { spread: None, expr })) =
                expr.as_array().and_then(|array| array.elems.first())
            {
                (**expr).clone()
            } else {
                (**expr).clone()
            }
        }
        None => {
            return Err(
                "You have to use JSX Expression inside your `v-text`.".to_string(),
            );
        }
        _ => unreachable!(),
    };

    Ok(Directive::Text(expr))
}

fn parse_v_html_directive(jsx_attr: &JSXAttr) -> Result<Directive, String> {
    let expr = match &jsx_attr.value {
        Some(JSXAttrValue::Str(str_lit)) => Expr::Lit(Lit::Str(str_lit.clone())),
        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        })) => {
            if let Some(Some(ExprOrSpread { spread: None, expr })) =
                expr.as_array().and_then(|array| array.elems.first())
            {
                (**expr).clone()
            } else {
                (**expr).clone()
            }
        }
        None => {
            return Err(
                "You have to use JSX Expression inside your `v-html`.".to_string(),
            );
        }
        _ => unreachable!(),
    };

    Ok(Directive::Html(expr))
}

fn parse_v_model_directive(
    jsx_attr: &JSXAttr,
    is_component: bool,
    mut argument: Option<Expr>,
    splitted_attr_name: Split<char>,
) -> Result<Directive, String> {
    let attr_value = match &jsx_attr.value {
        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        })) => (**expr).clone(),
        _ => {
            return Err(
                "You have to use JSX Expression inside your `v-model`.".to_string(),
            );
        }
    };

    let mut modifiers = None;
    let value;

    if let Expr::Array(ArrayLit { elems, .. }) = attr_value {
        value = match elems.first() {
            Some(Some(ExprOrSpread { spread: None, expr })) => (**expr).clone(),
            _ => Expr::Ident(quote_ident!("").into()),
        };
        if let Some(Some(ExprOrSpread { spread: None, expr })) = elems.get(1) {
            match &**expr {
                Expr::Array(ArrayLit { elems, .. }) => {
                    if is_component && argument.is_none() {
                        argument = Some(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })));
                    }
                    modifiers = Some(parse_modifiers(elems));
                }
                expr => {
                    if argument.is_none() {
                        argument = Some(expr.clone());
                    }
                    if let Some(Some(ExprOrSpread { spread: None, expr })) = elems.get(2) {
                        if let Expr::Array(ArrayLit { elems, .. }) = &**expr {
                            modifiers = Some(parse_modifiers(elems));
                        }
                    }
                }
            }
        } else {
            if is_component && argument.is_none() {
                argument = Some(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })));
            }
            modifiers = Some(splitted_attr_name.map(Atom::from).collect());
        }
    } else {
        modifiers = Some(splitted_attr_name.map(Atom::from).collect());
        value = attr_value.clone();
    }

    Ok(Directive::VModel(VModelDirective {
        argument: argument.clone(),
        transformed_argument: if !is_component
            && modifiers
                .as_ref()
                .map(|modifiers| !modifiers.is_empty())
                .unwrap_or_default()
        {
            argument.or_else(|| {
                Some(Expr::Unary(UnaryExpr {
                    span: DUMMY_SP,
                    op: op!("void"),
                    arg: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: 0.0,
                        raw: None,
                    }))),
                }))
            })
        } else {
            argument
        },
        modifiers: modifiers.and_then(|modifiers| transform_modifiers(modifiers, is_component)),
        value,
    }))
}

fn transform_modifiers(modifiers: BTreeSet<Atom>, quote_prop: bool) -> Option<Expr> {
    if modifiers.is_empty() {
        None
    } else {
        Some(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: modifiers
                .into_iter()
                .map(|modifier| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: if quote_prop {
                            PropName::Str(quote_str!(modifier))
                        } else {
                            PropName::Ident(quote_ident!(modifier))
                        },
                        value: Box::new(Expr::Lit(Lit::Bool(Bool {
                            span: DUMMY_SP,
                            value: true,
                        }))),
                    })))
                })
                .collect(),
        }))
    }
}

fn parse_v_slots_directive(jsx_attr: &JSXAttr) -> Result<Directive, String> {
    let expr = match &jsx_attr.value {
        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        })) => match &**expr {
            Expr::Ident(..) | Expr::Object(..) => Some(expr.clone()),
            _ => None,
        },
        _ => None,
    };
    Ok(Directive::Slots(expr))
}
