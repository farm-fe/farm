use std::{collections::HashMap, fs};
use std::sync::Arc;
use farmfe_core::{
  swc_common::Span, swc_ecma_ast::*, swc_ecma_parser::{Syntax, TsSyntax}
};
use farmfe_toolkit::{
  script::{parse_module, ParseScriptModuleResult},
  swc_ecma_visit::{Visit, VisitWith},
};

#[derive(Debug)]
pub enum ImportType {
  Named,
  Type,
  Namespace,
  // TODO: 暂时不支持动态导入
  Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportType {
  All,
  Type,
  Declaration,
  Named,
  DefaultDecl,
  DefaultExpr,
  Namespace,
}

impl Default for ExportType {
  fn default() -> Self {
    ExportType::Declaration
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeclarationType {
  Var,
  Let,
  Const,
  Class,
  Function,
  AsyncFunction,
  Using,
  TsEnum,
  TsInterface,
  TsModule,
  TsTypeAlias,
}

#[derive(Debug)]
pub struct ESMExport {
  pub export_type: ExportType,
  pub declaration_type: Option<DeclarationType>,
  pub name: Option<String>,
  pub default_name: Option<String>,
  pub named_exports: Option<HashMap<String, String>>,
  pub type_named_exports: Option<HashMap<String, String>>,
  pub specifier: Option<String>,
}

#[derive(Debug)]
pub struct ESMImport {
  pub import_type: ImportType,
  pub specifier: String,
  pub default_import: Option<String>,
  pub namespaced_import: Option<String>,
  pub named_imports: Option<HashMap<String, String>>,
  pub type_named_imports: Option<HashMap<String, String>>,
  pub span: Span,
}

pub struct ImportsVisitor {
  imports: Vec<ESMImport>,
}

impl ImportsVisitor {
  fn new() -> Self {
    Self {
      imports: Vec::new(),
    }
  }
}

pub struct ExportsVisitor {
  exports: Vec<ESMExport>,
}

impl ExportsVisitor {
  fn new() -> Self {
    Self {
      exports: Vec::new(),
    }
  }
}

impl Visit for ImportsVisitor {
  fn visit_module_decl(&mut self, n: &ModuleDecl) {
    match n {
      ModuleDecl::Import(import) => {
        let mut named_imports = HashMap::new();
        let mut type_named_imports = HashMap::new();
        for specifier in &import.specifiers {
          match specifier {
            ImportSpecifier::Named(named) => {
              let imported_name = named
                .imported
                .as_ref()
                .map(|ident| ident.atom().to_string())
                .unwrap_or(named.local.sym.to_string());
              if !named.is_type_only {
                type_named_imports.insert(named.local.sym.to_string(), imported_name);
              } else {
                named_imports.insert(named.local.sym.to_string(), imported_name);
              }
            }
            ImportSpecifier::Default(default) => {
              let default_name = default.local.sym.to_string();
              self.imports.push(ESMImport {
                import_type: ImportType::Named,
                specifier: import.src.value.to_string(),
                default_import: Some(default_name.clone()),
                namespaced_import: None,
                named_imports: None,
                type_named_imports: None,
                span: import.span
              });
            }
            ImportSpecifier::Namespace(namespace) => {
              let namespace_name = namespace.local.sym.to_string();
              self.imports.push(ESMImport {
                import_type: ImportType::Namespace,
                specifier: import.src.value.to_string(),
                namespaced_import: Some(namespace_name.clone()),
                default_import: None,
                named_imports: None,
                type_named_imports: None,
                span: import.span
              });
            }
          }
        }
        if !named_imports.is_empty() {
          self.imports.push(ESMImport {
            import_type: ImportType::Named,
            specifier: import.src.value.to_string(),
            default_import: None,
            namespaced_import: None,
            type_named_imports: None,
            named_imports: Some(named_imports),
            span: import.span
          });
        }
        if !type_named_imports.is_empty() {
          self.imports.push(ESMImport {
            import_type: ImportType::Type,
            specifier: import.src.value.to_string(),
            default_import: None,
            namespaced_import: None,
            type_named_imports: Some(type_named_imports),
            named_imports: None,
            span: import.span
          });
        }
      }
      _ => {}
    }
  }
}

impl Visit for ExportsVisitor {
  fn visit_module_decl(&mut self, n: &ModuleDecl) {
    match n {
      ModuleDecl::ExportNamed(named_export) => {
        let mut named_exports = HashMap::new();
        let mut type_named_exports = HashMap::new();
        let specifier_str = named_export.src.as_ref().map(|s| s.value.to_string());
        for specifier in &named_export.specifiers {
          match specifier {
            ExportSpecifier::Named(named) => {
              let exported_name = named
                .exported
                .as_ref()
                .map(|ident| ident.atom().to_string())
                .unwrap_or(named.orig.atom().to_string());
              if named.is_type_only {
                type_named_exports.insert(named.orig.atom().to_string(), exported_name);
              } else {
                named_exports.insert(named.orig.atom().to_string(), exported_name);
              }
            }
            ExportSpecifier::Default(default) => {
              let default_name = default.exported.sym.to_string();
              self.exports.push(ESMExport {
                export_type: ExportType::DefaultDecl,
                specifier: specifier_str.clone(),
                name: Some(default_name.clone()),
                declaration_type: None,
                default_name: None,
                named_exports: None,
                type_named_exports: None,
              });
            }
            ExportSpecifier::Namespace(namespace) => {
              let namespace_name = namespace.name.atom().to_string();
              self.exports.push(ESMExport {
                export_type: ExportType::Namespace,
                specifier: specifier_str.clone(),
                name: Some(namespace_name),
                declaration_type: None,
                default_name: None,
                named_exports: None,
                type_named_exports: None,
              });
            }
          }
        }
        if !named_exports.is_empty() {
          self.exports.push(ESMExport {
            export_type: ExportType::Named,
            named_exports: Some(named_exports),
            specifier: specifier_str.clone(),
            declaration_type: None,
            name: None,
            default_name: None,
            type_named_exports: None,
          });
        }
        if !type_named_exports.is_empty() {
          self.exports.push(ESMExport {
            export_type: ExportType::Type,
            named_exports: None,
            specifier: specifier_str.clone(),
            declaration_type: None,
            name: None,
            default_name: None,
            type_named_exports: Some(type_named_exports),
          });
        }
      }
      ModuleDecl::ExportDecl(export_decl) => {
        let (declaration_type, name) = match &export_decl.decl {
          Decl::Class(class_decl) => (DeclarationType::Class, class_decl.ident.sym.to_string()),
          Decl::Fn(fn_decl) => (
            if fn_decl.function.is_async {
              DeclarationType::AsyncFunction
            } else {
              DeclarationType::Function
            },
            fn_decl.ident.sym.to_string(),
          ),
          Decl::TsEnum(enum_decl) => (DeclarationType::TsEnum, enum_decl.id.sym.to_string()),
          Decl::TsInterface(interface_decl) => (
            DeclarationType::TsInterface,
            interface_decl.id.sym.to_string(),
          ),
          Decl::TsModule(module_decl) => (
            DeclarationType::TsModule,
            if module_decl.id.is_str() {
              module_decl
                .id
                .clone()
                .str()
                .is_some()
                .then_some(module_decl.id.clone().str().unwrap().value.to_string())
                .unwrap()
            } else {
              module_decl
                .id
                .clone()
                .ident()
                .is_some()
                .then_some(module_decl.id.clone().ident().unwrap().sym.to_string())
                .unwrap()
            },
          ),
          Decl::TsTypeAlias(type_alias_decl) => (
            DeclarationType::TsTypeAlias,
            type_alias_decl.id.sym.to_string(),
          ),
          // 其他声明类型
          Decl::Using(_using_decl) => (DeclarationType::Using, String::from("Using")),
          Decl::Var(var_decl) => {
            let declaration_type = match var_decl.kind {
              VarDeclKind::Var => DeclarationType::Var,
              VarDeclKind::Let => DeclarationType::Let,
              VarDeclKind::Const => DeclarationType::Const,
            };
            let name = match &var_decl.decls[0].name {
              Pat::Ident(ident) => ident.sym.to_string(),
              _ => String::from("Anonymous"),
            };
            (declaration_type, name)
          }
        };
        self.exports.push(ESMExport {
          export_type: ExportType::Declaration,
          declaration_type: Some(declaration_type),
          name: Some(name.clone()),
          default_name: None,
          named_exports: None,
          specifier: None,
          type_named_exports: None,
        });
      }
      ModuleDecl::ExportDefaultDecl(default_export) => {
        let (declaration_type, default_name) = match &default_export.decl {
          DefaultDecl::Fn(fn_decl) => (
            if fn_decl.function.is_async {
              DeclarationType::AsyncFunction
            } else {
              DeclarationType::Function
            },
            fn_decl
              .ident
              .as_ref()
              .map_or(String::from("Anonymous"), |ident| ident.sym.to_string()),
          ),
          DefaultDecl::Class(class_decl) => (
            DeclarationType::Class,
            class_decl
              .ident
              .as_ref()
              .map_or(String::from("Anonymous"), |ident| ident.sym.to_string()),
          ),
          DefaultDecl::TsInterfaceDecl(interface_decl) => (
            DeclarationType::TsInterface,
            interface_decl.id.sym.to_string(),
          ),
        };
        self.exports.push(ESMExport {
          export_type: ExportType::DefaultDecl,
          declaration_type: Some(declaration_type),
          name: None,
          default_name: Some(default_name),
          named_exports: None,
          specifier: None,
          type_named_exports: None,
        });
      }
      ModuleDecl::ExportDefaultExpr(default_expr) => {
        let declaration_type = match &*default_expr.expr {
          Expr::Arrow(arrow_expr) => {
            if arrow_expr.is_async {
              DeclarationType::AsyncFunction
            } else {
              DeclarationType::Function
            }
          }
          Expr::Fn(fn_expr) => {
            if fn_expr.function.is_async {
              DeclarationType::AsyncFunction
            } else {
              DeclarationType::Function
            }
          }
          _ => DeclarationType::Var,
        };
        self.exports.push(ESMExport {
          export_type: ExportType::DefaultExpr,
          declaration_type: Some(declaration_type),
          name: None,
          default_name: Some(String::from("Anonymous")),
          named_exports: None,
          specifier: None,
          type_named_exports: None,
        });
      }
      ModuleDecl::ExportAll(export_all) => {
        let specifier = export_all.src.value.to_string();
        self.exports.push(ESMExport {
          export_type: ExportType::All,
          declaration_type: None,
          name: None,
          default_name: None,
          named_exports: None,
          specifier: Some(specifier),
          type_named_exports: None,
        });
      }
      _ => {}
    }
  }
}

pub fn parse_esm_imports_exports(
  file_path: Option<&str>,
  content: Option<&str>,
) -> (Vec<ESMImport>, Vec<ESMExport>) {
  if file_path.is_none() && content.is_none() {
    return (vec![], vec![]);
  }
  let file_path = if file_path.is_none() {
    ""
  } else {
    file_path.unwrap()
  };
  let content = if content.is_none() {
    &fs::read_to_string(file_path)
      .unwrap_or_else(|_| panic!("Unable to read file: {:?}", file_path))
  } else {
    content.unwrap()
  };
  let content = Arc::new(content.to_string());
  let ParseScriptModuleResult { ast, .. } = match parse_module(
    &file_path.into(),
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: true,
      ..Default::default()
    }),
    EsVersion::latest(),
  ) {
    Ok(res) => res,
    Err(err) => {
      println!("{}", err.to_string());
      panic!("Parse {} failed. See error details above.", file_path);
    }
  };

