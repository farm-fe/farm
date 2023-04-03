<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="./assets/logo.png" width="550" />
  </a>
  <p>基于 Rust 的极速构建引擎</p>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/README.md">English</a> | 
    <span>简体中文</span>
  </p>
  <p align="center">
    <a href="https://discord.gg/mDErq9aFnF">
      <img src="https://img.shields.io/badge/chat-discord-blueviolet?style=flat&logo=discord" alt="discord chat" />
    </a>
    <a href="https://npmjs.com/package/@farmfe/core"><img src="https://img.shields.io/npm/v/@farmfe/core.svg" alt="npm package"></a>
    <a href="https://nodejs.org/en/about/releases/"><img src="https://img.shields.io/node/v/@farmfe/core.svg" alt="node compatibility"></a>
    <a href="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml"><img src="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml/badge.svg" alt="build status"></a>
  </p>
  <br/>
</div>

---

## 介绍

Farm 是一个使用 Rust 编写的超级快、轻量级的 Web 构建工具, 对照其他工具进行基准测试 (使用 Turbopack 的基准测试，1000 个 React 组件) 如下所示:

![xx](./assets/benchmark.png)

> 测试仓库地址: https://github.com/farm-fe/performance-compare
>
> 测试机器环境 (Linux Mint 21.1 Cinnamon, 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6, 15.5 GiB)

<br />

## 特性

- ⚡ **超级快**: 使用 Rust 编写, 可以在毫秒级别内启动一个 React 或 Vue 项目。 在大多数情况下, 可以在 10ms 内执行 HMR 的更新。

- 🧰 **完全可插拔**: Farm 由插件驱动, 通过创建插件来实现任何您想要的, 同时支持 Rust 和 JavaScript 插件。

- ⚙️ **丰富的编译能力支持**: 开箱即用, Farm 内置了 JS/TS/JSX/TSX、CSS、HTML 和静态资源的编译。
- ⏱️ **惰性编译**: 仅仅在请求时才编译动态导入的资源。
- 📦 **局部打包**: 自动根据依赖关系、资源大小，将项目打包成若干个资源，提升资源加载性能的同时，保证缓存命中率。
- 🔒 **一致性**: 开发环境和生产环境的表现一致。
- 🌳 **兼容性**: 同时支持传统(ES5)和现代浏览器。

<br/>

> **注意**:
>
> - 关于设计动机和原则请看 [RFC-001](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation)。
> - **项目仍在开发中，尚未准备好用于生产环境。欢迎贡献**。
>
> Farm 基于 SWC 项目构建，使用 SWC 进行 HTML/CSS/JS/TSX/TS/JSX 解析、转换、优化和生成代码。

<br/>

## 快速开始

创建一个 Farm 项目 :

使用 npm:

```bash
$ npm create farm@latest
```

使用 yarn:

```bash
$ yarn create farm
```

使用 pnpm:

```bash
$ pnpm create farm
```

请参考[文档](https://farm-fe.github.io)以了解有关 Farm 的更多信息。

## 计划

查看 [计划](https://github.com/farm-fe/farm/blob/main/ROADMAP.zh-CN.md)

## 贡献

查看 [贡献指南](https://github.com/farm-fe/farm/blob/main/CONTRIBUTING.zh-CN.md)

## 作者

brightwu（吴明亮）, 曾就职于字节跳动和腾讯, 技术爱好者。
