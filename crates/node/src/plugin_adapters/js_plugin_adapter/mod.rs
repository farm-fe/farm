use farmfe_core::{
  context::CompilationContext,
  error::Result,
  plugin::{
    FilteringHookParam, Plugin, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
  },
};
use napi::{bindgen_prelude::ToNapiValue, JsObject};

use self::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook;

mod thread_safe_js_plugin_hook;

pub struct JsPluginAdapter {
  name: String,
  js_resolve_hook: ThreadSafeJsPluginHook,
}

impl JsPluginAdapter {
  pub fn new(name: String, js_plugin_object: JsObject) -> Self {
    // TODO calculating hooks should execute
    Self {
      name,
      js_resolve_hook: ThreadSafeJsPluginHook::new(),
    }
  }
}

impl Plugin for JsPluginAdapter {
  fn name(&self) -> String {
    self.name.clone()
  }

  // TODO filtering hooks should be executed
  fn should_execute_hook(&self, _param: &FilteringHookParam<'_>) -> bool {
    true
  }

  fn resolve(
    &self,
    _param: &PluginResolveHookParam,
    _context: &CompilationContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(None)
  }
}

macro_rules! define_struct_with_from_and_into {
  ($ori_ty:ident, $name:ident { $(pub $field:ident: $ty:ty)+ }) => {
    #[napi(object)]
    pub struct $name {
      $(pub $field:$ty),+
    }

    impl From<$ori_ty> for $name {
      fn from(res: $ori_ty) -> Self {
        Self {
          $($field: res.$field.into()),+
        }
      }
    }

    impl From<$name> for $ori_ty{
      fn from(ori: $name) -> Self {
        Self {
          $($field: ori.$field.into()),+
        }
      }
    }
  };
}

macro_rules! define_enum_with_from_and_into {
    ($ori_ty:ident, $name:ident { $($field:ident)+ } ) => {
      #[napi]
      pub enum $name {
        $($field),+
      }

      impl From<$ori_ty> for $name {
        fn from(rk: $ori_ty) -> Self {
          match rk {
            $($ori_ty::$field => Self::$field),+
          }
        }
      }

      impl From<$name> for $ori_ty {
        fn from(ori: $name) -> Self {
          match ori {
            $($name::$field => Self::$field),+
          }
        }
      }
    };
}

define_enum_with_from_and_into! {
  ResolveKind,
  JsResolveKind {
    Entry
    Import
    DynamicImport
    Require
    AtImport
    Url
    ScriptSrc
    LinkHref
  }
}

define_struct_with_from_and_into! {
  PluginResolveHookParam,
  JsPluginResolveHookParam {
    pub specifier: String
    pub importer: Option<String>
    pub kind: JsResolveKind
  }
}

define_struct_with_from_and_into! {
  PluginResolveHookResult,
  JsPluginResolveHookResult {
    pub id: String
    pub external: bool
    pub side_effects: bool
  }
}
