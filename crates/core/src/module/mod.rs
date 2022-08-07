use std::{any::Any, hash::Hash, path::Path};

use blake2::{
  digest::{Update, VariableOutput},
  Blake2bVar,
};
use downcast_rs::{impl_downcast, Downcast};
use farm_macro_cache_item::cache_item;
use hashbrown::HashSet;
use pathdiff::diff_paths;
use rkyv::{Archive, Archived, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;
use swc_common::Mark;
use swc_ecma_ast::Module as SwcModule;

use crate::{config::Mode, resource::resource_pot::ResourcePotId};

use self::module_group::ModuleGroupId;

pub mod module_bucket;
pub mod module_graph;
pub mod module_group;

/// A [Module] is a basic compilation unit
/// The [Module] is created by plugins in the parse hook of build stage
#[cache_item]
pub struct Module {
  /// the id of this module, generated from the resolved id.
  pub id: ModuleId,
  /// the type of this module, for example [ModuleType::Js]
  pub module_type: ModuleType,
  /// the module groups this module belongs to, used to construct [crate::module::module_group::ModuleGroupMap]
  pub module_groups: HashSet<ModuleGroupId>,
  /// the resource pot this module belongs to
  pub resource_pot: Option<ResourcePotId>,
  /// the meta data of this module custom by plugins
  pub meta: ModuleMetaData,
}

impl Module {
  pub fn new(id: ModuleId, module_type: ModuleType) -> Self {
    Self {
      id,
      module_type,
      meta: ModuleMetaData::Custom(Box::new(EmptyModuleMetaData) as _),
      module_groups: HashSet::new(),
      resource_pot: None,
    }
  }
}

/// Module meta data shared by core plugins through the compilation
/// Meta data which is not shared by core plugins should be stored in [ModuleMetaData::Custom]
#[cache_item]
pub enum ModuleMetaData {
  Script(ModuleScriptMetaData),
  Custom(Box<dyn SerializeCustomModuleMetaData>),
}

impl ModuleMetaData {
  pub fn as_script_mut(&mut self) -> &mut ModuleScriptMetaData {
    match self {
      ModuleMetaData::Script(script) => script,
      ModuleMetaData::Custom(_) => panic!("ModuleMetaData is not Script"),
    }
  }

  pub fn as_script(&self) -> &ModuleScriptMetaData {
    match self {
      ModuleMetaData::Script(script) => script,
      ModuleMetaData::Custom(_) => panic!("ModuleMetaData is not Script"),
    }
  }

  pub fn as_custom_mut<T: SerializeCustomModuleMetaData + 'static>(&mut self) -> &mut T {
    match self {
      ModuleMetaData::Script(_) => panic!("ModuleMetaData is not Script"),
      ModuleMetaData::Custom(custom) => {
        if let Some(c) = custom.downcast_mut::<T>() {
          c
        } else {
          panic!("custom meta type is not serializable");
        }
      }
    }
  }

  pub fn as_custom<T: SerializeCustomModuleMetaData + 'static>(&self) -> &T {
    match self {
      ModuleMetaData::Script(_) => panic!("ModuleMetaData is not Script"),
      ModuleMetaData::Custom(custom) => {
        if let Some(c) = custom.downcast_ref::<T>() {
          c
        } else {
          panic!("custom meta type is not serializable");
        }
      }
    }
  }
}

/// Trait that makes sure the trait object implements [rkyv::Serialize] and [rkyv::Deserialize]
#[archive_dyn(deserialize)]
pub trait CustomModuleMetaData: Any + Send + Sync + Downcast {}

impl_downcast!(SerializeCustomModuleMetaData);

/// initial empty custom data, plugins may replace this
#[cache_item(CustomModuleMetaData)]
pub struct EmptyModuleMetaData;

/// Script specific meta data, for example, [swc_ecma_ast]
#[cache_item]
pub struct ModuleScriptMetaData {
  pub ast: SwcModule,
}

/// Internal support module types by the core plugins, other
/// ModuleType will be set after the load hook, but can be change in transform hook too.
#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModuleType {
  // native supported module type by the core plugins
  Js,
  Jsx,
  Ts,
  Tsx,
  Css,
  Html,
  Asset,
  // custom module type from using by custom plugins
  Custom(String),
}

impl ModuleType {
  pub fn is_typescript(&self) -> bool {
    matches!(self, ModuleType::Ts) || matches!(self, ModuleType::Tsx)
  }
}

impl ModuleType {
  /// transform native supported file type to [ModuleType]
  pub fn from_ext(ext: &str) -> Self {
    match ext {
      "js" => Self::Js,
      "jsx" => Self::Jsx,
      "ts" => Self::Ts,
      "tsx" => Self::Tsx,
      "css" => Self::Css,
      "html" => Self::Html,
      custom => Self::Custom(custom.to_string()),
    }
  }
}

/// Abstract ModuleId from the module's resolved id
#[cache_item]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
#[archive_attr(derive(Hash, Eq, PartialEq))]
pub struct ModuleId {
  relative_path: String,
}

