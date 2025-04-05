use std::{cell::RefCell, rc::Rc};

use farmfe_macro_cache_item::cache_item;
use feature_flag::FeatureFlag;
use swc_common::{
  comments::{
    Comment, SingleThreadedComments, SingleThreadedCommentsMap, SingleThreadedCommentsMapInner,
  },
  BytePos,
};
use swc_ecma_ast::Module as SwcModule;

use crate::module::ModuleId;
use crate::{HashMap, HashSet};

use super::custom::CustomMetaDataMap;

use statement::{Statement, SwcId};

pub mod feature_flag;
pub mod statement;

pub const EXPORT_NAMESPACE: &str = "namespace_farm_internal_";
pub const EXPORT_EXTERNAL_ALL: &str = "external_all_farm_internal_";
pub const EXPORT_EXTERNAL_NAMESPACE: &str = "external_namespace_farm_internal_";
pub const EXPORT_DEFAULT: &str = "default";

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
  /// export * from './module'; // where module is a external module
  /// ```
  ExternalAll,

  /// ```js
  /// import * as xx from './module';
  /// export * as xx from './module';
  /// ```
  ExternalNamespace,

  /// ```js
  /// import { foo as bar } from './foo.cjs';
  /// export { foo as default } from './foo.cjs'; // foo is not external using esm  
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
pub struct ModuleExportIdent {
  pub module_id: ModuleId,
  pub ident: SwcId,
  pub export_type: ModuleExportIdentType,
}

/// Script specific meta data, for example, [swc_ecma_ast::Module]
#[cache_item]
pub struct ScriptModuleMetaData {
  pub ast: SwcModule,
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub module_system: ModuleSystem,
  /// true if this module calls `import.meta.hot.accept()` or `import.meta.hot.accept(mod => {})`
  pub hmr_self_accepted: bool,
  pub hmr_accepted_deps: HashSet<ModuleId>,
  pub comments: CommentsMetaData,
  pub statements: Vec<Statement>,
  pub top_level_idents: HashSet<SwcId>,
  pub unresolved_idents: HashSet<SwcId>,
  pub is_async: bool,
  pub feature_flags: HashSet<FeatureFlag>,
  /// real export ident map, for example:
  /// export { m as bar }
  /// export { foo as default } from './module';
  /// =>
  /// Map<String, SwcId> { bar -> m#1, default -> foo#1 where foo#1 is defined in './module' }
  pub export_ident_map: HashMap<String, ModuleExportIdent>, // TODO add Arc for ModuleExportIdent
  pub custom: CustomMetaDataMap,
}

impl Default for ScriptModuleMetaData {
  fn default() -> Self {
    Self {
      ast: SwcModule::default(),
      top_level_mark: 0,
      unresolved_mark: 0,
      module_system: ModuleSystem::EsModule,
      hmr_self_accepted: false,
      hmr_accepted_deps: Default::default(),
      comments: Default::default(),
      statements: vec![],
      top_level_idents: Default::default(),
      unresolved_idents: Default::default(),
      is_async: false,
      feature_flags: Default::default(),
      export_ident_map: Default::default(),
      custom: Default::default(),
    }
  }
}

impl Clone for ScriptModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::default()
    } else {
      let mut custom = HashMap::default();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      top_level_mark: self.top_level_mark,
      unresolved_mark: self.unresolved_mark,
      module_system: self.module_system.clone(),
      hmr_self_accepted: self.hmr_self_accepted,
      hmr_accepted_deps: self.hmr_accepted_deps.clone(),
      comments: self.comments.clone(),
      statements: self.statements.clone(),
      top_level_idents: self.top_level_idents.clone(),
      unresolved_idents: self.unresolved_idents.clone(),
      is_async: self.is_async,
      feature_flags: self.feature_flags.clone(),
      export_ident_map: self.export_ident_map.clone(),
      custom: CustomMetaDataMap::from(custom),
    }
  }
}

