use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  module::{Module, ModuleId},
  swc_common::{source_map::DefaultSourceMapGenConfig, Globals, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ArrowExpr, AssignExpr, AssignTarget, AssignTargetPat, AwaitExpr, BlockStmtOrExpr, Callee,
    ClassDecl, ComputedPropName, Decl, DefaultDecl, ExportAll, ExportDecl, ExportDefaultDecl,
    ExportDefaultExpr, ExportDefaultSpecifier, ExportSpecifier, Expr, ExprOrSpread, ExprStmt,
    FnDecl, ForHead, ForInStmt, ForOfStmt, Id, Ident, ImportDecl, ImportSpecifier, KeyValuePatProp,
    KeyValueProp, Lit, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind, Module as SwcModule,
    ModuleDecl, ModuleExportName, ModuleItem, NamedExport, Pat, Prop, SimpleAssignTarget, Stmt,
    Str, TsEntityName, TsExportAssignment, TsImportEqualsDecl, TsModuleRef, UpdateExpr, VarDecl,
    VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::{
    codegen_module, create_codegen_config, parse_module, swc_try_with::resolve_module_mark,
    syntax_from_module_type,
  },
  swc_ecma_visit::{noop_visit_mut_type, Visit, VisitMut, VisitMutWith, VisitWith},
};
use swc_ecma_transforms_typescript::strip_type;

const FARM_SSR_IMPORT: &str = "__farm_ssr_import__";
const FARM_SSR_DYNAMIC_IMPORT: &str = "__farm_ssr_dynamic_import__";
const FARM_SSR_EXPORT_NAME: &str = "__farm_ssr_export_name__";
const FARM_SSR_EXPORT_ALL: &str = "__farm_ssr_export_all__";
const FARM_SSR_IMPORT_META: &str = "__farm_ssr_import_meta__";
const FARM_SSR_DEFAULT_EXPORT_IDENT: &str = "__farm_ssr_default_export__";
const FARM_SSR_IMPORT_PREFIX: &str = "__farm_ssr_imported__";
const FARM_SSR_REEXPORT_PREFIX: &str = "__farm_ssr_reexported__";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerTransformBailoutReason {
  UnsupportedTs,
  ImportMutation,
  UnhandledModuleDecl,
}

impl RunnerTransformBailoutReason {
  pub fn as_str(&self) -> &'static str {
    match self {
      RunnerTransformBailoutReason::UnsupportedTs => "unsupported-ts",
      RunnerTransformBailoutReason::ImportMutation => "import-mutation",
      RunnerTransformBailoutReason::UnhandledModuleDecl => "unhandled-module-decl",
    }
  }
}

pub fn transform_script_module_to_runner_code(
  module: &Module,
  module_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<String>), RunnerTransformBailoutReason> {
  let mut ast = module.meta.as_script().ast.clone();
  let mut cm = context.meta.get_module_source_map(module_id);

  if ast.body.is_empty() {
    let syntax = syntax_from_module_type(&module.module_type, context.config.script.parser.clone())
      .ok_or(RunnerTransformBailoutReason::UnhandledModuleDecl)?;
    let parsed = parse_module(
      module_id,
      Arc::new(module.content.to_string()),
      syntax,
      context.config.script.target,
    )
    .map_err(|_| RunnerTransformBailoutReason::UnhandledModuleDecl)?;
    ast = parsed.ast;
    cm = parsed.source_map;
  }

  transform_module_ast_to_runner(&mut ast, module.module_type.is_typescript())?;
  ast.visit_mut_with(&mut RunnerExprRewriter);

  let mut src_map = vec![];
  let code = codegen_module(
    &ast,
    cm.clone(),
    Some(&mut src_map),
    create_codegen_config(context),
    None,
  )
  .map_err(|_| RunnerTransformBailoutReason::UnhandledModuleDecl)?;

  let source_map = if context.config.sourcemap.is_false() {
    None
  } else {
    let map = cm.build_source_map(&src_map, None, DefaultSourceMapGenConfig);
    let mut serialized = vec![];
    map
      .to_writer(&mut serialized)
      .map_err(|_| RunnerTransformBailoutReason::UnhandledModuleDecl)?;
    Some(
      String::from_utf8(serialized)
        .map_err(|_| RunnerTransformBailoutReason::UnhandledModuleDecl)?,
    )
  };

  Ok((String::from_utf8_lossy(&code).to_string(), source_map))
}

fn resolve_module_scope(ast: &mut SwcModule, is_typescript: bool) {
  let globals = Globals::new();
  resolve_module_mark(ast, is_typescript, &globals);
}

fn prune_type_only_import_decls(ast: &mut SwcModule) {
  ast.body.retain(|item| match item {
    ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
      if import_decl.type_only {
        return false;
      }

      if import_decl.specifiers.is_empty() {
        return true;
      }

      import_decl
        .specifiers
        .iter()
        .any(|specifier| match specifier {
          ImportSpecifier::Named(named_specifier) => !named_specifier.is_type_only,
          _ => true,
        })
    }
    _ => true,
  });
}

fn strip_typescript_module_syntax(ast: &mut SwcModule) {
  prune_type_only_import_decls(ast);
  ast.visit_mut_with(&mut strip_type());
}

fn transform_module_ast_to_runner(
  ast: &mut SwcModule,
  is_typescript: bool,
) -> Result<(), RunnerTransformBailoutReason> {
  if is_typescript {
    strip_typescript_module_syntax(ast);
  }

  let eligibility = analyze_transform_eligibility(ast);
  if !eligibility.should_transform() {
    return if eligibility.has_unsupported_ts_syntax {
      Err(RunnerTransformBailoutReason::UnsupportedTs)
    } else {
      Err(RunnerTransformBailoutReason::UnhandledModuleDecl)
    };
  }

  resolve_module_scope(ast, is_typescript);
  let import_binding_ids = collect_import_binding_ids(ast);
  let mutated_import_bindings = collect_import_binding_mutations(ast, &import_binding_ids);

  let mut transformer = RunnerModuleTransformer::new(mutated_import_bindings);
  let original = std::mem::take(&mut ast.body);
  let mut transformed = Vec::with_capacity(original.len());

  for item in original {
    transformer.transform_module_item(item, &mut transformed)?;
  }

  ast.body = transformed;
  ast.visit_mut_with(&mut ImportBindingRewriter::new(
    transformer.import_bindings_map,
  ));
  Ok(())
}

struct RunnerModuleTransformer {
  import_counter: usize,
  reexport_counter: usize,
  import_bindings_map: HashMap<Id, Expr>,
  mutated_import_bindings: HashSet<Id>,
}

impl RunnerModuleTransformer {
  fn new(mutated_import_bindings: HashSet<Id>) -> Self {
    Self {
      import_counter: 0,
      reexport_counter: 0,
      import_bindings_map: HashMap::default(),
      mutated_import_bindings,
    }
  }

  fn transform_module_item(
    &mut self,
    item: ModuleItem,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    match item {
      ModuleItem::Stmt(stmt) => {
        out.push(ModuleItem::Stmt(stmt));
      }
      ModuleItem::ModuleDecl(decl) => self.transform_module_decl(decl, out)?,
    }

    Ok(())
  }

