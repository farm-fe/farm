<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="./assets/logo.png" width="550" />
  </a>
  <h3>基于 Rust 的极速构建引擎</h3>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/README.md">English</a> | 
    <span>简体中文</span>
  </p>
  <p align="center">
    <a href="https://discord.gg/mDErq9aFnF">
      <img src="https://img.shields.io/badge/chat-discord-blueviolet?style=flat&logo=discord&colorA=ffe3f5&colorB=711a5f" alt="discord chat" />
    </a>
    <a href="https://npmjs.com/package/@farmfe/core"><img src="https://img.shields.io/npm/v/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="npm package"></a>
    <a href="https://nodejs.org/en/about/releases/"><img src="https://img.shields.io/node/v/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="node compatibility"></a>
  <a href="https://npmcharts.com/compare/@farmfe/core?minimal=true">
    <img src="https://img.shields.io/npm/dm/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="downloads" />
  </a>
  <a href="https://github.com/farm-fe/farm/blob/main/LICENSE">
    <img src="https://img.shields.io/npm/l/@farmfe/core?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="license" />
  </a>
  </p>
  <br/>
</div>

---

## 介绍

Farm 是一个使用 Rust 编写的极速 Web 构建工具，类似 webpack 或者 vite，但是 Farm 更快。

Farm 采取性能优先的设计思路，对照其他工具进行基准测试 (使用 Turbopack 的基准测试，1000 个 React 组件) 如下所示:

![xx](./assets/benchmark.jpg)

> 测试仓库地址: https://github.com/farm-fe/performance-compare
>
> 测试机器环境 (Linux Mint 21.1 Cinnamon, 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6, 15.5 GiB)

<br />

## 特性

- ⚡ **超级快**: 使用 Rust 编写, 可以在毫秒级别内启动一个 React 或 Vue 项目。 在大多数情况下, 可以在 10ms 内执行 HMR 的更新。
- 🧰 **完全可插拔**: Farm 由插件驱动, 同时支持 Farm 编译插件（Rust 和 JavaScript 插件，以及 Swc 插件），Farm Runtime 插件，Farm Dev Server 插件。
- ⚙️ **丰富的编译能力支持**: 开箱即用, Farm 内置了 JS/TS/JSX/TSX、CSS、Css Modules、Sass、Less、Postcss，HTML 和静态资源的编译，为 React，Vue，Solid 等框架提供了官方插件。
- ⏱️ **懒编译**: 仅仅在请求时才编译动态导入的资源，极大提速大型项目的编译。通过 `dynamic import` 即可启用，被 `dynamic import` 的模块及其依赖仅在使用时才会编译。
- 📦 **局部打包**: 自动根据依赖关系、资源大小，将项目打包成若干个资源，提升资源加载性能的同时，保证缓存命中率。
- 🔒 **一致性**: 开发环境和生产环境的表现一致，所见即所得。
- 🌳 **兼容性**: 同时支持传统(ES5)和现代浏览器。

<br/>

Farm 设计动机和理念请看 [RFC-001](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation)。。

<br/>

## 快速开始

使用任意常用包管理器，即可快速创建一个 Farm 项目:

```bash
# npm
npm create farm@latest
# yarn
yarn create farm@latest
# pnpm
pnpm create farm@latest
```

然后启动项目:

```bash
cd farm-project && npm start
```

1 分钟快速开始视频演示:

https://github.com/farm-fe/farm/assets/8372439/51e8834b-584a-4d9f-ae6f-516da70d3173

请参考[文档](https://farm-fe.github.io) 进一步了解 Farm。

## 示例

Farm 支持编译 React、Vue、SolidJS、Sass、Less、Css Modules 等场景场景，具体可以查看我们的示例:

### React 项目示例

- [React-Basic](https://github.com/farm-fe/farm/tree/main/examples/react)
- [React-Ant-Design](https://github.com/farm-fe/farm/tree/main/examples/react-antd)
- [React-Sass-CssModules](https://github.com/farm-fe/farm/tree/main/examples/css-modules)
- [React-Multi-Page-Application](https://github.com/farm-fe/farm/tree/main/examples/multi-page-app)
- [React-SSR](https://github.com/farm-fe/farm/tree/main/examples/react-ssr)
- [React-TailwindCSS](https://github.com/farm-fe/farm/tree/main/examples/tailwind)
- [React-Emotion](https://github.com/farm-fe/farm/tree/main/examples/emotion)

### Vue 项目示例

- [Vue-Basic](https://github.com/farm-fe/farm/tree/main/examples/vue)
- [Vue-Jsx](https://github.com/farm-fe/farm/tree/main/examples/vue-jsx)
- [Vue-Antdv](https://github.com/farm-fe/farm/tree/main/examples/vue-antdv)

### SolidJS 项目示例

- [SolidJS-Basic](https://github.com/farm-fe/farm/tree/main/examples/solid)

## 开发计划

Farm 目前已经实现了一个编译引擎的所有能力，包括生产环境优化如 tree shake 以及产物压缩。我们已经将企业级 web 应用成功迁移到 Farm，极大提升了构建速度以及开发体验。

查看 [开发计划](https://github.com/farm-fe/farm/blob/main/ROADMAP.zh-CN.md)

## 贡献

查看 [贡献指南](https://github.com/farm-fe/farm/blob/main/CONTRIBUTING.zh-CN.md)

## 交流群

- 加入 [Discord](https://discord.gg/mDErq9aFnF)

- 微信群

<img src="https://github.com/farm-fe/farm/assets/66500121/10baac12-d074-4a3a-b040-136b9408718d" width="30%" />

## 作者

brightwu（吴明亮）, 曾就职于字节跳动和腾讯。
