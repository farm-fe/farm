#![feature(box_patterns)]

pub mod css;
pub mod fs;
pub mod hash;
pub mod html;
pub mod resolve;
pub mod rkyv;
pub mod script;
pub mod sourcemap;

// re-exports dependencies
pub use preset_env_base;
pub use swc_ecma_codegen;
pub use swc_ecma_minifier;
pub use swc_ecma_parser;
pub use swc_ecma_preset_env;
pub use swc_ecma_transforms;
pub use swc_ecma_transforms_base;
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
pub use base64;
pub use farmfe_core::regex;
pub use lazy_static;
pub use rand;

pub mod get_dynamic_resources_map;
