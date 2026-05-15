//! Rust core surface for the upstream `tailwindcss` package.
//!
//! This crate intentionally focuses on compile/build state and externally
//! supplied configuration data. It does **not** load JS/TS config files and
//! does **not** implement plugin-compat APIs.

pub mod apply;
pub mod ast;
pub mod candidate;
pub mod compiler;
pub mod design_system;
pub mod functions;
pub mod parser;
pub mod scanner;
pub mod theme;
pub mod utilities;
pub mod utils;
pub mod value_parser;
pub mod variants;
pub mod walk;

pub use candidate::parse_candidate;
pub use compiler::{
  compile, CompileError, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig,
};
