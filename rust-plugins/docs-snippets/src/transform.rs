use farmfe_core::swc_common::{Span, Spanned};
use farmfe_core::{HashMap, HashSet, swc_ecma_ast::*};
use farmfe_syntax_highlight::HighlightOptions;
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};
mod ast;
mod codeblock;
mod commands;
mod values;

pub const MARKER_MODULE: &str = "@farmfe/docs/snippets";

#[derive(serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
  pub renderer_module: String,
  pub highlight: HighlightOptions,
}

impl Options {
  pub fn validate(&self) -> Result<(), String> {
    if self.renderer_module.trim().is_empty() || self.renderer_module.contains('\0') {
      return Err(
        "[DOCS_SNIPPET_OPTIONS] renderer module must be nonempty and cannot contain NUL".into(),
      );
    }
    Ok(())
  }
}

impl Default for Options {
  fn default() -> Self {
    Self {
      renderer_module: "@farmfe/docs/snippets-runtime".into(),
      highlight: HighlightOptions::default(),
    }
  }
}

#[derive(Default, Debug)]
pub struct Report {
  pub transformed: usize,
  pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub struct Diagnostic {
  pub message: String,
  pub span: Option<Span>,
}
impl Diagnostic {
  pub(crate) fn at(message: impl Into<String>, span: Span) -> Self {
    Self {
      message: message.into(),
      span: Some(span),
    }
  }
}
impl From<String> for Diagnostic {
  fn from(message: String) -> Self {
    Self {
      message,
      span: None,
    }
  }
}
impl std::fmt::Display for Diagnostic {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{}", self.message)?;
    if let Some(span) = self.span {
      write!(formatter, " (byte {})", span.lo.0)?;
    }
    Ok(())
  }
}
impl std::error::Error for Diagnostic {}

pub fn lower(module: &mut Module, options: &Options) -> Result<Report, String> {
  lower_diagnostic(module, options).map_err(|error| error.to_string())
}

pub fn lower_diagnostic(module: &mut Module, options: &Options) -> Result<Report, Diagnostic> {
  options.validate()?;
  let mut bindings = HashMap::default();
  for item in &module.body {
    if matches!(item,
      ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export)) if export.src.value == MARKER_MODULE && !export.type_only)
      || matches!(item,
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export)) if !export.type_only && export.src.as_ref().is_some_and(|source| source.value == MARKER_MODULE))
    {
      return Err(Diagnostic::at(
        "[DOCS_SNIPPET_IMPORT] marker re-exports are unsupported; import named markers at their JSX use site",
        item.span(),
      ));
    }
    let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item else {
      continue;
    };
    if import.src.value != MARKER_MODULE || import.type_only {
      continue;
    }
    if import.specifiers.is_empty() {
      return Err(Diagnostic::at(
        "[DOCS_SNIPPET_IMPORT] side-effect marker imports are unsupported",
        import.src.span,
      ));
    }
    for specifier in &import.specifiers {
      let ImportSpecifier::Named(named) = specifier else {
        return Err(Diagnostic::at(
          "[DOCS_SNIPPET_IMPORT] use named marker imports",
          specifier.span(),
        ));
      };
      if named.is_type_only {
        continue;
      }
      let imported = named
        .imported
        .as_ref()
        .map(|value| value.atom().to_string())
        .unwrap_or_else(|| named.local.sym.to_string());
      if imported != "CodeBlock" && imported != "CommandTabs" {
        return Err(Diagnostic::at(
          format!("[DOCS_SNIPPET_IMPORT] unsupported marker {imported}"),
          named.local.span,
        ));
      }
      bindings.insert(named.local.to_id(), imported);
    }
  }
  if bindings.is_empty() {
    return Ok(Report::default());
  }
  let mut usage = Usage {
    bindings: bindings.keys().cloned().collect(),
    escaped: None,
  };
  module.visit_with(&mut usage);
  if let Some(span) = usage.escaped {
    return Err(Diagnostic::at(
      "[DOCS_SNIPPET_STATIC] marker components cannot escape as runtime values",
      span,
    ));
  }
  let mut working = module.clone();
  let mut compiler = Compiler {
    bindings,
    options,
    report: Report::default(),
    error: None,
  };
  working.visit_mut_with(&mut compiler);
  if let Some(error) = compiler.error {
    return Err(error);
  }
  for item in &mut working.body {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
      if import.src.value == MARKER_MODULE {
        import.src.value = options.renderer_module.clone().into();
        import.src.raw = None;
      }
    }
  }
  *module = working;
  Ok(compiler.report)
}

struct Usage {
  bindings: HashSet<Id>,
  escaped: Option<Span>,
}
impl Visit for Usage {
  fn visit_import_decl(&mut self, _import: &ImportDecl) {}
  fn visit_ident(&mut self, ident: &Ident) {
    if self.bindings.contains(&ident.to_id()) {
      self.escaped.get_or_insert(ident.span);
    }
  }
  fn visit_jsx_element(&mut self, element: &JSXElement) {
    if matches!(&element.opening.name,JSXElementName::Ident(name) if self.bindings.contains(&name.to_id()))
    {
      element.opening.attrs.visit_with(self);
      element.children.visit_with(self);
    } else {
      element.visit_children_with(self);
    }
  }
}

struct Compiler<'a> {
  bindings: HashMap<Id, String>,
  options: &'a Options,
  report: Report,
  error: Option<Diagnostic>,
}
impl VisitMut for Compiler<'_> {
  fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
    if self.error.is_some() {
      return;
    }
    element.visit_mut_children_with(self);
    if self.error.is_some() {
      return;
    }
    if let JSXElementName::Ident(name) = &element.opening.name {
      if let Some(kind) = self.bindings.get(&name.to_id()) {
        let output = if kind == "CommandTabs" {
          commands::lower(element, self.options, &mut self.report)
        } else {
          codeblock::lower(element, self.options, &mut self.report)
        };
        match output {
          Ok(output) => {
            *element = output;
            self.report.transformed += 1;
          }
          Err(error) => {
            self.error = Some(Diagnostic::at(
              format!("[DOCS_SNIPPET_STATIC] {error}"),
              element.opening.name.span(),
            ))
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod mapping_tests;
#[cfg(test)]
mod tests;