  fn transform_module_decl(
    &mut self,
    decl: ModuleDecl,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    match decl {
      ModuleDecl::Import(import_decl) => self.transform_import_decl(import_decl, out),
      ModuleDecl::ExportDecl(export_decl) => self.transform_export_decl(export_decl, out),
      ModuleDecl::ExportDefaultDecl(default_decl) => {
        self.transform_export_default_decl(default_decl, out)
      }
      ModuleDecl::ExportDefaultExpr(default_expr) => {
        self.transform_export_default_expr(default_expr, out)
      }
      ModuleDecl::ExportNamed(named_export) => self.transform_named_export(named_export, out),
      ModuleDecl::ExportAll(export_all) => self.transform_export_all(export_all, out),
      ModuleDecl::TsExportAssignment(export_assignment) => {
        self.transform_ts_export_assignment(export_assignment, out)
      }
      ModuleDecl::TsNamespaceExport(_) => Ok(()),
      ModuleDecl::TsImportEquals(import_equals) => {
        self.transform_ts_import_equals(*import_equals, out)
      }
    }
  }

  fn transform_import_decl(
    &mut self,
    import_decl: ImportDecl,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    let ImportDecl {
      specifiers,
      src,
      type_only,
      with,
      ..
    } = import_decl;

    if type_only {
      return Ok(());
    }

    let source = src.value.to_string();
    let import_with = with.map(|with_obj| Expr::Object(*with_obj));

    if specifiers.is_empty() {
      out.push(ModuleItem::Stmt(create_import_stmt(
        None,
        &source,
        import_with,
      )));
      return Ok(());
    }

    let runtime_specifiers = specifiers
      .into_iter()
      .filter(|specifier| match specifier {
        ImportSpecifier::Named(named_specifier) => !named_specifier.is_type_only,
        _ => true,
      })
      .collect::<Vec<_>>();

    if runtime_specifiers.is_empty() {
      return Ok(());
    }

    let ns_ident = self.next_import_ident();
    out.push(ModuleItem::Stmt(create_import_stmt(
      Some(ns_ident.clone()),
      &source,
      import_with,
    )));

    for specifier in runtime_specifiers {
      match specifier {
        ImportSpecifier::Default(default_specifier) => {
          let local = default_specifier.local;
          let local_id = local.to_id();
          let replacement = create_member_expr(Expr::Ident(ns_ident.clone()), "default");
          if self.mutated_import_bindings.contains(&local_id) {
            out.push(ModuleItem::Stmt(create_const_from_expr(local, replacement)));
          } else {
            self.import_bindings_map.insert(local_id, replacement);
          }
        }
        ImportSpecifier::Namespace(namespace_specifier) => {
          let local = namespace_specifier.local;
          let local_id = local.to_id();
          let replacement = Expr::Ident(ns_ident.clone());
          if self.mutated_import_bindings.contains(&local_id) {
            out.push(ModuleItem::Stmt(create_const_from_expr(local, replacement)));
          } else {
            self.import_bindings_map.insert(local_id, replacement);
          }
        }
        ImportSpecifier::Named(named_specifier) => {
          let imported = named_specifier
            .imported
            .as_ref()
            .map(module_export_name_to_string)
            .unwrap_or_else(|| named_specifier.local.sym.to_string());
          let local = named_specifier.local;
          let local_id = local.to_id();
          let replacement = create_member_expr(Expr::Ident(ns_ident.clone()), &imported);
          if self.mutated_import_bindings.contains(&local_id) {
            out.push(ModuleItem::Stmt(create_const_from_expr(local, replacement)));
          } else {
            self.import_bindings_map.insert(local_id, replacement);
          }
        }
      }
    }

    Ok(())
  }