  let mut imports_visitor = ImportsVisitor::new();
  ast.visit_with(&mut imports_visitor);
  let mut exports_visitor = ExportsVisitor::new();
  ast.visit_with(&mut exports_visitor);
  (imports_visitor.imports, exports_visitor.exports)
}

pub fn parse_esm_imports(file_path: Option<&str>, content: Option<&str>) -> Vec<ESMImport> {
  if file_path.is_none() && content.is_none() {
    return vec![];
  }
  let file_path = if file_path.is_none() {
    ""
  } else {
    file_path.unwrap()
  };
  let content = if content.is_none() {
    &fs::read_to_string(file_path)
      .unwrap_or_else(|_| panic!("Unable to read file: {:?}", file_path))
  } else {
    content.unwrap()
  };
  let content = Arc::new(content.to_string());
  let ParseScriptModuleResult { ast, .. } = match parse_module(
    &file_path.into(),
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: false,
      dts: false,
      no_early_errors: true,
      disallow_ambiguous_jsx_like: true,
    }),
    EsVersion::latest(),
  ) {
    Ok(res) => res,
    Err(err) => {
      println!("{}", err.to_string());
      panic!("Parse {} failed. See error details above.", file_path);
    }
  };

  let mut imports_visitor = ImportsVisitor::new();
  ast.visit_with(&mut imports_visitor);
  imports_visitor.imports
}

pub fn parse_esm_exports(file_path: Option<&str>, content: Option<&str>) -> Vec<ESMExport> {
  if file_path.is_none() && content.is_none() {
    return vec![];
  }
  let file_path = if file_path.is_none() {
    ""
  } else {
    file_path.unwrap()
  };
  let content = if content.is_none() {
    &fs::read_to_string(file_path)
      .unwrap_or_else(|_| panic!("Unable to read file: {:?}", file_path))
  } else {
    content.unwrap()
  };
  let content = Arc::new(content.to_string());
  let ParseScriptModuleResult { ast, .. } = match parse_module(
    &file_path.into(),
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: true,
      ..Default::default()
    }),
    EsVersion::latest(),
  ) {
    Ok(res) => res,
    Err(err) => {
      println!("{}", err.to_string());
      panic!("Parse {} failed. See error details above.", file_path);
    }
  };

  let mut exports_visitor = ExportsVisitor::new();
  ast.visit_with(&mut exports_visitor);
  exports_visitor.exports
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_esm_imports_exports() {
    let (imports, exports) = parse_esm_imports_exports(Some("./tests/test.ts"), None);
    println!("imports: {:#?}", imports);
    println!("exports: {:#?}", exports);
  }
}
