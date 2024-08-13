use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  path::PathBuf,
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  farm_profile_function,
  module::{module_graph::ModuleGraph, Module, ModuleId, ModuleSystem, ModuleType},
  resource::resource_pot::ResourcePotId,
  swc_common::{Mark, SourceMap},
  swc_ecma_ast::{Id, Module as EcmaAstModule},
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  script::swc_try_with::try_with,
  swc_ecma_visit::VisitWith,
};

use crate::resource_pot_to_bundle::{
  bundle::reference::ReferenceMap, common::get_module_mark, targets::cjs::CjsModuleAnalyzer,
  uniq_name::BundleVariable, Var,
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
  ///
  /// export default function() {}
  /// export default class {}
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

pub struct ModuleAnalyzer {
  pub statements: Vec<Statement>,
  pub statement_actions: HashSet<StmtAction>,
  pub cm: Arc<SourceMap>,
  pub ast: EcmaAstModule,
  pub module_id: ModuleId,
  pub resource_pot_id: ResourcePotId,
  pub export_names: Option<Arc<ReferenceMap>>,
  pub entry: bool,
  pub external: bool,
  pub dynamic: bool,
  pub is_runtime: bool,
  pub cjs_module_analyzer: CjsModuleAnalyzer,
  pub mark: (Mark, Mark),
  pub module_system: ModuleSystem,
  pub module_type: ModuleType,
}

impl Debug for ModuleAnalyzer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleAnalyzer")
      .field("statements", &self.statements)
      .field("statement_actions", &self.statement_actions)
      .field("cm", &"[skip]")
      .field("ast", &self.ast)
      .field("module_id", &self.module_id)
      .field("resource_pot_id", &self.resource_pot_id)
      .field("export_names", &self.export_names)
      .field("entry", &self.entry)
      .field("external", &self.external)
      .field("dynamic", &self.dynamic)
      .field("is_runtime", &self.is_runtime)
      .field("cjs_module_analyzer", &"[skip]")
      .field("mark", &self.mark)
      .field("module_system", &self.module_system)
      .finish()
  }
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

    let mut mark = None;
    try_with(cm.clone(), &context.meta.script.globals, || {
      mark = Some(get_module_mark(module, &mut ast, context));
    })?;

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
      module_type: module.module_type.clone(),
    })
  }

  pub fn is_commonjs(&self) -> bool {
    matches!(
      self.module_system,
      ModuleSystem::CommonJs | ModuleSystem::Hybrid
    )
  }

  pub fn is_hybrid_esm(&self) -> bool {
    matches!(
      self.module_system,
      ModuleSystem::EsModule | ModuleSystem::Hybrid
    )
  }

  fn collect_unresolved_ident(&self, bundle_variable: &mut BundleVariable) {
    farm_profile_function!();
    let mut collection = CollectUnresolvedIdent::new(self.mark.0);

    self.ast.visit_with(&mut collection);

    let uniq_name = bundle_variable.uniq_name_mut();
    let mut ordered_unresolved_ident = collection.unresolved_ident.into_iter().collect::<Vec<_>>();

    ordered_unresolved_ident.sort();

    for item in ordered_unresolved_ident {
      uniq_name.insert(&item);
    }
  }

  pub fn extract_statement(
    &mut self,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut BundleVariable,
  ) -> Result<()> {
    farm_profile_function!("");
    try_with(self.cm.clone(), &context.meta.script.globals, || {
      for (statement_id, stmt) in self.ast.body.iter().enumerate() {
        let statement = analyze::analyze_imports_and_exports(
          statement_id,
          stmt,
          &self.module_id,
          module_graph,
          self.mark.1,
          &mut |ident, strict| {
            bundle_variable.register_var(&self.module_id.to_string(), ident, strict)
          },
        )
        .unwrap();

        if statement.export.is_none() && statement.import.is_none() && statement.defined.is_empty()
        {
          continue;
        }

        self.statements.push(statement);
      }

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

  pub fn export_names(&self) -> Arc<ReferenceMap> {
    self
      .export_names
      .clone()
      .unwrap_or_else(|| Arc::new(ReferenceMap::new(self.module_system.clone())))
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
              .unwrap_or_default(),
          )
          .map(|item| bundle_variable.var_by_index(item))
          .filter(|item| item.rename.is_some())
          .map(|item| (&item.var, item))
      })
      .collect::<HashMap<_, _>>()
  }
}