  fn transform_export_decl(
    &mut self,
    export_decl: ExportDecl,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    match export_decl.decl {
      Decl::Class(class_decl) => {
        let export_ident = class_decl.ident.clone();
        let export_name = export_ident.sym.to_string();
        out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(class_decl))));
        out.push(ModuleItem::Stmt(create_export_name_stmt(
          &export_name,
          Expr::Ident(export_ident),
        )));
      }
      Decl::Fn(fn_decl) => {
        let export_ident = fn_decl.ident.clone();
        let export_name = export_ident.sym.to_string();
        out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl))));
        out.push(ModuleItem::Stmt(create_export_name_stmt(
          &export_name,
          Expr::Ident(export_ident),
        )));
      }
      Decl::Var(var_decl) => {
        let exported_idents = collect_var_decl_idents(&var_decl);
        out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))));

        for ident in exported_idents {
          let export_name = ident.sym.to_string();
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            &export_name,
            Expr::Ident(ident),
          )));
        }
      }
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        // Type-only exports do not have runtime side effects.
      }
      _ => return Err(RunnerTransformBailoutReason::UnhandledModuleDecl),
    }

    Ok(())
  }

  fn transform_export_default_decl(
    &mut self,
    default_decl: ExportDefaultDecl,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    match default_decl.decl {
      DefaultDecl::Fn(fn_expr) => {
        if let Some(ident) = fn_expr.ident {
          out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
            ident: ident.clone(),
            declare: false,
            function: fn_expr.function,
          }))));
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            "default",
            Expr::Ident(ident),
          )));
          return Ok(());
        }

        out.push(ModuleItem::Stmt(create_const_from_expr(
          create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT),
          Expr::Fn(fn_expr),
        )));
        out.push(ModuleItem::Stmt(create_export_name_stmt(
          "default",
          Expr::Ident(create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT)),
        )));
      }
      DefaultDecl::Class(class_expr) => {
        if let Some(ident) = class_expr.ident {
          out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
            ident: ident.clone(),
            declare: false,
            class: class_expr.class,
          }))));
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            "default",
            Expr::Ident(ident),
          )));
          return Ok(());
        }

        out.push(ModuleItem::Stmt(create_const_from_expr(
          create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT),
          Expr::Class(class_expr),
        )));
        out.push(ModuleItem::Stmt(create_export_name_stmt(
          "default",
          Expr::Ident(create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT)),
        )));
      }
      DefaultDecl::TsInterfaceDecl(_) => {
        // Type-only export
      }
    }

    Ok(())
  }

  fn transform_export_default_expr(
    &mut self,
    default_expr: ExportDefaultExpr,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    out.push(ModuleItem::Stmt(create_const_from_expr(
      create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT),
      *default_expr.expr,
    )));
    out.push(ModuleItem::Stmt(create_export_name_stmt(
      "default",
      Expr::Ident(create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT)),
    )));
    Ok(())
  }

  fn transform_named_export(
    &mut self,
    named_export: NamedExport,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    let NamedExport {
      specifiers,
      src,
      type_only,
      with,
      ..
    } = named_export;

    if type_only {
      return Ok(());
    }

    let export_with = with.map(|with_obj| Expr::Object(*with_obj));

    if let Some(source) = src {
      let runtime_specifiers = specifiers
        .into_iter()
        .filter(|specifier| match specifier {
          ExportSpecifier::Named(named_specifier) => !named_specifier.is_type_only,
          _ => true,
        })
        .collect::<Vec<_>>();

      if runtime_specifiers.is_empty() {
        return Ok(());
      }

      let reexport_namespace = self.next_reexport_ident();
      out.push(ModuleItem::Stmt(create_import_stmt(
        Some(reexport_namespace.clone()),
        source.value.as_ref(),
        export_with,
      )));

      for specifier in runtime_specifiers {
        match specifier {
          ExportSpecifier::Namespace(namespace_specifier) => {
            let exported_name = module_export_name_to_string(&namespace_specifier.name);
            out.push(ModuleItem::Stmt(create_export_name_stmt(
              &exported_name,
              Expr::Ident(reexport_namespace.clone()),
            )));
          }
          ExportSpecifier::Default(default_specifier) => {
            let exported_name = default_specifier.exported.sym.to_string();
            out.push(ModuleItem::Stmt(create_export_name_stmt(
              &exported_name,
              create_member_expr(Expr::Ident(reexport_namespace.clone()), "default"),
            )));
          }
          ExportSpecifier::Named(named_specifier) => {
            let imported_name = module_export_name_to_string(&named_specifier.orig);
            let exported_name = named_specifier
              .exported
              .as_ref()
              .map(module_export_name_to_string)
              .unwrap_or_else(|| imported_name.clone());

            out.push(ModuleItem::Stmt(create_export_name_stmt(
              &exported_name,
              create_member_expr(Expr::Ident(reexport_namespace.clone()), &imported_name),
            )));
          }
        }
      }

      return Ok(());
    }

    for specifier in specifiers {
      match specifier {
        ExportSpecifier::Named(named_specifier) => {
          if named_specifier.is_type_only {
            continue;
          }

          let local_ident = module_export_name_to_ident(&named_specifier.orig)
            .ok_or(RunnerTransformBailoutReason::UnhandledModuleDecl)?;
          let exported_name = named_specifier
            .exported
            .as_ref()
            .map(module_export_name_to_string)
            .unwrap_or_else(|| local_ident.sym.to_string());

          out.push(ModuleItem::Stmt(create_export_name_stmt(
            &exported_name,
            Expr::Ident(local_ident),
          )));
        }
        ExportSpecifier::Default(ExportDefaultSpecifier { exported, .. }) => {
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            exported.sym.as_ref(),
            Expr::Ident(exported.clone()),
          )));
        }
        ExportSpecifier::Namespace(namespace_specifier) => {
          let local_ident = module_export_name_to_ident(&namespace_specifier.name)
            .ok_or(RunnerTransformBailoutReason::UnhandledModuleDecl)?;
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            local_ident.sym.as_ref(),
            Expr::Ident(local_ident.clone()),
          )));
        }
      }
    }

    Ok(())
  }

  fn transform_export_all(
    &mut self,
    export_all: ExportAll,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    let ExportAll {
      src,
      type_only,
      with,
      ..
    } = export_all;

    if type_only {
      return Ok(());
    }

    let export_with = with.map(|with_obj| Expr::Object(*with_obj));
    let reexport_namespace = self.next_reexport_ident();
    out.push(ModuleItem::Stmt(create_import_stmt(
      Some(reexport_namespace.clone()),
      src.value.as_ref(),
      export_with,
    )));

    out.push(ModuleItem::Stmt(create_export_all_stmt(Expr::Ident(
      reexport_namespace,
    ))));

    Ok(())
  }

  fn transform_ts_export_assignment(
    &mut self,
    export_assignment: TsExportAssignment,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    out.push(ModuleItem::Stmt(create_const_from_expr(
      create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT),
      *export_assignment.expr,
    )));
    out.push(ModuleItem::Stmt(create_export_name_stmt(
      "default",
      Expr::Ident(create_runtime_ident(FARM_SSR_DEFAULT_EXPORT_IDENT)),
    )));

    Ok(())
  }

  fn transform_ts_import_equals(
    &mut self,
    import_equals: TsImportEqualsDecl,
    out: &mut Vec<ModuleItem>,
  ) -> Result<(), RunnerTransformBailoutReason> {
    let TsImportEqualsDecl {
      id,
      module_ref,
      is_export,
      is_type_only,
      ..
    } = import_equals;

    if is_type_only {
      return Ok(());
    }

    let source = match module_ref {
      TsModuleRef::TsExternalModuleRef(external_module_ref) => {
        external_module_ref.expr.value.to_string()
      }
      TsModuleRef::TsEntityName(entity_name) => {
        let resolved_expr = create_expr_from_ts_entity_name(&entity_name);
        out.push(ModuleItem::Stmt(create_const_from_expr(
          id.clone(),
          resolved_expr,
        )));

        if is_export {
          let export_name = id.sym.to_string();
          out.push(ModuleItem::Stmt(create_export_name_stmt(
            &export_name,
            Expr::Ident(id),
          )));
        }

        return Ok(());
      }
    };

    out.push(ModuleItem::Stmt(create_import_stmt(
      Some(id.clone()),
      &source,
      None,
    )));

    if is_export {
      let export_name = id.sym.to_string();
      out.push(ModuleItem::Stmt(create_export_name_stmt(
        &export_name,
        Expr::Ident(id),
      )));
    }

    Ok(())
  }

  fn next_import_ident(&mut self) -> Ident {
    let ident = create_runtime_ident(&format!("{FARM_SSR_IMPORT_PREFIX}{}", self.import_counter));
    self.import_counter += 1;
    ident
  }

  fn next_reexport_ident(&mut self) -> Ident {
    let ident = create_runtime_ident(&format!(
      "{FARM_SSR_REEXPORT_PREFIX}{}",
      self.reexport_counter
    ));
    self.reexport_counter += 1;
    ident
  }
}

fn create_expr_from_ts_entity_name(entity_name: &TsEntityName) -> Expr {
  match entity_name {
    TsEntityName::Ident(ident) => Expr::Ident(ident.clone()),
    TsEntityName::TsQualifiedName(qualified_name) => create_member_expr(
      create_expr_from_ts_entity_name(&qualified_name.left),
      qualified_name.right.sym.as_ref(),
    ),
  }
}

struct ImportBindingRewriter {
  import_bindings_map: HashMap<Id, Expr>,
}

impl ImportBindingRewriter {
  fn new(import_bindings_map: HashMap<Id, Expr>) -> Self {
    Self {
      import_bindings_map,
    }
  }
}

impl VisitMut for ImportBindingRewriter {
  noop_visit_mut_type!();

  fn visit_mut_prop(&mut self, prop: &mut Prop) {
    if let Prop::Shorthand(shorthand) = prop {
      if let Some(replacement) = self.import_bindings_map.get(&shorthand.to_id()) {
        *prop = Prop::KeyValue(KeyValueProp {
          key: shorthand.clone().into(),
          value: Box::new(replacement.clone()),
        });
      }
      return;
    }

    prop.visit_mut_children_with(self);
  }

  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Expr::Ident(ident) = expr {
      if let Some(replacement) = self.import_bindings_map.get(&ident.to_id()) {
        *expr = replacement.clone();
      }
      return;
    }

    expr.visit_mut_children_with(self);
  }
}

struct RunnerExprRewriter;

impl VisitMut for RunnerExprRewriter {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    expr.visit_mut_children_with(self);

