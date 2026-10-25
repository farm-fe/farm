# Farm 生态

Farm 以 Rust 原生编译器和官方 Rust 插件为基础，生态还包括文档工具及可复用的 Rust 库。按你要构建的项目选择入口。

## 使用和扩展 Farm

在[官方插件文档](/zh/docs/plugins/official-plugins/overview)中了解编译扩展；需要开发插件时，阅读[编写 Rust 插件](/zh/docs/plugins/writing-plugins/rust-plugin)。

## 复用 Rust 库

[Rust crate 目录](./rust-crates.md)将已核实的库按用途分类，提供真实源码来源，并区分已发布版本、仅源码项目和本地候选。

## 构建文档站

[Farm Docs](/zh/docs/farm-docs/overview)提供文档集合、路由、主题以及面向 Agent 的 Markdown 入口。Rust 负责内容解析、校验和编译扩展；Node 编排工作流，React 渲染界面与静态页面。

每个项目独立说明发布状态。本地工作区构建成功，不代表所有 crate、npm 包和平台原生文件都已正式发布。
