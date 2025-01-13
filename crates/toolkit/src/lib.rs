#![feature(box_patterns)]
#![feature(let_chains)]

pub mod css;
pub mod fs;
pub mod hash;
pub mod html;
pub mod resolve;
pub mod script;
pub mod source_map;

// re-exports dependencies
pub use preset_env_base;
pub use swc_ecma_codegen;
pub use swc_ecma_minifier;
pub use swc_ecma_parser;
pub use swc_ecma_preset_env;
pub use swc_ecma_transforms;
pub use swc_ecma_transforms_base;
pub use swc_ecma_utils;
pub use swc_ecma_visit;

pub use swc_css_codegen;
pub use swc_css_minifier;
pub use swc_css_modules;
pub use swc_css_parser;
pub use swc_css_prefixer;
pub use swc_css_visit;

pub use swc_html_codegen;
pub use swc_html_minifier;
pub use swc_html_parser;
pub use swc_html_visit;

pub use swc_atoms;

pub use anyhow;
pub use farmfe_core::regex;
pub use itertools;
pub use lazy_static;
pub use sourcemap;

pub mod get_dynamic_resources_map;
pub mod minify;

pub mod constant;
pub mod plugin_utils;