    match expr {
      Expr::Call(call_expr) => {
        if matches!(call_expr.callee, Callee::Import(_)) {
          call_expr.callee = Callee::Expr(Box::new(Expr::Ident(create_runtime_ident(
            FARM_SSR_DYNAMIC_IMPORT,
          ))));
          return;
        }

        if let Callee::Expr(callee_expr) = &mut call_expr.callee {
          if let Expr::Ident(ident) = callee_expr.as_ref() {
            if ident.sym == "import" {
              *callee_expr = Box::new(Expr::Ident(create_runtime_ident(FARM_SSR_DYNAMIC_IMPORT)));
            }
          }
        }
      }
      Expr::MetaProp(MetaPropExpr {
        kind: MetaPropKind::ImportMeta,
        ..
      }) => {
        *expr = Expr::Ident(create_runtime_ident(FARM_SSR_IMPORT_META));
      }
      Expr::Member(member_expr) => {
        if is_import_meta_member(member_expr) {
          *expr = Expr::Ident(create_runtime_ident(FARM_SSR_IMPORT_META));
        }
      }
      _ => {}
    }
  }
}

fn collect_import_binding_ids(ast: &SwcModule) -> HashSet<Id> {
  let mut ids = HashSet::default();

  for item in &ast.body {
    let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item else {
      continue;
    };

    if import_decl.type_only {
      continue;
    }

    for specifier in &import_decl.specifiers {
      match specifier {
        ImportSpecifier::Default(default_specifier) => {
          ids.insert(default_specifier.local.to_id());
        }
        ImportSpecifier::Namespace(namespace_specifier) => {
          ids.insert(namespace_specifier.local.to_id());
        }
        ImportSpecifier::Named(named_specifier) => {
          if !named_specifier.is_type_only {
            ids.insert(named_specifier.local.to_id());
          }
        }
      }
    }
  }

  ids
}

struct ImportBindingMutationCollector<'a> {
  import_binding_ids: &'a HashSet<Id>,
  mutated_ids: HashSet<Id>,
}

impl Visit for ImportBindingMutationCollector<'_> {
  fn visit_update_expr(&mut self, update_expr: &UpdateExpr) {
    if let Some(id) = extract_ident_id_from_expr(update_expr.arg.as_ref()) {
      if self.import_binding_ids.contains(&id) {
        self.mutated_ids.insert(id);
      }
    }

    update_expr.visit_children_with(self);
  }

  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    collect_import_binding_assign_target(
      &assign_expr.left,
      self.import_binding_ids,
      &mut self.mutated_ids,
    );
    assign_expr.visit_children_with(self);
  }

  fn visit_for_of_stmt(&mut self, for_of_stmt: &ForOfStmt) {
    collect_import_binding_for_head(
      &for_of_stmt.left,
      self.import_binding_ids,
      &mut self.mutated_ids,
    );
    for_of_stmt.visit_children_with(self);
  }

  fn visit_for_in_stmt(&mut self, for_in_stmt: &ForInStmt) {
    collect_import_binding_for_head(
      &for_in_stmt.left,
      self.import_binding_ids,
      &mut self.mutated_ids,
    );
    for_in_stmt.visit_children_with(self);
  }
}

fn collect_import_binding_assign_target(
  target: &AssignTarget,
  import_binding_ids: &HashSet<Id>,
  mutated_ids: &mut HashSet<Id>,
) {
  match target {
    AssignTarget::Simple(simple_target) => {
      collect_import_binding_simple_assign_target(simple_target, import_binding_ids, mutated_ids)
    }
    AssignTarget::Pat(assign_target_pat) => {
      collect_import_binding_assign_target_pat(assign_target_pat, import_binding_ids, mutated_ids)
    }
  }
}

fn collect_import_binding_simple_assign_target(
  target: &SimpleAssignTarget,
  import_binding_ids: &HashSet<Id>,
  mutated_ids: &mut HashSet<Id>,
) {
  match target {
    SimpleAssignTarget::Ident(binding_ident) => {
      let id = binding_ident.id.to_id();
      if import_binding_ids.contains(&id) {
        mutated_ids.insert(id);
      }
    }
    SimpleAssignTarget::Paren(paren_expr) => {
      if let Some(id) = extract_ident_id_from_expr(paren_expr.expr.as_ref()) {
        if import_binding_ids.contains(&id) {
          mutated_ids.insert(id);
        }
      }
    }
    _ => {}
  }
}

fn collect_import_binding_assign_target_pat(
  target: &AssignTargetPat,
  import_binding_ids: &HashSet<Id>,
  mutated_ids: &mut HashSet<Id>,
) {
  let pat: Pat = target.clone().into();
  collect_import_binding_pat(&pat, import_binding_ids, mutated_ids);
}

fn collect_import_binding_for_head(
  head: &ForHead,
  import_binding_ids: &HashSet<Id>,
  mutated_ids: &mut HashSet<Id>,
) {
  if let ForHead::Pat(pat) = head {
    collect_import_binding_pat(pat.as_ref(), import_binding_ids, mutated_ids);
  }
}

fn collect_import_binding_pat(
  pat: &Pat,
  import_binding_ids: &HashSet<Id>,
  mutated_ids: &mut HashSet<Id>,
) {
  let mut idents = vec![];
  collect_pat_idents(pat, &mut idents);
  for ident in idents {
    let id = ident.to_id();
    if import_binding_ids.contains(&id) {
      mutated_ids.insert(id);
    }
  }
}

fn collect_import_binding_mutations(
  ast: &SwcModule,
  import_binding_ids: &HashSet<Id>,
) -> HashSet<Id> {
  if import_binding_ids.is_empty() {
    return HashSet::default();
  }

  let mut collector = ImportBindingMutationCollector {
    import_binding_ids,
    mutated_ids: HashSet::default(),
  };
  ast.visit_with(&mut collector);
  collector.mutated_ids
}

fn extract_ident_id_from_expr(expr: &Expr) -> Option<Id> {
  match expr {
    Expr::Ident(ident) => Some(ident.to_id()),
    Expr::Paren(paren_expr) => extract_ident_id_from_expr(paren_expr.expr.as_ref()),
    _ => None,
  }
}

fn is_import_meta_member(member_expr: &MemberExpr) -> bool {
  let prop_is_meta = match &member_expr.prop {
    MemberProp::Ident(ident_name) => ident_name.sym == "meta",
    MemberProp::Computed(computed) => {
      if let Expr::Lit(Lit::Str(str)) = computed.expr.as_ref() {
        str.value == "meta"
      } else {
        false
      }
    }
    _ => false,
  };

  if !prop_is_meta {
    return false;
  }

  if let Expr::Ident(ident) = member_expr.obj.as_ref() {
    return ident.sym == "import";
  }

  false
}

fn is_dynamic_import_call(call_expr: &farmfe_core::swc_ecma_ast::CallExpr) -> bool {
  if matches!(call_expr.callee, Callee::Import(_)) {
    return true;
  }

  if let Callee::Expr(callee_expr) = &call_expr.callee {
    if let Expr::Ident(ident) = callee_expr.as_ref() {
      return ident.sym == "import";
    }
  }

  false
}

struct RunnerTransformEligibility {
  has_module_decl: bool,
  has_dynamic_import: bool,
  has_import_meta: bool,
  has_unsupported_ts_syntax: bool,
}

