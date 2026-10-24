use std::sync::Arc;

use farmfe_core::{
  HashMap,
  context::CompilationContext,
  error::Result,
  plugin::PluginFinalizeResourcesHookParam,
  resource::Resource,
  serde::{Deserialize, Serialize},
};
use napi::{
  Env, JsValue, Unknown,
  bindgen_prelude::{FromNapiValue, Function, JsObjectValue, Object},
};

use crate::plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::{
  HookParamEncoder, ThreadSafeJsPluginHook,
};

struct FinalizeResourcesEncoder;

// Keep the input schema in step with Resource's serde representation.
const RESOURCE_INPUT_FIELDS: [&str; 8] = [
  "name",
  "nameHash",
  "bytes",
  "emitted",
  "shouldTransformOutputFilename",
  "resourceType",
  "origin",
  "specialPlaceholders",
];

impl HookParamEncoder<JsPluginFinalizeResourcesHookParams> for FinalizeResourcesEncoder {
  fn encode<'env>(
    env: &'env Env,
    param: JsPluginFinalizeResourcesHookParams,
  ) -> napi::Result<Unknown<'env>> {
    let mut object = Object::new(env)?;
    let mut resources = Object::new(env)?;
    for (key, resource) in &param.resources_map {
      let length = u32::try_from(resource.bytes.len())
        .map_err(|_| napi::Error::from_reason("Resource bytes exceed JavaScript array length"))?;
      env.run_in_scope(|| {
        let mut encoded_resource = Object::new(env)?;
        // Set each field once, in serde order. Serializing empty bytes then
        // replacing them would expose extra writes to inherited setters.
        encoded_resource
          .set_named_property(RESOURCE_INPUT_FIELDS[0], env.to_js_value(&resource.name)?)?;
        encoded_resource.set_named_property(
          RESOURCE_INPUT_FIELDS[1],
          env.to_js_value(&resource.name_hash)?,
        )?;
        let mut array = env.create_array(length)?;
        // A byte has only 256 possible values. Keep these handles in this local
        // scope instead of creating one NAPI number handle per byte.
        let mut numbers = [None; 256];
        for (index, byte) in resource.bytes.iter().enumerate() {
          let number = match numbers[*byte as usize] {
            Some(number) => number,
            None => {
              let number = env.create_uint32(u32::from(*byte))?;
              numbers[*byte as usize] = Some(number);
              number
            }
          };
          array.set(index as u32, number)?;
        }
        encoded_resource.set_named_property(RESOURCE_INPUT_FIELDS[2], array)?;
        encoded_resource.set_named_property(
          RESOURCE_INPUT_FIELDS[3],
          env.to_js_value(&resource.emitted)?,
        )?;
        encoded_resource.set_named_property(
          RESOURCE_INPUT_FIELDS[4],
          env.to_js_value(&resource.should_transform_output_filename)?,
        )?;
        encoded_resource.set_named_property(
          RESOURCE_INPUT_FIELDS[5],
          env.to_js_value(&resource.resource_type)?,
        )?;
        encoded_resource
          .set_named_property(RESOURCE_INPUT_FIELDS[6], env.to_js_value(&resource.origin)?)?;
        encoded_resource.set_named_property(
          RESOURCE_INPUT_FIELDS[7],
          env.to_js_value(&resource.special_placeholders)?,
        )?;
        resources.set_property(env.create_string(key)?, encoded_resource)?;
        Ok(())
      })?;
    }
    object.set_named_property("resourcesMap", resources)?;
    object.set_named_property("config", env.to_js_value(&param.config)?)?;
    unsafe { Unknown::from_napi_value(env.raw(), object.raw()) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::BTreeSet;

  #[test]
  fn resource_input_fields_match_serde_schema_and_order() {
    let resource = Resource::default();
    let value = farmfe_core::serde_json::to_value(&resource).unwrap();
    let actual: BTreeSet<_> = value
      .as_object()
      .unwrap()
      .keys()
      .map(String::as_str)
      .collect();
    assert_eq!(actual, RESOURCE_INPUT_FIELDS.into_iter().collect());
    let serialized = farmfe_core::serde_json::to_string(&resource).unwrap();
    let positions: Vec<_> = RESOURCE_INPUT_FIELDS
      .iter()
      .map(|field| serialized.find(&format!("\"{field}\":")).unwrap())
      .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
  }
}

pub struct JsPluginFinalizeResourcesHook {
  tsfn: ThreadSafeJsPluginHook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
pub struct JsPluginFinalizeResourcesHookParams {
  pub resources_map: HashMap<String, Resource>,
  pub config: farmfe_core::config::Config,
}

impl From<&mut PluginFinalizeResourcesHookParam<'_>> for JsPluginFinalizeResourcesHookParams {
  fn from(value: &mut PluginFinalizeResourcesHookParam) -> Self {
    Self {
      resources_map: value.resources_map.clone(),
      config: value.config.clone(),
    }
  }
}

impl JsPluginFinalizeResourcesHook {
  pub fn new(env: &napi::Env, obj: Object) -> Self {
    let func = obj
      .get_named_property::<Function>("executor")
      .expect("executor should be checked in js side");

    Self {
      tsfn: ThreadSafeJsPluginHook::new_with_encoder::<
        JsPluginFinalizeResourcesHookParams,
        HashMap<String, Resource>,
        FinalizeResourcesEncoder,
      >(env, func),
    }
  }

  pub fn call(
    &self,
    param: JsPluginFinalizeResourcesHookParams,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<HashMap<String, Resource>>> {
    self.tsfn.call(param, ctx, None)
  }
}
