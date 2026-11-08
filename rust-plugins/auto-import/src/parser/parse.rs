#![allow(dead_code)]

use farmfe_core::{
  error::{CompilationError, Result},
  swc_common::{Span, Spanned},
  swc_ecma_ast::*,
  swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax},
};
use farmfe_toolkit::{
  sourcemap::create_swc_source_map,
  swc_ecma_visit::{Visit, VisitWith},
};
use std::sync::Arc;
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub enum ImportType {
  Named,
  Type,
  Namespace,
  // TODO: 暂时不支持动态导入
  Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExportType {
  All,
  Type,
  #[default]
  Declaration,
  Named,
  DefaultDecl,
  DefaultExpr,
  Namespace,
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
    if let ModuleDecl::Import(import) = n {
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
              specifier: import.src.value.to_string_lossy().into_owned(),
              default_import: Some(default_name.clone()),
              namespaced_import: None,
              named_imports: None,
              type_named_imports: None,
              span: import.span,
            });
          }
          ImportSpecifier::Namespace(namespace) => {
            let namespace_name = namespace.local.sym.to_string();
            self.imports.push(ESMImport {
              import_type: ImportType::Namespace,
              specifier: import.src.value.to_string_lossy().into_owned(),
              namespaced_import: Some(namespace_name.clone()),
              default_import: None,
              named_imports: None,
              type_named_imports: None,
              span: import.span,
            });
          }
        }
      }
      if !named_imports.is_empty() {
        self.imports.push(ESMImport {
          import_type: ImportType::Named,
          specifier: import.src.value.to_string_lossy().into_owned(),
          default_import: None,
          namespaced_import: None,
          type_named_imports: None,
          named_imports: Some(named_imports),
          span: import.span,
        });
      }
      if !type_named_imports.is_empty() {
        self.imports.push(ESMImport {
          import_type: ImportType::Type,
          specifier: import.src.value.to_string_lossy().into_owned(),
          default_import: None,
          namespaced_import: None,
          type_named_imports: Some(type_named_imports),
          named_imports: None,
          span: import.span,
        });
      }
    }
  }
}

impl Visit for ExportsVisitor {
  fn visit_module_decl(&mut self, n: &ModuleDecl) {
    match n {
      ModuleDecl::ExportNamed(named_export) => {
        let mut named_exports = HashMap::new();
        let mut type_named_exports = HashMap::new();
        let specifier_str = named_export
          .src
          .as_ref()
          .map(|s| s.value.to_string_lossy().into_owned());
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
                .then_some(
                  module_decl
                    .id
                    .clone()
                    .str()
                    .unwrap()
                    .value
                    .to_string_lossy()
                    .into_owned(),
                )
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
        let specifier = export_all.src.value.to_string_lossy().into_owned();
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

fn parse_source(file_path: Option<&str>, content: Option<&str>, syntax: Syntax) -> Result<Module> {
  let file_path = file_path.unwrap_or("<auto-import>");
  let content = match content {
    Some(content) => content.to_owned(),
    None => fs::read_to_string(file_path).map_err(|error| CompilationError::LoadError {
      resolved_path: file_path.to_owned(),
      source: Some(Box::new(error)),
    })?,
  };
  let (source_map, source_file) = create_swc_source_map(&file_path.into(), Arc::new(content));
  let mut parser = Parser::new(syntax, StringInput::from(&*source_file), None);
  let parsed = parser.parse_module();
  // Discovery parses syntax; it must not turn this recoverable TypeScript
  // semantic diagnostic into a build error (including @ts-ignore sources).
  // Keep every other recovered diagnostic, including malformed syntax.
  let mut errors = parser
    .take_errors()
    .into_iter()
    .filter(|error| {
      !matches!(
        error.kind(),
        farmfe_core::swc_ecma_parser::error::SyntaxError::TS4112
      )
    })
    .collect::<Vec<_>>();
  let ast = match parsed {
    Ok(ast) => Some(ast),
    Err(error) => {
      errors.push(error);
      None
    }
  };
  if !errors.is_empty() {
    let msg = errors
      .into_iter()
      .map(|error| {
        let location = source_map.lookup_char_pos(error.span().lo);
        format!(
          "{}:{}: {}",
          location.line,
          location.col_display + 1,
          error.kind().msg()
        )
      })
      .collect::<Vec<_>>()
      .join("\n");
    return Err(CompilationError::ParseError {
      resolved_path: file_path.to_owned(),
      msg,
    });
  }
  ast.ok_or_else(|| CompilationError::ParseError {
    resolved_path: file_path.to_owned(),
    msg: "Parser produced no module".into(),
  })
}

pub fn parse_esm_imports_exports(
  file_path: Option<&str>,
  content: Option<&str>,
) -> Result<(Vec<ESMImport>, Vec<ESMExport>)> {
  if file_path.is_none() && content.is_none() {
    return Ok((vec![], vec![]));
  }
  let ast = parse_source(
    file_path,
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: true,
      ..Default::default()
    }),
  )?;
  let mut imports = ImportsVisitor::new();
  ast.visit_with(&mut imports);
  let mut exports = ExportsVisitor::new();
  ast.visit_with(&mut exports);
  Ok((imports.imports, exports.exports))
}

pub fn parse_esm_imports(file_path: Option<&str>, content: Option<&str>) -> Result<Vec<ESMImport>> {
  if file_path.is_none() && content.is_none() {
    return Ok(vec![]);
  }
  let ast = parse_source(
    file_path,
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: false,
      dts: false,
      no_early_errors: false,
      disallow_ambiguous_jsx_like: true,
    }),
  )?;
  let mut imports = ImportsVisitor::new();
  ast.visit_with(&mut imports);
  Ok(imports.imports)
}

