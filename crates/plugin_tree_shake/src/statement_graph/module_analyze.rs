use std::collections::HashMap;
use std::hash::Hash;

use farmfe_core::module::Module;
use farmfe_core::petgraph::visit::EdgeRef;
use farmfe_core::petgraph::Direction::Outgoing;
use farmfe_core::swc_common::Mark;
use farmfe_core::swc_ecma_ast::{
  op, AssignTarget, BlockStmtOrExpr, Class, Decl, DefaultDecl, ExportDecl, ExportDefaultDecl, Expr,
  ExprStmt, Id, ImportSpecifier, MemberProp, ModuleDecl, PropName, Stmt,
};
use farmfe_core::{
  petgraph::{self, stable_graph::NodeIndex},
  swc_ecma_ast::ModuleItem,
};
use farmfe_toolkit::swc_ecma_utils::find_pat_ids;
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
  Item { index: usize, kind: ItemIdType },
}

impl ItemId {
  pub fn index(&self) -> usize {
    match self {
      ItemId::Item { index, kind: _ } => index.clone(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemIdType {
  Var(usize),
  Import(usize),
  Normal,
}

#[derive(Debug, Default, Clone)]
pub struct ItemData {
  pub var_decls: Vec<Id>,
  pub read_vars: Vec<Id>,
  pub write_vars: Vec<Id>,
  pub nested_read_vars: Vec<Id>,
  pub nested_write_vars: Vec<Id>,
  pub side_effects: bool,
  pub side_effect_call: Vec<Id>,
}

fn class_collection(class: &Class) -> ItemData {
  let collect = collect_all_usage(class, None);

  let data = ItemData {
    var_decls: vec![],
    read_vars: collect.vars.read,
    write_vars: collect.vars.write,
    nested_read_vars: collect.vars.nested_read,
    nested_write_vars: collect.vars.nested_write,
    side_effect_call: collect.call_reads,
    ..Default::default()
  };

  data
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct ModuleAnalyzeItemEdge {
  pub mode: Mode,
  pub nested: bool,
}

pub struct ModuleAnalyze {
  g: petgraph::stable_graph::StableDiGraph<ItemId, ModuleAnalyzeItemEdge>,
  id_map: HashMap<ItemId, NodeIndex>,
  vars: HashMap<Id, ItemId>,
  index_map: HashMap<usize, Vec<ItemId>>,
}

impl ModuleAnalyze {
  pub fn new() -> Self {
    Self {
      g: petgraph::stable_graph::StableDiGraph::new(),
      id_map: HashMap::new(),
      vars: HashMap::new(),
      index_map: HashMap::new(),
    }
  }

  fn add_edge(&mut self, from: ItemId, to: ItemId, edge: ModuleAnalyzeItemEdge) {
    if !self.id_map.contains_key(&from) {
      self
        .id_map
        .insert(from.clone(), self.g.add_node(from.clone()));
    }

    if !self.id_map.contains_key(&to) {
      self.id_map.insert(to.clone(), self.g.add_node(to.clone()));
    }

    let from = self.id_map[&from];
    let to = self.id_map[&to];

    self.g.add_edge(from, to, edge);
  }

  pub fn reference_edges(&self, n: &ItemId) -> Vec<(&ItemId, &ItemId, ModuleAnalyzeItemEdge)> {
    if self.id_map.contains_key(n) {
      let mut walk = self.g.neighbors_undirected(self.id_map[n]).detach();

      let mut res = vec![];

      while let Some((edge_index, _)) = walk.next(&self.g) {
        let (source, target) = self.g.edge_endpoints(edge_index).unwrap();

        let edge = self.g.edge_weight(edge_index).unwrap().clone();

        res.push((&self.g[source], &self.g[target], edge));
      }

      res
    } else {
      vec![]
    }
  }

  pub fn has_node(&self, n: &ItemId) -> bool {
    self.id_map.contains_key(n)
  }

  pub fn analyze(module: &Module) -> (Vec<ItemId>, HashMap<ItemId, ItemData>) {
    let ast = &module.meta.as_script().ast;

    // let mut graph = petgraph::stable_graph::StableGraph::new();
    let mut items_map = HashMap::new();
    let mut ids = Vec::new();

    for (index, stmt) in ast.body.iter().enumerate() {
      match stmt {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import_info)) => {
          for (i, specify) in import_info.specifiers.iter().enumerate() {
            let id = match specify {
              ImportSpecifier::Named(named) => named.local.to_id(),
              ImportSpecifier::Default(default) => default.local.to_id(),
              ImportSpecifier::Namespace(ns) => ns.local.to_id(),
            };

            // Imports are considered side-effect variables by default
            let data = ItemData {
              var_decls: vec![id],
              side_effects: true,
              ..Default::default()
            };

            let id = ItemId::Item {
              index,
              kind: ItemIdType::Import(i),
            };
            ids.push(id.clone());
            items_map.insert(id, data);
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
          decl: Decl::Fn(f), ..
        }))
        | ModuleItem::Stmt(Stmt::Decl(Decl::Fn(f))) => {
          let id = f.ident.to_id();

          let collect = collect_all_usage(&f.function, None);

          let is_side_effect_fn = collect
            .vars
            .write
            .iter()
            .any(|id| !collect.vars.read.contains(id));

          let data = ItemData {
            var_decls: vec![id],
            read_vars: collect.vars.read,
            write_vars: collect.vars.write,
            nested_read_vars: collect.vars.nested_read,
            nested_write_vars: collect.vars.nested_write,
            side_effects: is_side_effect_fn,
            side_effect_call: collect.call_reads,
            ..Default::default()
          };

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };
          ids.push(id.clone());
          items_map.insert(id, data);
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
          decl: Decl::Var(vars),
          ..
        }))
        | ModuleItem::Stmt(Stmt::Decl(Decl::Var(vars))) => {
          for (i, decl) in vars.decls.iter().enumerate() {
            let decl_ids: Vec<Id> = find_pat_ids(&decl.name);

            let collect_ident = collect_all_usage(&decl.init, None);

            let data = ItemData {
              var_decls: decl_ids.clone(),
              read_vars: collect_ident.vars.read,
              write_vars: [collect_ident.vars.write, decl_ids].concat(),
              nested_read_vars: collect_ident.vars.nested_read,
              nested_write_vars: collect_ident.vars.nested_write,
              side_effect_call: collect_ident.call_reads,
              ..Default::default()
            };

            let id = ItemId::Item {
              index,
              kind: ItemIdType::Var(i),
            };

            ids.push(id.clone());
            items_map.insert(id, data);
          }
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
          decl: DefaultDecl::Fn(f),
          ..
        })) => {
          let id = f.ident.clone().map(|ident| ident.to_id());

          let collect = collect_all_usage(&f.function, None);

          let is_side_effect_fn = collect
            .vars
            .write
            .iter()
            .any(|id| !collect.vars.read.contains(id));

          let data = ItemData {
            var_decls: id.map(|id| vec![id]).unwrap_or_default(),
            nested_read_vars: collect.vars.read,
            nested_write_vars: collect.vars.write,
            side_effects: is_side_effect_fn,
            side_effect_call: collect.call_reads,
            ..Default::default()
          };

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };
          ids.push(id.clone());
          items_map.insert(id, data);
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
          decl: DefaultDecl::Class(class),
          ..
        })) => {
          let mut data = class_collection(&class.class);

          if let Some(ident) = &class.ident {
            data.var_decls.push(ident.to_id());
          }

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };

          ids.push(id.clone());

          items_map.insert(id, data);
        }

        ModuleItem::Stmt(Stmt::Decl(Decl::Class(class)))
        | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
          decl: Decl::Class(class),
          ..
        })) => {
          let id = class.ident.to_id();
          let mut data = class_collection(&class.class);

          data.var_decls.push(id);

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };

          ids.push(id.clone());
          items_map.insert(id, data);
        }

        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(_))
        | ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(_)) => {
          let used_ids = collect_usage_ignore_nested(stmt, Some(Mode::Read));
          let data = ItemData {
            read_vars: used_ids.vars.read,
            write_vars: used_ids.vars.write,
            ..Default::default()
          };

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };
          ids.push(id.clone());
          items_map.insert(id, data);
        }

        ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          expr: box Expr::Assign(assign),
          ..
        })) => {
          let mut used_ident = collect_all_usage(stmt, None);

          if assign.op != op!("=") {
            let extra_ids = collect_usage_ignore_nested(&assign.left, None);
            used_ident.vars.read.extend(extra_ids.vars.read);
            used_ident.vars.write.extend(extra_ids.vars.write);
            used_ident
              .vars
              .nested_read
              .extend(extra_ids.vars.nested_read);
            used_ident
              .vars
              .nested_write
              .extend(extra_ids.vars.nested_write);
          }

          let data = ItemData {
            read_vars: used_ident.vars.read,
            write_vars: used_ident.vars.write,
            nested_read_vars: used_ident.vars.nested_read,
            nested_write_vars: used_ident.vars.nested_write,
            side_effect_call: used_ident.call_reads,
            ..Default::default()
          };

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };
          ids.push(id.clone());
          items_map.insert(id, data);
        }

        _ => {
          let used_ids = collect_all_usage(stmt, None);

          let data = ItemData {
            read_vars: used_ids.vars.read,
            write_vars: used_ids.vars.write,
            nested_read_vars: used_ids.vars.nested_read,
            nested_write_vars: used_ids.vars.nested_write,
            side_effect_call: used_ids.call_reads,
            ..Default::default()
          };

          let id = ItemId::Item {
            index,
            kind: ItemIdType::Normal,
          };
          ids.push(id.clone());
          items_map.insert(id, data);
        }
      }
    }

    (ids, items_map)
  }

  pub fn connect(&mut self, items_map: &HashMap<ItemId, ItemData>, ids: &Vec<ItemId>) {
    for item_id in ids {
      self
        .index_map
        .entry(item_id.index())
        .or_insert_with(Vec::new)
        .push(item_id.clone());

      let item = &items_map[&item_id];
      for decl in &item.var_decls {
        self
          .vars
          .entry(decl.clone())
          .or_insert_with(|| item_id.clone());
      }
    }

    for item_id in ids {
      let item = &items_map[&item_id];

      for read in &item.read_vars {
        if let Some(read_id) = self.vars.get(read).cloned() {
          if *item_id == read_id {
            continue;
          }
          self.add_edge(
            item_id.clone(),
            read_id,
            ModuleAnalyzeItemEdge {
              mode: Mode::Read,
              ..Default::default()
            },
          );
        }
      }

      for write in &item.write_vars {
        if let Some(write_id) = self.vars.get(write).cloned() {
          if *item_id == write_id {
            continue;
          }
          self.add_edge(
            item_id.clone(),
            write_id,
            ModuleAnalyzeItemEdge {
              mode: Mode::Write,
              ..Default::default()
            },
          );
        }
      }

      for nested_read in &item.nested_read_vars {
        if let Some(nested_read_id) = self.vars.get(nested_read).cloned() {
          self.add_edge(
            item_id.clone(),
            nested_read_id,
            ModuleAnalyzeItemEdge {
              mode: Mode::Read,
              nested: true,
            },
          );
        }
      }

      for nested_write in &item.nested_write_vars {
        if let Some(nested_write_id) = self.vars.get(nested_write).cloned() {
          self.add_edge(
            item_id.clone(),
            nested_write_id,
            ModuleAnalyzeItemEdge {
              mode: Mode::Write,
              nested: true,
            },
          );
        }
      }
    }
  }

  pub fn items(&self, index: &usize) -> Vec<ItemId> {
    self
      .index_map
      .get(index)
      .map(|ids| ids.clone())
      .unwrap_or_default()
  }

  pub fn global_reference(
    &self,
    item_id: &Vec<ItemId>,
    items_map: &HashMap<ItemId, ItemData>,
    unresolved_mark: Mark,
  ) -> Vec<ItemId> {
    let mut side_effect_ids: Vec<ItemId> = vec![];
    for id in item_id {
      let item = &items_map[id];

      if item
        .write_vars
        .iter()
        .chain(item.read_vars.iter())
        .any(|write| write.1.outer() == unresolved_mark)
      {
        side_effect_ids.push(id.clone());
      };
    }

    return side_effect_ids;
  }

  ///
  /// ```js
  /// var a = 10;
  ///
  /// function foo() {
  ///   a = 20;
  /// }
  ///
  /// // side_effect
  /// var b = foo();
  ///
  /// export default a;
  /// ```
  ///
  pub fn reference_side_effect_ids(
    &self,
    ids: &Vec<ItemId>,
    items_map: &HashMap<ItemId, ItemData>,
  ) -> Vec<ItemId> {
    let mut side_effect_ids: Vec<ItemId> = vec![];

    for id in ids {
      if !self.has_node(id) {
        continue;
      }
      let source = &items_map[id];
      source.side_effect_call.iter().for_each(|side_effect_id| {
        if source.read_vars.contains(side_effect_id)
          && !source.nested_read_vars.contains(side_effect_id)
        {
          side_effect_ids.push(id.clone());
        }
      });
    }

    side_effect_ids
  }

  pub fn edge(&self, from: &ItemId, to: &ItemId) -> Option<&ModuleAnalyzeItemEdge> {
    self
      .g
      .edges_connecting(
        *self.id_map.get(from).unwrap(),
        *self.id_map.get(to).unwrap(),
      )
      .next()
      .map(|e| e.weight())
  }

  #[allow(unused)]
  pub fn print_graph(&mut self) {
    println!("nodes: ");
    for node in self.g.node_indices() {
      let item = &self.g[node];
      println!("{:?}", item);
    }

    println!("edges: ");
    for node in self.g.node_indices() {
      let edges = self.g.edges_directed(node, Outgoing);
      for edge in edges {
        let from = &self.g[edge.source()];
        let to = &self.g[edge.target()];
        println!("{:?} -> {:?} {:?}", from, to, edge.weight().mode);
      }
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Mode {
  Read,
  #[default]
  Write,
}

#[derive(Debug, Default)]
struct Vars {
  read: Vec<Id>,
  write: Vec<Id>,
  nested_read: Vec<Id>,
  nested_write: Vec<Id>,
}

fn collect_usage_ignore_nested<N>(n: &N, c: Option<Mode>) -> CollectIdent
where
  N: VisitWith<CollectIdent>,
{
  let mut v = CollectIdent {
    ignore_nested: true,
    vars: Vars {
      read: vec![],
      write: vec![],
      nested_read: vec![],
      nested_write: vec![],
    },
    mode: c.unwrap_or(Mode::Write),
    enforce: None,
    ..Default::default()
  };

  n.visit_with(&mut v);

  v
}

fn collect_all_usage<N>(n: &N, c: Option<Mode>) -> CollectIdent
where
  N: VisitWith<CollectIdent>,
{
  let mut v = CollectIdent {
    ignore_nested: false,
    vars: Vars {
      read: vec![],
      write: vec![],
      nested_read: vec![],
      nested_write: vec![],
    },
    mode: c.unwrap_or(Mode::Write),
    enforce: None,
    ..Default::default()
  };

  n.visit_with(&mut v);

  v
}

#[derive(Debug, Default)]
struct CollectIdent {
  ignore_nested: bool,
  pub vars: Vars,
  mode: Mode,
  enforce: Option<Mode>,
  call: bool,
  call_reads: Vec<Id>,
  nested: bool,
}

impl CollectIdent {
  fn with_mode<F>(&mut self, mode: Mode, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.mode.clone();

    self.mode = mode;
    f(self);
    self.mode = prev;
  }

  fn with_enforce<F>(&mut self, mode: Mode, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.enforce.clone();

    self.enforce = Some(mode);
    f(self);
    self.enforce = prev;
  }

  fn with_call<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.call;

    self.call = true;
    f(self);
    self.call = prev;
  }

  fn with_nested<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.nested;

    self.nested = true;
    f(self);
    self.nested = prev;
  }
}
impl Visit for CollectIdent {
  fn visit_block_stmt_or_expr(&mut self, n: &BlockStmtOrExpr) {
    if self.ignore_nested {
      return;
    }

    self.with_nested(|this| n.visit_children_with(this));
  }

