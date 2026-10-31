# Rust crate 目录

以下 11 个 crate 来自 Farm 仓库的 `rust-ecosystem/`，不是任意第三方依赖清单。用途和版本依据固定源码提交，registry 观测截至 **2026-10-24**。版本列为源码 manifest 版本，不代表该版本已发布或具有稳定公共 API。

[固定源码与清单](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem)

## Docs 构建组件

这些是已集成的 Docs 支撑库，不是独立文档站 CLI。应用开发者应从 [Farm Docs 使用指南](/zh/docs/farm-docs/usage)开始。

### farmfe_docs_config

Docs 配置、校验与 schema 协议，面向原生 Docs 集成开发者。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-config) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-config/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。

### farmfe_docs_content

Markdown/MDX 语义、标题、代码、资源与源码映射，面向内容工具开发者。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-content) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-content/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。

### farmfe_docs_plan

路由、链接、资源与内容构建计划，面向原生文档构建集成。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-plan) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-plan/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。

### farmfe_docs_seo

基于文档计划生成确定性 SEO 与订阅资源，面向文档工具开发者。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-seo) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-seo/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。

### farmfe_docs_snapshot

源码观测、输入输出凭据和构建校验，面向构建系统集成。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-snapshot) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-snapshot/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。

## 共享高亮库

### farmfe_syntax_highlight

Docs 与编译插件共享的 Rust 语法高亮、主题和静态渲染库，面向 Rust 工具开发者。

- 源码版本: `0.1.0`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。 [THIRD_PARTY_NOTICES.md](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight/THIRD_PARTY_NOTICES.md) 和主题声明也须保留。

## 编译与转换库

### farmfe_svgr_rs

SVG 到 React JSX/SWC 的转换，面向 Rust 转换工具集成；这是 Farm 的 svgr-rs 分支。

- 源码版本: `0.2.0`.
- 状态: [crates.io 0.2.0](https://crates.io/crates/farmfe_svgr_rs/0.2.0) 已存在；这不能证明 registry 归档与当前源码相同。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs/LICENSE): MIT. 保留原始 2023 Cong-Cong Pan 版权声明与 Farm 分支归属。

### farmfe_grass

带 Farm 源码映射与导入修复的 Sass 编译器分支，由原生 Sass 插件使用，面向 CSS 工具开发者。

- 源码版本: `0.13.4-farm.1`.
- 状态: 本地候选/集成源码；审计时未发现该名称的 crates.io 条目。此处不提供提前发布的安装命令。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass/LICENSE): MIT. 保留 2020 Connor Skees 版权及 [Grass 上游](https://github.com/connorskees/grass)归属。

### farmfe_ecosystem_tailwindcss

Rust Tailwind 编译、工具类、变体、主题和扫描实现；JS/TS 配置及插件兼容仍有缺口。

- 源码版本: `0.1.0`.
- 状态: 内部集成；自动发布配置 `release = false`。本轮没有确认 registry 是否存在，不能标为已正式发布。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。 [Tailwind 上游声明](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-plugins/tailwindcss/THIRD_PARTY_NOTICES.md)应随相应分发保留；不宣称已完全兼容上游。

### farmfe_ecosystem_tailwindcss_node

Rust CSS/模块解析、导入、URL、源码映射与依赖追踪，是 Tailwind 编译器的集成组件。

- 源码版本: `0.1.0`.
- 状态: 内部集成；自动发布配置 `release = false`。本轮没有确认 registry 是否存在，不能标为已正式发布。
- [源码](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss-node) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss-node/Cargo.toml).
- [许可证](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. 声明 MIT；不是全部上游与传递依赖许可已闭合的保证。 [Tailwind 上游声明](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-plugins/tailwindcss/THIRD_PARTY_NOTICES.md)应随相应分发保留；不宣称已完全兼容上游。

## Glob 匹配与遍历

### farmfe_ecosystem_glob

面向 Farm 集成的纯 Rust glob 匹配与 import-glob 遍历。限定参考配置为 Vite **8.3.5** 的 JavaScript transform/tinyglobby 0.2.17（picomatch 4.0.7、fdir 6.5.0）以及直接 JavaScript HMR 匹配。不宣称完整兼容 picomatch、minimatch 或 Vite，也不包含 Vite 独立的原生 Rolldown 路径。

- 源码版本: `0.1.0`.
- 状态: 本地未发布集成；不声明已有发布版本，不提供 registry 安装命令。
- [源码](https://github.com/farm-fe/farm/tree/79cdbe44c/rust-ecosystem/glob) · [Cargo.toml](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/Cargo.toml) · [兼容范围](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/README.md)。
- 来源: 派生自 picomatch-rs `0.1.0`，上游提交 `ceb39c90fc719fbbab79216a139f31c174272ed3`；不使用 NAPI 包装或 JavaScript RegExp。须保留 MIT [UPSTREAM-LICENSE](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/UPSTREAM-LICENSE)（含 Jon Schlinkert 版权）及 Farm 的 MIT 许可证。[UPSTREAM.patch](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/UPSTREAM.patch) 记录 Farm 对匹配器的修改。

## 发布与来源边界

本目录描述源码能力，不能替代分发验证或许可证审查。移植、复制代码及依赖的上游声明须在实际 npm/crate 归档中核查。上述 glob 集成是本地限定范围实现，其源码和兼容配置单独固定；不声明已发布或完整兼容。
