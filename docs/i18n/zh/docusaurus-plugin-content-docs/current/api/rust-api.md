# Rust Api
您可以在 Rust 代码中创建 Farm Rust 编译器。 例子：

```rust
use farmfe_compiler::Compiler;
use farmfe_core::config::Config;

// 创建 farm 编译器
pub fn create_farm_compiler() {
  let config = Config::default();
  let extra_plugins = vec![];

  let compiler = Compiler::new(config, extra_plugins);

  compiler
}

// 编译项目
pub fn compile() {
  let compiler = create_farm_compiler();
  compiler.compile()
}

// 执行热更新
pub fn update(compiler: Compiler) {
  let update_result = compiler.update(vec![(String::from("/root/index.ts"), UpdateType:Update)], || {
    // 当所有更新（包括资源再生）完成时调用
  }, true);

  // 处理 update_result...
}
```

Farm Rust 编译器由 [`farmfe_compiler`](https://docs.rs/farmfe_core/latest/farmfe_compiler) crate 导出。 请参阅 [`farmfe_compiler`](https://docs.rs/farmfe_core/latest/farmfe_compiler) 文档。