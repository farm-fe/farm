use std::collections::HashMap;

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, CallExpr, Callee, ExportNamedSpecifier, ExportSpecifier, Expr, ExprOrSpread,
    ExprStmt, Id, Ident, KeyValuePatProp, KeyValueProp, Lit, MemberExpr, MemberProp, ModuleDecl,
    ModuleExportName, ModuleItem, NamedExport, ObjectLit, Pat, Prop, PropName, PropOrSpread, Stmt,
    Str,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::{ident_generator::MinifiedIdentsGenerator, util::get_module_export_name};

pub struct ImportsMinifier<'a> {
  pub id_to_replace_map: HashMap<Id, String>,
  module_id: &'a ModuleId,
  minified_module_exports_map: &'a mut HashMap<ModuleId, HashMap<String, String>>,
  module_graph: &'a ModuleGraph,
  ident_generator: &'a mut MinifiedIdentsGenerator,
  unresolved_mark: Mark,

  imported_ident_count_map: HashMap<String, usize>,
  exported_ident_count_map: HashMap<String, usize>,
}

impl<'a> ImportsMinifier<'a> {
  pub fn new(
    module_id: &'a ModuleId,
    minified_module_exports_map: &'a mut HashMap<ModuleId, HashMap<String, String>>,
    module_graph: &'a ModuleGraph,
    ident_generator: &'a mut MinifiedIdentsGenerator,
    unresolved_mark: Mark,
  ) -> Self {
    let mut exported_ident_count_map = HashMap::new();

    if let Some(current_minified_exports_map) = minified_module_exports_map.get(module_id) {
      for (_, minified_ident) in current_minified_exports_map {
        exported_ident_count_map.insert(minified_ident.to_string(), 1);
      }
    }

    let mut imported_ident_count_map = HashMap::new();
    // used idents mean the idents that are used in the current module, they should not be used to avoid ident decl conflict
    for ident in ident_generator.used_idents() {
      imported_ident_count_map.insert(ident.to_string(), 1);
    }

    Self {
      module_id,
      minified_module_exports_map,
      module_graph,
      ident_generator,
      unresolved_mark,
      id_to_replace_map: HashMap::new(),

      imported_ident_count_map,
      exported_ident_count_map,
    }
  }

  pub fn inc_imported_ident_count(&mut self, ident: String) {
    *self.imported_ident_count_map.entry(ident).or_default() += 1;
  }

  pub fn get_imported_ident_with_count(&mut self, ident: &str) -> String {
    let count = self.imported_ident_count_map.get(ident).unwrap_or(&0);
    let mut used_ident = if *count == 0 {
      ident.to_string()
    } else {
      format!("{}{}", ident, count)
    };

    let mut count = *count;

    while self.imported_ident_count_map.contains_key(&used_ident) {
      count += 1;
      used_ident = format!("{}{}", ident, count);
    }

    self.inc_imported_ident_count(used_ident.clone());

    used_ident
  }

  pub fn inc_exported_ident_count(&mut self, ident: String) {
    *self.exported_ident_count_map.entry(ident).or_default() += 1;
  }

  pub fn get_exported_ident_with_count(&mut self, ident: &str) -> String {
    let count = self.exported_ident_count_map.get(ident).unwrap_or(&0);
    let mut used_ident = if *count == 0 {
      ident.to_string()
    } else {
      format!("{}{}", ident, count)
    };

    let mut count = *count;

    while self.exported_ident_count_map.contains_key(&used_ident) {
      count += 1;
      used_ident = format!("{}{}", ident, count);
    }

    self.inc_exported_ident_count(used_ident.clone());

    used_ident
  }
}

