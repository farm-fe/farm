use std::sync::Arc;

use farmfe_core::{
  config::external::ExternalConfig,
  context::CompilationContext,
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
  swc_common::{Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, BlockStmt, Expr, FnExpr, Function, Ident, Lit, Module, ModuleItem, Param, Pat,
    Stmt,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::{
  script::{create_top_level_ident, is_commonjs_require, parse_stmt, swc_try_with::try_with},
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use crate::{
  formats::{cjs::transform_resource_pot_to_cjs, GenerateLibraryFormatResourcesOptions},
  utils::{add_format_to_generated_resources, emit_resource_pot},
};

pub fn emit_umd_resources(
  resource_pot: &mut ResourcePot,
  runtime_module_helper_ast: &Module,
  all_used_helper_idents: &HashSet<String>,
  options: &GenerateLibraryFormatResourcesOptions,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  // 1. transform resource pot to cjs first
  transform_resource_pot_to_cjs(
    resource_pot,
    runtime_module_helper_ast,
    all_used_helper_idents,
    options,
    context,
  );

  let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
  let globals = context.meta.get_resource_pot_globals(&resource_pot.id);

  try_with(cm, globals.value(), || {
    // 2. find all require('xxx') expr and replace it with identifier
    let meta = resource_pot.meta.as_js_mut();
    let external_identifier_map = transform_require_expr_to_identifier(
      &mut meta.ast,
      Mark::from_u32(meta.unresolved_mark),
      Mark::from_u32(meta.top_level_mark),
      context,
    );
    // make external identifiers ordered by name
    let mut external_identifiers = external_identifier_map
      .into_iter()
      .map(|(k, (v, mark))| (k, (v, mark)))
      .collect::<Vec<_>>();
    external_identifiers.sort_by(|a, b| a.0.cmp(&b.0));

    let ast = std::mem::take(&mut meta.ast);

    // 3. generate a umd function expr
    let umd_function_expr = create_umd_function_expr(
      &external_identifiers,
      ast,
      Mark::from_u32(meta.unresolved_mark),
    );

    // 4. inject umd function expr to wrapper stmt
    let umd_stmt = create_umd_wrapper_stmt(
      umd_function_expr,
      &external_identifiers,
      &context.config.output.name,
    );
    meta.ast = Module {
      span: DUMMY_SP,
      body: vec![ModuleItem::Stmt(umd_stmt)],
      shebang: None,
    };
  })
  .unwrap();

  let mut resources = emit_resource_pot(resource_pot, context, hook_context)?;
  add_format_to_generated_resources(&mut resources, "umd");

  Ok(resources)
}

/// Example:
/// ```
/// // before
/// console.log(require('node:fs'));
///
/// // after
/// console.log(node_fs);
/// ```
fn transform_require_expr_to_identifier(
  ast: &mut Module,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  context: &Arc<CompilationContext>,
) -> HashMap<String, (String, Mark)> {
  let mut transformer =
    RequireExprTransformer::new(unresolved_mark, top_level_mark, context.clone());

  ast.visit_mut_with(&mut transformer);

  transformer.external_identifiers
}

struct RequireExprTransformer {
  /// Map from module name to (normalized_module_name, (raw_module_name, mark))
  pub external_identifiers: HashMap<String, (String, Mark)>,

  unresolved_mark: Mark,
  top_level_mark: Mark,
  context: Arc<CompilationContext>,

  used_identifiers: HashMap<String, u32>,
}

impl RequireExprTransformer {
  pub fn new(
    unresolved_mark: Mark,
    top_level_mark: Mark,
    context: Arc<CompilationContext>,
  ) -> Self {
    Self {
      external_identifiers: Default::default(),
      unresolved_mark,
      top_level_mark,
      context,
      used_identifiers: Default::default(),
    }
  }
}

impl VisitMut for RequireExprTransformer {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Some(call_expr) = expr.as_mut_call() {
      if is_commonjs_require(self.unresolved_mark, self.top_level_mark, call_expr) {
        if let box Expr::Lit(Lit::Str(str)) = &call_expr.args[0].expr {
          // replace all no A-Za-z0-9_ to _
          let raw_module_name = str.value.to_string();

          let external_config = ExternalConfig::from(&*self.context.config);

          let normalized_module_name = external_config
            .find_match(&raw_module_name)
            .map(|v| v.source(&raw_module_name))
            // it's maybe from plugin
            .unwrap_or(raw_module_name.clone())
            .replace(|c: char| !c.is_alphanumeric(), "_");

          let module_name =
            if let Some(count) = self.used_identifiers.get_mut(&normalized_module_name) {
              *count += 1;
              format!("__f_umd_{}_{}", normalized_module_name, count)
            } else {
              self
                .used_identifiers
                .insert(normalized_module_name.clone(), 0);
              format!("__f_umd_{}", normalized_module_name)
            };

          let mark = Mark::new();
          self
            .external_identifiers
            .insert(module_name.clone(), (raw_module_name, mark));

          *expr = Expr::Ident(Ident::new(
            module_name.as_str().into(),
            DUMMY_SP,
            SyntaxContext::empty().apply_mark(mark),
          ));
        }
      }
    }

    expr.visit_mut_children_with(self);
  }
}

/// Create
/// ```js
/// function (exports, dep1, dep2, ...) {
///   // code
/// }
/// ```
fn create_umd_function_expr(
  external_identifiers: &Vec<(String, (String, Mark))>,
  ast: Module,
  unresolved_mark: Mark,
) -> Expr {
  let mut params = vec![Param {
    span: DUMMY_SP,
    decorators: vec![],
    pat: Pat::Ident(BindingIdent {
      id: create_top_level_ident("exports", unresolved_mark),
      type_ann: None,
    }),
  }];

  params.append(
    &mut external_identifiers
      .into_iter()
      .map(|(k, (_, mark))| Param {
        span: DUMMY_SP,
        decorators: vec![],
        pat: Pat::Ident(BindingIdent {
          id: create_top_level_ident(k, *mark),
          type_ann: None,
        }),
      })
      .collect::<Vec<_>>(),
  );

  Expr::Fn(FnExpr {
    ident: None,
    function: Box::new(Function {
      params,
      decorators: vec![],
      span: DUMMY_SP,
      ctxt: SyntaxContext::empty(),
      body: Some(BlockStmt {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        stmts: ast
          .body
          .into_iter()
          .map(|item| item.expect_stmt())
          .collect(),
      }),
      is_generator: false,
      is_async: false,
      type_params: None,
      return_type: None,
    }),
  })
}

/// Create umd wrapper stmt
/// ```js
/// (function (global, factory) {
///   // code
/// })(this, factory_expr);
/// ```
fn create_umd_wrapper_stmt(
  factory_expr: Expr,
  external_identifiers: &Vec<(String, (String, Mark))>,
  name: &str,
) -> Stmt {
  let raw_external_identifiers = external_identifiers
    .iter()
    .map(|(_, (raw, _))| raw)
    .collect::<Vec<_>>();
  let cjs_require_deps = raw_external_identifiers
    .iter()
    .map(|s| format!("require('{s}')"))
    .collect::<Vec<_>>()
    .join(", ");
  let amd_deps = raw_external_identifiers
    .iter()
    .map(|s| format!("'{s}'"))
    .collect::<Vec<_>>()
    .join(", ");
  let external_identifiers = raw_external_identifiers
    .iter()
    .map(|s| format!("global['{s}']"))
    .collect::<Vec<_>>()
    .join(", ");

  let umd_wrapper_template = format!(
    r#"(function (global, factory) {{
	typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports, {cjs_require_deps}) :
	typeof define === 'function' && define.amd ? define(['exports', {amd_deps}], factory) :
	(global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global['{name}'] = {{}}, {external_identifiers}));
}})(this, factory_expr);
"#,
  );

  // parse umd template code
  let mut wrapper_stmt =
    parse_stmt("__farm_umd_wrapper_template__", &umd_wrapper_template).unwrap();

  // replace factory_expr
  wrapper_stmt
    .as_mut_expr()
    .and_then(|expr_stmt| expr_stmt.expr.as_mut_call())
    .and_then(|call_expr| call_expr.args.get_mut(1))
    .and_then(|arg| Some(arg.expr = Box::new(factory_expr)))
    .expect("replace factory_expr failed");

  wrapper_stmt
}
