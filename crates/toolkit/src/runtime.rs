use std::sync::Arc;

use farmfe_core::{
  config::{Mode, TargetEnv},
  context::CompilationContext,
  module::meta_data::script::feature_flag::{
    FeatureFlag, FARM_ENABLE_EXPORT_ALL_HELPER, FARM_ENABLE_EXPORT_HELPER,
    FARM_ENABLE_IMPORT_ALL_HELPER, FARM_ENABLE_IMPORT_DEFAULT_HELPER, FARM_ENABLE_TOP_LEVEL_AWAIT,
    FARM_IMPORT_EXPORT_FROM_HELPER,
  },
  swc_common::{util::take::Take, DUMMY_SP},
  swc_ecma_ast::{BinaryOp, EmptyStmt, Expr, Ident, Lit, Stmt, Str},
  HashMap, HashSet,
};
use lazy_static::lazy_static;
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub const FARM_RUNTIME_TARGET_ENV: &str = "__FARM_RUNTIME_TARGET_ENV__";
pub const FARM_ENABLE_RUNTIME_PLUGIN: &str = "__FARM_ENABLE_RUNTIME_PLUGIN__";
pub const FARM_ENABLE_EXTERNAL_MODULES: &str = "__FARM_ENABLE_EXTERNAL_MODULES__"; // always true if target env is not library

// Init full runtime features map
lazy_static! {
  pub static ref FULL_RUNTIME_FEATURES: HashSet<&'static str> = {
    let mut features = HashSet::default();
    features.insert(FARM_RUNTIME_TARGET_ENV);
    features.insert(FARM_ENABLE_RUNTIME_PLUGIN);
    features.insert(FARM_ENABLE_EXTERNAL_MODULES);
    features.insert(FARM_ENABLE_TOP_LEVEL_AWAIT);
    features.insert(FARM_ENABLE_EXPORT_HELPER);
    features.insert(FARM_ENABLE_EXPORT_ALL_HELPER);
    features.insert(FARM_ENABLE_IMPORT_ALL_HELPER);
    features.insert(FARM_IMPORT_EXPORT_FROM_HELPER);
    features.insert(FARM_ENABLE_IMPORT_DEFAULT_HELPER);

    features
  };
}

pub struct RuntimeFeatureGuardRemover<'a> {
  /// __FARM_ENABLE_RUNTIME_PLUGIN__
  bool_features: HashSet<&'a str>,
  /// __FARM_RUNTIME_TARGET_ENV__
  string_features: HashMap<&'a str, String>,
}

impl<'a> RuntimeFeatureGuardRemover<'a> {
  pub fn new(feature_flags: &'a HashSet<FeatureFlag>, context: &Arc<CompilationContext>) -> Self {
    let bool_features = init_bool_features(feature_flags, context);
    let string_features = init_string_features(context);

    Self {
      bool_features,
      string_features,
    }
  }
}

impl VisitMut for RuntimeFeatureGuardRemover<'_> {
  fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
    // remove children first
    stmt.visit_mut_children_with(self);

    if let Stmt::If(if_stmt) = stmt {
      match &*if_stmt.test {
        Expr::Ident(Ident { sym, .. }) => {
          if !FULL_RUNTIME_FEATURES.contains(sym.as_str()) {
            return;
          }

          // 1. remove if (__FARM_ENABLE_RUNTIME_PLUGIN__) { ... }
          if self.bool_features.contains(sym.as_str()) {
            // if (xxx) { 123 } => { 123 }
            let cons = if_stmt.cons.take();
            *stmt = *cons;
          } else {
            // remove if branch with else branch or empty statement
            // if (xxx) { 123 } else { 456 } => { 456 }
            let alt = if_stmt.alt.take().map(|alt| *alt);
            *stmt = alt.unwrap_or(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
          }
        }
        Expr::Bin(bin) => {
          // 2. remove if (__FARM_RUNTIME_TARGET_ENV__ === 'browser') { ... }
          if let (Expr::Ident(Ident { sym, .. }), Expr::Lit(Lit::Str(Str { value, .. }))) =
            (&*bin.left, &*bin.right)
          {
            if !FULL_RUNTIME_FEATURES.contains(sym.as_str()) {
              return;
            }

            if bin.op == BinaryOp::EqEqEq && self.string_features.contains_key(sym.as_str()) {
              let expect_value = self.string_features.get(sym.as_str());
              // if (xxx) { 123 } => { 123 }
              if expect_value.is_some() && value.as_str() == expect_value.unwrap() {
                let cons = if_stmt.cons.take();
                *stmt = *cons;
              } else {
                // remove if branch with else branch or empty statement
                // if (xxx) { 123 } else { 456 } => { 456 }
                let alt = if_stmt.alt.take().map(|alt| *alt);
                *stmt = alt.unwrap_or(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
              }
            }
          }
        }
        _ => {}
      }
    }
  }
}

fn init_bool_features<'a>(
  feature_flags: &'a HashSet<FeatureFlag>,
  context: &Arc<CompilationContext>,
) -> HashSet<&'a str> {
  // enable all features in development mode
  if matches!(context.config.mode, Mode::Development) {
    return FULL_RUNTIME_FEATURES.clone();
  }

  let mut bool_features = HashSet::default();

  if context.config.runtime.plugins.len() > 0 {
    bool_features.insert(FARM_ENABLE_RUNTIME_PLUGIN);
  }

  if context.config.output.target_env != TargetEnv::Library {
    bool_features.insert(FARM_ENABLE_EXTERNAL_MODULES);
  }

  let feature_names = feature_flags
    .iter()
    .map(|flag| flag.as_str())
    .collect::<HashSet<_>>();

  for feature_name in feature_names {
    bool_features.insert(feature_name);
  }

  bool_features
}

fn init_string_features(context: &Arc<CompilationContext>) -> HashMap<&'static str, String> {
  let mut string_features = HashMap::default();
  string_features.insert(
    FARM_RUNTIME_TARGET_ENV,
    context.config.output.target_env.to_string(),
  );

  string_features
}