impl RunnerTransformEligibility {
  fn should_transform(&self) -> bool {
    !self.has_unsupported_ts_syntax
      && (self.has_module_decl || self.has_dynamic_import || self.has_import_meta)
  }
}

impl Visit for RunnerTransformEligibility {
  fn visit_module_decl(&mut self, decl: &ModuleDecl) {
    self.has_module_decl = true;
    decl.visit_children_with(self);
  }

  fn visit_decl(&mut self, decl: &Decl) {
    match decl {
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) => {
        self.has_unsupported_ts_syntax = true;
        return;
      }
      Decl::TsEnum(enum_decl) => {
        if !enum_decl.declare {
          self.has_unsupported_ts_syntax = true;
          return;
        }
      }
      Decl::TsModule(module_decl) => {
        if !module_decl.declare {
          self.has_unsupported_ts_syntax = true;
          return;
        }
      }
      _ => {}
    }

    decl.visit_children_with(self);
  }

  fn visit_ts_type_ann(&mut self, _: &farmfe_core::swc_ecma_ast::TsTypeAnn) {
    self.has_unsupported_ts_syntax = true;
  }

  fn visit_ts_type_param_decl(&mut self, _: &farmfe_core::swc_ecma_ast::TsTypeParamDecl) {
    self.has_unsupported_ts_syntax = true;
  }

  fn visit_ts_type_param_instantiation(
    &mut self,
    _: &farmfe_core::swc_ecma_ast::TsTypeParamInstantiation,
  ) {
    self.has_unsupported_ts_syntax = true;
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if matches!(
      expr,
      Expr::TsTypeAssertion(_)
        | Expr::TsConstAssertion(_)
        | Expr::TsNonNull(_)
        | Expr::TsAs(_)
        | Expr::TsInstantiation(_)
        | Expr::TsSatisfies(_)
    ) {
      self.has_unsupported_ts_syntax = true;
      return;
    }

    expr.visit_children_with(self);
  }

  fn visit_simple_assign_target(&mut self, target: &SimpleAssignTarget) {
    if matches!(
      target,
      SimpleAssignTarget::TsAs(_)
        | SimpleAssignTarget::TsSatisfies(_)
        | SimpleAssignTarget::TsNonNull(_)
        | SimpleAssignTarget::TsTypeAssertion(_)
        | SimpleAssignTarget::TsInstantiation(_)
    ) {
      self.has_unsupported_ts_syntax = true;
      return;
    }

    target.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &farmfe_core::swc_ecma_ast::CallExpr) {
    if is_dynamic_import_call(call_expr) {
      self.has_dynamic_import = true;
    }

    call_expr.visit_children_with(self);
  }

  fn visit_meta_prop_expr(&mut self, meta_prop_expr: &MetaPropExpr) {
    if matches!(meta_prop_expr.kind, MetaPropKind::ImportMeta) {
      self.has_import_meta = true;
    }
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    if is_import_meta_member(member_expr) {
      self.has_import_meta = true;
    }

    member_expr.visit_children_with(self);
  }
}

#[allow(dead_code)]
fn can_transform_to_runner(ast: &SwcModule) -> bool {
  analyze_transform_eligibility(ast).should_transform()
}

fn analyze_transform_eligibility(ast: &SwcModule) -> RunnerTransformEligibility {
  let mut eligibility = RunnerTransformEligibility {
    has_module_decl: false,
    has_dynamic_import: false,
    has_import_meta: false,
    has_unsupported_ts_syntax: false,
  };
  ast.visit_with(&mut eligibility);
  eligibility
}

fn create_runtime_ident(value: &str) -> Ident {
  Ident::new(value.into(), DUMMY_SP, SyntaxContext::empty())
}

fn create_string_lit(value: &str) -> Str {
  Str {
    span: DUMMY_SP,
    value: value.into(),
    raw: None,
  }
}

fn create_helper_call_expr(name: &str, args: Vec<Expr>) -> Expr {
  Expr::Call(farmfe_core::swc_ecma_ast::CallExpr {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    callee: Callee::Expr(Box::new(Expr::Ident(create_runtime_ident(name)))),
    args: args
      .into_iter()
      .map(|expr| ExprOrSpread {
        spread: None,
        expr: Box::new(expr),
      })
      .collect(),
    type_args: None,
  })
}

fn create_import_stmt(binding: Option<Ident>, source: &str, import_with: Option<Expr>) -> Stmt {
  let mut import_args = vec![Expr::Lit(Lit::Str(create_string_lit(source)))];
  if let Some(import_with) = import_with {
    import_args.push(import_with);
  }

  let awaited_import = Expr::Await(AwaitExpr {
    span: DUMMY_SP,
    arg: Box::new(create_helper_call_expr(FARM_SSR_IMPORT, import_args)),
  });

  if let Some(binding) = binding {
    return create_const_from_expr(binding, awaited_import);
  }

  Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(awaited_import),
  })
}

fn create_member_expr(target: Expr, prop: &str) -> Expr {
  Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(target),
    prop: MemberProp::Computed(ComputedPropName {
      span: DUMMY_SP,
      expr: Box::new(Expr::Lit(Lit::Str(create_string_lit(prop)))),
    }),
  })
}

fn create_const_from_expr(binding: Ident, expr: Expr) -> Stmt {
  Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(binding.into()),
      init: Some(Box::new(expr)),
      definite: false,
    }],
    declare: false,
    ctxt: SyntaxContext::empty(),
  })))
}

fn create_export_name_stmt(exported_name: &str, value: Expr) -> Stmt {
  let getter = Expr::Arrow(ArrowExpr {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    params: vec![],
    body: Box::new(BlockStmtOrExpr::Expr(Box::new(value))),
    is_async: false,
    is_generator: false,
    type_params: None,
    return_type: None,
  });

  Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(create_helper_call_expr(
      FARM_SSR_EXPORT_NAME,
      vec![
        Expr::Lit(Lit::Str(create_string_lit(exported_name))),
        getter,
      ],
    )),
  })
}

fn create_export_all_stmt(namespace: Expr) -> Stmt {
  Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(create_helper_call_expr(
      FARM_SSR_EXPORT_ALL,
      vec![namespace],
    )),
  })
}

fn collect_var_decl_idents(var_decl: &VarDecl) -> Vec<Ident> {
  let mut idents = vec![];

  for declarator in &var_decl.decls {
    collect_pat_idents(&declarator.name, &mut idents);
  }

  idents
}

fn collect_pat_idents(pat: &Pat, idents: &mut Vec<Ident>) {
  match pat {
    Pat::Ident(binding_ident) => {
      idents.push(binding_ident.id.clone());
    }
    Pat::Array(array_pat) => {
      for pat in array_pat.elems.iter().flatten() {
        collect_pat_idents(pat, idents);
      }
    }
    Pat::Rest(rest_pat) => {
      collect_pat_idents(&rest_pat.arg, idents);
    }
    Pat::Object(object_pat) => {
      for property in &object_pat.props {
        match property {
          farmfe_core::swc_ecma_ast::ObjectPatProp::Assign(assign_prop) => {
            idents.push(assign_prop.key.id.clone());
          }
          farmfe_core::swc_ecma_ast::ObjectPatProp::KeyValue(KeyValuePatProp { value, .. }) => {
            collect_pat_idents(value, idents);
          }
          farmfe_core::swc_ecma_ast::ObjectPatProp::Rest(rest) => {
            collect_pat_idents(&rest.arg, idents);
          }
        }
      }
    }
    Pat::Assign(assign_pat) => {
      collect_pat_idents(&assign_pat.left, idents);
    }
    _ => {}
  }
}

