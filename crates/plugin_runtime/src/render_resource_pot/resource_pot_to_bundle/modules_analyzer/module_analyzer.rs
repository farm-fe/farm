use std::{
  cell::RefMut,
  collections::{HashMap, HashSet},
  mem,
  path::PathBuf,
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::{module_graph::ModuleGraph, Module, ModuleId, ModuleSystem},
  resource::resource_pot::ResourcePotId,
  swc_common::{Mark, SourceMap},
  swc_ecma_ast::{Id, Module as EcmaAstModule},
};
use farmfe_toolkit::{
  common::{create_swc_source_map, Source},
  script::swc_try_with::{resolve_module_mark, try_with},
};

use crate::resource_pot_to_bundle::{
  bundle::ModuleAnalyzerManager, targets::cjs::CjsModuleAnalyzer, uniq_name::BundleVariable, Var,
};

use super::analyze;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StmtAction {
  StripExport(usize),
  StripDefaultExport(usize, usize),
  // StripImport(usize),
  DeclDefaultExpr(usize, usize),
  RemoveImport(usize),
  ReplaceCjsImport(usize, ModuleId),
  ReplaceCjsExport(ModuleId),
  // RemoveExport(usize),
  // PatchNamespaceDecl(usize),
}

impl StmtAction {
  pub fn index(&self) -> Option<usize> {
    match self {
      StmtAction::StripExport(index) => Some(*index),
      StmtAction::StripDefaultExport(index, _) => Some(*index),
      // StmtAction::StripImport(index) => Some(*index),
      StmtAction::DeclDefaultExpr(index, _) => Some(*index),
      StmtAction::RemoveImport(index) => Some(*index),
      StmtAction::ReplaceCjsImport(index, _) => Some(*index),
      StmtAction::ReplaceCjsExport(_) => None,
      // StmtAction::RemoveExport(index) => Some(*index),
      // StmtAction::PatchNamespaceDecl(_) => None,
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
  All(Option<Vec<usize>>),
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
  pub export_names: Option<Vec<(ExportInfo, ModuleId)>>,
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

  pub fn extract_statement(
    &mut self,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut RefMut<BundleVariable>,
  ) -> Result<()> {
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
      let mut cjs_module_analyzer =
        mem::replace(&mut self.cjs_module_analyzer, CjsModuleAnalyzer::new());
      cjs_module_analyzer.analyze_modules(self, module_graph);
      self.cjs_module_analyzer = cjs_module_analyzer;
    })?;

    Ok(())
  }

  pub fn patch_ast(&self, module_analyzer_manager: ModuleAnalyzerManager) {
    // module_analyzer_manager
    //   .module_global_uniq_name
    //   .commonjs_name(&self.module_id);
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
