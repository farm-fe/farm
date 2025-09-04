use std::{hash::Hash, sync::Arc};

use farmfe_macro_cache_item::cache_item;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use swc_atoms::Atom;

use crate::module::{meta_data::script::statement::SwcId, ModuleId};

/// How the module export ident is defined.
/// Where resolving this type, [Declaration] will be resolved first when expand exports.
/// Other 3 types will be resolved when expand imports.
#[cache_item]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ModuleExportIdentType {
  /// ```js
  /// function m() {}
  /// export { m as bar }; // or
  /// export const bar = m;
  /// ```
  Declaration,

  /// ```js
  /// import { foo as bar } from 'module';
  /// export { foo as default } from 'module'; // where module is a external module and we don't know how foo is defined
  /// ```
  External,

  /// ```js
  /// // deep.js
  /// export * from 'module'; // where module is a external module
  ///
  /// // index.js
  /// import { bar } from'./deep'; // bar is a ExternalReExport ident
  /// ```
  ExternalExportAll,

  /// ```js
  /// export * from './module'; // where module is not a es module
  /// ```
  AmbiguousExportAll,

  /// ```js
  /// import { foo as bar } from './foo.cjs';
  /// export { foo as default } from './foo.cjs'; // foo is not external but using cjs  
  /// ```
  Unresolved,

  /// ```js
  /// // deep.js
  /// export * from './foo.cjs';
  ///
  /// // index.js
  /// import { foo } from './deep';
  /// ```
  UnresolvedExportAll,

  /// namespace import/export placeholder
  /// ```js
  /// import * as m from './foo';
  /// export * as m from './foo';
  /// ```
  VirtualNamespace,
}

#[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
#[rkyv(remote = SharedModuleExportIdent)]
#[rkyv(archived = ArchivedSharedModuleExportIdent)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InternalModuleExportIdent {
  #[rkyv(getter = get_module_id)]
  pub module_id: ModuleId,
  #[rkyv(getter = get_ident)]
  pub ident: SwcId,
  #[rkyv(getter = get_export_type)]
  pub export_type: ModuleExportIdentType,
}

fn get_module_id(value: &SharedModuleExportIdent) -> ModuleId {
  value.internal.read().module_id.clone()
}

fn get_ident(value: &SharedModuleExportIdent) -> SwcId {
  value.internal.read().ident.clone()
}

fn get_export_type(value: &SharedModuleExportIdent) -> ModuleExportIdentType {
  value.internal.read().export_type.clone()
}

#[derive(Debug, Clone)]
struct SharedModuleExportIdent {
  internal: Arc<RwLock<InternalModuleExportIdent>>,
}

impl From<InternalModuleExportIdent> for SharedModuleExportIdent {
  fn from(value: InternalModuleExportIdent) -> Self {
    Self {
      internal: Arc::new(RwLock::new(value)),
    }
  }
}

#[cache_item]
#[derive(Debug, Clone)]
pub struct ModuleExportIdent {
  #[rkyv(with = InternalModuleExportIdent)]
  shared: SharedModuleExportIdent,
}

impl ModuleExportIdent {
  pub fn new(module_id: ModuleId, ident: SwcId, export_type: ModuleExportIdentType) -> Self {
    Self {
      shared: SharedModuleExportIdent::from(InternalModuleExportIdent {
        module_id,
        ident,
        export_type,
      }),
    }
  }

  pub fn as_internal(&self) -> RwLockReadGuard<'_, InternalModuleExportIdent> {
    self.shared.internal.read()
  }

  pub fn as_internal_mut(&self) -> RwLockWriteGuard<'_, InternalModuleExportIdent> {
    self.shared.internal.write()
  }
}

impl Hash for ModuleExportIdent {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.shared.internal.read().hash(state);
  }
}

impl PartialEq for ModuleExportIdent {
  fn eq(&self, other: &Self) -> bool {
    self
      .shared
      .internal
      .read()
      .eq(&other.shared.internal.read())
  }
}

impl Eq for ModuleExportIdent {}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleReExportIdentType {
  /// ```js
  /// export * from './module';
  /// ```
  FromExportAll(ModuleId),
  /// ```js
  /// export { foo as default } from './module'; // foo is local ident
  /// ```
  FromExportNamed {
    local: Atom,
    from_module_id: ModuleId,
  },
}