impl<'a> VisitMut for ImportsMinifier<'a> {
  fn visit_mut_module(&mut self, n: &mut farmfe_core::swc_ecma_ast::Module) {
    let mut id_to_replace = HashMap::new();

    let mut renamed_re_exports = HashMap::<String, HashMap<String, String>>::new();
    let mut reverted_import_export_star = HashMap::<ModuleId, (usize, Id)>::new();

    // visit all import/export from/export * from statement and minify the imported ident
    for (index, item) in n.body.iter_mut().enumerate() {
      match item {
        farmfe_core::swc_ecma_ast::ModuleItem::ModuleDecl(module_decl) => match module_decl {
          farmfe_core::swc_ecma_ast::ModuleDecl::Import(import_decl) => {
            let source = import_decl.src.value.to_string();
            let dep_module_id = self.module_graph.get_dep_by_source(
              &self.module_id,
              &source,
              Some(ResolveKind::Import),
            );

            for sp in &mut import_decl.specifiers {
              match sp {
                farmfe_core::swc_ecma_ast::ImportSpecifier::Named(named) => {
                  // 1. import { hello } from './hello' -> import { a } from './hello' (hello is minified in ExportsMinifier)
                  // 2. import { hello as hello1 } from './hello' -> import { a as hello1 } from './hello' and minify hello1
                  if self
                    .minified_module_exports_map
                    .get(&dep_module_id)
                    .is_none()
                  {
                    continue;
                  }

                  let imported = if let Some(imported) = &mut named.imported {
                    match imported {
                      farmfe_core::swc_ecma_ast::ModuleExportName::Ident(ident) => {
                        ident.sym.to_string()
                      }
                      farmfe_core::swc_ecma_ast::ModuleExportName::Str(_) => unreachable!(),
                    }
                  } else {
                    named.local.sym.to_string()
                  };

                  if imported.as_str() == "default" {
                    let ident = self.ident_generator.generate();
                    self.inc_imported_ident_count(ident.clone());
                    id_to_replace.insert(named.local.to_id(), ident.clone());
                    named.local.sym = ident.as_str().into();
                    continue;
                  }

                  let dep_minified_exports = self
                    .minified_module_exports_map
                    .get(&dep_module_id)
                    .unwrap();
                  let mut ident_to_inc = vec![];

                  if let Some(orig_minified_export) = dep_minified_exports.get(&imported) {
                    let orig_minified_export = orig_minified_export.clone();
                    // `minified_export`` should not be used in this module again
                    self.ident_generator.add_used_ident(&orig_minified_export);

                    if let Some(imported) = &mut named.imported {
                      match imported {
                        farmfe_core::swc_ecma_ast::ModuleExportName::Ident(ident) => {
                          ident.sym = orig_minified_export.as_str().into();
                        }
                        farmfe_core::swc_ecma_ast::ModuleExportName::Str(_) => unreachable!(),
                      }

                      // minify named.local and add it to id_to_replace
                      let ident = self.ident_generator.generate();
                      id_to_replace.insert(named.local.to_id(), ident.clone());
                      named.local.sym = ident.as_str().into();
                      ident_to_inc.push(ident);
                    } else {
                      // import { hello } from './hello' -> import { a as a1 } from './hello' if a is already defined
                      if self
                        .imported_ident_count_map
                        .contains_key(&orig_minified_export)
                      {
                        let minified_export =
                          self.get_imported_ident_with_count(&orig_minified_export);
                        id_to_replace.insert(named.local.to_id(), minified_export.clone());
                        named.imported = Some(farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
                          Ident::new(orig_minified_export.as_str().into(), DUMMY_SP),
                        ));
                        named.local.sym = minified_export.as_str().into();
                      } else {
                        id_to_replace.insert(named.local.to_id(), orig_minified_export.clone());
                        named.local.sym = orig_minified_export.as_str().into();
                      }
                      ident_to_inc.push(orig_minified_export);
                    }
                  }

                  for ident in ident_to_inc {
                    self.inc_imported_ident_count(ident);
                  }
                }
                farmfe_core::swc_ecma_ast::ImportSpecifier::Default(default) => {
                  let ident = self.ident_generator.generate();
                  self.inc_imported_ident_count(ident.clone());
                  id_to_replace.insert(default.local.to_id(), ident.clone());
                  default.local.sym = ident.as_str().into();
                }
                farmfe_core::swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                  let ident = self.ident_generator.generate();
                  self.inc_imported_ident_count(ident.clone());
                  id_to_replace.insert(ns.local.to_id(), ident.clone());
                  ns.local.sym = ident.as_str().into();
                  reverted_import_export_star
                    .insert(dep_module_id.clone(), (index, ns.local.to_id()));
                }
              }
            }
          }
          /* For export * from. Append all minified export info of the dependency module to current module's minified_export  */
          farmfe_core::swc_ecma_ast::ModuleDecl::ExportAll(export_all) => {
            let dep_module_id = self.module_graph.get_dep_by_source(
              &self.module_id,
              &export_all.src.value,
              Some(ResolveKind::ExportFrom),
            );

            let dep_minified_exports = self
              .minified_module_exports_map
              .get(&dep_module_id)
              .cloned();

            if let Some(dep_minified_exports) = dep_minified_exports {
              let mut filtered_dep_minified_exports = HashMap::new();

              for (export, minified) in dep_minified_exports {
                if self.exported_ident_count_map.contains_key(&minified) {
                  let dep_entry = renamed_re_exports
                    .entry(export_all.src.value.to_string())
                    .or_default();
                  let re_minified = self.get_exported_ident_with_count(&minified);
                  self.inc_exported_ident_count(minified.clone());
                  self.ident_generator.add_used_ident(&minified);
                  dep_entry.insert(minified.clone(), re_minified.clone());

                  filtered_dep_minified_exports.insert(export, re_minified);
                } else {
                  self.inc_exported_ident_count(minified.clone());
                  self.ident_generator.add_used_ident(&minified);

                  filtered_dep_minified_exports.insert(export, minified);
                }
              }

              self
                .minified_module_exports_map
                .entry(self.module_id.clone())
                .or_default()
                .extend(filtered_dep_minified_exports);
            }
          }
          /* For export { xx } from. Append all minified export info of the dependency module to current module's minified_export  */
          farmfe_core::swc_ecma_ast::ModuleDecl::ExportNamed(export_named) => {
            if let Some(src) = &export_named.src {
              for sp in &mut export_named.specifiers {
                match sp {
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(ns) => {
                    let ident = self.ident_generator.generate();
                    let mut ns_ident = get_module_export_name(ns.name.clone());
                    id_to_replace.insert(ns_ident.to_id(), ident.clone());
                    ns_ident.sym = ident.as_str().into();

                    let dep_module_id = self.module_graph.get_dep_by_source(
                      &self.module_id,
                      &src.value,
                      Some(ResolveKind::ExportFrom),
                    );
                    reverted_import_export_star.insert(dep_module_id, (index, ns_ident.to_id()));
                    ns.name = ModuleExportName::Ident(ns_ident);
                  }
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named) => {
                    let dep_module_id = self.module_graph.get_dep_by_source(
                      &self.module_id,
                      &src.value,
                      Some(ResolveKind::ExportFrom),
                    );

                    let mut current_minified_exports = HashMap::new();

                    if let Some(dep_minified_exports) =
                      self.minified_module_exports_map.get(&dep_module_id)
                    {
                      let mut orig = get_module_export_name(named.orig.clone());
                      let orig_str = orig.sym.to_string();

                      // export { hello as hello1 } from './hello' -> export { x as a } from './hello'
                      if let Some(exported) = &named.exported {
                        if let Some(minified_export) = dep_minified_exports.get(&orig_str) {
                          orig.sym = minified_export.as_str().into();
                          named.orig = ModuleExportName::Ident(orig);
                        }

                        let mut exported = get_module_export_name(exported.clone());
                        let ident = self.ident_generator.generate();
                        self.inc_exported_ident_count(ident.clone());
                        current_minified_exports.insert(exported.sym.to_string(), ident.clone());
                        exported.sym = ident.as_str().into();
                        named.exported = Some(ModuleExportName::Ident(exported));
                      } else if let Some(orig_minified_export) = dep_minified_exports.get(&orig_str)
                      {
                        let orig_minified_export = orig_minified_export.clone();
                        // export { hello } from './hello' -> export { x } from './hello'
                        // or export { x as x1 } from './hello' only if x is already exported
                        if self
                          .exported_ident_count_map
                          .contains_key(&orig_minified_export)
                        {
                          orig.sym = orig_minified_export.as_str().into();
                          named.orig = ModuleExportName::Ident(orig);

                          let minified_export =
                            self.get_exported_ident_with_count(&orig_minified_export);
                          named.exported = Some(ModuleExportName::Ident(Ident::new(
                            minified_export.as_str().into(),
                            DUMMY_SP,
                          )));
                          current_minified_exports.insert(orig_str, minified_export);
                        } else {
                          orig.sym = orig_minified_export.as_str().into();
                          named.orig = ModuleExportName::Ident(orig);

                          current_minified_exports.insert(orig_str, orig_minified_export.clone());
                        }

                        self.ident_generator.add_used_ident(&orig_minified_export);
                        self.inc_exported_ident_count(orig_minified_export);
                      }
                    }

                    if !current_minified_exports.is_empty() {
                      self
                        .minified_module_exports_map
                        .entry(self.module_id.clone())
                        .or_default()
                        .extend(current_minified_exports);
                    }
                  }
                  farmfe_core::swc_ecma_ast::ExportSpecifier::Default(_) => unreachable!(),
                }
              }
            }
          }
          _ => {}
        },
        _ => {}
      }
    }

    // rename conflicting re-exports when there are multiple export * from
    let mut renamed_re_exports = renamed_re_exports.into_iter().collect::<Vec<_>>();
    renamed_re_exports.sort_by(|a, b| b.0.cmp(&a.0));

    for (src, re_minified) in renamed_re_exports {
      let mut re_minified = re_minified.into_iter().collect::<Vec<_>>();
      re_minified.sort_by(|a, b| a.0.cmp(&b.0));

      let specifiers = re_minified
        .into_iter()
        .map(|(from, to)| {
          ExportSpecifier::Named(ExportNamedSpecifier {
            span: DUMMY_SP,
            orig: ModuleExportName::Ident(Ident::new(from.as_str().into(), DUMMY_SP)),
            exported: Some(ModuleExportName::Ident(Ident::new(
              to.as_str().into(),
              DUMMY_SP,
            ))),
            is_type_only: false,
          })
        })
        .collect();

      n.body.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
        NamedExport {
          span: DUMMY_SP,
          specifiers,
          src: Some(Box::new(Str {
            span: DUMMY_SP,
            value: src.as_str().into(),
            raw: None,
          })),
          type_only: false,
          with: None,
        },
      )))
    }

    // revert minified idents for import * as xx and export * as xx to avoid
    let mut reverted_import_export_star =
      reverted_import_export_star.into_iter().collect::<Vec<_>>();
    reverted_import_export_star.sort_by(|a, b| a.0.cmp(&b.0));

    for (dep_module_id, (index, star_ident)) in reverted_import_export_star {
      let mut object_lit = ObjectLit {
        span: DUMMY_SP,
        props: vec![],
      };

      if let Some(dep_minified_exports) = self.minified_module_exports_map.get(&dep_module_id) {
        let mut dep_minified_exports = dep_minified_exports.into_iter().collect::<Vec<_>>();
        dep_minified_exports.sort_by(|a, b| a.1.cmp(&b.1));

        for (export, minified) in dep_minified_exports {
          object_lit
            .props
            .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
              key: PropName::Str(Str {
                span: DUMMY_SP,
                value: minified.as_str().into(),
                raw: None,
              }),
              value: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: export.as_str().into(),
                raw: None,
              }))),
            }))));
        }
      }

      n.body.insert(
        index,
        ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          span: DUMMY_SP,
          expr: Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident(Ident::new(
                "module".into(),
                DUMMY_SP.apply_mark(self.unresolved_mark),
              ))),
              prop: MemberProp::Ident(Ident::new("p".into(), DUMMY_SP)),
            }))),
            args: vec![
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Object(object_lit)),
              },
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(Ident::new(
                  star_ident.0,
                  DUMMY_SP.with_ctxt(star_ident.1),
                ))),
              },
            ],
            type_args: None,
          })),
        })),
      )
    }

    self.id_to_replace_map = id_to_replace;
  }
}

