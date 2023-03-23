# 贡献指南

Farm 整个项目分为两个部分, `JavaScript` 和 `Rust`。

- **JavaScript 部分**: 查看 `packages` 文件夹中的代码, 包含核心包(开发服务, 文件监听, 编译器包装), 脚手架, 运行时和运行时插件 (模块系统, HMR 热更新)。
- **Rust 部分**: 查看 `packages` 文件夹中的代码, 包含核心包 (编译上下文, 插件驱动等), 编译器 (编译进程、HMR 更新等), Rust 插件。

开发步骤 :

1. 安装 Rust 开发环境 （如果您没有使用过 Rust ,请搜索 "Rustup Book") 并且保证您的 Node 版本 >= 16。
2. 使用 `pnpm bootstrap` 安装依赖项，并构建核心包。
3. 运行示例 Demo (打开一个新的终端): `cd examples/react && pnpm start`, 如果启动失败, 请新建一个 issues 并提供错误信息。
4. 如果你修改了在 `crates` 包中的 Rust 代码, 请在 `packages/core` 包中执行 `pnpm run build:rs` 来编译最新的代码。
