# Rust Api
You can create a Farm Rust compiler in your rust code. Example:

```rust
use farmfe_compiler::Compiler;
use farmfe_core::config::Config;

// create farm compiler
pub fn create_farm_compiler() {
  let config = Config::default();
  let extra_plugins = vec![];

  let compiler = Compiler::new(config, extra_plugins);

  compiler
}

// compile the project
pub fn compile() {
  let compiler = create_farm_compiler();
  compiler.compile()
}

// perform hot update
pub fn update(compiler: Compiler) {
  let update_result = compiler.update(vec![(String::from("/root/index.ts"), UpdateType:Update)], || {
    // called when all update(including resource regeneration) finished
  }, true);

  // handle update_result...
}
```

Farm Rust compiler is exported by [`farmfe_compiler`](https://docs.rs/farmfe_core/latest/farmfe_compiler) crate. Refer to [`farmfe_compiler`](https://docs.rs/farmfe_core/latest/farmfe_compiler) documentation.