  fn visit_constructor(&mut self, n: &farmfe_core::swc_ecma_ast::Constructor) {
    if self.ignore_nested {
      return;
    }

    self.with_nested(|this| n.visit_children_with(this));
  }

  fn visit_function(&mut self, n: &farmfe_core::swc_ecma_ast::Function) {
    if self.ignore_nested {
      return;
    }

    self.with_nested(|this| n.visit_children_with(this));
  }

  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if self.call {
      self.call_reads.push(n.to_id());
    }

    match self.enforce.as_ref().unwrap_or(&self.mode) {
      Mode::Read => {
        if self.nested {
          self.vars.nested_read.push(n.to_id());
        } else {
          self.vars.read.push(n.to_id())
        }
      }
      Mode::Write => {
        if self.nested {
          self.vars.nested_write.push(n.to_id())
        } else {
          self.vars.write.push(n.to_id())
        }
      }
    }
  }

  fn visit_expr(&mut self, e: &Expr) {
    self.with_mode(Mode::Read, |this| e.visit_children_with(this));
  }

  fn visit_pat(&mut self, p: &farmfe_core::swc_ecma_ast::Pat) {
    self.with_enforce(Mode::Write, |this| {
      this.with_mode(Mode::Write, |this| p.visit_children_with(this));
    });
  }

  fn visit_assign_target(&mut self, n: &AssignTarget) {
    self.with_enforce(Mode::Write, |this| {
      n.visit_children_with(this);
    });
  }

  fn visit_member_prop(&mut self, n: &MemberProp) {
    if let MemberProp::Computed(..) = n {
      n.visit_children_with(self);
    }
  }

  fn visit_prop_name(&mut self, n: &PropName) {
    if let PropName::Computed(..) = n {
      n.visit_children_with(self);
    }
  }

  fn visit_callee(&mut self, n: &farmfe_core::swc_ecma_ast::Callee) {
    match n {
      farmfe_core::swc_ecma_ast::Callee::Expr(expr) => {
        self.with_call(|this| expr.visit_children_with(this))
      }
      _ => {}
    }
  }

  fn visit_new_expr(&mut self, n: &farmfe_core::swc_ecma_ast::NewExpr) {
    self.with_call(|this| n.callee.visit_children_with(this))
  }
}