const LEN: usize = 4;

// get platform independent relative path
pub fn relative(from: &str, to: &str) -> String {
  let rp =
    diff_paths(to, from).unwrap_or_else(|| panic!("{} or {} is not absolute path", from, to));

  // make sure the relative path is platform independent
  // this can ensure that the relative path and hash stable across platforms
  let mut result = String::new();

  for comp in rp.components() {
    match comp {
      std::path::Component::Prefix(_)
      | std::path::Component::RootDir
      | std::path::Component::CurDir => unreachable!(),
      std::path::Component::ParentDir => {
        if result.is_empty() {
          result += "..";
        } else {
          result += "/..";
        }
      }
      std::path::Component::Normal(c) => {
        let c = c.to_string_lossy().to_string();

        if result.is_empty() {
          result += &c;
        } else {
          result = format!("{}/{}", result, c);
        }
      }
    }
  }

  result
}

impl ModuleId {
  pub fn new(resolved_id: &str, cwd: &str) -> Self {
    let rp = Path::new(resolved_id);
    let relative_path = if rp.is_absolute() {
      relative(cwd, resolved_id)
    } else {
      resolved_id.to_string()
    };

    Self { relative_path }
  }

  /// return self.relative_path in dev,
  /// return hash(self.relative_path) in prod
  pub fn id(&self, mode: Mode) -> String {
    match mode {
      Mode::Development => self.relative_path.to_string(),
      Mode::Production => self.hash(),
    }
  }

  pub fn path(&self) -> &str {
    &self.relative_path
  }

  pub fn hash(&self) -> String {
    let mut hasher = Blake2bVar::new(LEN).unwrap();
    hasher.update(self.relative_path.as_bytes());
    let mut buf = [0u8; LEN];
    hasher.finalize_variable(&mut buf).unwrap();
    hex::encode(buf)
  }
}

impl From<&str> for ModuleId {
  fn from(rp: &str) -> Self {
    Self {
      relative_path: rp.to_string(),
    }
  }
}

impl From<String> for ModuleId {
  fn from(rp: String) -> Self {
    Self { relative_path: rp }
  }
}

impl ToString for ModuleId {
  fn to_string(&self) -> String {
    self.relative_path.clone()
  }
}

#[cfg(test)]
mod tests {
  use crate::config::Mode;
  use farm_macro_cache_item::cache_item;
  use hashbrown::HashSet;
  use rkyv::{Archive, Archived, Deserialize, Serialize};
  use rkyv_dyn::archive_dyn;
  use rkyv_typename::TypeName;

  use super::{
    CustomModuleMetaData, DeserializeCustomModuleMetaData, Module, ModuleId, ModuleMetaData,
    ModuleType, SerializeCustomModuleMetaData,
  };

  #[test]
  fn module_id() {
    #[cfg(not(target_os = "windows"))]
    let resolved_path = "/root/module.html";
    #[cfg(not(target_os = "windows"))]
    let module_id = ModuleId::new(resolved_path, "/root");

    #[cfg(target_os = "windows")]
    let resolved_path = "C:\\root\\module.html";
    #[cfg(target_os = "windows")]
    let module_id = ModuleId::new(resolved_path, "C:\\root");

    assert_eq!(module_id.id(Mode::Development), "module.html");
    assert_eq!(module_id.id(Mode::Production), "5de94ab0");
    assert_eq!(module_id.path(), "module.html");
    assert_eq!(module_id.hash(), "5de94ab0");
  }

  #[test]
  fn module_serialization() {
    let mut module = Module::new(
      ModuleId::new("/root/index.ts", "/root"),
      ModuleType::from_ext("ts"),
    );

    #[cache_item(CustomModuleMetaData)]
    struct StructModuleData {
      ast: String,
      imports: Vec<String>,
    }

    module.module_groups = HashSet::from([ModuleId::new("1", ""), ModuleId::new("2", "")]);

    module.meta = ModuleMetaData::Custom(Box::new(StructModuleData {
      ast: String::from("ast"),
      imports: vec![String::from("./index")],
    }) as _);

    let bytes = rkyv::to_bytes::<_, 256>(&module).unwrap();

    let archived = unsafe { rkyv::archived_root::<Module>(&bytes[..]) };
    let mut deserialized_module: Module = archived
      .deserialize(&mut rkyv::de::deserializers::SharedDeserializeMap::new())
      .unwrap();

    assert_eq!(deserialized_module.id.path(), module.id.path());

    assert_eq!(
      deserialized_module
        .meta
        .as_custom_mut::<StructModuleData>()
        .ast,
      "ast"
    );

    assert_eq!(
      deserialized_module
        .meta
        .as_custom::<StructModuleData>()
        .imports,
      vec![String::from("./index")]
    );

    assert!(deserialized_module
      .module_groups
      .contains(&ModuleId::new("1", "")));
    assert!(deserialized_module
      .module_groups
      .contains(&ModuleId::new("2", "")));
  }
}
