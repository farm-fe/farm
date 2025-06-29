use std::cell::RefCell;

use farmfe_core::{
  module::meta_data::script::ScriptModuleMetaData, swc_common::Mark, swc_ecma_ast::Expr, HashSet,
};
use farmfe_toolkit::script::{
  create_top_level_ident,
  module2cjs::{self, RuntimeCalleeAllocator, TransformModuleDeclsOptions},
};

pub fn transform_hybrid_to_cjs(meta: &mut ScriptModuleMetaData) -> HashSet<&'static str> {
  let unresolved_mark = Mark::from_u32(meta.unresolved_mark);
  let top_level_mark = Mark::from_u32(meta.top_level_mark);

  let callee_allocator = Hybrid2CjsCalleeAllocator::new(top_level_mark);

  module2cjs::transform_module_decls(
    &mut meta.ast,
    unresolved_mark,
    &callee_allocator,
    TransformModuleDeclsOptions {
      is_target_legacy: false,
    },
  );

  let mut used_helper_idents = callee_allocator.used_helper_idents.borrow_mut();
  std::mem::take(&mut used_helper_idents)
}

pub struct Hybrid2CjsCalleeAllocator {
  top_level_mark: Mark,
  pub used_helper_idents: RefCell<HashSet<&'static str>>,
}

impl Hybrid2CjsCalleeAllocator {
  pub fn new(top_level_mark: Mark) -> Self {
    Self {
      top_level_mark,
      used_helper_idents: RefCell::new(HashSet::default()),
    }
  }
}

impl Hybrid2CjsCalleeAllocator {
  fn create_expr_ident(&self, ident: &'static str) -> Box<Expr> {
    self.used_helper_idents.borrow_mut().insert(ident);

    Box::new(Expr::Ident(create_top_level_ident(
      ident,
      self.top_level_mark,
    )))
  }
}

impl RuntimeCalleeAllocator for Hybrid2CjsCalleeAllocator {
  fn define_property_callee(&self) -> Box<Expr> {
    self.create_expr_ident("exportByDefineProperty")
  }

  fn es_module_flag_callee(&self) -> Box<Expr> {
    self.create_expr_ident("defineExportEsModule")
  }

  fn export_star_callee(&self) -> Box<Expr> {
    self.create_expr_ident("defineExportStar")
  }

  fn esm_export_named_callee(&self) -> Box<Expr> {
    self.create_expr_ident("defineExportFrom")
  }

  fn import_namespace_callee(&self) -> Box<Expr> {
    self.create_expr_ident("interopRequireWildcard")
  }

  fn cjs_export_named_callee(&self) -> Box<Expr> {
    self.create_expr_ident("defineExport")
  }

  fn interop_default(&self) -> Box<Expr> {
    self.create_expr_ident("importDefault")
  }

  fn import_default_callee(&self) -> Box<Expr> {
    self.create_expr_ident("interopRequireDefault")
  }
}
