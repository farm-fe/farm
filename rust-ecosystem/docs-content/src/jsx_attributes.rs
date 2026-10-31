use mdxjs::hast::Node;
use std::collections::BTreeSet;
use swc_core::{
  common::Span,
  ecma::{
    ast::{
      Expr, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXElementName, JSXExpr, JSXExprContainer,
      Lit, Module,
    },
    visit::{VisitMut, VisitMutWith},
  },
};

/// Preserve the distinct semantics of generated strings and authored JSX.
pub fn preserve(module: &mut Module, tree: &Node, is_authored: impl Fn(Span) -> bool) {
  fn collect(node: &Node, positions: &mut BTreeSet<(usize, usize)>) {
    if let Node::Element(element) = node {
      if element.tag_name == "pre"
        && element
          .properties
          .iter()
          .any(|(name, _)| name == "data-code")
      {
        if let Some(position) = &element.position {
          positions.insert((position.start.offset + 1, position.end.offset + 1));
        }
      }
    }
    if let Some(children) = node.children() {
      for child in children {
        collect(child, positions);
      }
    }
  }
  struct Preserve<F> {
    generated: BTreeSet<(usize, usize)>,
    authored: F,
  }
  impl<F: Fn(Span) -> bool> VisitMut for Preserve<F> {
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
      element.visit_mut_children_with(self);
      let generated = self
        .generated
        .contains(&(element.span.lo.0 as usize, element.span.hi.0 as usize));
      let authored = (self.authored)(element.span);
      if !generated && !authored {
        return;
      }
      // mdxjs retains the authored element span but creates its tag identifier
      // with a dummy span. Give downstream AST transforms a mapped name anchor;
      // otherwise generated nodes trace to the preceding Markdown expression.
      if authored {
        if let JSXElementName::Ident(name) = &mut element.opening.name {
          if name.span.is_dummy() {
            name.span = element.span.shrink_to_lo();
          }
        }
      }
      for attribute in &mut element.opening.attrs {
        let JSXAttrOrSpread::JSXAttr(attribute) = attribute else {
          continue;
        };
        let Some(JSXAttrValue::Lit(Lit::Str(value))) = &mut attribute.value else {
          continue;
        };
        if generated {
          // JSX quoted values do not decode JavaScript escapes. A JS string
          // expression preserves the exact Rust code text, including newlines.
          attribute.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: value.span,
            expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Str(value.clone())))),
          }));
        } else if value.raw.is_none() {
          // MDX literals still contain authored entities. Retain JSX quoting
          // and backslashes; encode only the quote chosen for serialization.
          // ESM/expression JSX already has its original raw literal and is left alone.
          value.raw = Some(format!("\"{}\"", value.value.replace('"', "&quot;")).into());
        }
      }
    }
  }
  let mut generated = BTreeSet::new();
  collect(tree, &mut generated);
  module.visit_mut_with(&mut Preserve {
    generated,
    authored: is_authored,
  });
}
