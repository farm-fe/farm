use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  mem,
};

use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  swc_common::{Globals, Mark},
  swc_ecma_ast::Ident,
};
use farmfe_toolkit::script::swc_try_with::try_with;

use crate::statement_graph::{
  module_analyze::{ItemId, ItemIdType, Mode, ModuleAnalyze, ModuleAnalyzeItemEdge},
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
      _ => {}
    }
  }

  pub fn to_string_vec(&self) -> Vec<String> {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports.values().flatten().cloned().collect()
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
    module: &Module,
    globals: &Globals,
  ) -> HashMap<StatementId, HashSet<String>> {
    farmfe_core::farm_profile_function!(format!(
      "used_statements {:?}",
      self.module_id.to_string()
    ));

    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();

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
      let (ids, mut items_map) = ModuleAnalyze::analyze(module);
      let mut module_analyze = ModuleAnalyze::new();
      let mut entries: HashSet<ItemId> = HashSet::new();

      // remove unused export specifiers
      for (stmt, used_idents) in stmt_used_idents_map.iter() {
        let default_str = UsedIdent::Default.to_string();
        let used_idents_string = used_idents
          .iter()
          .filter_map(|ident| match ident {
            UsedIdent::SwcIdent(ident) => Some(ident),
            UsedIdent::Default => Some(&default_str),
            _ => None,
          })
          .collect::<HashSet<_>>();

        let is_need_removed = used_idents.iter().any(|ident| match ident {
          // need remove
          UsedIdent::SwcIdent(_) => true,
          // save all
          UsedIdent::Default => true,
          _ => false,
        });

        if !is_need_removed
          || (!used_idents_string.is_empty() && used_idents.contains(&UsedIdent::Default))
        {
          continue;
        }

        for item_id in ids.iter().filter(|item_id| &item_id.index() == stmt) {
          let item = items_map.get_mut(item_id).unwrap();

          let mut removed_idents = item
            .read_vars
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
              if !used_idents_string.contains(&Ident::from(id.clone()).to_string()) {
                Some(index)
              } else {
                None
              }
            })
            .collect::<Vec<_>>();

          removed_idents.sort();
          removed_idents.reverse();

          for index in removed_idents {
            item.read_vars.remove(index);
          }
        }
      }

      // connect stmt's node
      module_analyze.connect(&items_map, &ids);

      // used global ident
      try_with(Default::default(), globals, || {
        let side_effect_ids = module_analyze.global_reference(&ids, &items_map, unresolved_mark);
        entries.extend(side_effect_ids);
      })
      .unwrap();

      // effect stmt
      entries.extend(module_analyze.reference_side_effect_ids(&ids, &items_map));

      // self executed stmts
      for index in self.stmt_graph.stmts().iter().filter_map(|stmt| {
        if stmt.is_self_executed {
          Some(stmt.id)
        } else {
          None
        }
      }) {
        entries.extend(module_analyze.items(&index));
      }

      // used export need to be the entry point to traversal
      for stmt in stmt_used_idents_map.keys() {
        entries.extend(module_analyze.items(stmt))
      }

      fn dfs<'a>(
        entry: &'a ItemId,
        stack: &mut Vec<&'a ItemId>,
        result: &mut HashSet<ItemId>,
        visited: &mut HashSet<&'a ItemId>,
        module_define_graph: &'a ModuleAnalyze,
        stmt_graph: &StatementGraph,
        reverse_terser_chain: &mut Vec<Mode>,
      ) {
        let collection_result = |reverse_terser_chain: &mut Vec<Mode>,
                                 result: &mut HashSet<ItemId>,
                                 stack: &mut Vec<&'a ItemId>| {
          println!(
            "reverse_terser_chain: {:?}, stack: {:?}",
            reverse_terser_chain,
            stack.iter().map(|item| item.index()).collect::<Vec<_>>()
          );
          if !reverse_terser_chain.is_empty() {
            if reverse_terser_chain
              .iter()
              .any(|mode| matches!(mode, Mode::Write))
            {
              result.extend(stack.iter().map(|item| (*item).clone()).collect::<Vec<_>>());
            }
          } else {
            result.extend(stack.iter().map(|item| (*item).clone()).collect::<Vec<_>>());
          }
        };

        let break_tenser = |stack: &mut Vec<&'a ItemId>| {
          stack.pop();
        };

        if visited.contains(entry) {
          collection_result(reverse_terser_chain, result, stack);
          break_tenser(stack);
          return;
        }

        stack.push(entry);

        if !module_define_graph.has_node(entry) {
          collection_result(reverse_terser_chain, result, stack);
          break_tenser(stack);
          return;
        }

        println!("\n\nentry: {:?} {:?}", entry.index(), reverse_terser_chain);

        let edges = module_define_graph.reference_edges(entry);

        println!("edges: {:?}", edges);

        if edges.is_empty() || edges.iter().all(|(source, target, _)| source == target) {
          collection_result(reverse_terser_chain, result, stack);
        } else {
          visited.insert(entry);
          let push_reverse_terser_chain =
            |reverse_terser_chain: &mut Vec<Mode>, mode: Mode, create| {
              if !reverse_terser_chain.is_empty() || create {
                reverse_terser_chain.push(mode);
              }
            };

          let pop_reverse_terser_chain = |reverse_terser_chain: &mut Vec<Mode>| {
            reverse_terser_chain.pop();
          };

          for (source, target, edge) in edges {
            if source == target {
              continue;
            }
            println!(
              "source: {}, target: {}, edge: {:?}",
              source.index(),
              target.index(),
              edge
            );
            match (edge.mode, edge.nested) {
              (Mode::Read, _) => {
                // push_reverse_terser_chain(reverse_terser_chain, Mode::Read, true);
                push_reverse_terser_chain(reverse_terser_chain, Mode::Read, false);
                dfs(
                  target,
                  stack,
                  result,
                  visited,
                  module_define_graph,
                  stmt_graph,
                  reverse_terser_chain,
                );
                pop_reverse_terser_chain(reverse_terser_chain);

                // ignore nested
                // cache -> readCache { cache }
                //
                //```js
                // const cache = {};
                //
                // function readCache(key) {
                //   cache[key]
                // }
                //```
                //

                // ignore bar
                //
                // foo:2 -> foo:1,bar:1 -> bar:3
                // ```js
                // // bar.js
                // 1: import { bar, foo } from './foo';
                // 2: foo()
                // 3: export { bar, foo }
                //```

                if !edge.nested
                  && !matches!(
                    target,
                    ItemId::Item {
                      kind: ItemIdType::Import(_),
                      ..
                    }
                  )
                {
                  push_reverse_terser_chain(reverse_terser_chain, Mode::Read, true);
                  dfs(
                    source,
                    stack,
                    result,
                    visited,
                    module_define_graph,
                    stmt_graph,
                    reverse_terser_chain,
                  );
                  pop_reverse_terser_chain(reverse_terser_chain);
                }
              }

              (Mode::Write, false) => {
                push_reverse_terser_chain(reverse_terser_chain, Mode::Write, false);
                dfs(
                  source,
                  stack,
                  result,
                  visited,
                  module_define_graph,
                  stmt_graph,
                  reverse_terser_chain,
                );
                pop_reverse_terser_chain(reverse_terser_chain);
              }

              (Mode::Write, true) => {
                push_reverse_terser_chain(reverse_terser_chain, Mode::Write, false);
                dfs(
                  target,
                  stack,
                  result,
                  visited,
                  module_define_graph,
                  stmt_graph,
                  reverse_terser_chain,
                );
                pop_reverse_terser_chain(reverse_terser_chain);
              }
            }
          }
        }

        break_tenser(stack);
      }

      let mut visited = HashSet::new();
      let mut reference_chain = HashSet::new();

      let mut entries = entries.into_iter().collect::<Vec<_>>();

      entries.sort_by(|a, b| a.index().cmp(&b.index()));

      println!("entries: {:#?}", entries);

      for stmt in &entries {
        dfs(
          stmt,
          &mut vec![],
          &mut reference_chain,
          &mut visited,
          &module_analyze,
          &self.stmt_graph,
          &mut vec![],
        );
      }

      println!("reference_chain: {:?}", reference_chain);

      let reference_stmts = reference_chain
        .iter()
        .map(|item| item.index())
        .collect::<HashSet<_>>();

      let mut used_self_execute_stmts = HashSet::new();

      used_self_execute_stmts.extend(
        reference_stmts
          .into_iter()
          .filter(|index| self.stmt_graph.stmt(&index).import_info.is_none()),
      );

      // dependencies cannot be used to obtain the ident that export stmt depends on.
      for stmt in stmt_used_idents_map.keys() {
        used_self_execute_stmts.remove(stmt);
      }

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

        let contain_export_all = idents.contains(&&UsedIdent::ExportAll.to_string());
        if contain_export_all {
          // for all content introduced, all export information needs to be collected
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
          return used_idents;
        }

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
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            for export_info in self.exports() {
              if export_info
                .specifiers
                .iter()
                .any(|sp| matches!(sp, ExportSpecifierInfo::All(_)))
              {
                let stmt_id = export_info.stmt_id;
                used_idents.push((UsedIdent::InExportAll(ident.to_string()), stmt_id));
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
