#![feature(box_patterns)]

pub mod fs;
pub mod npm_package;
pub mod script;
pub mod testing_helpers;

// re-exports dependencies
pub use swc_ecma_codegen;
pub use swc_ecma_parser;
pub use swc_ecma_transforms;
pub use swc_ecma_visit;
pub use testing_macros;
