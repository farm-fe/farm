use std::sync::Arc;

use farmfe_core::{config::minify::MinifyOptions, serde_json, swc_common::SourceMap};
use swc_ecma_minifier::option::{
  terser::{TerserCompressorOptions, TerserTopLevelOptions},
  MangleOptions, MinifyOptions as JsMinifyOptions,
};

#[derive(Clone)]
pub struct NormalizedMinifyOptions {
  pub compress: Option<TerserCompressorOptions>,
  pub mangle: Option<MangleOptions>,
}

impl NormalizedMinifyOptions {
  pub fn minify_options_for_resource_pot(minify: &MinifyOptions) -> NormalizedMinifyOptions {
    // compress
    let mut compress = minify
      .compress
      .clone()
      .map(|value| {
        serde_json::from_value::<TerserCompressorOptions>(value)
          .expect("FarmPluginMinify.compress option is invalid")
      })
      .unwrap_as_option(|default| match default {
        Some(true) => Some(Default::default()),
        _ => None,
      });

    if let Some(compress) = &mut compress {
      if compress.const_to_let.is_none() {
        compress.const_to_let = Some(true);
      }

      if compress.toplevel.is_none() {
        compress.toplevel = Some(TerserTopLevelOptions::Bool(true));
      }
    }

    // mangle
    let mangle = minify
      .mangle
      .clone()
      .map(|value| {
        serde_json::from_value::<MangleOptions>(value)
          .expect("FarmPluginMinify.mangle option is invalid")
      })
      .unwrap_as_option(|default| match default {
        Some(true) => Some(Default::default()),
        _ => None,
      })
      .map(|mut mangle| {
        if mangle.top_level.is_none() {
          mangle.top_level = Some(true);
        }

        mangle
      });

    NormalizedMinifyOptions { compress, mangle }
  }

  pub fn minify_options_for_module(minify: &MinifyOptions) -> NormalizedMinifyOptions {
    let mut minify_options = Self::minify_options_for_resource_pot(minify);

    minify_options.compress = minify_options.compress.map(|mut v| {
      v.toplevel = None;
      v
    });

    minify_options
  }

  pub fn into_js_minify_options(self, cm: Arc<SourceMap>) -> JsMinifyOptions {
    JsMinifyOptions {
      compress: self.compress.map(|item| item.into_config(cm)),
      mangle: self.mangle,
      ..Default::default()
    }
  }
}
