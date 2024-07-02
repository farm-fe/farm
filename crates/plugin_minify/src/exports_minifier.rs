use std::collections::HashMap;

use farmfe_core::{
  swc_common::{util::take::Take, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, Decl, ExportDecl, Expr, Ident, Module, ModuleDecl, ModuleExportName, ModuleItem,
    NamedExport, Pat, Stmt, VarDecl, VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::defined_idents_collector::DefinedIdentsCollector,
  swc_ecma_visit::{VisitMut, VisitWith},
};

use crate::{ident_generator::MinifiedIdentsGenerator, util::get_module_export_name};

pub struct ExportsMinifier<'a> {
  pub minified_exports_map: HashMap<String, String>,

  ident_generator: &'a mut MinifiedIdentsGenerator,
}

impl<'a> ExportsMinifier<'a> {
  pub fn new(ident_generator: &'a mut MinifiedIdentsGenerator) -> Self {
    Self {
      minified_exports_map: HashMap::new(),
      ident_generator,
    }
  }
}

impl<'a> ExportsMinifier<'a> {
  fn create_export_var_item(&self, minified_ident: String, ident: Ident) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
      span: DUMMY_SP,
      decl: Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: Ident::new(minified_ident.into(), DUMMY_SP),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Ident(ident))),
          definite: false,
        }],
      })),
    }))
  }
}

