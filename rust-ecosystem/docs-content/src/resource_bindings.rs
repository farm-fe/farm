use std::collections::{BTreeMap, HashMap, HashSet};
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::*,
    visit::{Visit, VisitMut, VisitMutWith, VisitWith},
  },
};

#[derive(Clone, Default)]
pub struct ResourceBindings(pub BTreeMap<String, (String, String)>);
struct Names(HashSet<String>);
impl Visit for Names {
  fn visit_ident(&mut self, ident: &Ident) {
    self.0.insert(ident.sym.to_string());
  }
}
struct Rewriter {
  values: HashMap<String, (Ident, String)>,
  used: HashSet<String>,
}
impl VisitMut for Rewriter {
  fn visit_mut_jsx_attr(&mut self, attribute: &mut JSXAttr) {
    attribute.visit_mut_children_with(self);
    if !matches!(&attribute.name, JSXAttrName::Ident(name) if matches!(name.sym.as_ref(),"src"|"href"))
    {
      return;
    }
    let literal = match &attribute.value {
      Some(JSXAttrValue::Lit(Lit::Str(value))) => Some(value),
      Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
        expr: JSXExpr::Expr(value),
        ..
      })) => match value.as_ref() {
        Expr::Lit(Lit::Str(value)) => Some(value),
        _ => None,
      },
      _ => None,
    };
    let Some(literal) = literal else {
      return;
    };
    let key = literal.value.to_string();
    let Some((name, suffix)) = self.values.get(&key) else {
      return;
    };
    let span = literal.span;
    let expression = if suffix.is_empty() {
      Expr::Ident(name.clone())
    } else {
      Expr::Bin(BinExpr {
        span,
        op: BinaryOp::Add,
        left: Box::new(Expr::Ident(name.clone())),
        right: Box::new(Expr::Lit(Lit::Str(Str {
          span,
          value: suffix.as_str().into(),
          raw: None,
        }))),
      })
    };
    attribute.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
      span,
      expr: JSXExpr::Expr(Box::new(expression)),
    }));
    self.used.insert(key);
  }
}
pub fn apply(module: &mut Module, bindings: ResourceBindings) -> Result<(), String> {
  if bindings.0.is_empty() {
    return Ok(());
  }
  let mut names = Names(HashSet::new());
  module.visit_with(&mut names);
  let mut rewriter = Rewriter {
    values: HashMap::new(),
    used: HashSet::new(),
  };
  let mut imports = vec![];
  let mut index = 0;
  for (url, (source, suffix)) in bindings.0 {
    let name = loop {
      let candidate = format!("_farmDocsAsset{index}");
      index += 1;
      if names.0.insert(candidate.clone()) {
        break candidate;
      }
    };
    let ident = Ident::new_no_ctxt(name.into(), DUMMY_SP);
    imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
        span: DUMMY_SP,
        local: ident.clone(),
      })],
      src: Box::new(Str {
        span: DUMMY_SP,
        value: source.into(),
        raw: None,
      }),
      type_only: false,
      with: None,
      phase: Default::default(),
    })));
    rewriter.values.insert(url, (ident, suffix));
  }
  module.visit_mut_with(&mut rewriter);
  if rewriter.used.len() != rewriter.values.len() {
    return Err(
      "[DOCS_ASSET_BINDING] resource JSX value was not retained in the compiled tree".into(),
    );
  }
  imports.append(&mut module.body);
  module.body = imports;
  Ok(())
}
