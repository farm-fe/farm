#![allow(clippy::all)]

use std::sync::Arc;
use std::{borrow::Cow, collections::BTreeMap, mem};

use crate::directive::{is_directive, parse_directive, Directive, NormalDirective};
use crate::options::Options;
use crate::patch_flags::PatchFlags;
use crate::slot_flag::SlotFlag;
use fxhash::FxHashMap;
use indexmap::IndexSet;

use farmfe_core::serde_json;
use farmfe_core::swc_common::{comments::Comments, Globals, Mark, Span, Spanned, SyntaxContext, DUMMY_SP, GLOBALS};
use farmfe_core::swc_ecma_ast::*;
use farmfe_toolkit::swc_atoms::Atom;
use farmfe_toolkit::swc_ecma_utils::{private_ident, quote_ident, quote_str};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

pub mod directive;
pub mod options;
pub mod patch_flags;
pub mod resolve_type;
pub mod slot_flag;
pub mod util;

type FnvHashMap<K, V> = FxHashMap<K, V>;

const FRAGMENT: &str = "Fragment";
const KEEP_ALIVE: &str = "KeepAlive";

struct AttrsTransformationResult<'a> {
    attrs: Expr,
    patch_flags: PatchFlags,
    dynamic_props: Option<IndexSet<Cow<'a, str>>>,
    slots: Option<Box<Expr>>,
}

pub struct VueJsxTransformVisitor<C>
where
    C: Comments,
{
    pub(crate) options: Options,
    pub(crate) vue_imports: BTreeMap<&'static str, Ident>,
    pub(crate) transform_on_helper: Option<Ident>,

    pub(crate) define_component: Option<SyntaxContext>,
    pub(crate) interfaces: FnvHashMap<(Atom, SyntaxContext), TsInterfaceDecl>,
    pub(crate) type_aliases: FnvHashMap<(Atom, SyntaxContext), TsType>,

    pub(crate) unresolved_mark: Mark,
    pub(crate) comments: Option<C>,

    pub(crate) pragma: Option<String>,
    pub(crate) slot_helper_ident: Option<Ident>,
    pub(crate) injecting_vars: Vec<VarDeclarator>,
    pub(crate) slot_counter: usize,
    pub(crate) slot_flag_stack: Vec<SlotFlag>,

    pub(crate) assignment_left: Option<Ident>,
    pub(crate) injecting_consts: Vec<VarDeclarator>,
    pub(crate) resolved_path: String,
}

impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    pub fn new(options: Options, unresolved_mark: Mark, comments: Option<C>, resolved_path: String) -> Self {
        Self {
            options,
            vue_imports: Default::default(),
            transform_on_helper: None,

            define_component: None,
            interfaces: Default::default(),
            type_aliases: Default::default(),

            unresolved_mark,
            comments,

            pragma: None,
            slot_helper_ident: None,
            injecting_vars: Default::default(),
            slot_counter: 1,
            slot_flag_stack: Default::default(),

            assignment_left: None,
            injecting_consts: Default::default(),
            resolved_path,
        }
    }

    fn import_from_vue(&mut self, item: &'static str) -> Ident {
        self.vue_imports
            .entry(item)
            .or_insert_with_key(|name| private_ident!(format!("_{name}")))
            .clone()
    }

    fn generate_slot_helper(&mut self) -> Ident {
        self.slot_helper_ident
            .get_or_insert_with(|| private_ident!("_isSlot"))
            .clone()
    }

    fn generate_unique_slot_ident(&mut self) -> Ident {
        let ident = if self.slot_counter == 1 {
            private_ident!("_slot")
        } else {
            private_ident!(format!("_slot{}", self.slot_counter))
        };
        self.injecting_vars.push(VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: ident.clone(),
                type_ann: None,
            }),
            init: None,
            definite: false,
        });

        self.slot_counter += 1;
        ident
    }

    fn get_pragma(&mut self) -> Ident {
        self.pragma
            .as_ref()
            .or(self.options.pragma.as_ref())
            .map(|name| quote_ident!(name.as_str()).into())
            .unwrap_or_else(|| self.import_from_vue("createVNode"))
    }

    fn search_jsx_pragma(&mut self, span: Span) {
        if let Some(comments) = &self.comments {
            comments.with_leading(span.lo, |comments| {
                let pragma = comments.iter().find_map(|comment| {
                    let trimmed = comment.text.trim();
                    trimmed
                        .strip_prefix('*')
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix("@jsx")
                        .map(str::trim)
                });
                if let Some(pragma) = pragma {
                    self.pragma = Some(pragma.to_string());
                }
            });
        }
    }

    fn is_component(&self, element_name: &JSXElementName) -> bool {
        let name = match element_name {
            JSXElementName::Ident(Ident { sym, .. }) => sym,
            JSXElementName::JSXMemberExpr(JSXMemberExpr { prop, .. }) => &*prop.sym,
            JSXElementName::JSXNamespacedName(JSXNamespacedName { name, .. }) => &*name.sym,
        };
        let should_transformed_to_slots = !self
            .vue_imports
            .get(FRAGMENT)
            .map(|ident| &*ident.sym == name)
            .unwrap_or_default()
            && name != KEEP_ALIVE;

        if matches!(element_name, JSXElementName::JSXMemberExpr(..)) {
            should_transformed_to_slots
        } else {
            self.options
                .custom_element_patterns
                .iter()
                .all(|pattern| !pattern.is_match(name))
                && should_transformed_to_slots
                && !(name.as_bytes()[0].is_ascii_lowercase()
                    && (css_dataset::tags::STANDARD_HTML_TAGS.contains(name)
                        || css_dataset::tags::SVG_TAGS.contains(name)))
        }
    }

    fn is_define_component_call(&self, CallExpr { callee, .. }: &CallExpr) -> bool {
        callee
            .as_expr()
            .and_then(|expr| expr.as_ident())
            .and_then(|ident| {
                self.define_component
                    .map(|ctxt| ctxt == ident.ctxt && ident.sym == "defineComponent")
            })
            .unwrap_or_default()
    }

    fn build_iife(&mut self, elems: Vec<Option<ExprOrSpread>>) -> Vec<Option<ExprOrSpread>> {
        let left = self.assignment_left.take();
        if let Some(left) = left {
            elems
                .into_iter()
                .map(|elem| match elem {
                    Some(ExprOrSpread { spread: None, expr }) => match *expr {
                        Expr::Ident(ident) if ident.sym == left.sym => {
                            let name = private_ident!(format!("_{}", ident.sym));
                            self.injecting_consts.push(VarDeclarator {
                                span: DUMMY_SP,
                                name: Pat::Ident(BindingIdent {
                                    id: name.clone(),
                                    type_ann: None,
                                }),
                                init: Some(Box::new(Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Callee::Expr(Box::new(Expr::Fn(FnExpr {
                                        ident: None,
                                        function: Box::new(Function {
                                            params: vec![],
                                            decorators: vec![],
                                            span: DUMMY_SP,
                                            body: Some(BlockStmt {
                                                span: DUMMY_SP,
                                                stmts: vec![Stmt::Return(ReturnStmt {
                                                    span: DUMMY_SP,
                                                    arg: Some(Box::new(Expr::Ident(ident))),
                                                })],
                                                ..Default::default()
                                            }),
                                            is_generator: false,
                                            is_async: false,
                                            ..Default::default()
                                        }),
                                    }))),
                                    args: vec![],
                                    ..Default::default()
                                }))),
                                definite: false,
                            });
                            Some(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Ident(name)),
                            })
                        }
                        expr => Some(ExprOrSpread {
                            spread: None,
                            expr: Box::new(expr),
                        }),
                    },
                    _ => elem,
                })
                .collect()
        } else {
            elems
        }
    }
}


impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    fn transform_jsx_element(&mut self, jsx_element: &JSXElement) -> Expr {
        if self.options.optimize {
            self.slot_flag_stack.push(SlotFlag::Stable);
        }

        let is_component = self.is_component(&jsx_element.opening.name);
        let mut directives = vec![];
        let AttrsTransformationResult {
            attrs,
            patch_flags,
            dynamic_props,
            slots,
        } = self.transform_attrs(&jsx_element.opening.attrs, is_component, &mut directives);
        let mut vnode_call_args = vec![
            ExprOrSpread {
                spread: None,
                expr: Box::new(self.transform_tag(&jsx_element.opening.name)),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(attrs),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(self.transform_children(&jsx_element.children, is_component, slots)),
            },
        ];
        if self.options.optimize {
            if !patch_flags.is_empty() {
                vnode_call_args.push(ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: patch_flags.bits() as f64,
                        raw: None,
                    }))),
                });
            }
            match dynamic_props {
                Some(dynamic_props) if !dynamic_props.is_empty() => {
                    vnode_call_args.push(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Array(ArrayLit {
                            span: DUMMY_SP,
                            elems: dynamic_props
                                .into_iter()
                                .map(|prop| {
                                    Some(ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Lit(Lit::Str(quote_str!(prop)))),
                                    })
                                })
                                .collect(),
                        })),
                    })
                }
                _ => {}
            }
        }

        let create_vnode_call = Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident(self.get_pragma()))),
            args: vnode_call_args,
            ..Default::default()
        });

        if directives.is_empty() {
            create_vnode_call
        } else {
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.import_from_vue("withDirectives"),
                ))),
                args: vec![
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(create_vnode_call),
                    },
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Array(ArrayLit {
                            span: DUMMY_SP,
                            elems: directives
                                .into_iter()
                                .map(|directive| {
                                    let mut elems =
                                        vec![
                                            Some(ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(self.resolve_directive(
                                                    &directive.name,
                                                    jsx_element,
                                                )),
                                            }),
                                            Some(ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(directive.value),
                                            }),
                                        ];
                                    if let Some(argument) = directive.argument {
                                        elems.push(Some(ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(argument),
                                        }));
                                    }
                                    if let Some(modifiers) = directive.modifiers {
                                        elems.push(Some(ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(modifiers),
                                        }));
                                    }
                                    Some(ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Array(ArrayLit {
                                            span: DUMMY_SP,
                                            elems,
                                        })),
                                    })
                                })
                                .collect(),
                        })),
                    },
                ],
                ..Default::default()
            })
        }
    }

    fn transform_jsx_fragment(&mut self, jsx_fragment: &JSXFragment) -> Expr {
        if self.options.optimize {
            self.slot_flag_stack.push(SlotFlag::Stable);
        }

        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident(self.get_pragma()))),
            args: vec![
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(self.import_from_vue(FRAGMENT))),
                },
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
                },
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(self.transform_children(&jsx_fragment.children, false, None)),
                },
            ],
            ..Default::default()
        })
    }
}


impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    fn transform_tag(&mut self, jsx_element_name: &JSXElementName) -> Expr {
        match jsx_element_name {
            JSXElementName::Ident(ident) => {
                let name = &*ident.sym;
                if name.as_bytes()[0].is_ascii_lowercase()
                    && (css_dataset::tags::STANDARD_HTML_TAGS.contains(name)
                        || css_dataset::tags::SVG_TAGS.contains(name))
                {
                    Expr::Lit(Lit::Str(quote_str!(name)))
                } else if name == FRAGMENT {
                    Expr::Ident(self.import_from_vue(FRAGMENT))
                } else if self
                    .options
                    .custom_element_patterns
                    .iter()
                    .any(|pattern| pattern.is_match(name))
                {
                    Expr::Lit(Lit::Str(quote_str!(name)))
                } else if ident.to_id().1.has_mark(self.unresolved_mark) {
                    // for components that can't be resolved from current file
                    Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(
                            self.import_from_vue("resolveComponent"),
                        ))),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Str(quote_str!(name)))),
                        }],
                        ..Default::default()
                    })
                } else {
                    Expr::Ident(ident.clone())
                }
            }
            JSXElementName::JSXMemberExpr(expr) => Expr::JSXMember(expr.clone()),
            JSXElementName::JSXNamespacedName(name) => Expr::JSXNamespacedName(name.clone()),
        }
    }

    fn resolve_directive(&mut self, directive_name: &str, jsx_element: &JSXElement) -> Expr {
        match directive_name {
            "show" => Expr::Ident(self.import_from_vue("vShow")),
            "model" => match &jsx_element.opening.name {
                JSXElementName::Ident(ident) if &ident.sym == "select" => {
                    Expr::Ident(self.import_from_vue("vModelSelect"))
                }
                JSXElementName::Ident(ident) if &ident.sym == "textarea" => {
                    Expr::Ident(self.import_from_vue("vModelText"))
                }
                _ => {
                    let typ = jsx_element
                        .opening
                        .attrs
                        .iter()
                        .find_map(|jsx_attr_or_spread| match jsx_attr_or_spread {
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                name: JSXAttrName::Ident(ident),
                                value,
                                ..
                            }) if &ident.sym == "type" => value.as_ref(),
                            _ => None,
                        });
                    match typ {
                        Some(JSXAttrValue::Str(str)) if str.value.as_str() == Some("checkbox") => {
                            Expr::Ident(self.import_from_vue("vModelCheckbox"))
                        }
                        Some(JSXAttrValue::Str(str)) if str.value.as_str() == Some("radio") => {
                            Expr::Ident(self.import_from_vue("vModelRadio"))
                        }
                        Some(JSXAttrValue::Str(..)) | None => {
                            Expr::Ident(self.import_from_vue("vModelText"))
                        }
                        Some(..) => Expr::Ident(self.import_from_vue("vModelDynamic")),
                    }
                }
            },
            _ => Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.import_from_vue("resolveDirective"),
                ))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(quote_str!(directive_name)))),
                }],
                ..Default::default()
            }),
        }
    }
}


impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    fn transform_attrs<'a>(
        &mut self,
        attrs: &'a [JSXAttrOrSpread],
        is_component: bool,
        directives: &mut Vec<NormalDirective>,
    ) -> AttrsTransformationResult<'a> {
        let mut slots = None;

        if attrs.is_empty() {
            return AttrsTransformationResult {
                attrs: Expr::Lit(Lit::Null(Null { span: DUMMY_SP })),
                patch_flags: PatchFlags::empty(),
                dynamic_props: None,
                slots,
            };
        }

        let mut dynamic_props = IndexSet::new();

        // patch flags analysis
        let mut has_ref = false;
        let mut has_class_binding = false;
        let mut has_style_binding = false;
        let mut has_hydration_event_binding = false;
        let mut has_dynamic_keys = false;

        let (mut props, mut merge_args) = attrs.iter().fold(
            (
                Vec::with_capacity(attrs.len()),
                Vec::with_capacity(attrs.len()),
            ),
            |(mut props, mut merge_args), jsx_attr_or_spread| {
                match jsx_attr_or_spread {
                    JSXAttrOrSpread::JSXAttr(jsx_attr) if is_directive(jsx_attr) => {
                        match parse_directive(jsx_attr, is_component) {
                            Ok(Directive::Normal(directive)) => directives.push(directive),
                            Ok(Directive::Html(expr)) => {
                                props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                    KeyValueProp {
                                        key: PropName::Str(quote_str!("innerHTML")),
                                        value: Box::new(expr),
                                    },
                                ))));
                                dynamic_props.insert("innerHTML".into());
                            }
                            Ok(Directive::Text(expr)) => {
                                props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                    KeyValueProp {
                                        key: PropName::Str(quote_str!("textContent")),
                                        value: Box::new(expr),
                                    },
                                ))));
                                dynamic_props.insert("textContent".into());
                            }
                            Ok(Directive::VModel(directive)) => {
                                if is_component {
                                    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                        KeyValueProp {
                                            key: match &directive.argument {
                                                Some(Expr::Lit(Lit::Null(..))) | None => {
                                                    dynamic_props.insert("modelValue".into());
                                                    PropName::Str(quote_str!("modelValue"))
                                                }
                                                Some(Expr::Lit(Lit::Str(Str {
                                                    value,
                                                    ..
                                                }))) => {
                                                    dynamic_props
                                                        .insert(Cow::from(value.as_str().unwrap_or("").to_string()));
                                                    PropName::Str(quote_str!(&**value))
                                                }
                                                Some(expr) => {
                                                    PropName::Computed(ComputedPropName {
                                                        span: DUMMY_SP,
                                                        expr: Box::new(expr.clone()),
                                                    })
                                                }
                                            },
                                            value: Box::new(directive.value.clone()),
                                        },
                                    ))));
                                    if let Some(modifiers) = directive.modifiers {
                                        props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                            KeyValueProp {
                                                key: match &directive.argument {
                                                    Some(Expr::Lit(Lit::Null(..))) | None => {
                                                        PropName::Str(quote_str!("modelModifiers"))
                                                    }
                                                    Some(Expr::Lit(Lit::Str(Str {
                                                        value,
                                                        ..
                                                    }))) => PropName::Str(quote_str!(format!(
                                                        "{}Modifiers", value.as_str().unwrap_or("")
                                                    ))),
                                                    Some(expr) => {
                                                        PropName::Computed(ComputedPropName {
                                                            span: DUMMY_SP,
                                                            expr: Box::new(Expr::Bin(BinExpr {
                                                                span: DUMMY_SP,
                                                                op: op!(bin, "+"),
                                                                left: Box::new(expr.clone()),
                                                                right: Box::new(Expr::Lit(
                                                                    Lit::Str(quote_str!(
                                                                        "Modifiers"
                                                                    )),
                                                                )),
                                                            })),
                                                        })
                                                    }
                                                },
                                                value: Box::new(modifiers),
                                            },
                                        ))))
                                    }
                                } else {
                                    directives.push(NormalDirective {
                                        name: Atom::from("model"),
                                        argument: directive.transformed_argument,
                                        modifiers: directive.modifiers.clone(),
                                        value: directive.value.clone(),
                                    });
                                }

                                props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                    KeyValueProp {
                                        key: match directive.argument {
                                            Some(Expr::Lit(Lit::Null(..))) | None => {
                                                dynamic_props.insert("onUpdate:modelValue".into());
                                                PropName::Str(quote_str!("onUpdate:modelValue"))
                                            }
                                            Some(Expr::Lit(Lit::Str(Str { value, .. }))) => {
                                                let name = format!("onUpdate:{}", value.as_str().unwrap_or(""));
                                                let prop_name = PropName::Str(quote_str!(&*name));
                                                dynamic_props.insert(name.into());
                                                prop_name
                                            }
                                            Some(expr) => {
                                                has_dynamic_keys = true;
                                                PropName::Computed(ComputedPropName {
                                                    span: DUMMY_SP,
                                                    expr: Box::new(Expr::Bin(BinExpr {
                                                        span: DUMMY_SP,
                                                        op: op!(bin, "+"),
                                                        left: Box::new(Expr::Lit(Lit::Str(
                                                            quote_str!("onUpdate"),
                                                        ))),
                                                        right: Box::new(expr),
                                                    })),
                                                })
                                            }
                                        },
                                        value: Box::new(Expr::Arrow(ArrowExpr {
                                            span: DUMMY_SP,
                                            params: vec![Pat::Ident(BindingIdent {
                                                id: quote_ident!("$event").into(),
                                                type_ann: None,
                                            })],
                                            body: Box::new(BlockStmtOrExpr::Expr(Box::new(
                                                Expr::Assign(AssignExpr {
                                                    span: DUMMY_SP,
                                                    op: op!("="),
                                                    left: AssignTarget::Simple(
                                                        SimpleAssignTarget::Paren(ParenExpr {
                                                            span: DUMMY_SP,
                                                            expr: Box::new(directive.value),
                                                        }),
                                                    ),
                                                    right: Box::new(Expr::Ident(
                                                        quote_ident!("$event").into(),
                                                    )),
                                                }),
                                            ))),
                                            is_async: false,
                                            is_generator: false,
                                            ..Default::default()
                                        })),
                                    },
                                ))));
                            }
                            Ok(Directive::Slots(expr)) => slots = expr,
                            Err(_) => {}
                        }
                    }
                    JSXAttrOrSpread::JSXAttr(jsx_attr) => {
                        let attr_name = match &jsx_attr.name {
                            JSXAttrName::Ident(ident) => Cow::from(&*ident.sym),
                            JSXAttrName::JSXNamespacedName(name) => {
                                Cow::from(format!("{}:{}", name.ns.sym, name.name.sym))
                            }
                        };
                        let attr_value = jsx_attr
                            .value
                            .as_ref()
                            .map(|value| match value {
                                JSXAttrValue::Str(str) => Box::new(Expr::Lit(Lit::Str(
                                    quote_str!(util::transform_text(&format!("{}", str.value.as_str().unwrap_or("")))),
                                ))),
                                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    expr: JSXExpr::Expr(expr),
                                    ..
                                }) => expr.clone(),
                                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    expr: JSXExpr::JSXEmptyExpr(expr),
                                    ..
                                }) => Box::new(Expr::JSXEmpty(*expr)),
                                JSXAttrValue::JSXElement(element) => {
                                    Box::new(Expr::JSXElement(element.clone()))
                                }
                                JSXAttrValue::JSXFragment(fragment) => {
                                    Box::new(Expr::JSXFragment(fragment.clone()))
                                }
                            })
                            .unwrap_or_else(|| {
                                Box::new(Expr::Lit(Lit::Bool(Bool {
                                    span: DUMMY_SP,
                                    value: true,
                                })))
                            });

                        if attr_name == "ref" {
                            has_ref = true;
                        } else if !jsx_attr
                            .value
                            .as_ref()
                            .map(util::is_jsx_attr_value_constant)
                            .unwrap_or_default()
                        {
                            if !is_component && util::is_on(&attr_name)
                                // omit the flag for click handlers becaues hydration gives click
                                // dedicated fast path.
                                && !attr_name.eq_ignore_ascii_case("onclick")
                                // omit v-model handlers
                                && attr_name != "onUpdate:modelValue"
                            {
                                has_hydration_event_binding = true;
                            }
                            match &*attr_name {
                                "class" if !is_component => has_class_binding = true,
                                "style" if !is_component => has_style_binding = true,
                                "key" | "on" | "ref" => {}
                                _ => {
                                    dynamic_props.insert(attr_name.clone());
                                }
                            }
                        }

                        if self.options.transform_on
                            && (attr_name == "on" || attr_name == "nativeOn")
                        {
                            merge_args.push(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Callee::Expr(Box::new(Expr::Ident(
                                    self.transform_on_helper
                                        .get_or_insert_with(|| private_ident!("_transformOn"))
                                        .clone(),
                                ))),
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: attr_value,
                                }],
                                ..Default::default()
                            }));
                        } else {
                            props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
                                    key: PropName::Str(quote_str!(attr_name)),
                                    value: attr_value,
                                },
                            ))));
                        }
                    }
                    JSXAttrOrSpread::SpreadElement(spread) => {
                        has_dynamic_keys = true;

                        if !props.is_empty() && self.options.merge_props {
                            merge_args.push(Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: util::dedupe_props(mem::take(&mut props)),
                            }));
                        }

                        if let Expr::Object(object) = &*spread.expr {
                            if self.options.merge_props {
                                merge_args.push(Expr::Object(object.clone()));
                            } else {
                                props.extend_from_slice(&object.props);
                            }
                        } else if self.options.merge_props {
                            merge_args.push(*spread.expr.clone());
                        } else {
                            props.push(PropOrSpread::Spread(spread.clone()));
                        }
                    }
                }
                (props, merge_args)
            },
        );

        let expr = if !merge_args.is_empty() {
            if !props.is_empty() {
                merge_args.push(Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: if self.options.merge_props {
                        util::dedupe_props(mem::take(&mut props))
                    } else {
                        mem::take(&mut props)
                    },
                }));
            }
            match merge_args.as_slice() {
                [expr] => expr.clone(),
                _ => Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(Expr::Ident(self.import_from_vue("mergeProps")))),
                    args: merge_args
                        .into_iter()
                        .map(|expr| ExprOrSpread {
                            spread: None,
                            expr: Box::new(expr),
                        })
                        .collect(),
                    ..Default::default()
                }),
            }
        } else if !props.is_empty() {
            if let [PropOrSpread::Spread(SpreadElement { expr, .. })] = props.as_slice() {
                *expr.clone()
            } else {
                Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: if self.options.merge_props {
                        util::dedupe_props(props)
                    } else {
                        props
                    },
                })
            }
        } else {
            Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))
        };

        let mut patch_flags = PatchFlags::empty();
        if has_dynamic_keys {
            patch_flags.insert(PatchFlags::FULL_PROPS);
        } else {
            if has_class_binding {
                patch_flags.insert(PatchFlags::CLASS);
            }
            if has_style_binding {
                patch_flags.insert(PatchFlags::STYLE);
            }
            if !dynamic_props.is_empty() {
                patch_flags.insert(PatchFlags::PROPS);
            }
            if has_hydration_event_binding {
                patch_flags.insert(PatchFlags::HYDRATE_EVENTS);
            }
        }
        if (patch_flags.is_empty() || patch_flags == PatchFlags::HYDRATE_EVENTS)
            && (has_ref || !directives.is_empty())
        {
            patch_flags.insert(PatchFlags::NEED_PATCH);
        }

        AttrsTransformationResult {
            attrs: expr,
            patch_flags,
            dynamic_props: Some(dynamic_props),
            slots,
        }
    }
}


impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    fn transform_children(
        &mut self,
        children: &[JSXElementChild],
        is_component: bool,
        slots: Option<Box<Expr>>,
    ) -> Expr {
        let elems = children
            .iter()
            .filter_map(|child| match child {
                JSXElementChild::JSXText(jsx_text) => {
                    self.transform_jsx_text(jsx_text).map(|expr| ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    })
                }
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::JSXEmptyExpr(..),
                    ..
                }) => None,
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(expr),
                    ..
                }) => {
                    if self.options.optimize {
                        match &**expr {
                            Expr::Ident(ident)
                                if !ident.to_id().1.has_mark(self.unresolved_mark) =>
                            {
                                self.slot_flag_stack.fill(SlotFlag::Dynamic);
                            }
                            _ => {}
                        }
                    }
                    Some(ExprOrSpread {
                        spread: None,
                        expr: expr.clone(),
                    })
                }
                JSXElementChild::JSXSpreadChild(JSXSpreadChild { expr, .. }) => {
                    if self.options.optimize {
                        match &**expr {
                            Expr::Ident(ident)
                                if !ident.to_id().1.has_mark(self.unresolved_mark) =>
                            {
                                self.slot_flag_stack.fill(SlotFlag::Dynamic);
                            }
                            _ => {}
                        }
                    }
                    Some(ExprOrSpread {
                        spread: Some(DUMMY_SP),
                        expr: expr.clone(),
                    })
                }
                JSXElementChild::JSXElement(jsx_element) => Some(ExprOrSpread {
                    spread: None,
                    expr: Box::new(self.transform_jsx_element(jsx_element)),
                }),
                JSXElementChild::JSXFragment(jsx_fragment) => Some(ExprOrSpread {
                    spread: None,
                    expr: Box::new(self.transform_jsx_fragment(jsx_fragment)),
                }),
            })
            .map(Some)
            .collect::<Vec<_>>();

        let slot_flag = if self.options.optimize {
            self.slot_flag_stack.pop().unwrap_or(SlotFlag::Stable)
        } else {
            SlotFlag::Stable
        };

        match elems.as_slice() {
            [] => {
                if let Some(slots) = slots {
                    *slots
                } else {
                    Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))
                }
            }
            [Some(ExprOrSpread { spread: None, expr })] => match &**expr {
                expr @ Expr::Ident(..) if is_component => {
                    let elems = self.build_iife(elems.clone());
                    if self.options.enable_object_slots {
                        Expr::Cond(CondExpr {
                            span: DUMMY_SP,
                            test: Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Callee::Expr(Box::new(Expr::Ident(
                                    self.generate_slot_helper(),
                                ))),
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(expr.clone()),
                                }],
                                ..Default::default()
                            })),
                            cons: Box::new(expr.clone()),
                            alt: Box::new(self.wrap_children(elems, slot_flag, slots)),
                        })
                    } else {
                        self.wrap_children(elems, slot_flag, slots)
                    }
                }
                expr @ Expr::Call(..) if expr.span() != DUMMY_SP && is_component => {
                    // the element was generated and doesn't have location information
                    if self.options.enable_object_slots {
                        let slot_ident = self.generate_unique_slot_ident();
                        Expr::Cond(CondExpr {
                            span: DUMMY_SP,
                            test: Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Callee::Expr(Box::new(Expr::Ident(
                                    self.generate_slot_helper(),
                                ))),
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Assign(AssignExpr {
                                        span: DUMMY_SP,
                                        op: op!("="),
                                        left: AssignTarget::Simple(SimpleAssignTarget::Paren(
                                            ParenExpr {
                                                span: DUMMY_SP,
                                                expr: Box::new(Expr::Ident(slot_ident.clone())),
                                            },
                                        )),
                                        right: Box::new(expr.clone()),
                                    })),
                                }],
                                ..Default::default()
                            })),
                            cons: Box::new(Expr::Ident(slot_ident.clone())),
                            alt: {
                                let elems = self.build_iife(vec![Some(ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(slot_ident)),
                                })]);
                                Box::new(self.wrap_children(elems, slot_flag, slots))
                            },
                        })
                    } else {
                        self.wrap_children(elems, slot_flag, slots)
                    }
                }
                expr @ Expr::Fn(..) | expr @ Expr::Arrow(..) => Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(quote_ident!("default")),
                        value: Box::new(expr.clone()),
                    })))],
                }),
                Expr::Object(ObjectLit { props, .. }) => {
                    let mut props = props.clone();
                    if self.options.optimize {
                        props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("_")),
                            value: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: slot_flag as u8 as f64,
                                raw: None,
                            }))),
                        }))));
                    }
                    Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props,
                    })
                }
                _ => {
                    if is_component {
                        self.wrap_children(elems, slot_flag, slots)
                    } else {
                        Expr::Array(ArrayLit {
                            span: DUMMY_SP,
                            elems,
                        })
                    }
                }
            },
            _ => {
                if is_component {
                    self.wrap_children(elems, slot_flag, slots)
                } else {
                    Expr::Array(ArrayLit {
                        span: DUMMY_SP,
                        elems,
                    })
                }
            }
        }
    }

    fn wrap_children(
        &self,
        elems: Vec<Option<ExprOrSpread>>,
        slot_flag: SlotFlag,
        slots: Option<Box<Expr>>,
    ) -> Expr {
        let mut props = vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(quote_ident!("default")),
            value: Box::new(Expr::Arrow(ArrowExpr {
                span: DUMMY_SP,
                params: vec![],
                body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems,
                })))),
                is_async: false,
                is_generator: false,
                ..Default::default()
            })),
        })))];

        if let Some(expr) = slots {
            match *expr {
                Expr::Object(ObjectLit {
                    props: slot_props, ..
                }) => props.extend_from_slice(&slot_props),
                _ => props.push(PropOrSpread::Spread(SpreadElement {
                    dot3_token: DUMMY_SP,
                    expr,
                })),
            }
        }

        if self.options.optimize {
            props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("_")),
                value: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: slot_flag as u8 as f64,
                    raw: None,
                }))),
            }))));
        }

        Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
        })
    }

    fn transform_jsx_text(&mut self, jsx_text: &JSXText) -> Option<Expr> {
        let text = util::transform_text(&jsx_text.value);
        if text.is_empty() {
            None
        } else {
            Some(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.import_from_vue("createTextVNode"),
                ))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(quote_str!(text)))),
                }],
                ..Default::default()
            }))
        }
    }
}


