use farmfe_core::module::meta_data::script::{statement::Statement, ScriptModuleMetaData};
use farmfe_toolkit::script::analyze_statement::{analyze_statement_info, AnalyzedStatementInfo};

pub fn analyze_statements(meta: &ScriptModuleMetaData) -> Vec<Statement> {
  let mut statements = vec![];

  for (i, item) in meta.ast.body.iter().enumerate() {
    let AnalyzedStatementInfo {
      import_info,
      export_info,
      defined_idents,
      top_level_await,
    } = analyze_statement_info(&i, item);
    let stmt = Statement::new(i, export_info, import_info, defined_idents, top_level_await);
    statements.push(stmt);
  }

  statements
}
