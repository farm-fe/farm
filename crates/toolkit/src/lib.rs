#![feature(box_patterns)]

pub mod css;
pub mod fs;
pub mod html;
pub mod resolve;
pub mod rkyv;
pub mod script;

// re-exports dependencies
pub use swc_ecma_codegen;
pub use swc_ecma_parser;
pub use swc_ecma_transforms;
pub use swc_ecma_visit;

pub use swc_css_codegen;
pub use swc_css_parser;
pub use swc_css_visit;

pub use swc_html_codegen;
pub use swc_html_parser;
pub use swc_html_visit;

pub use lazy_static;
pub use regex;