impl<C> VisitMut for VueJsxTransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_module(&mut self, module: &mut Module) {
        self.search_jsx_pragma(module.span);
        module
            .body
            .iter()
            .for_each(|item| self.search_jsx_pragma(item.span()));

        module.visit_mut_children_with(self);

        if !self.injecting_consts.is_empty() {
            module.body.insert(
                0,
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    decls: mem::take(&mut self.injecting_consts),
                    ..Default::default()
                })))),
            );
        }

        if !self.injecting_vars.is_empty() {
            module.body.insert(
                0,
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Let,
                    decls: mem::take(&mut self.injecting_vars),
                    ..Default::default()
                })))),
            );
            self.slot_counter = 1;
        }

        if let Some(slot_helper) = &self.slot_helper_ident {
            module.body.insert(
                0,
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(util::build_slot_helper(
                    slot_helper.clone(),
                    self.import_from_vue("isVNode"),
                )))),
            )
        }

        if let Some(helper) = &self.transform_on_helper {
            module.body.insert(
                0,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: DUMMY_SP,
                        local: helper.clone(),
                    })],
                    src: Box::new(quote_str!("@vue/babel-helper-vue-transform-on")),
                    type_only: false,
                    with: None,
                    phase: Default::default(),
                })),
            )
        }

        if !self.vue_imports.is_empty() {
            module.body.insert(
                0,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers: self
                        .vue_imports
                        .iter()
                        .map(|(imported, local)| {
                            ImportSpecifier::Named(ImportNamedSpecifier {
                                span: DUMMY_SP,
                                local: local.clone(),
                                imported: Some(ModuleExportName::Ident(
                                    quote_ident!(*imported).into(),
                                )),
                                is_type_only: false,
                            })
                        })
                        .collect(),
                    src: Box::new(quote_str!("vue")),
                    type_only: false,
                    with: None,
                    phase: Default::default(),
                })),
            );
        }
    }

    fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        stmts.visit_mut_children_with(self);

        if !self.injecting_consts.is_empty() {
            stmts.insert(
                0,
                Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    decls: mem::take(&mut self.injecting_consts),
                    ..Default::default()
                }))),
            );
        }

        if !self.injecting_vars.is_empty() {
            stmts.insert(
                0,
                Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Let,
                    decls: mem::take(&mut self.injecting_vars),
                    ..Default::default()
                }))),
            );
            self.slot_counter = 1;
        }
    }

    fn visit_mut_arrow_expr(&mut self, arrow_expr: &mut ArrowExpr) {
        arrow_expr.visit_mut_children_with(self);

        if !self.injecting_consts.is_empty() || !self.injecting_vars.is_empty() {
            if let BlockStmtOrExpr::Expr(ret) = &*arrow_expr.body {
                let mut stmts = Vec::with_capacity(3);

                if !self.injecting_consts.is_empty() {
                    stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Const,
                        decls: mem::take(&mut self.injecting_consts),
                        ..Default::default()
                    }))));
                }

                if !self.injecting_vars.is_empty() {
                    stmts.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Let,
                        decls: mem::take(&mut self.injecting_vars),
                        ..Default::default()
                    }))));
                    self.slot_counter = 1;
                }

                stmts.push(Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(ret.clone()),
                }));

                arrow_expr.body = Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                    span: DUMMY_SP,
                    stmts,
                    ..Default::default()
                }));
            }
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        match expr {
            Expr::JSXElement(jsx_element) => *expr = self.transform_jsx_element(jsx_element),
            Expr::JSXFragment(jsx_fragment) => *expr = self.transform_jsx_fragment(jsx_fragment),
            Expr::Assign(AssignExpr {
                left: AssignTarget::Simple(SimpleAssignTarget::Ident(binding_ident)),
                ..
            }) => self.assignment_left = Some(binding_ident.id.clone()),
            _ => {}
        }
    }

    // decouple `v-models`
    fn visit_mut_jsx_opening_element(&mut self, jsx_opening_element: &mut JSXOpeningElement) {
        jsx_opening_element.visit_mut_children_with(self);

        let Some(index) =
            jsx_opening_element
                .attrs
                .iter()
                .enumerate()
                .find_map(|(i, jsx_attr_or_spread)| match jsx_attr_or_spread {
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        name: JSXAttrName::Ident(IdentName { sym, .. }),
                        ..
                    }) if sym == "v-models" => Some(i),
                    _ => None,
                })
        else {
            return;
        };

        let JSXAttrOrSpread::JSXAttr(JSXAttr { value, .. }) =
            jsx_opening_element.attrs.remove(index)
        else {
            unreachable!()
        };

        let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        })) = value
        else {
            eprintln!("[vue-jsx] you should pass a Two-dimensional Arrays to v-models");
            return;
        };
        let Expr::Array(ArrayLit { elems, .. }) = *expr else {
            eprintln!("[vue-jsx] you should pass a Two-dimensional Arrays to v-models");
            return;
        };

        jsx_opening_element
            .attrs
            .splice(index..index, util::decouple_v_models(elems));
    }

    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        import_decl.visit_mut_children_with(self);

        if import_decl.src.value != "vue" {
            return;
        }

        let ctxt = import_decl.specifiers.iter().find_map(|specifier| {
            if let ImportSpecifier::Named(ImportNamedSpecifier {
                local,
                imported: None,
                ..
            }) = specifier
            {
                (local.sym == "defineComponent").then_some(local.ctxt)
            } else {
                None
            }
        });
        if let Some(ctxt) = ctxt {
            self.define_component = Some(ctxt);
        }
    }

    fn visit_mut_ts_interface_decl(&mut self, ts_interface_decl: &mut TsInterfaceDecl) {
        ts_interface_decl.visit_mut_children_with(self);
        if self.options.resolve_type {
            let key = (ts_interface_decl.id.sym.clone(), ts_interface_decl.id.ctxt);
            if let Some(interface) = self.interfaces.get_mut(&key) {
                interface
                    .body
                    .body
                    .extend_from_slice(&ts_interface_decl.body.body);
            } else {
                self.interfaces.insert(key, ts_interface_decl.clone());
            }
        }
    }

    fn visit_mut_ts_type_alias_decl(&mut self, ts_type_alias_decl: &mut TsTypeAliasDecl) {
        ts_type_alias_decl.visit_mut_children_with(self);
        if self.options.resolve_type {
            self.type_aliases.insert(
                (
                    ts_type_alias_decl.id.sym.clone(),
                    ts_type_alias_decl.id.ctxt,
                ),
                (*ts_type_alias_decl.type_ann).clone(),
            );
        }
    }

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        call_expr.visit_mut_children_with(self);

        if !self.options.resolve_type {
            return;
        }

        if !self.is_define_component_call(call_expr) {
            return;
        }

        let Some(maybe_setup) = call_expr.args.first() else {
            return;
        };

        let props_types = match self.extract_props_type(maybe_setup) {
            Ok(pt) => pt,
            Err(err) => {
                eprintln!("[vue-jsx] {err}");
                return;
            }
        };
        let emits_types = match self.extract_emits_type(maybe_setup) {
            Ok(et) => et,
            Err(err) => {
                eprintln!("[vue-jsx] {err}");
                return;
            }
        };
        if let Some(prop_types) = props_types {
            inject_define_component_option(call_expr, "props", prop_types);
        }
        if let Some(emits_type) = emits_types {
            inject_define_component_option(call_expr, "emits", Expr::Array(emits_type));
        }
    }

    fn visit_mut_var_declarator(&mut self, var_declarator: &mut VarDeclarator) {
        var_declarator.visit_mut_children_with(self);

        if !self.options.resolve_type {
            return;
        }
        let Pat::Ident(name) = &var_declarator.name else {
            return;
        };
        let Some(Expr::Call(call)) = var_declarator.init.as_deref_mut() else {
            return;
        };
        if !self.is_define_component_call(call) {
            return;
        }

        inject_define_component_option(
            call,
            "name",
            Expr::Lit(Lit::Str(quote_str!(name.sym.clone()))),
        );
    }
}


