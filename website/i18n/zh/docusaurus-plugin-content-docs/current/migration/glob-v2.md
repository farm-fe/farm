---
title: V2 glob 迁移
---

Farm V2 的 import glob、HMR 匹配和 fixture 遍历统一使用 `farmfe_ecosystem_glob`。Rust Core 的公开入口改为 `farmfe_core::glob`，删除旧 `farmfe_core::wax` 导出及其可选诊断 API。这是正式 V2 发布前明确允许的破坏性变更。

## Rust 使用方

将旧 `wax::{Glob, Pattern}` 导入替换为 `farmfe_core::glob::{Glob, WalkOptions}`。`is_match` 返回 `Result<bool, GlobError>`，必须区分错误与 `Ok(false)`。

```rust
use farmfe_core::glob::{Glob, WalkOptions};

let glob = Glob::with_options("src/**/*.{js,ts}", WalkOptions::default())?;
for entry in glob.walk(root) {
  let entry = entry?;
  println!("{}", entry.path().display());
}
let matches = glob.is_match("src/main.ts")?;
```

匹配路径使用正斜杠。默认遍历排除隐藏路径和 `node_modules`；需要时显式设置 `exhaustive: true`。Farm 的 fixture helper 使用完整遍历，保留隐藏目录及以依赖目录命名的测试夹具。遍历 I/O 和符号链接循环错误会明确报错，不再静默丢弃条目。

新入口不要求旧 `Pattern` trait，也不提供 `Glob::diagnosed`、`Glob::diagnose` 或 Wax 的默认 `miette::Diagnostic` 实现。可以使用普通 `Display`、`Debug` 和 `std::error::Error`；错误类型区分模式编译、遍历、I/O 和根目录外路径。不会自动回退到旧匹配方言。

## JavaScript 使用方

受支持的 `import.meta.glob` adapter 保留 lazy/eager/import/query、alias/root/base、负模式及 HMR 新增/删除行为；query 不会污染对象键。兼容范围以固定 Vite 8.3.5 profile 为准，不承诺覆盖所有 Wax、picomatch 或 minimatch 语义。未闭合括号按所选 profile 处理，直接使用 matcher 时可以启用严格括号检查；不支持的编译和运行期匹配错误不会被转换为空结果。

transform/walker profile 禁用 extglob、启用 POSIX 字符类；直接 HMR 匹配使用独立的 `posix: false` profile，不能全局强制为同一种配置。

## 分发和版本

本次移除旧 Wax 依赖，而不只是关闭诊断 feature。发布前仍需检查全部 target 的默认/全部 feature 依赖图，并重建、核验新的 native 产物；旧二进制不能证明依赖已经排除。本次迁移没有补写或假造上游缺失的许可材料。

Core/compiler/toolkit 保持 Rust V2。Core 数据布局及动态插件接口未变，ABI 保持 `0.5.4`；正式分发的 Core 与插件仍必须由同一轮新源码重建。独立 glob crate 保持 `0.1.0`。

配套 Vuetify 修复的准确版本是 `@farmfe/js-plugin-vuetify@0.0.2`，不会覆盖已发布的 `0.0.1`。这些包及官网仍需经过正式发布批准和纯注册表消费验证。