impl<'a> VisitMut for ExportsMinifier<'a> {
  fn visit_mut_module(&mut self, n: &mut Module) {
    let mut new_items = Vec::new();

    for item in n.body.take() {
      if let ModuleItem::ModuleDecl(decl) = item {
        match decl {
          ModuleDecl::ExportDecl(export_decl) => {
            match export_decl.decl {
              farmfe_core::swc_ecma_ast::Decl::Class(class) => {
                let ident = class.ident.clone();
                new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(class))));
                let minified_ident = self.ident_generator.generate();
                self
                  .minified_exports_map
                  .insert(ident.sym.to_string(), minified_ident.clone());
                new_items.push(self.create_export_var_item(minified_ident, ident));
              }
              farmfe_core::swc_ecma_ast::Decl::Fn(func) => {
                let ident = func.ident.clone();
                new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(func))));
                let minified_ident = self.ident_generator.generate();
                self
                  .minified_exports_map
                  .insert(ident.sym.to_string(), minified_ident.clone());
                new_items.push(self.create_export_var_item(minified_ident, ident));
              }
              farmfe_core::swc_ecma_ast::Decl::Var(var_decl) => {
                let mut decl_items = Vec::new();

                for var_decl in &var_decl.decls {
                  let mut defined_idents_collector = DefinedIdentsCollector::new();
                  var_decl.name.visit_with(&mut defined_idents_collector);

                  for defined_ident in defined_idents_collector.defined_idents {
                    let minified_ident = self.ident_generator.generate();
                    self
                      .minified_exports_map
                      .insert(defined_ident.0.to_string(), minified_ident.clone());
                    decl_items.push(self.create_export_var_item(
                      minified_ident,
                      Ident::new(defined_ident.0, DUMMY_SP.with_ctxt(defined_ident.1)),
                    ));
                  }
                }

                new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))));
                new_items.extend(decl_items);
              }
              _ => { /* do nothing */ }
            }
          }
          ModuleDecl::ExportNamed(mut export_named) => {
            // ignore export from, it will be handled in `minify_import` (expect export * as xxx from './xx')
            if export_named.src.is_some() {
              for sp in &mut export_named.specifiers {
                if let farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(ns) = sp {
                  let name = get_module_export_name(ns.name.clone()).sym.to_string();
                  if &name == "default" {
                    continue;
                  }

                  let minified_ident = self.ident_generator.generate();
                  self
                    .minified_exports_map
                    .insert(name, minified_ident.clone());
                  ns.name = ModuleExportName::Ident(Ident::new(minified_ident.into(), DUMMY_SP));
                }
              }
              new_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                export_named,
              )));
            } else {
              let mut preserved_specifiers = vec![];

              for sp in export_named.specifiers {
                let minified_ident = self.ident_generator.generate();
                // export { hello as hello1 } => export var a = hello; // minified_idents_map: hello1 -> a
                match sp {
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(_) => unreachable!(),
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named) => {
                    let exported = if let Some(exported) = named.exported.clone() {
                      get_module_export_name(exported)
                    } else {
                      get_module_export_name(named.orig.clone())
                    };
                    if exported.sym == "default" {
                      preserved_specifiers
                        .push(farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named));
                      continue;
                    }
                    self
                      .minified_exports_map
                      .insert(exported.sym.to_string(), minified_ident.clone());
                    new_items.push(
                      self
                        .create_export_var_item(minified_ident, get_module_export_name(named.orig)),
                    );
                  }
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Default(_) => unreachable!(),
                }
              }

              if !preserved_specifiers.is_empty() {
                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                  NamedExport {
                    span: DUMMY_SP,
                    specifiers: preserved_specifiers,
                    src: None,
                    type_only: false,
                    with: None,
                  },
                )));
              }
            }
          }
          ModuleDecl::ExportDefaultDecl(_)
          | ModuleDecl::ExportDefaultExpr(_)
          | ModuleDecl::ExportAll(_)
          | ModuleDecl::TsImportEquals(_)
          | ModuleDecl::Import(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => {
            new_items.push(ModuleItem::ModuleDecl(decl));
          }
        }
      } else {
        new_items.push(item);
      }
    }

    n.body = new_items;
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashSet, sync::Arc};

  use farmfe_core::{swc_common::Globals, swc_ecma_ast::EsVersion, swc_ecma_parser::Syntax};
  use farmfe_toolkit::{
    common::{create_swc_source_map, Source},
    script::{codegen_module, parse_module, swc_try_with::try_with},
    swc_ecma_visit::VisitMutWith,
  };

  use crate::top_level_idents_collector::TopLevelIdentsCollector;

  use super::*;

  #[test]
  fn test_export_minifier() {
    let path = "any";
    let content = r#"
var a = 'a', b = 'b';
function c() {
  var d = 'd';
}
class e {}

const long1 = 'long1', long2 = 'long2';
export { long1, long2 as default }
export const long3 = 'long3', long4 = 'long4';
export function long5() {}
export class long6 {}
export default function long7() {}
export default class long8 {}
export default 'hello';

export * from './dep';
export * as long9 from './dep1';
export { long10 } from './dep2';
    "#;
    let (cm, _) = create_swc_source_map(Source {
      path: std::path::PathBuf::from(path),
      content: Arc::new(content.to_string()),
    });
    let mut ast = parse_module(
      path,
      content,
      Syntax::Es(Default::default()),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;

    let mut top_level_idents_collector = TopLevelIdentsCollector::new();
    ast.visit_mut_with(&mut top_level_idents_collector);

    assert_eq!(
      top_level_idents_collector.top_level_idents,
      HashSet::from([
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "e".to_string(),
        "long1".to_string(),
        "long2".to_string(),
        "long3".to_string(),
        "long4".to_string(),
        "long5".to_string(),
        "long6".to_string(),
        "long7".to_string(),
        "long8".to_string(),
      ])
    );

    let mut ident_generator =
      MinifiedIdentsGenerator::new(top_level_idents_collector.top_level_idents);
    let mut export_minifier = ExportsMinifier::new(&mut ident_generator);

    try_with(cm.clone(), &Globals::new(), || {
      ast.visit_mut_with(&mut export_minifier);

      let code_bytes =
        codegen_module(&mut ast, EsVersion::latest(), cm, None, false, None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      println!("{}", code);

      assert_eq!(
        code,
        r#"var a = 'a', b = 'b';
function c() {
    var d = 'd';
}
class e {
}
const long1 = 'long1', long2 = 'long2';
export var d = long1;
export { long2 as default };
const long3 = 'long3', long4 = 'long4';
export var g = long3;
export var h = long4;
function long5() {}
export var i = long5;
class long6 {
}
export var j = long6;
export default function long7() {}
export default class long8 {
}
export default 'hello';
export * from './dep';
export * as k from './dep1';
export { long10 } from './dep2';
"#
      );

      assert_eq!(
        export_minifier.minified_exports_map,
        HashMap::from([
          ("long1".to_string(), "d".to_string(),),
          ("long3".to_string(), "g".to_string(),),
          ("long4".to_string(), "h".to_string(),),
          ("long5".to_string(), "i".to_string(),),
          ("long6".to_string(), "j".to_string(),),
          ("long9".to_string(), "k".to_string(),),
        ])
      );
    })
    .unwrap();
  }
}
