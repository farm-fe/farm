use std::{
  cell::RefMut,
  collections::{HashMap, HashSet},
  path::PathBuf,
  rc::Rc,
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  farm_profile_function,
  module::{module_graph::ModuleGraph, Module, ModuleId, ModuleSystem},
  resource::resource_pot::ResourcePotId,
  swc_common::{Mark, SourceMap},
  swc_ecma_ast::{Id, Module as EcmaAstModule},
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  script::swc_try_with::{resolve_module_mark, try_with},
  swc_ecma_visit::VisitWith,
};

use crate::resource_pot_to_bundle::{
  bundle::reference::ReferenceMap, targets::cjs::CjsModuleAnalyzer, uniq_name::BundleVariable, Var,
};

use super::analyze::{self, CollectUnresolvedIdent};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StmtAction {
  ///
  ///
  /// ```ts
  /// export var foo = 1;
  /// // =>
  /// var foo = 1;
  ///
  /// export function foo() {}
  /// // =>
  /// function foo() {}
  ///
  /// ```
  ///
  ///
  StripExport(usize),

  ///
  /// ```ts
  /// export default function foo() {}
  /// export default class Foo() {}
  /// // =>
  /// function foo() {}
  /// class Foo() {}
  /// ```
  ///
  StripDefaultExport(usize, usize),
  ///
  /// ```ts
  /// export default 1 + 1;
  /// // =>
  /// var module_default = 1 + 1;
  /// ```
  DeclDefaultExpr(usize, usize),
  ///
  /// ```ts
  /// import { name } from "shulan";
  /// import person from "shulan";
  /// import * as personNs from "shulan"
  /// ```
  /// remove all
  ///
  RemoveImport(usize),
  ///
  /// ```ts
  /// import { foo as bar } from './cjs_module';
  /// import * as ns from './cjs_module';
  /// import cjs from './cjs_module';
  /// // =>
  /// remove
  /// // or
  ///
  /// ```
  StripCjsImport(usize, Option<ModuleId>),
  //
  // ```ts
  // export { name as cjsName } from "./cjs";
  // export { age as cjsAge } from "./cjs";
  // // =>
  // const { name, age } = require_cjs();
  // ```
  //
  // ReplaceCjsExport(ModuleId),
}

impl StmtAction {
  pub fn index(&self) -> Option<usize> {
    match self {
      StmtAction::StripExport(index) => Some(*index),
      StmtAction::StripDefaultExport(index, _) => Some(*index),
      StmtAction::DeclDefaultExpr(index, _) => Some(*index),
      StmtAction::RemoveImport(index) => Some(*index),
      StmtAction::StripCjsImport(index, _) => Some(*index),
      // StmtAction::ReplaceCjsExport(_) => None,
    }
  }
}

pub type StatementId = usize;

#[derive(Debug, Clone)]
// export { foo as bar }; Variable(foo, Some(bar))
// import { foo as bar }; Variable(bar, Some(foo))
pub struct Variable(pub usize, pub Option<usize>);

impl From<usize> for Variable {
  fn from(value: usize) -> Self {
    Variable(value, None)
  }
}

impl From<(usize, Option<usize>)> for Variable {
  fn from(value: (usize, Option<usize>)) -> Self {
    Variable(value.0, value.1)
  }
}

impl Variable {
  pub fn export_as(&self) -> usize {
    self.1.unwrap_or(self.0)
  }
  pub fn export_from(&self) -> usize {
    self.0
  }

  pub fn import_origin(&self) -> usize {
    self.1.unwrap_or(self.0)
  }

  pub fn local(&self) -> usize {
    self.0
  }

  pub fn rev(&self) -> Self {
    if let Some(b) = self.1 {
      Variable(b, Some(self.0))
    } else {
      Variable(self.0, None)
    }
  }
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
  pub source: ModuleId,
  pub specifiers: Vec<ImportSpecifierInfo>,
  pub stmt_id: StatementId,
}

