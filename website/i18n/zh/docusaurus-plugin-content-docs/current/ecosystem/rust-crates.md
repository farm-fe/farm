# Rust crate 目录

以下 10 个 crate 来自 Farm 仓库的 `rust-ecosystem/`，不是任意第三方依赖清单。用途和版本依据固定源码提交，registry 观测截至 **2026-10-24**。版本列为源码 manifest 版本，不代表该版本已发布或具有稳定公共 API。

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

## 发布与来源边界

目录描述源码中的能力，不替代分发包验收与许可证审计。移植、复制及依赖各自的上游声明需要随实际 npm/crate 归档核查；尚未解决的授权证据不能标为已齐。新的 glob/minimatch 兼容方案仍在评估，此处没有将拟议 crate 列为已实现或已发布。
