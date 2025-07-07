use farmfe_core::{module::meta_data::script::statement::SwcId, HashMap, HashSet};

use super::{
  analyze_used_import_all_fields::UsedImportAllFields, StatementGraph, StatementSideEffects,
  WriteTopLevelVar,
};

pub fn analyze_written_imported_idents(graph: &StatementGraph) -> HashSet<WriteTopLevelVar> {
  let mut written_imported_idents = HashSet::default();

  for stmt in graph.stmts() {
    if let StatementSideEffects::WriteTopLevelVar(write_top_level_vars) = &stmt.side_effects {
      let idents = write_top_level_vars
        .iter()
        .map(|v| v.ident.clone())
        .collect::<HashSet<_>>();

      let used_import_all_field = write_top_level_vars
        .iter()
        .map(|v| {
          (
            v.ident.clone(),
            v.fields
              .as_ref()
              .map(|f| f.iter().map(|f| f.clone()).collect())
              .unwrap_or_default(),
          )
        })
        .collect::<HashMap<_, _>>();

      written_imported_idents.extend(find_written_imported_idents_dfs(
        &idents,
        &used_import_all_field,
        graph,
        &mut HashSet::default(),
      ))
    }
  }

  written_imported_idents
}

fn find_written_imported_idents_dfs(
  idents: &HashSet<SwcId>,
  used_import_all_fields: &HashMap<SwcId, HashSet<UsedImportAllFields>>,
  graph: &StatementGraph,
  visited: &mut HashSet<SwcId>,
) -> HashSet<WriteTopLevelVar> {
  let mut written_imported_idents = HashSet::default();

  for ident in idents {
    if visited.contains(ident) {
      continue;
    }

    visited.insert(ident.clone());

    if let Some(stmt_id) = graph.reverse_defined_idents_map.get(ident) {
      let dep_stmt = graph.stmt(stmt_id);

      if dep_stmt.import_info.is_some() {
        let fields = used_import_all_fields.get(ident).cloned();
        written_imported_idents.insert(WriteTopLevelVar {
          ident: ident.clone(),
          fields: fields.map(|f| f.into_iter().collect()),
        });
      } else {
        for (_, edge) in graph.dependencies(stmt_id) {
          if let Some(dep_idents) = edge.used_idents_map.get(ident) {
            written_imported_idents.extend(find_written_imported_idents_dfs(
              dep_idents,
              &edge.used_import_all_fields,
              graph,
              visited,
            ));
          }
        }
      }
    }
  }

  written_imported_idents
}