fn inject_define_component_option(call: &mut CallExpr, name: &'static str, value: Expr) {
    let options = call.args.get_mut(1);
    if options
        .as_ref()
        .and_then(|options| options.spread)
        .is_some()
    {
        return;
    }

    match options.map(|options| &mut *options.expr) {
        Some(Expr::Object(object)) => {
            if !object.props.iter().any(|prop| {
                prop.as_prop()
                    .and_then(|prop| prop.as_key_value())
                    .and_then(|key_value| key_value.key.as_ident())
                    .map(|ident| ident.sym == name)
                    .unwrap_or_default()
            }) {
                object
                    .props
                    .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(quote_ident!(name)),
                        value: Box::new(value),
                    }))));
            }
        }
        Some(..) => {
            let expr = Expr::Object(ObjectLit {
                props: vec![
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(quote_ident!(name)),
                        value: Box::new(value),
                    }))),
                    PropOrSpread::Spread(SpreadElement {
                        expr: call.args.remove(1).expr,
                        dot3_token: DUMMY_SP,
                    }),
                ],
                span: DUMMY_SP,
            });
            call.args.insert(
                1,
                ExprOrSpread {
                    expr: Box::new(expr),
                    spread: None,
                },
            );
        }
        None => {
            call.args.push(ExprOrSpread {
                expr: Box::new(Expr::Object(ObjectLit {
                    props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(quote_ident!(name)),
                        value: Box::new(value),
                    })))],
                    span: DUMMY_SP,
                })),
                spread: None,
            });
        }
    }
}