// collect all exports and gathering them into a simpler structure
#[derive(Debug, Clone)]
pub enum ExportSpecifierInfo {
  /// ```js
  /// export * from 'foo';
  /// ```
  All(Option<usize>),
  /// ```js
  /// // (default, Some(zoo))
  /// export { foo, bar, default as zoo } from 'foo';
  ///
  /// export { foo, bar };
  ///
  /// export const foo = 'foo';
  /// ```
  Named(Variable),
  /// ```js
  /// export default n;
  /// export default 1 + 1;
  /// ```
  Default(usize),
  /// ```js
  /// export * as foo from 'foo';
  /// ```
  Namespace(usize),
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
  pub source: Option<ModuleId>,
  pub specifiers: Vec<ExportSpecifierInfo>,
  pub stmt_id: StatementId,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ExportType {
  ///
  ///
  /// ```ts
  /// // index.ts
  /// export * from "./cjs_module"
  /// export const name = 10;
  ///
  /// // cjs_module.ts
  /// module.exports.name = "shulan"
  /// module.exports.age = 18;
  /// ```
  ///
  HybridDynamic,
  ///
  /// only esm
  ///
  #[default]
  Static,
}

impl ExportType {
  pub fn merge(&mut self, other: Self) {
    match self {
      ExportType::HybridDynamic => {}
      ExportType::Static => {
        if matches!(other, ExportType::HybridDynamic) {
          *self = other;
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct ExportAllSet {
  pub data: Vec<(ExportInfo, ModuleId)>,
  pub ty: ExportType,
}

impl ExportAllSet {
  pub fn new() -> Self {
    Self {
      data: vec![],
      ty: ExportType::Static,
    }
  }

  pub fn add(&mut self, data: (ExportInfo, ModuleId)) {
    self.data.push(data);
  }

  pub fn inner(self) -> Vec<(ExportInfo, ModuleId)> {
    self.data
  }

  pub fn merge(&mut self, other: Self) {
    self.data.extend(other.data);

    self.ty.merge(other.ty);
  }
}

impl From<Vec<(ExportInfo, ModuleId)>> for ExportAllSet {
  fn from(value: Vec<(ExportInfo, ModuleId)>) -> Self {
    Self {
      data: value,
      ty: ExportType::Static,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSpecifierInfo {
  /// ```js
  /// import * as foo from 'foo';
  /// ```
  Namespace(usize),
  /// ```js
  /// // local bar
  /// // imported Some(foo)
  /// import { foo as bar } from 'foo';
  ///
  /// // local foo
  /// // imported None
  /// import { foo } from 'foo';
  /// ```
  Named {
    local: usize,
    /// as foo
    imported: Option<usize>,
  },
  /// ```js
  /// import xxx from 'foo';
  /// ```
  Default(usize),
}

#[derive(Debug, Clone)]
pub struct Statement {
  pub id: StatementId,
  pub import: Option<ImportInfo>,
  pub export: Option<ExportInfo>,
  pub defined: Vec<usize>,
}

struct ModuleMetadata {}

pub struct ModuleAnalyzer {
  pub statements: Vec<Statement>,
  pub statement_actions: HashSet<StmtAction>,
  pub cm: Arc<SourceMap>,
  pub ast: EcmaAstModule,
  pub module_id: ModuleId,
  pub resource_pot_id: ResourcePotId,
  pub export_names: Option<Rc<ReferenceMap>>,
  pub entry: bool,
  pub external: bool,
  pub dynamic: bool,
  pub is_runtime: bool,
  pub cjs_module_analyzer: CjsModuleAnalyzer,
  pub mark: (Mark, Mark),
  pub module_system: ModuleSystem,
}

impl ModuleAnalyzer {
  pub fn new(
    module: &Module,
    context: &Arc<CompilationContext>,
    resource_pot_id: ResourcePotId,
    is_entry: bool,
    is_dynamic: bool,
    is_runtime: bool,
  ) -> Result<Self> {
    farm_profile_function!(format!("module analyzer {}", module.id.to_string()));
    let mut ast = module.meta.as_script().ast.clone();

    let (cm, _) = create_swc_source_map(Source {
      path: PathBuf::from(module.id.resolved_path_with_query(&context.config.root)),
      content: module.content.clone(),
    });

    // println!("module_id: {}\n", module.id.to_string());
    // TODO: resolve_module_mark
    let mut mark = None;
    try_with(cm.clone(), &context.meta.script.globals, || {
      // let (unresolved_mark, top_level_mark) = if module.meta.as_script().unresolved_mark == 0
      //   && module.meta.as_script().top_level_mark == 0
      // {
      //   resolve_module_mark(&mut ast, module.module_type.is_typescript(), context)
      // } else {
      //   let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
      //   let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
      //   (unresolved_mark, top_level_mark)
      // };
      mark = Some(resolve_module_mark(
        &mut ast,
        module.module_type.is_typescript(),
        context,
      ));
    })?;

    // println!("mark: {:?}\nast: {:#?}", mark, ast);

    Ok(Self {
      statements: vec![],
      statement_actions: HashSet::new(),
      cm,
      ast,
      module_id: module.id.clone(),
      export_names: None,
      resource_pot_id,
      external: module.external,
      entry: is_entry,
      dynamic: is_dynamic,
      is_runtime,
      cjs_module_analyzer: CjsModuleAnalyzer::new(),
      mark: mark.unwrap(),
      module_system: module.meta.as_script().module_system.clone(),
    })
  }

  pub fn is_commonjs(&self) -> bool {
    matches!(
      self.module_system,
      ModuleSystem::CommonJs | ModuleSystem::Hybrid
    )
  }

  pub fn is_hybrid_esm(&self) -> bool {
    matches!(self.module_system, ModuleSystem::EsModule | ModuleSystem::Hybrid)
  }

  fn collect_unresolved_ident(&self, bundle_variable: &mut BundleVariable) {
    farm_profile_function!("");
    let mut collection = CollectUnresolvedIdent::new(self.mark.0);

    self.ast.visit_with(&mut collection);

    let uniq_name = bundle_variable.uniq_name_mut();
    for item in collection.unresolved_ident {
      uniq_name.insert(&item);
    }
  }

  pub fn extract_statement(
    &mut self,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut RefMut<BundleVariable>,
  ) -> Result<()> {
    farm_profile_function!("");
    for (statement_id, stmt) in self.ast.body.iter().enumerate() {
      let statement = analyze::analyze_imports_and_exports(
        statement_id,
        stmt,
        &self.module_id,
        module_graph,
        &mut |ident, strict| bundle_variable.register_var(&self.module_id, ident, strict),
      )?;

      if statement.export.is_none() && statement.import.is_none() && statement.defined.is_empty() {
        continue;
      }

      self.statements.push(statement);
    }

    try_with(self.cm.clone(), &context.meta.script.globals, || {
      // unresolved is write to global, so, we need to avoid having the same declaration as unresolved ident in the bundle
      self.collect_unresolved_ident(bundle_variable);

      self.cjs_module_analyzer.require_modules = self.cjs_module_analyzer.analyze_modules(
        &self.module_id,
        self.mark.0,
        self.mark.1,
        &self.ast,
        module_graph,
      );
    })?;

    Ok(())
  }

  pub fn exports_stmts(&self) -> Vec<&ExportInfo> {
    self
      .statements
      .iter()
      .filter_map(|stmt| stmt.export.as_ref())
      .collect()
  }

  pub fn variables(&self) -> HashSet<usize> {
    let mut variables = HashSet::new();

    for statement in &self.statements {
      for defined in &statement.defined {
        variables.insert(*defined);
      }
    }

    variables
  }

  pub fn export_names(&self) -> Rc<ReferenceMap> {
    return self.export_names.clone().unwrap_or_default();
  }

  pub fn build_rename_map<'a>(
    &self,
    bundle_variable: &'a BundleVariable,
  ) -> HashMap<&'a Id, &'a Var> {
    self
      .statements
      .iter()
      .flat_map(|statement| {
        statement
          .export
          .as_ref()
          .map(|item| {
            let mut idents: Vec<usize> = vec![];
            for specify in &item.specifiers {
              idents.extend(match specify {
                ExportSpecifierInfo::All(_) => {
                  vec![]
                }
                ExportSpecifierInfo::Named(var) => vec![var.local()],
                ExportSpecifierInfo::Default(index) => {
                  vec![*index]
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  vec![*ns]
                }
              })
            }
            idents
          })
          .unwrap_or_default()
          .into_iter()
          .chain(statement.defined.iter().cloned())
          .chain(
            statement
              .import
              .as_ref()
              .map(|item| {
                let mut idents = vec![];
                for specify in &item.specifiers {
                  match specify {
                    ImportSpecifierInfo::Namespace(local) => {
                      idents.push(*local);
                    }
                    ImportSpecifierInfo::Named { local, imported: _ } => {
                      idents.push(*local);
                    }
                    ImportSpecifierInfo::Default(local) => {
                      idents.push(*local);
                    }
                  }
                }
                idents
              })
              .unwrap_or_default()
              .into_iter(),
          )
          .map(|item| bundle_variable.var_by_index(item))
          .filter(|item| item.rename.is_some())
          .map(|item| (&item.var, item))
      })
      .collect::<HashMap<_, _>>()
  }
}
