use std::sync::Arc;

use farmfe_core::HashSet;
use farmfe_core::config::external::ExternalConfig;
use farmfe_core::config::{FARM_REQUIRE, ModuleFormat};
use farmfe_core::context::CompilationContext;
use farmfe_core::resource::resource_pot::ResourcePotId;
use farmfe_core::swc_ecma_ast::EsVersion;
use farmfe_core::swc_ecma_ast::Module as SwcModule;
use farmfe_core::swc_ecma_ast::{CallExpr, Callee, Expr, MemberProp};
use farmfe_core::swc_ecma_parser::Syntax;
use farmfe_toolkit::html::get_farm_global_this;
use farmfe_toolkit::script::parse_module;
use farmfe_toolkit::script::swc_try_with::ResetSpanVisitMut;
use farmfe_toolkit::swc_ecma_utils::StmtLikeInjector;
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitMutWith, VisitWith};

#[derive(Default)]
struct NativeRequireDetector {
  used: bool,
}

impl Visit for NativeRequireDetector {
  fn visit_call_expr(&mut self, call: &CallExpr) {
    if let Callee::Expr(expr) = &call.callee {
      if let Expr::Member(member) = expr.as_ref() {
        if matches!(&member.prop, MemberProp::Ident(prop) if prop.sym == "nr")
          && matches!(member.obj.as_ref(), Expr::Ident(obj) if obj.sym.as_str() == FARM_REQUIRE)
        {
          self.used = true;
        }
      }
    }
    call.visit_children_with(self);
  }
}

pub fn handle_external_modules(
  resource_pot_id: &ResourcePotId,
  resource_pot_ast: SwcModule,
  external_modules: &HashSet<String>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<SwcModule> {
  let mut native_require = NativeRequireDetector::default();
  if context.config.output.target_env.is_node() {
    resource_pot_ast.visit_with(&mut native_require);
  }
  if external_modules.is_empty() && !native_require.used {
    return Ok(resource_pot_ast);
  }

  let mut external_modules = external_modules.iter().collect::<Vec<_>>();
  external_modules.sort();

  // library won't be handled here, they will be handled in plugin_library
  if context.config.output.target_env.is_node() {
    handle_node_external_modules(
      resource_pot_id,
      resource_pot_ast,
      external_modules,
      native_require.used,
      context,
    )
  } else {
    handle_browser_external_modules(resource_pot_id, resource_pot_ast, external_modules, context)
  }
}

/// For node external modules, we need to import/require the external modules at the top of the resource pot. e.g
/// ```js
/// import * as fs from 'fs';
/// import * as path from 'path';
/// global['__farm_default_namespace__'].m.se({"fs": fs, "path": path});
/// ```
fn handle_node_external_modules(
  resource_pot_id: &ResourcePotId,
  mut resource_pot_ast: SwcModule,
  external_modules: Vec<&String>,
  native_require: bool,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<SwcModule> {
  let farm_global_this = get_farm_global_this(
    &context.config.runtime.namespace,
    &context.config.output.target_env,
  );

  let mut import_strings = vec![];
  let mut source_to_names = vec![];

  for external_module in external_modules {
    // replace all invalid characters with `_`
    let mut name = external_module
      .chars()
      .map(|c| if c.is_alphanumeric() { c } else { '_' })
      .collect::<String>();
    name = format!("__farm_external_module_{name}");

    let import_str = if context.config.output.format.as_single() == ModuleFormat::EsModule {
      format!("import * as {name} from {external_module:?};")
    } else {
      format!("var {name} = require({external_module:?});")
    };
    import_strings.push(import_str);
    source_to_names.push((name, external_module));
  }

  let mut prepend_str = import_strings.join("\n");
  // Keep literal CommonJS external requires separate from ESM namespaces.
  if native_require && context.config.output.format.as_single() == ModuleFormat::EsModule {
    prepend_str.push_str(&format!(
      "\nimport {{ createRequire as __farm_create_require__ }} from 'node:module';\n{farm_global_this}.m.r.nr = __farm_create_require__(import.meta.url);"
    ));
  } else if native_require {
    prepend_str.push_str(&format!("\n{farm_global_this}.m.r.nr = require;"));
  }
  if !source_to_names.is_empty() {
    prepend_str.push_str(&format!(
      "{farm_global_this}.m.se({{{}}});",
      source_to_names
        .into_iter()
        .map(|(name, source)| format!("{source:?}: {name}"))
        .collect::<Vec<_>>()
        .join(",")
    ));
  }

  let mut prepend_ast = parse_module(
    &resource_pot_id.as_str().into(),
    Arc::new(prepend_str),
    Syntax::Es(Default::default()),
    EsVersion::Es5,
    // None,
  )?;
  prepend_ast.ast.visit_mut_with(&mut ResetSpanVisitMut);

  resource_pot_ast.body.prepend_stmts(prepend_ast.ast.body);

  Ok(resource_pot_ast)
}

/// For browser external modules, we need to read the external modules from the window object. e.g
/// ```js
/// global['__farm_default_namespace__'].m.se({"fs": window['fs'], "path": window['path']});
/// ```
fn handle_browser_external_modules(
  resource_pot_id: &ResourcePotId,
  mut resource_pot_ast: SwcModule,
  external_modules: Vec<&String>,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<SwcModule> {
  let farm_global_this = get_farm_global_this(
    &context.config.runtime.namespace,
    &context.config.output.target_env,
  );
  let external_config = ExternalConfig::from(&*context.config);
  let mut external_objs = Vec::new();

  for source in external_modules {
    let replace_source = external_config
      .find_match(&source.to_string())
      .map(|v| v.source(&source.to_string()))
      // it's maybe from plugin
      .unwrap_or(source.to_string());

    let source_obj = format!("window['{replace_source}']||{{}}");
    external_objs.push(format!("{source:?}: {source_obj}"));
  }

  let prepend_str = format!("{farm_global_this}.m.se({{{}}});", external_objs.join(","));

  let mut prepend_ast = parse_module(
    &resource_pot_id.as_str().into(),
    Arc::new(prepend_str),
    Syntax::Es(Default::default()),
    EsVersion::Es5,
    // None,
  )?;
  prepend_ast.ast.visit_mut_with(&mut ResetSpanVisitMut);

  resource_pot_ast.body.prepend_stmts(prepend_ast.ast.body);

  Ok(resource_pot_ast)
}