pub fn parse_esm_exports(file_path: Option<&str>, content: Option<&str>) -> Result<Vec<ESMExport>> {
  if file_path.is_none() && content.is_none() {
    return Ok(vec![]);
  }
  let ast = parse_source(
    file_path,
    content,
    Syntax::Typescript(TsSyntax {
      tsx: true,
      decorators: true,
      ..Default::default()
    }),
  )?;
  let mut exports = ExportsVisitor::new();
  ast.visit_with(&mut exports);
  Ok(exports.exports)
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_esm_imports_exports() {
    let (imports, exports) = parse_esm_imports_exports(Some("./tests/test.ts"), None).unwrap();
    println!("imports: {imports:#?}");
    println!("exports: {exports:#?}");
  }
}

#[cfg(test)]
mod recovery_regression_tests {
  use super::*;

  #[test]
  fn incomplete_source_does_not_unwind() {
    for content in [
      "export const value =",
      "type Props = { onChange: (value: string) => v",
      "export default () => <div>",
    ] {
      assert!(
        std::panic::catch_unwind(|| parse_esm_imports(Some("stream.tsx"), Some(content))).is_ok()
      );
      assert!(
        std::panic::catch_unwind(|| parse_esm_exports(Some("stream.tsx"), Some(content))).is_ok()
      );
      assert!(
        std::panic::catch_unwind(|| parse_esm_imports_exports(Some("stream.tsx"), Some(content)))
          .is_ok()
      );
    }
  }

  #[test]
  fn missing_source_does_not_unwind() {
    assert!(
      std::panic::catch_unwind(|| parse_esm_imports(
        Some("/farm-auto-import-missing-test.tsx"),
        None
      ))
      .is_ok()
    );
  }
}

#[cfg(test)]
mod diagnostic_regression_tests {
  use super::*;
  #[test]
  fn invalid_then_valid_returns_diagnostic_and_recovers() {
    for content in [
      "export const value =",
      "export default () => <div>",
      "type Props = { onChange: (value: string) => v",
    ] {
      for result in [
        parse_esm_imports(Some("stream.tsx"), Some(content)).map(|_| ()),
        parse_esm_exports(Some("stream.tsx"), Some(content)).map(|_| ()),
        parse_esm_imports_exports(Some("stream.tsx"), Some(content)).map(|_| ()),
      ] {
        match result {
          Err(CompilationError::ParseError { resolved_path, msg }) => {
            assert_eq!(resolved_path, "stream.tsx");
            assert!(!msg.is_empty());
          }
          other => panic!("expected diagnostic: {other:?}"),
        }
      }
      assert!(
        parse_esm_imports(
          Some("stream.tsx"),
          Some("import React from 'react'; export const value = <div />;")
        )
        .is_ok()
      );
    }
  }
  #[test]
  fn override_semantic_diagnostic_does_not_block_discovery() {
    for content in [
      "export class Example { override method() {} }",
      "export class Example { // @ts-ignore\n override method() {} }",
    ] {
      assert!(parse_esm_imports(Some("semantic.ts"), Some(content)).is_ok());
      assert!(parse_esm_exports(Some("semantic.ts"), Some(content)).is_ok());
      assert!(parse_esm_imports_exports(Some("semantic.ts"), Some(content)).is_ok());
    }
    assert!(parse_esm_exports(Some("syntax.ts"), Some("export const value = 1 2;")).is_err());
  }

  #[test]
  fn recoverable_parser_errors_are_not_success() {
    assert!(parse_esm_imports(Some("stream.tsx"), Some("const value = 1 2;")).is_err());
  }
  #[test]
  fn missing_file_keeps_path_in_load_error() {
    assert!(
      matches!(parse_esm_exports(Some("/farm-auto-import-missing-test.tsx"), None), Err(CompilationError::LoadError { resolved_path, .. }) if resolved_path == "/farm-auto-import-missing-test.tsx")
    );
  }
}