fn module_export_name_to_string(name: &ModuleExportName) -> String {
  match name {
    ModuleExportName::Ident(ident) => ident.sym.to_string(),
    ModuleExportName::Str(str) => str.value.to_string(),
  }
}

fn module_export_name_to_ident(name: &ModuleExportName) -> Option<Ident> {
  match name {
    ModuleExportName::Ident(ident) => Some(ident.clone()),
    ModuleExportName::Str(_) => None,
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    module::ModuleId,
    swc_ecma_parser::{EsSyntax, Syntax, TsSyntax},
  };
  use farmfe_toolkit::script::{codegen_module, parse_module};

  use super::{
    can_transform_to_runner, transform_module_ast_to_runner, RunnerExprRewriter,
    RunnerTransformBailoutReason,
  };
  use farmfe_toolkit::swc_ecma_visit::VisitMutWith;

  #[test]
  fn should_transform_esm_to_runner_helpers() {
    let module_id = ModuleId::from("index.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo as localFoo } from './dep';
export const value = localFoo + 1;
export { value as renamed };
export * from './dep2';
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("__farm_ssr_import__"));
    assert!(output.contains("__farm_ssr_export_name__"));
    assert!(output.contains("__farm_ssr_export_all__"));
    assert!(!output.contains("import "));
    assert!(!output.contains("export "));
  }

  #[test]
  fn should_rewrite_dynamic_import_and_import_meta() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
const mod = await import('./dep');
export const metaUrl = import.meta.url;
export default mod;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("__farm_ssr_dynamic_import__"), "{output}");
    assert!(output.contains("__farm_ssr_import_meta__"), "{output}");
  }

  #[test]
  fn should_transform_reexport_namespace_and_named_from() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
export { foo as bar } from './dep';
export * as depNs from './dep2';
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"bar\""),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"depNs\""),
      "{output}"
    );
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_export_var_destructuring() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
const dep = { a: 1, b: 2 };
export const { a, b } = dep;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("__farm_ssr_export_name__(\"a\""),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"b\""),
      "{output}"
    );
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_ignore_type_only_import_and_export() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import type { Foo } from './dep';
export type { Foo } from './dep';
export const value = 1;
"#
        .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_transform_plain_cjs_like_script() {
    let module_id = ModuleId::from("entry.js");
    let parsed = parse_module(
      &module_id,
      Arc::new("const dep = require('./dep'); module.exports = dep;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    assert!(can_transform_to_runner(&ast).eq(&false));
    assert!(matches!(
      transform_module_ast_to_runner(&mut ast, false),
      Err(RunnerTransformBailoutReason::UnhandledModuleDecl)
    ));

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(!output.contains("__farm_ssr_export_name__"), "{output}");
  }

  #[test]
  fn should_ignore_inline_type_only_import_specifier() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { type Foo } from './dep'; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_ignore_default_type_only_import_declaration() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import type Foo from './dep'; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_ignore_namespace_type_only_import_declaration() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import type * as dep from './dep'; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_keep_runtime_binding_for_mixed_type_and_value_import() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { type Foo, bar } from './dep'; export const value = bar;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains("Foo"), "{output}");
  }

  #[test]
  fn should_rewrite_import_binding_references_to_namespace_members() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { counter as localCounter } from './dep';
