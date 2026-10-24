use std::{collections::HashMap, sync::Arc};

use farmfe_core::{error::CompilationError, swc_ecma_ast::*};
use farmfe_toolkit::{script::parse_module, swc_ecma_parser::Syntax};

/// Register exported Vue components only. Ordinary JSX functions must continue
/// propagating updates to their importing boundary.
pub fn inject(module: &mut Module, module_id: &str) -> farmfe_core::error::Result<()> {
  let mut factories = Vec::new();
  let mut namespaces = Vec::new();
  let mut exports = HashMap::<String, String>::new();
  for item in &module.body {
    match item {
      ModuleItem::ModuleDecl(ModuleDecl::Import(import)) if import.src.value == "vue" => {
        for specifier in &import.specifiers {
          match specifier {
            ImportSpecifier::Named(named) if !named.is_type_only => {
              let imported = named
                .imported
                .as_ref()
                .map(|name| match name {
                  ModuleExportName::Ident(id) => id.sym.to_string(),
                  ModuleExportName::Str(value) => value.value.to_string_lossy().into_owned(),
                })
                .unwrap_or_else(|| named.local.sym.to_string());
              if imported == "defineComponent" {
                factories.push(named.local.sym.to_string());
              }
            }
            ImportSpecifier::Namespace(namespace) => {
              namespaces.push(namespace.local.sym.to_string())
            }
            _ => {}
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
        if let Decl::Var(var) = &export.decl {
          for declaration in &var.decls {
            if let Pat::Ident(id) = &declaration.name {
              exports.insert(id.id.sym.to_string(), id.id.sym.to_string());
            }
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)) if export.src.is_none() => {
        for specifier in &export.specifiers {
          if let ExportSpecifier::Named(named) = specifier {
            if !named.is_type_only {
              if let ModuleExportName::Ident(local) = &named.orig {
                let exported = named
                  .exported
                  .as_ref()
                  .map(|name| match name {
                    ModuleExportName::Ident(id) => id.sym.to_string(),
                    ModuleExportName::Str(value) => value.value.to_string_lossy().into_owned(),
                  })
                  .unwrap_or_else(|| local.sym.to_string());
                exports.insert(local.sym.to_string(), exported);
              }
            }
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export)) => {
        if let Expr::Ident(local) = &*export.expr {
          exports.insert(local.sym.to_string(), "default".into());
        }
      }
      _ => {}
    }
  }
  let is_component = |expr: &Expr| {
    let Expr::Call(call) = expr else {
      return false;
    };
    let Callee::Expr(callee) = &call.callee else {
      return false;
    };
    match &**callee {
      Expr::Ident(id) => factories.iter().any(|name| name == id.sym.as_ref()),
      Expr::Member(member) => {
        matches!((&*member.obj, &member.prop), (Expr::Ident(id), MemberProp::Ident(prop)) if prop.sym == "defineComponent" && namespaces.iter().any(|name| name == id.sym.as_ref()))
      }
      _ => false,
    }
  };
  for item in &mut module.body {
    match item {
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export))
        if is_component(&export.expr) =>
      {
        export.expr = wrap(export.expr.clone(), module_id, "default")?;
      }
      ModuleItem::Stmt(Stmt::Decl(Decl::Var(var)))
      | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
        decl: Decl::Var(var),
        ..
      })) => {
        for declaration in &mut var.decls {
          if let (Pat::Ident(id), Some(init)) = (&declaration.name, &mut declaration.init) {
            if let Some(exported) = exports.get(id.id.sym.as_ref()) {
              if is_component(init) {
                *init = wrap(init.clone(), module_id, exported)?;
              }
            }
          }
        }
      }
      _ => {}
    }
  }
  Ok(())
}

fn wrap(
  component: Box<Expr>,
  module_id: &str,
  export: &str,
) -> farmfe_core::error::Result<Box<Expr>> {
  let id = farmfe_core::serde_json::json!(format!("{}:{export}", module_id.replace('\\', "/")))
    .to_string();
  let export = farmfe_core::serde_json::json!(export).to_string();
  let code = format!(
    r#"((component) => {{
      if (import.meta.hot && typeof window !== 'undefined' && window.__VUE_HMR_RUNTIME__) {{
        component.__hmrId = {id};
        window.__VUE_HMR_RUNTIME__.createRecord({id}, component);
        import.meta.hot.accept((mod) => {{
          if (mod && mod[{export}]) window.__VUE_HMR_RUNTIME__.reload({id}, mod[{export}]);
        }});
      }}
      return component;
    }})(null);"#
  );
  let parsed = parse_module(
    &module_id.into(),
    Arc::new(code),
    Syntax::Es(Default::default()),
    Default::default(),
  )
  .map_err(|error| CompilationError::TransformError {
    resolved_path: module_id.into(),
    msg: format!("[vue-jsx] HMR generation failed: {error}"),
  })?;
  if let Some(ModuleItem::Stmt(Stmt::Expr(statement))) = parsed.ast.body.into_iter().next() {
    if let Expr::Call(mut call) = *statement.expr {
      if let Some(argument) = call.args.first_mut() {
        argument.expr = component;
        return Ok(Box::new(Expr::Call(call)));
      }
    }
  }
  Err(CompilationError::TransformError {
    resolved_path: module_id.into(),
    msg: "[vue-jsx] invalid generated HMR expression".into(),
  })
}
