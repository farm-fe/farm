use farmfe_core::module::meta_data::script::{ScriptModuleMetaData, statement::Statement};
use farmfe_toolkit::script::analyze_statement::analyze_statements as raw_analyze_statements;

pub fn analyze_statements(meta: &ScriptModuleMetaData) -> Vec<Statement> {
  raw_analyze_statements(&meta.ast)
}
