//! Rust core surface for the upstream `tailwindcss` package.
//!
//! This crate intentionally focuses on compile/build state and externally
//! supplied configuration data. It does **not** load JS/TS config files and
//! does **not** implement plugin-compat APIs.

pub mod compiler;
pub mod scanner;

pub use compiler::{compile, Compiler, CompilerOptions, Features, Polyfills, TailwindConfig};