const read = () => localCounter + 1;
export { localCounter, read };
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("__farm_ssr_import__"), "{output}");
    assert!(!output.contains("const localCounter ="), "{output}");
    assert!(
      output.contains("__farm_ssr_imported__0[\"counter\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"localCounter\""),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"read\""),
      "{output}"
    );
  }

  #[test]
  fn should_rewrite_shorthand_props_using_import_bindings() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import { foo } from './dep'; const obj = { foo }; export const value = obj.foo;"
          .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("foo: __farm_ssr_imported__0[\"foo\"]")
        || output.contains("foo:__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_treat_shadowed_binding_update_as_import_mutation() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
function mutate(foo) {
  foo++;
  return foo;
}
export const value = foo;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("function mutate(foo)"), "{output}");
    assert!(
      output.contains("foo++") || output.contains("foo ++"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_treat_shadowed_parenthesized_binding_update_as_import_mutation() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
function mutate(foo) {
  ((foo))++;
  return foo;
}
export const value = foo;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("function mutate(foo)"), "{output}");
    assert!(
      output.contains("((foo))++")
        || output.contains("((foo)) ++")
        || output.contains("(foo)++")
        || output.contains("(foo) ++"),
      "{output}"
    );
    assert!(
      !output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        && !output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_treat_shadowed_parenthesized_binding_assignment_as_import_mutation() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
function mutate(foo) {
  ((foo)) = 1;
  return foo;
}
export const value = foo;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("function mutate(foo)"), "{output}");
    assert!(
      output.contains("((foo)) = 1")
        || output.contains("((foo))=1")
        || output.contains("(foo) = 1")
        || output.contains("(foo)=1"),
      "{output}"
    );
    assert!(
      !output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        && !output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_treat_block_shadowed_parenthesized_binding_update_as_import_mutation() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
{
  let foo = 1;
  ((foo))++;
}
export const value = foo;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("let foo = 1") || output.contains("let foo=1"),
      "{output}"
    );
    assert!(
      !output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        && !output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_preserve_ordering_with_parenthesized_import_mutation_and_reexport() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
((foo))++;
export { bar as reBar } from './dep2';
export const value = foo;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let dep_import_idx = output.find("\"./dep\"").expect("dep import");
    let foo_alias_idx = output
      .find("const foo = __farm_ssr_imported__0[\"foo\"]")
      .or_else(|| output.find("const foo=__farm_ssr_imported__0[\"foo\"]"))
      .expect("foo alias");
    let dep2_import_idx = output.find("\"./dep2\"").expect("dep2 import");
    let reexport_idx = output
      .find("__farm_ssr_export_name__(\"reBar\"")
      .expect("reexport call");
    let value_export_idx = output
      .find("__farm_ssr_export_name__(\"value\"")
      .expect("value export call");

    assert!(dep_import_idx < foo_alias_idx, "{output}");
    assert!(foo_alias_idx < dep2_import_idx, "{output}");
    assert!(dep2_import_idx < reexport_idx, "{output}");
    assert!(reexport_idx < value_export_idx, "{output}");
  }

  #[test]
  fn should_preserve_ordering_with_import_attributes_and_parenthesized_mutation() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { foo } from './dep';
import data from './data.json' with { type: 'json' };
((foo))++;
export { bar as reBar } from './dep2.json' with { type: 'json' };
export const value = data;
"#
        .to_string(),
      ),
      Syntax::Es(EsSyntax {
        import_attributes: true,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let dep_import_idx = output.find("\"./dep\"").expect("dep import");
    let foo_alias_idx = output
      .find("const foo = __farm_ssr_imported__0[\"foo\"]")
      .or_else(|| output.find("const foo=__farm_ssr_imported__0[\"foo\"]"))
      .expect("foo alias");
    let data_import_idx = output.find("\"./data.json\"").expect("data import");
    let dep2_import_idx = output.find("\"./dep2.json\"").expect("dep2 import");
    let reexport_idx = output
      .find("__farm_ssr_export_name__(\"reBar\"")
      .expect("reexport call");
    let value_export_idx = output
      .find("__farm_ssr_export_name__(\"value\"")
      .expect("value export call");

    assert!(dep_import_idx < foo_alias_idx, "{output}");
    assert!(foo_alias_idx < data_import_idx, "{output}");
    assert!(data_import_idx < dep2_import_idx, "{output}");
    assert!(dep2_import_idx < reexport_idx, "{output}");
    assert!(reexport_idx < value_export_idx, "{output}");
    assert!(
      output.contains("__farm_ssr_import__(\"./data.json\","),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_import__(\"./dep2.json\","),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_import_binding_is_updated() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { foo } from './dep'; foo++; export const value = 1;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_parenthesized_import_binding_is_updated() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { foo } from './dep'; ((foo))++; export const value = foo;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("((foo))++")
        || output.contains("((foo)) ++")
        || output.contains("(foo)++")
        || output.contains("(foo) ++"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_import_binding_is_assigned() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { foo } from './dep'; foo = 1; export const value = 1;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_parenthesized_import_binding_is_assigned() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { foo } from './dep'; ((foo)) = 1; export const value = foo;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("((foo)) = 1")
        || output.contains("((foo))=1")
        || output.contains("(foo) = 1")
        || output.contains("(foo)=1"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_import_binding_is_destructured_assigned() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import { foo } from './dep'; ({ foo } = { foo: 1 }); export const value = 1;".to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_import_binding_is_for_of_target() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import { foo } from './dep'; for (foo of [1, 2]) {} export const value = 1;".to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_when_import_binding_is_for_in_target() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import { foo } from './dep'; for (foo in { a: 1 }) {} export const value = 1;".to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_keep_non_mutated_import_binding_live_when_another_binding_is_mutated() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import { foo, bar } from './dep'; foo++; export const value = bar;".to_string()),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_imported__0[\"bar\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_keep_non_mutated_import_binding_live_when_parenthesized_binding_is_mutated() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import { foo, bar } from './dep'; ((foo))++; export const value = bar;".to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("const foo = __farm_ssr_imported__0[\"foo\"]")
        || output.contains("const foo=__farm_ssr_imported__0[\"foo\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_imported__0[\"bar\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_dynamic_import_with_options() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "const mod = await import('./dep', { with: { type: 'json' } }); export default mod;"
          .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    assert!(can_transform_to_runner(&ast));
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();

    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("__farm_ssr_dynamic_import__"), "{output}");
    assert!(!output.contains("import("), "{output}");
    assert!(
      output.contains("__farm_ssr_dynamic_import__(\"./dep\",")
        || output.contains("__farm_ssr_dynamic_import__('./dep',"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
  }

  #[test]
  fn should_ignore_type_only_reexport_from() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("export { type Foo } from './dep'; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_ignore_export_all_type_only() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("export type * from './dep'; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(!output.contains("__farm_ssr_export_all__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_keep_runtime_binding_for_mixed_type_and_value_reexport_from() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("export { type Foo, bar as baz } from './dep';".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"baz\""),
      "{output}"
    );
    assert!(!output.contains("Foo"), "{output}");
  }

  #[test]
  fn should_keep_runtime_binding_for_mixed_type_and_value_local_export() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import type { Foo } from './dep'; const bar = 1; export { type Foo, bar as baz };"
          .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"baz\""),
      "{output}"
    );
    assert!(
      !output.contains("__farm_ssr_export_name__(\"Foo\""),
      "{output}"
    );
  }

  #[test]
  fn should_not_transform_module_with_ts_decl_syntax() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("type Foo = number; export const value = 1;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    assert!(!can_transform_to_runner(&parsed.ast));
  }

  #[test]
  fn should_transform_ts_export_assignment_as_default_export() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value = 1; export = value;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("const __farm_ssr_default_export__ = value")
        || output.contains("const __farm_ssr_default_export__=value"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_ts_import_equals_external_module_ref() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import foo = require('./dep'); export const value = foo;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("const foo = await __farm_ssr_import__(\"./dep\")")
        || output.contains("const foo=await __farm_ssr_import__(\"./dep\")"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_exported_ts_import_equals_external_module_ref() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("export import foo = require('./dep');".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("const foo = await __farm_ssr_import__(\"./dep\")")
        || output.contains("const foo=await __farm_ssr_import__(\"./dep\")"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"foo\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_ts_import_equals_entity_name_module_ref() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import foo = Bar.Baz; export const value = foo;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = Bar[\"Baz\"]") || output.contains("const foo=Bar[\"Baz\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_ts_import_equals_nested_entity_name_module_ref() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("import foo = A.B.C; export default foo;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("const foo = A[\"B\"][\"C\"]")
        || output.contains("const foo=A[\"B\"][\"C\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
  }

  #[test]
  fn should_transform_declare_enum_and_module_decls() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
declare enum Color {
  Red
}
declare namespace Foo {
  const value: number;
}
export const value = 1;
"#
        .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains("declare enum"), "{output}");
    assert!(!output.contains("declare namespace"), "{output}");
  }

  #[test]
  fn should_bailout_non_declare_ts_enum_decl() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("enum Color { Red } export const value = Color.Red;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    let transformed = transform_module_ast_to_runner(&mut ast, true);
    assert!(matches!(
      transformed,
      Err(RunnerTransformBailoutReason::UnsupportedTs)
    ));
  }

  #[test]
  fn should_bailout_non_declare_ts_module_decl() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("namespace Foo { export const a = 1; } export const value = Foo.a;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    let transformed = transform_module_ast_to_runner(&mut ast, true);
    assert!(matches!(
      transformed,
      Err(RunnerTransformBailoutReason::UnsupportedTs)
    ));
  }

  #[test]
  fn should_transform_declare_interface_and_type_alias_decls() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
declare interface Foo {
  value: number;
}
declare type Bar = string;
export const value = 1;
"#
        .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains("declare interface"), "{output}");
    assert!(!output.contains("declare type"), "{output}");
  }

  #[test]
  fn should_preserve_ordering_with_ts_entity_import_equals_and_reexport() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import dep = A.B;
export { foo as reFoo } from './dep2';
export const value = dep.foo;
"#
        .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let dep_assign_idx = output
      .find("const dep = A[\"B\"]")
      .or_else(|| output.find("const dep=A[\"B\"]"))
      .expect("dep assign");
    let dep2_import_idx = output.find("\"./dep2\"").expect("dep2 import");
    let reexport_idx = output
      .find("__farm_ssr_export_name__(\"reFoo\"")
      .expect("reexport call");
    let value_export_idx = output
      .find("__farm_ssr_export_name__(\"value\"")
      .expect("value export call");

    assert!(dep_assign_idx < dep2_import_idx, "{output}");
    assert!(dep2_import_idx < reexport_idx, "{output}");
    assert!(reexport_idx < value_export_idx, "{output}");
  }

  #[test]
  fn should_transform_import_attributes() {
    let module_id = ModuleId::from("entry.js");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "import data from './foo.json' with { type: 'json' }; export default data;".to_string(),
      ),
      Syntax::Es(EsSyntax {
        import_attributes: true,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("__farm_ssr_import__"), "{output}");
    assert!(
      output.contains("__farm_ssr_import__(\"./foo.json\",")
        || output.contains("__farm_ssr_import__('./foo.json',"),
      "{output}"
    );
    assert!(output.contains("type"), "{output}");
    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
    assert!(!output.contains("import "), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_rewrite_destructure_default_initializer_with_import_binding() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { fallback } from './dep';
const source = {};
const { value = fallback } = source;
export { value };
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("value = __farm_ssr_imported__0[\"fallback\"]")
        || output.contains("value=__farm_ssr_imported__0[\"fallback\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
  }

  #[test]
  fn should_rewrite_class_computed_key_with_import_binding() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { key } from './dep';
class Box {
  [key]() {
    return 1;
  }
}
export default Box;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("[__farm_ssr_imported__0[\"key\"]]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
  }

  #[test]
  fn should_preserve_import_export_ordering_for_reexport() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { before } from './before';
export { value as reexported } from './dep';
export const tail = before;
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let before_import_idx = output.find("\"./before\"").expect("before import");
    let dep_import_idx = output.find("\"./dep\"").expect("dep import");
    let reexport_idx = output
      .find("__farm_ssr_export_name__(\"reexported\"")
      .expect("reexport call");
    let tail_export_idx = output
      .find("__farm_ssr_export_name__(\"tail\"")
      .expect("tail export call");

    assert!(before_import_idx < dep_import_idx, "{output}");
    assert!(dep_import_idx < reexport_idx, "{output}");
    assert!(reexport_idx < tail_export_idx, "{output}");
  }

  #[test]
  fn should_not_rewrite_shadowed_binding_in_class_computed_key() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { key } from './dep';
function create(key) {
  class Box {
    [key]() {
      return key;
    }
  }
  return Box;
}
export const box = create('local');
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("function create(key)"), "{output}");
    assert!(output.contains("[key]"), "{output}");
    assert!(
      !output.contains("__farm_ssr_imported__0[\"key\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"box\""),
      "{output}"
    );
  }

  #[test]
  fn should_preserve_ordering_with_import_attributes_and_reexport() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import base from './base.json' with { type: 'json' };
export { dep as alias } from './dep.json' with { type: 'json' };
const local = base;
export { local };
"#
        .to_string(),
      ),
      Syntax::Es(EsSyntax {
        import_attributes: true,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let base_import_idx = output.find("\"./base.json\"").expect("base import");
    let dep_import_idx = output.find("\"./dep.json\"").expect("dep import");
    let alias_export_idx = output
      .find("__farm_ssr_export_name__(\"alias\"")
      .expect("alias export");
    let local_export_idx = output
      .find("__farm_ssr_export_name__(\"local\"")
      .expect("local export");

    assert!(base_import_idx < dep_import_idx, "{output}");
    assert!(dep_import_idx < alias_export_idx, "{output}");
    assert!(alias_export_idx < local_export_idx, "{output}");
    assert!(
      output.contains("__farm_ssr_import__(\"./base.json\","),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_import__(\"./dep.json\","),
      "{output}"
    );
    assert!(output.contains("type"), "{output}");
  }

  #[test]
  fn should_rewrite_nested_destructure_default_with_scope_shadow() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import { fallback } from './dep';
const source = { nested: {} };
const fallbackValue = 'local';
const {
  nested: { value = fallback }
} = source;
export { value, fallbackValue };
"#
        .to_string(),
      ),
      Syntax::Es(Default::default()),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, false).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("value = __farm_ssr_imported__0[\"fallback\"]")
        || output.contains("value=__farm_ssr_imported__0[\"fallback\"]"),
      "{output}"
    );
    assert!(
      output.contains("__farm_ssr_export_name__(\"fallbackValue\""),
      "{output}"
    );
  }

  #[test]
  fn should_preserve_ordering_with_ts_import_equals_and_reexport() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        r#"