impl ScriptModuleMetaData {
  pub fn take_ast(&mut self) -> SwcModule {
    std::mem::replace(
      &mut self.ast,
      SwcModule {
        span: Default::default(),
        body: Default::default(),
        shebang: None,
      },
    )
  }

  pub fn set_ast(&mut self, ast: SwcModule) {
    self.ast = ast;
  }

  pub fn take_comments(&mut self) -> CommentsMetaData {
    std::mem::take(&mut self.comments)
  }

  pub fn set_comments(&mut self, comments: CommentsMetaData) {
    self.comments = comments;
  }

  pub fn is_cjs(&self) -> bool {
    matches!(self.module_system, ModuleSystem::CommonJs)
  }

  pub fn is_esm(&self) -> bool {
    matches!(self.module_system, ModuleSystem::EsModule)
  }

  pub fn is_hybrid(&self) -> bool {
    matches!(self.module_system, ModuleSystem::Hybrid)
  }

  pub fn get_export_idents(&self) -> Vec<(String, ModuleExportIdent)> {
    let mut export_idents = self
      .export_ident_map
      .iter()
      .filter(|(k, _)| *k != EXPORT_NAMESPACE)
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect::<Vec<_>>();
    export_idents.sort_by_key(|a| a.0.clone());

    export_idents
  }
}

#[cache_item]
#[derive(Clone)]
pub struct CommentsMetaDataItem {
  pub byte_pos: BytePos,
  pub comment: Vec<Comment>,
}

#[cache_item]
#[derive(Clone, Default)]
pub struct CommentsMetaData {
  pub leading: Vec<CommentsMetaDataItem>,
  pub trailing: Vec<CommentsMetaDataItem>,
}

impl From<SingleThreadedComments> for CommentsMetaData {
  fn from(value: SingleThreadedComments) -> Self {
    let (swc_leading_map, swc_trailing_map) = value.take_all();
    let transform_comment_map = |map: SingleThreadedCommentsMap| {
      map
        .take()
        .into_iter()
        .map(|(byte_pos, comments)| CommentsMetaDataItem {
          byte_pos,
          comment: comments,
        })
        .collect::<Vec<CommentsMetaDataItem>>()
    };

    let leading = transform_comment_map(swc_leading_map);
    let trailing = transform_comment_map(swc_trailing_map);

    Self { leading, trailing }
  }
}

impl From<CommentsMetaData> for SingleThreadedComments {
  fn from(value: CommentsMetaData) -> Self {
    let transform_comment_map = |comments: Vec<CommentsMetaDataItem>| {
      Rc::new(RefCell::new(
        comments
          .into_iter()
          .map(|item| (item.byte_pos, item.comment))
          .collect::<SingleThreadedCommentsMapInner>(),
      ))
    };

    let leading = transform_comment_map(value.leading);
    let trailing = transform_comment_map(value.trailing);

    SingleThreadedComments::from_leading_and_trailing(leading, trailing)
  }
}

#[cache_item]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleSystem {
  UnInitial,
  EsModule,
  CommonJs,
  // Hybrid of commonjs and es-module
  Hybrid,
  Custom(String),
}

impl ModuleSystem {
  pub fn merge(&self, module_system: ModuleSystem) -> ModuleSystem {
    if matches!(module_system, ModuleSystem::UnInitial) {
      return self.clone();
    }

    match self {
      ModuleSystem::UnInitial => module_system,
      ModuleSystem::EsModule => {
        if matches!(module_system, ModuleSystem::CommonJs) {
          ModuleSystem::Hybrid
        } else {
          module_system
        }
      }

      ModuleSystem::CommonJs => {
        if matches!(module_system, ModuleSystem::EsModule) {
          ModuleSystem::Hybrid
        } else {
          module_system
        }
      }

      ModuleSystem::Hybrid => ModuleSystem::Hybrid,

      ModuleSystem::Custom(_) => module_system,
    }
  }

  pub fn contains_commonjs(&self) -> bool {
    matches!(self, ModuleSystem::CommonJs | ModuleSystem::Hybrid)
  }
}
