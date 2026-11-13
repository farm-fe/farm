//! Turn Farm's generated ESM entry envelope into a classic script container.
//! Only emitted side-effect imports are folded; externals/named imports fail closed.
use super::error;
use farmfe_core::{
  HashMap,
  context::CompilationContext,
  error::Result,
  swc_ecma_ast::{ExprStmt, ModuleDecl, ModuleItem, Stmt},
};
use farmfe_toolkit::script::{codegen_module, create_codegen_config, parse_module};
use std::{collections::HashSet, sync::Arc};

pub fn bundle(
  entry: &str,
  sources: &HashMap<String, String>,
  context: &CompilationContext,
) -> Result<Vec<u8>> {
  fn visit(
    name: &str,
    sources: &HashMap<String, String>,
    context: &CompilationContext,
    visiting: &mut HashSet<String>,
    done: &mut HashSet<String>,
    output: &mut Vec<u8>,
  ) -> Result<()> {
    if done.contains(name) {
      return Ok(());
    }
    if !visiting.insert(name.into()) {
      return Err(error("Cycle in generated entry resources"));
    }
    let source = sources
      .get(name)
      .ok_or_else(|| error(format!("Entry dependency not emitted: {name}")))?;
    let mut parsed = parse_module(
      &name.into(),
      Arc::new(source.clone()),
      Default::default(),
      Default::default(),
    )?;
    let mut statements = vec![];
    for item in parsed.ast.body {
      match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => {
          if !import.specifiers.is_empty() {
            return Err(error(
              "Named external imports cannot be folded into a global container",
            ));
          }
          let src = import.src.value.to_string_lossy();
          let dependency = src
            .strip_prefix("./")
            .ok_or_else(|| error("Global container cannot include external static imports"))?;
          visit(dependency, sources, context, visiting, done, output)?;
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)) if export.src.is_none() => {}
        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export)) => {
          statements.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: export.span,
            expr: export.expr,
          })))
        }
        ModuleItem::ModuleDecl(_) => {
          return Err(error("Unexpected export in generated container resources"));
        }
        statement => statements.push(statement),
      }
    }
    parsed.ast.body = statements;
    let code = codegen_module(
      &parsed.ast,
      parsed.source_map,
      None,
      create_codegen_config(context),
      None,
    )
    .map_err(|e| error(e.to_string()))?;
    output.extend(code);
    output.extend(b";\n");
    visiting.remove(name);
    done.insert(name.into());
    Ok(())
  }
  let mut output = b"(function(){\"use strict\";\n".to_vec();
  visit(
    entry,
    sources,
    context,
    &mut HashSet::new(),
    &mut HashSet::new(),
    &mut output,
  )?;
  output.extend(b"})();");
  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::config::Config;
  fn context() -> CompilationContext {
    CompilationContext::new(Config::default(), vec![]).expect("context")
  }
  #[test]
  fn folds_side_effect_dependencies_before_entry_and_drops_esm_exports() {
    let sources = HashMap::from_iter([
      (
        "remoteEntry.js".into(),
        "import './runtime.js'; import './chunk.js'; var entry=42; export {entry};".into(),
      ),
      ("runtime.js".into(), "globalThis.runtimeReady=true;".into()),
      ("chunk.js".into(), "globalThis.chunkReady=true;".into()),
    ]);
    let result = String::from_utf8(bundle("remoteEntry.js", &sources, &context()).expect("bundle"))
      .expect("utf8");
    assert!(!result.contains("import "));
    assert!(!result.contains("export "));
    assert!(
      result.find("runtimeReady").expect("runtime") < result.find("chunkReady").expect("chunk")
    );
    assert!(result.find("chunkReady").expect("chunk") < result.find("entry").expect("entry"));
  }
  #[test]
  fn rejects_named_external_imports_and_missing_dependencies() {
    for source in [
      "import x from './other.js';",
      "import './missing.js';",
      "import 'external';",
    ] {
      let sources = HashMap::from_iter([("entry.js".into(), source.into())]);
      assert!(bundle("entry.js", &sources, &context()).is_err());
    }
  }
  #[test]
  fn generated_resource_cycles_are_diagnostics() {
    let sources = HashMap::from_iter([("entry.js".into(), "import './entry.js';".into())]);
    assert!(bundle("entry.js", &sources, &context()).is_err());
  }
}
