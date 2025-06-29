use std::{hash::Hash, sync::Arc};

use farmfe_macro_cache_item::cache_item;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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
  ExternalReExportAll,

  /// ```js
  /// export * from './module'; // where module is a external module
  /// ```
  ExternalAll,

  /// ```js
  /// import { foo as bar } from './foo.cjs';
  /// export { foo as default } from './foo.cjs'; // foo is not external but using cjs  
  /// ```
  Unresolved,

  /// namespace import/export placeholder
  /// ```js
  /// import * as m from './foo';
  /// export * as m from './foo';
  /// ```
  VirtualNamespace,
}

#[cache_item]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleExportIdentInternal {
  pub module_id: ModuleId,
  pub ident: SwcId,
  pub export_type: ModuleExportIdentType,
}

#[derive(Debug, Clone)]
pub struct ModuleExportIdent {
  shared: Arc<RwLock<ModuleExportIdentInternal>>,
}

impl ModuleExportIdent {
  pub fn new(module_id: ModuleId, ident: SwcId, export_type: ModuleExportIdentType) -> Self {
    Self {
      shared: Arc::new(RwLock::new(ModuleExportIdentInternal {
        module_id,
        ident,
        export_type,
      })),
    }
  }

  pub fn as_internal(&self) -> RwLockReadGuard<'_, ModuleExportIdentInternal> {
    self.shared.read()
  }

  pub fn as_internal_mut(&self) -> RwLockWriteGuard<'_, ModuleExportIdentInternal> {
    self.shared.write()
  }
}

impl Hash for ModuleExportIdent {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.shared.read().hash(state);
  }
}

impl PartialEq for ModuleExportIdent {
  fn eq(&self, other: &Self) -> bool {
    self.shared.read().eq(&other.shared.read())
  }
}

impl Eq for ModuleExportIdent {}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleReExportIdentType {
  /// ```js
  /// export * from './module';
  /// ```
  FromExportAll,
  /// ```js
  /// export { foo as default } from './module'; // foo is local ident
  /// ```
  FromExportNamed { local: String },
}

impl<__D: Fallible + ?Sized> Deserialize<ModuleExportIdent, __D> for Archived<ModuleExportIdent> {
  #[inline]
  fn deserialize(
    &self,
    deserializer: &mut __D,
  ) -> ::core::result::Result<ModuleExportIdent, __D::Error> {
    let internal =
      Deserialize::<ModuleExportIdentInternal, __D>::deserialize(&self.internal, deserializer)?;
    let res = ModuleExportIdent {
      shared: Arc::new(RwLock::new(internal)),
    };

    Ok(res)
  }
}

impl<__S: Fallible + ?Sized> Serialize<__S> for ModuleExportIdent
where
  __S: rkyv::ser::Serializer + rkyv::ser::ScratchSpace,
{
  #[inline]
  fn serialize(&self, serializer: &mut __S) -> ::core::result::Result<Self::Resolver, __S::Error> {
    let internal = self.shared.read();
    let resolver_internal = Serialize::<__S>::serialize(&*internal, serializer)?;

    Ok(ModuleExportIdentResolver {
      internal: resolver_internal,
    })
  }
}

pub struct ArchivedModuleExportIdent {
  pub internal: ::rkyv::Archived<ModuleExportIdentInternal>,
}

pub struct ModuleExportIdentResolver {
  internal: ::rkyv::Resolver<ModuleExportIdentInternal>,
}

impl Archive for ModuleExportIdent {
  type Archived = ArchivedModuleExportIdent;
  type Resolver = ModuleExportIdentResolver;
  #[allow(clippy::unit_arg)]
  #[inline]
  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    let (fp, fo) = {
      #[allow(unused_unsafe)]
      unsafe {
        let fo = &raw mut (*out).internal;
        (fo.cast::<u8>().offset_from(out.cast::<u8>()) as usize, fo)
      }
    };

    let internal = self.shared.read();

    ::rkyv::Archive::resolve(&*internal, pos + fp, resolver.internal, fo);
  }
}