import dep = require('./dep');
export { foo as reFoo } from './dep2';
export const value = dep.foo;
"#
        .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    let dep_import_idx = output.find("\"./dep\"").expect("dep import");
    let dep2_import_idx = output.find("\"./dep2\"").expect("dep2 import");
    let reexport_idx = output
      .find("__farm_ssr_export_name__(\"reFoo\"")
      .expect("reexport call");
    let value_export_idx = output
      .find("__farm_ssr_export_name__(\"value\"")
      .expect("value export call");

    assert!(dep_import_idx < dep2_import_idx, "{output}");
    assert!(dep2_import_idx < reexport_idx, "{output}");
    assert!(reexport_idx < value_export_idx, "{output}");
  }

  #[test]
  fn should_skip_transform_for_ts_type_annotation_syntax() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value: number = 1; export { value };".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    assert!(!can_transform_to_runner(&parsed.ast));
    let mut ast = parsed.ast;
    let transformed = transform_module_ast_to_runner(&mut ast, false);
    assert!(matches!(
      transformed,
      Err(RunnerTransformBailoutReason::UnsupportedTs)
    ));
  }

  #[test]
  fn should_skip_transform_for_ts_as_expression_syntax() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value = 1 as number; export { value };".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    assert!(!can_transform_to_runner(&parsed.ast));
  }

  #[test]
  fn should_transform_ts_type_annotation_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value: number = 1; export { value };".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains(": number"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_ts_as_expression_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value = 1 as number; export default value;".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
    assert!(!output.contains(" as number"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_ts_satisfies_expression_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value = { a: 1 } satisfies { a: number }; export { value };".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains("satisfies"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_ts_non_null_expression_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "const maybe: string | undefined = 'ok'; const value = maybe!; export default value;"
          .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
    assert!(!output.contains("maybe!"), "{output}");
    assert!(!output.contains(": string | undefined"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_ts_const_assertion_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new("const value = { a: 1 } as const; export { value };".to_string()),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"value\""),
      "{output}"
    );
    assert!(!output.contains("as const"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }

  #[test]
  fn should_transform_ts_instantiation_expression_after_strip() {
    let module_id = ModuleId::from("entry.ts");
    let parsed = parse_module(
      &module_id,
      Arc::new(
        "function id<T>(v: T) { return v; } const value = id<string>('ok'); export default value;"
          .to_string(),
      ),
      Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }),
      Default::default(),
    )
    .unwrap();

    let mut ast = parsed.ast;
    transform_module_ast_to_runner(&mut ast, true).unwrap();
    ast.visit_mut_with(&mut RunnerExprRewriter);

    let output = codegen_module(
      &ast,
      parsed.source_map,
      None,
      farmfe_toolkit::swc_ecma_codegen::Config::default(),
      None,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
      output.contains("__farm_ssr_export_name__(\"default\""),
      "{output}"
    );
    assert!(!output.contains("<string>"), "{output}");
    assert!(!output.contains("function id<T>"), "{output}");
    assert!(!output.contains("export "), "{output}");
  }
}
