# 贡献指南

非常感谢您对 Farm 感兴趣, 在您提交 Pull Request 之前, 请先阅读以下指南。


## 行为规范准则

所有贡献者都应该遵循 Rust [行为规范](https://www.rust-lang.org/policies/code-of-conduct)。


## 错误报告

目前 Farm 正在快速开发和迭代中, 在开发和使用中可能会遇到一些问题, 如果您遇到了一些我们不可预料的问题, 请提交 issues 用来报告错误问题, 通过新建一个 [issues](https://github.com/farm-fe/farm/issues/new/choose) 来报告您所遇到的问题。

## 创建新特性

如果要创建新的功能或者特性, 请在 issues 中添加 [feature request](https://github.com/farm-fe/farm/issues/new/choose)。


## 提交代码指南

- Fock [Farm](https://github.com/farm-fe/farm) 仓库。
- 编写代码的时候, 请遵循代码编写规范。
- 设置您的本地开发环境。
- 在您的本地从 `main` 分支切出一个新的功能特性分支。
- 使用 `cargo test` 确保所有测试均能通过。
- 如果您已经更改了一些包并准备更新版本，则您应该在根目录中输出`npx changeset` 用来发布新版本并且提交。


## 设置您的本地开发环境

### 依赖

- 安装 Rust 环境 [rustup](https://www.rust-lang.org/tools/install)。

- 确保您的 [Node.js](https://nodejs.org) 版本在 **16** 以上。

- 确保您的 [Pnpm](https://pnpm.io) 版本在 **7.28.0** 以上。

### 设置其他依赖

- 在构建 `sass-embedded` 需要用到 [protoc](https://grpc.io/docs/protoc-installation/) 所以您的本地开发环境还需要安装 [protoc](https://grpc.io/docs/protoc-installation/)。

**TIP:** 当您在初次开发时, 请确保您的本地环境已经安装了 `protoc`。如果您的本地环境没有安装 `protoc`，则在执行 `pnpm bootstrap` 时会触发脚本, 针对 `mac` `linux` 用户会自动安装 `protoc`, 针对 `windows` 用户不会自动安装, 但是可以根据提示自行下载安装。

## 运行项目

Farm 的开发启动非常简单, 您只需在根目录中执行 `pnpm bootstrap` 一条命令即可构建所有子包中需要构建的代码。

```bash
$ pnpm bootstrap # install the dependencies of the project with series of initialization operations.
```

- 使用`pnpm bootstRap`安装依赖项，并通过一系列初始化操作构建核心包。

- 使用示例(打开新终端)：`cd Examples/Reaction&&pnpm start`，如果示例不能正常启动，则上报问题。

- 如果`Examples/react`正常运行，则表明开发环境配置成功。

- 如果您更改了`crates`中的 `Rust` 代码，请再次运行 `Packages/core` 下的 `npm run build：rs` 以获取最新的二进制代码。

## 通过脚手架快速创建插件

Farm 提供了一个脚手架来帮助您快速创建一个插件, 您可以通过以下命令来创建一个插件。
您可以 `cd packages/cli` 目录下, 运行 `npm link` 或者全局安装 `@farmfe/cli` 来使用,
安装完成之后, 您可以通过 `farm plugin create` 来创建一个插件, 支持 `rust` 和 `js` 插件

  ```bash
  $ farm plugin create <plugin-name> # create a plugin support js or rust
  ```


## 小提示

Farm 整个项目分为两个部分, JavaScript 和 Rust。

- **JavaScript** 部分: 查看 packages 文件夹中的代码, 包含核心包(开发服务, 文件监听, 编译器包装), 脚手架, 运行时和运行时插件 (模块系统, HMR 热更新)。
- **Rust** 部分: 查看 crates 以及 rust-plugin 文件夹中的代码, 包含核心包 (编译上下文, 插件驱动等), 编译器 (编译进程、HMR 更新等), Rust 插件。