pub struct IdentReplacer {
  id_to_replace: HashMap<farmfe_core::swc_ecma_ast::Id, String>,
}

impl IdentReplacer {
  pub fn new(id_to_replace: HashMap<farmfe_core::swc_ecma_ast::Id, String>) -> Self {
    Self { id_to_replace }
  }
}

impl<'a> VisitMut for IdentReplacer {
  fn visit_mut_object_lit(&mut self, n: &mut farmfe_core::swc_ecma_ast::ObjectLit) {
    for prop in &mut n.props {
      match prop {
        PropOrSpread::Prop(prop) => match &**prop {
          farmfe_core::swc_ecma_ast::Prop::Shorthand(s) => {
            if let Some(replaced) = self.id_to_replace.get(&s.to_id()) {
              *prop = Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(s.clone()),
                value: Box::new(Expr::Ident(Ident::new(
                  replaced.as_str().into(),
                  DUMMY_SP.with_ctxt(s.span.ctxt()),
                ))),
              }))
            }
          }
          _ => {
            prop.visit_mut_children_with(self);
          }
        },
        PropOrSpread::Spread(spread) => {
          spread.expr.visit_mut_children_with(self);
        }
      }
    }
  }

  // fix #1644. Do not replace ident of member expression
  fn visit_mut_member_prop(&mut self, n: &mut MemberProp) {
    // ignore ident and private name of member expression
    if let MemberProp::Computed(computed) = n {
      computed.expr.visit_mut_with(self);
    }
  }

  fn visit_mut_object_pat(&mut self, pat: &mut farmfe_core::swc_ecma_ast::ObjectPat) {
    for n in &mut pat.props {
      match n {
        farmfe_core::swc_ecma_ast::ObjectPatProp::KeyValue(key) => {
          // fix #1672. Replace ident of computed property of object pattern
          if let PropName::Computed(c) = &mut key.key {
            c.expr.visit_mut_with(self);
          }

          key.value.visit_mut_with(self);
        }
        farmfe_core::swc_ecma_ast::ObjectPatProp::Assign(a) => {
          if let Some(value) = &mut a.value {
            value.visit_mut_with(self);
          } else if let Some(replaced) = self.id_to_replace.get(&a.key.id.to_id()) {
            *n = farmfe_core::swc_ecma_ast::ObjectPatProp::KeyValue(KeyValuePatProp {
              key: PropName::Ident(a.key.id.clone()),
              value: Box::new(Pat::Ident(BindingIdent {
                id: Ident::new(
                  replaced.as_str().into(),
                  DUMMY_SP.apply_mark(a.key.id.span.ctxt().outer()),
                ),
                type_ann: None,
              })),
            })
          }
        }
        farmfe_core::swc_ecma_ast::ObjectPatProp::Rest(r) => r.visit_mut_with(self),
      }
    }
  }

  fn visit_mut_ident(&mut self, n: &mut farmfe_core::swc_ecma_ast::Ident) {
    if let Some(replaced) = self.id_to_replace.get(&n.to_id()) {
      n.sym = replaced.as_str().into();
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    module::{
      module_graph::{ModuleGraph, ModuleGraphEdge, ModuleGraphEdgeDataItem},
      Module, ModuleId,
    },
    plugin::ResolveKind,
    swc_common::{Globals, Mark},
    swc_ecma_ast::EsVersion,
    swc_ecma_parser::Syntax,
  };
  use farmfe_toolkit::{
    common::{create_swc_source_map, Source},
    script::{codegen_module, parse_module, swc_try_with::try_with},
    swc_ecma_visit::VisitMutWith,
  };

  use crate::{
    ident_generator::MinifiedIdentsGenerator, imports_minifier::ImportsMinifier,
    top_level_idents_collector::TopLevelIdentsCollector,
  };

  #[test]
  fn test_minify_imports() {
    let path = "any";
    let content = r#"
import { hello1, hello2 as hello3, default as hello4 } from './dep';
import { foo, zoo, bar } from './dep2';
import hello5 from './dep';
import * as hello6 from './dep';
import hello7 from './dep1';

console.log(hello1, hello3, hello4, hello5, hello6, hello7);
console.log(foo, zoo, bar);

export * from './dep1';
export { hello1, hello2, default as world1 } from './dep';
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
    let mut ident_generator =
      MinifiedIdentsGenerator::new(top_level_idents_collector.top_level_idents);

    // create module and module graph
    let (module_id, module_graph) = {
      let module_id = ModuleId::from("current");
      let module = Module::new(module_id.clone());
      let dep_module = Module::new(ModuleId::from("dep"));
      let dep1_module = Module::new(ModuleId::from("dep1"));
      let dep2_module = Module::new(ModuleId::from("dep2"));
      let mut module_graph = ModuleGraph::new();
      module_graph.add_module(module);
      module_graph.add_module(dep1_module);
      module_graph.add_module(dep_module);
      module_graph.add_module(dep2_module);
      module_graph
        .add_edge(
          &module_id,
          &ModuleId::from("dep"),
          ModuleGraphEdge::new(vec![
            ModuleGraphEdgeDataItem {
              source: "./dep".to_string(),
              kind: ResolveKind::Import,
              order: 0,
            },
            ModuleGraphEdgeDataItem {
              source: "./dep".to_string(),
              kind: ResolveKind::ExportFrom,
              order: 2,
            },
          ]),
        )
        .unwrap();
      module_graph
        .add_edge(
          &module_id,
          &ModuleId::from("dep1"),
          ModuleGraphEdge::new(vec![
            ModuleGraphEdgeDataItem {
              source: "./dep1".to_string(),
              kind: ResolveKind::Import,
              order: 2,
            },
            ModuleGraphEdgeDataItem {
              source: "./dep1".to_string(),
              kind: ResolveKind::ExportFrom,
              order: 1,
            },
          ]),
        )
        .unwrap();
      module_graph
        .add_edge(
          &module_id,
          &ModuleId::from("dep2"),
          ModuleGraphEdge::new(vec![ModuleGraphEdgeDataItem {
            source: "./dep2".to_string(),
            kind: ResolveKind::Import,
            order: 0,
          }]),
        )
        .unwrap();
      (module_id, module_graph)
    };
    let mut minified_module_exports_map = HashMap::from([
      (
        ModuleId::from("dep"),
        HashMap::from([
          ("hello1".to_string(), "aa".to_string()),
          ("hello2".to_string(), "bb".to_string()),
          ("hello3".to_string(), "cc".to_string()),
        ]),
      ),
      (
        ModuleId::from("dep1"),
        HashMap::from([("world".to_string(), "dd".to_string())]),
      ),
      (
        ModuleId::from("dep2"),
        HashMap::from([
          ("foo".to_string(), "aa".to_string()),
          ("zoo".to_string(), "bb".to_string()),
          ("bar".to_string(), "cc".to_string()),
        ]),
      ),
    ]);

    try_with(cm.clone(), &Globals::new(), || {
      let mut imports_minifier = ImportsMinifier::new(
        &module_id,
        &mut minified_module_exports_map,
        &module_graph,
        &mut ident_generator,
        Mark::new(),
      );

      ast.visit_mut_with(&mut imports_minifier);

      let code_bytes =
        codegen_module(&mut ast, EsVersion::latest(), cm, None, false, None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      println!("{}", code);

      assert_eq!(
        code,
        r#"import { aa, bb as a, default as b } from './dep';
import { aa as aa1, bb, cc } from './dep2';
import c from './dep';
module.p({
    "aa": "hello1",
    "bb": "hello2",
    "cc": "hello3"
}, d);
import * as d from './dep';
import e from './dep1';
console.log(hello1, hello3, hello4, hello5, hello6, hello7);
console.log(foo, zoo, bar);
export * from './dep1';
export { aa, bb, default as f } from './dep';
"#
      );
    })
    .unwrap();

    let minified_exports_map = minified_module_exports_map.get(&module_id).unwrap();

    assert_eq!(
      minified_exports_map,
      &HashMap::from([
        ("world".to_string(), "dd".to_string(),),
        ("world1".to_string(), "f".to_string(),),
        ("hello1".to_string(), "aa".to_string(),),
        ("hello2".to_string(), "bb".to_string(),),
      ])
    );
  }
}
