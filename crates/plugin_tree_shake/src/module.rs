use std::{
  collections::{HashMap, HashSet},
  mem,
  slice::Iter,
};

use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  swc_common::{Globals, Mark},
  swc_ecma_ast::Id,
};
use farmfe_toolkit::script::swc_try_with::try_with;

use crate::statement_graph::{
  module_analyze::{ItemId, Mode, ModuleAnalyze},
  ExportInfo, ExportSpecifierInfo, ImportInfo, StatementGraph, StatementId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedIdent {
  /// Local ident
  SwcIdent(String),
  /// Default ident
  Default,
  /// This ident is used and may be exported from other module
  InExportAll(String),
  /// All idents is used and may be exported from other module
  ExportAll,
}

impl ToString for UsedIdent {
  fn to_string(&self) -> String {
    match self {
      UsedIdent::SwcIdent(ident) => ident.to_string(),
      UsedIdent::Default => "default".to_string(),
      UsedIdent::InExportAll(ident) => ident.to_string(),
      UsedIdent::ExportAll => "*".to_string(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum UsedExports {
  All(HashMap<ModuleId, Vec<String>>),
  Partial(HashMap<ModuleId, Vec<String>>),
}

impl UsedExports {
  pub fn add_used_export(&mut self, module_id: &ModuleId, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports
          .entry(module_id.clone())
          .or_insert_with(Vec::new)
          .push(used_export.to_string());

        // if let UsedExports::Partial(self_used_exports) = self {
        //   if ns == used_export.to_string() {
        //     *self = UsedExports::All(self_used_exports.drain(..).collect());
        //   }
        // }
      }
    }
  }

  pub fn remove_used_export(&mut self, module_id: &ModuleId, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        let s = used_export.to_string();
        if !self_used_exports.contains_key(module_id) {
          return;
        }
        if let Some(pos) = self_used_exports[module_id]
          .iter()
          .position(|item| item == &s)
        {
          self_used_exports.get_mut(module_id).unwrap().remove(pos);

          // let ns = UsedIdent::Namespace.to_string();
          // if let UsedExports::All(self_used_exports) = self {
          //   if self_used_exports.iter().any(|item| item != &ns) {
          //     *self = UsedExports::Partial(self_used_exports.drain(..).collect());
          //   }
          // }
        }
      }
    };
  }

  pub fn remove_all_used_export(&mut self, module_id: &ModuleId) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports.remove(module_id);
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      UsedExports::All(_) => false,
      UsedExports::Partial(self_used_exports) => {
        !self_used_exports.values().any(|item| !item.is_empty())
      }
    }
  }

  pub fn change_all(&mut self) {
    match self {
      UsedExports::Partial(self_used_exports) => {
        *self = UsedExports::All(mem::take(self_used_exports));
      }
      // UsedExports::All(_) => todo!(),
      _ => {}
    }
  }

  pub fn to_string_vec(&self) -> Vec<String> {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        // let ns = UsedIdent::Namespace.to_string();
        self_used_exports
          .values()
          .flatten()
          .cloned()
          // .map(|ident| if ident == ns { "*".to_string() } else { ident })
          .collect()
      }
    }
  }

  pub fn idents(&mut self) -> HashSet<&String> {
    match self {
      UsedExports::Partial(self_used_exports) => {
        self_used_exports.values().flatten().collect::<HashSet<_>>()
      }
      _ => Default::default(),
    }
  }
}

pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  pub contains_self_executed_stmt: bool,
  pub is_self_executed_import: bool,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    farmfe_core::farm_profile_function!(format!(
      "TreeShakeModule::new {:?}",
      module.id.to_string()
    ));
    let module_system = module.meta.as_script().module_system.clone();

    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = if module_system == ModuleSystem::EsModule {
      StatementGraph::new(ast)
    } else {
      StatementGraph::empty()
    };

    // 2. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All(Default::default())
    } else {
      UsedExports::Partial(Default::default())
    };

    Self {
      module_id: module.id.clone(),
      contains_self_executed_stmt: stmt_graph.contains_self_executed_stmt(),
      stmt_graph,
      used_exports,
      side_effects: module.side_effects,
      module_system,
      is_self_executed_import: false,
    }
  }

  pub fn imports(&self) -> Vec<ImportInfo> {
    let mut imports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(import) = &stmt.import_info {
        imports.push(import.clone());
      }
    }

    imports
  }

  pub fn exports(&self) -> Vec<ExportInfo> {
    let mut exports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(export) = &stmt.export_info {
        exports.push(export.clone());
      }
    }

    exports
  }

  pub fn used_statements(
    &self,
    module: &mut Module,
    globals: &Globals,
  ) -> HashMap<StatementId, HashSet<String>> {
    println!("used_statements: {}", self.module_id.to_string());
    farmfe_core::farm_profile_function!(format!(
      "used_statements {:?}",
      self.module_id.to_string()
    ));
    // 通过 used_exports_idents 获取到所有的 ident
    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();

    if used_exports_idents.is_empty() && !self.is_self_executed_import {
      return HashMap::new();
    }

    println!("used_exports_idents: {:#?}", used_exports_idents);
    let mut stmt_used_idents_map = HashMap::new();

    for (used_ident, stmt_id) in used_exports_idents {
      let used_idents = stmt_used_idents_map
        .entry(stmt_id)
        .or_insert(HashSet::new());
      used_idents.insert(used_ident);
    }

    {
      farmfe_core::farm_profile_scope!(format!(
        "analyze self executed stmts {:?}",
        self.module_id.to_string()
      ));
      let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
      let (ids, items_map) = ModuleAnalyze::analyze(module);
      let mut stmt_graph = ModuleAnalyze::new();
      let mut entries: HashSet<ItemId> = HashSet::new();

      // print ids and items_map

      let print_items = |headers: &'static str, iter: Iter<Id>| {
        if iter.is_empty() {
          return;
        }

        print!("    {}", headers);

        for (index, var_decls) in iter.enumerate() {
          print!(
            "{}",
            format!(
              "{:?} {}",
              var_decls.0.to_string(),
              if index > 0 { "" } else { ", " }
            )
          );
        }

        print!("\n");
      };

      // print_items("reads: ", &|item| &item.read_vars);

      for id in &ids {
        let item = &items_map[id];
        match id {
          ItemId::Item { index, kind } => println!("index: {} kind: {:?}", index, kind),
        }

        print_items("var_decls: ", item.var_decls.iter());

        print_items("reads: ", item.read_vars.iter());

        print_items("writes: ", item.write_vars.iter());

        print_items("nested_read_vars: ", item.nested_read_vars.iter());

        print_items("nested_write_vars: ", item.nested_write_vars.iter());

        if item.side_effects {
          println!("    side_effect")
        }

        print!("\n")
      }

      stmt_graph.connect(&items_map, &ids);

      // global side used global variables
      try_with(Default::default(), globals, || {
        let side_effect_ids = stmt_graph.global_reference(&ids, &items_map, unresolved_mark);
        println!("___side_effect_ids: {:#?}", side_effect_ids);
        entries.extend(side_effect_ids);
      })
      .unwrap();

      entries.extend(stmt_graph.reference_side_effect_ids(&ids, &items_map));
      println!(
        "___reference_side_effect_ids: {:#?}",
        stmt_graph.reference_side_effect_ids(&ids, &items_map)
      );

      // self executed stmts
      for index in self.stmt_graph.stmts().iter().filter_map(|stmt| {
        if stmt.is_self_executed {
          Some(stmt.id)
        } else {
          None
        }
      }) {
        println!(
          "___self_executed: stmt:{} {:?}",
          index,
          stmt_graph.items(&index)
        );
        entries.extend(stmt_graph.items(&index));
      }

      stmt_graph.print_graph();

      let mut used_self_execute_stmts = HashSet::new();

      // entries.extend(stmt_used_idents_map.keys());

      // println!("___stmt_graph: {:#?}", self.stmt_graph.stmts());

      for stmt in stmt_used_idents_map.keys() {
        entries.extend(stmt_graph.items(stmt))
      }

      let write_stmt = stmt_graph.write_edges();

      println!("___entries: {:?}", entries,);

      fn dfs<'a>(
        entry: &'a ItemId,
        stack: &mut Vec<&'a ItemId>,
        result: &mut Vec<Vec<ItemId>>,
        visited: &mut HashSet<&'a ItemId>,
        module_define_graph: &'a ModuleAnalyze,
        stmt_graph: &StatementGraph,
      ) {
        stack.push(entry);

        let edges = module_define_graph.reference_edges(entry);
        println!(
          "___index: {}, visited: {:?}, edges: {:#?}",
          entry.index(),
          edges.is_empty()
            || visited.contains(entry)
            || !module_define_graph.has_node(entry)
            || !edges.iter().any(|(_, mode)| matches!(mode, Mode::Read)),
          edges
        );

        if edges.is_empty()
          || visited.contains(entry)
          || !module_define_graph.has_node(entry)
          || !edges.iter().any(|(_, mode)| matches!(mode, Mode::Read))
        {
          result.push(stack.iter().map(|item| (*item).clone()).collect());
        } else {
          visited.insert(entry);
          for (node, mode) in edges {
            match mode {
              Mode::Read => {
                dfs(
                  node,
                  stack,
                  result,
                  visited,
                  module_define_graph,
                  stmt_graph,
                );
              }
              _ => {}
            }
          }
        }

        stack.pop();
      }

      let mut visited = HashSet::new();
      let mut reference_chain = Vec::new();
      for stmt in &entries {
        dfs(
          stmt,
          &mut vec![],
          &mut reference_chain,
          &mut visited,
          &stmt_graph,
          &self.stmt_graph,
        );
      }

      println!("___reference_chain: {:#?}", reference_chain);

      for chain in reference_chain {
        used_self_execute_stmts.extend(
          &chain
            .iter()
            .filter(|item| self.stmt_graph.stmt(&item.index()).import_info.is_none())
            .map(|item| item.index())
            .collect::<Vec<_>>(),
        );
      }

      println!("___write_stmt: {:?}", write_stmt);

      for (source, target) in write_stmt {
        if used_self_execute_stmts.contains(&target.index()) {
          used_self_execute_stmts.insert(source.index());
        }
      }

      println!("___used_self_execute_stmts: {:?}", used_self_execute_stmts);

      for stmt_id in used_self_execute_stmts {
        stmt_used_idents_map
          .entry(stmt_id)
          .or_insert(HashSet::new());

        let dep_stmts = self.stmt_graph.dependencies(&stmt_id);
        // let mut is_contain_stmts = vec![];
        for (dep_stmt, referred_idents) in dep_stmts {
          let used_idents = stmt_used_idents_map
            .entry(dep_stmt.id)
            .or_insert(HashSet::new());
          used_idents.extend(referred_idents.into_iter().map(UsedIdent::SwcIdent));
        }
      }
    }

    println!("___stmt_used_idents_map: {:?}", stmt_used_idents_map);

    // 2. analyze used statements starting from used exports

    self
      .stmt_graph
      .analyze_used_statements_and_idents(stmt_used_idents_map)
  }

  pub fn used_exports_idents(&self) -> Vec<(UsedIdent, StatementId)> {
    farmfe_core::farm_profile_function!(format!(
      "used_exports_idents {:?}",
      self.module_id.to_string()
    ));
    println!(
      "used_exports: {} {:#?}",
      self.module_id.to_string(),
      self.used_exports
    );

    match &self.used_exports {
      UsedExports::All(_) => {
        // all exported identifiers are used
        let mut used_idents = vec![];

        for export_info in self.exports() {
          for sp in export_info.specifiers {
            match sp {
              ExportSpecifierInfo::Default => {
                used_idents.push((UsedIdent::Default, export_info.stmt_id));
              }
              ExportSpecifierInfo::Named { local, .. } => {
                used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::Namespace(ns) => {
                used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::All(_) => {
                used_idents.push((UsedIdent::ExportAll, export_info.stmt_id));
              }
            }
          }
        }

        used_idents
      }
      UsedExports::Partial(idents) => {
        let mut used_idents = vec![];

        let idents = idents.values().flatten().collect::<Vec<_>>();
        // if idents.contains(&UsedIdent::ExportAll.to_string()) {
        //   // all exported identifiers are used
        //   for export_info in self.exports() {

        //   }

        //   return used_idents;
        // };

        for ident in &idents {
          // find the export info that contains the ident
          let export_info = self.exports().into_iter().find(|export_info| {
            export_info.specifiers.iter().any(|sp| match sp {
              ExportSpecifierInfo::Default => *ident == "default",
              ExportSpecifierInfo::Named { local, exported } => {
                let exported_ident = if let Some(exported) = exported {
                  exported
                } else {
                  local
                };

                is_ident_equal(ident, exported_ident)
              }
              ExportSpecifierInfo::Namespace(ns) => is_ident_equal(ident, ns),
              ExportSpecifierInfo::All(_) => {
                /* Deal with All later */
                false
              }
            })
          });

          if let Some(export_info) = export_info {
            for sp in export_info.specifiers {
              match sp {
                ExportSpecifierInfo::Default => {
                  if *ident == "default" {
                    used_idents.push((UsedIdent::Default, export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Named { local, exported } => {
                  if let Some(exported) = exported {
                    if is_ident_equal(ident, &exported) {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                  } else if is_ident_equal(ident, &local) {
                    used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  if is_ident_equal(ident, &ns) {
                    used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::All(_) => {
                  unreachable!()
                }
              }
            }
          } else {
            let contain_all = idents.contains(&&UsedIdent::ExportAll.to_string());
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            for export_info in self.exports() {
              if export_info
                .specifiers
                .iter()
                .any(|sp| matches!(sp, ExportSpecifierInfo::All(_)))
              {
                let stmt_id = export_info.stmt_id;
                used_idents.push((UsedIdent::InExportAll(ident.to_string()), stmt_id));
              } else if contain_all {
                for sp in export_info.specifiers {
                  match sp {
                    ExportSpecifierInfo::Default => {
                      used_idents.push((UsedIdent::Default, export_info.stmt_id));
                    }
                    ExportSpecifierInfo::Named { local, .. } => {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                    ExportSpecifierInfo::Namespace(ns) => {
                      used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                    }
                    ExportSpecifierInfo::All(_) => {
                      used_idents.push((UsedIdent::ExportAll, export_info.stmt_id));
                    }
                  }
                }
              }
            }
          }
        }

        used_idents
      }
    }
  }
}

fn is_ident_equal(ident1: &String, ident2: &String) -> bool {
  let split1 = ident1.split('#').collect::<Vec<_>>();
  let split2 = ident2.split('#').collect::<Vec<_>>();

  if split1.len() == 2 && split2.len() == 2 {
    split1[0] == split2[0] && split1[1] == split2[1]
  } else {
    split1[0] == split2[0]
  }
}