use farmfe_core::{
    config::Config,
    context::CompilationContext,
    error::CompilationError,
    module::ModuleType,
    plugin::{Plugin, PluginTransformHookParam, PluginTransformHookResult},
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
    script::{codegen_module, create_codegen_config, parse_module},
    swc_ecma_parser::{Syntax, TsSyntax},
};

#[farm_plugin]
pub struct FarmPluginVueJsx {
    options_json: String,
}

impl FarmPluginVueJsx {
    pub fn new(_config: &Config, options: String) -> Self {
        let _opts: Options = if options.trim().is_empty() {
            Options::default()
        } else {
            serde_json::from_str(&options).unwrap_or_default()
        };
        Self {
            options_json: options,
        }
    }

    fn get_options(&self) -> Options {
        if self.options_json.trim().is_empty() {
            Options::default()
        } else {
            serde_json::from_str(&self.options_json).unwrap_or_default()
        }
    }
}

impl Plugin for FarmPluginVueJsx {
    fn name(&self) -> &str {
        "FarmPluginVueJsx"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn transform(
        &self,
        param: &PluginTransformHookParam,
        _context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
        // Guard: only process Jsx/Tsx module types
        if !matches!(&param.module_type, ModuleType::Jsx | ModuleType::Tsx) {
            return Ok(None);
        }

        let options = self.get_options();

        let syntax = match &param.module_type {
            ModuleType::Tsx => Syntax::Typescript(TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            _ => Syntax::Es(Default::default()),
        };

        let globals = Globals::default();
        GLOBALS.set(&globals, || {
            let parse_result = parse_module(
                &param.module_id.clone().into(),
                Arc::new(param.content.clone()),
                syntax,
                Default::default(),
            )
            .map_err(|e| CompilationError::TransformError {
                resolved_path: param.resolved_path.to_string(),
                msg: format!("[vue-jsx] failed to parse module: {e}"),
            })?;

            let unresolved_mark = Mark::new();
            let mut ast = parse_result.ast;
            let comments = parse_result.comments;
            let cm = parse_result.source_map;

            let mut visitor = VueJsxTransformVisitor::new(
                options,
                unresolved_mark,
                Some(&comments),
                param.resolved_path.to_string(),
            );

            ast.visit_mut_with(&mut visitor);

            let codegen_config = create_codegen_config(_context);
            let code_bytes = codegen_module(
                &ast,
                cm,
                None,
                codegen_config,
                Some(farmfe_toolkit::script::CodeGenCommentsConfig {
                    comments: &comments,
                    config: &Default::default(),
                }),
            )
            .map_err(|e| CompilationError::TransformError {
                resolved_path: param.resolved_path.to_string(),
                msg: format!("[vue-jsx] codegen error: {e}"),
            })?;

            let content = String::from_utf8(code_bytes).map_err(|e| {
                CompilationError::TransformError {
                    resolved_path: param.resolved_path.to_string(),
                    msg: format!("[vue-jsx] invalid utf-8 output: {e}"),
                }
            })?;

            Ok(Some(PluginTransformHookResult {
                content,
                module_type: Some(param.module_type.clone()),
                source_map: None,
                ignore_previous_source_map: false,
            }))
        })
    }
}
