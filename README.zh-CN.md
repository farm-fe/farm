<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="./assets/logo.png" width="550" />
  </a>
  <h3>Rust native Web 构建工具与官方 Rust 插件生态</h3>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/README.md">English</a> | 
    <span>简体中文</span>
  </p>
  <p align="center">
    <a href="https://discord.gg/mDErq9aFnF">
      <img src="https://img.shields.io/badge/chat-discord-blueviolet?style=flat&logo=discord&colorA=ffe3f5&colorB=711a5f" alt="discord chat" />
    </a>
    <a href="https://codecov.io/gh/farm-fe/farm" > 
      <img src="https://codecov.io/gh/farm-fe/farm/graph/badge.svg?token=AJM5HMP07F"/> 
    </a>
    <a href="https://npmjs.com/package/@farmfe/core"><img src="https://img.shields.io/npm/v/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="npm package"></a>
    <a href="https://nodejs.org/en/about/releases/"><img src="https://img.shields.io/node/v/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="node compatibility"></a>
  <a href="https://npmcharts.com/compare/@farmfe/core?minimal=true">
    <img src="https://img.shields.io/npm/dm/@farmfe/core.svg?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="downloads" />
  </a>
  <a href="https://github.com/farm-fe/farm/blob/main/LICENSE">
    <img src="https://img.shields.io/npm/l/@farmfe/core?style=flat-square&colorA=ffe3f5&colorB=711a5f" alt="license" />
  </a>
  <a href="https://gurubase.io/g/farm">
    <img src="https://img.shields.io/badge/Gurubase-Ask%20Farm%20Guru-711a5f?style=flat-square&labelColor=ffe3f5" alt="Gurubase" />
  </a>
  </p>
  <br/>
</div>

---

## 介绍

Farm 是 Rust native Web 构建工具。编译器及内置编译插件在 Rust 中处理模块图、转换、打包与优化，官方 Rust 插件进一步覆盖框架、样式、资源与文档构建。

Node.js 负责配置、CLI 与开发服务器编排；需要时仍会使用 JavaScript 插件、React 渲染与应用运行时。Vite 插件适配器用于渐进迁移，而不是 Farm 架构的定义。

本分支包含 v2 发布候选。源码中的版本与插件实现不代表对应包已在 registry 发布或已完成正式版验收。评估 v2 时应使用匹配且经过验证的候选；`@latest` 命令安装的是 registry 当前已发布版本。

## 在线体验

[![Edit Farm](https://codesandbox.io/static/img/play-codesandbox.svg)](https://codesandbox.io/p/github/ErKeLost/react/main)

## 为什么选择 Farm？

Farm 以 Rust 原生编译器和编译扩展为基础，为开发与生产提供统一的插件架构，并结合增量编译、可配置打包和现有 JavaScript 工具的适配能力。架构与迁移说明见[为什么需要 Farm](https://farmfe.org/zh/docs/why-farm)。

## 特性

- **Rust native 编译**：Rust 编译器与内置插件覆盖脚本、CSS、HTML、JSON、解析及生产优化。
- **官方 Rust 插件家族**：覆盖 React/Vue、Sass/Tailwind CSS、自动导入、SVG/图标、数据与资源、Worker/Wasm 及文档工具。功能范围和候选状态见[插件总览](website/i18n/zh/docusaurus-plugin-content-docs/current/plugins/official-plugins/overview.md)，不同插件的覆盖程度并不相同。
- **增量工作流**：持久缓存、懒编译与热更新；效果取决于项目、配置和缓存状态。
- **局部打包**：控制产物分组，开发与生产使用同一编译器基础。
- **可扩展与渐进迁移**：支持 Rust/JavaScript 编译插件、运行时插件和 Vite 适配器。请验证项目实际使用的 hooks 与框架功能，不假设所有 Vite 插件都可直接替换。

v2 包发布、各平台原生产物及正式版验收，与源码是否存在分别跟踪。

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

查看 [Farm 文档](https://farmfe.org) 了解更多用法。

## 基准测试

历史基准数据与对应工作负载保留在 [performance-compare](https://github.com/farm-fe/performance-compare)。这些结果不是本次 v2 候选的实测，也不是对项目耗时的保证。评估 v2 时应固定内容、配置、硬件和缓存状态后再比较。

## 贡献

查看 [贡献指南](https://github.com/farm-fe/farm/blob/main/CONTRIBUTING.zh-CN.md)

## 交流群

- [作者推特](https://twitter.com/brightwwu46799), [官方推特](https://twitter.com/fe_farm)

- 加入 [Discord](https://discord.gg/mDErq9aFnF)

- 微信交流群

<br><img src="https://github.com/user-attachments/assets/16189074-f05b-4d24-b432-0736b0ab8421" width="30%" />

- QQ 群

<br><img src="https://github.com/ErKeLost/react/assets/66500121/a9312045-584b-44fa-87b3-2827372ae849" width="30%" />

## 感谢贡献者们做出的努力

<a href="https://github.com/farm-fe/farm/graphs/contributors" target="_blank">
  <table>
    <tr>
      <th colspan="2">
        <br/>
        <img src="https://contrib.rocks/image?repo=farm-fe/farm"><br/><br/>
      </th>
    </tr>
    <tr>
      <td>
        <picture>
          <source
            media="(prefers-color-scheme: dark)"
            srcset="https://next.ossinsight.io/widgets/official/compose-org-active-contributors/thumbnail.png?activity=active&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=2x3&color_scheme=dark"
          />
          <img
            alt="Contributors of farm-fe/farm"
            src="https://next.ossinsight.io/widgets/official/compose-org-active-contributors/thumbnail.png?activity=active&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=2x3&color_scheme=light"
          />
        </picture>
      </td>
      <td rowspan="2">
       <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://next.ossinsight.io/widgets/official/compose-org-participants-growth/thumbnail.png?activity=new&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=4x7&color_scheme=dark">
        <img alt="New trends of farm-fe" src="https://next.ossinsight.io/widgets/official/compose-org-participants-growth/thumbnail.png?activity=new&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=4x7&color_scheme=light">
      </picture>
      </td>
    </tr>
    <tr>
      <td>
        <picture>
          <source
            media="(prefers-color-scheme: dark)"
            srcset="https://next.ossinsight.io/widgets/official/compose-org-active-contributors/thumbnail.png?activity=new&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=2x3&color_scheme=dark"
          />
          <img
            alt="Contributors of farm-fe/farm"
            src="https://next.ossinsight.io/widgets/official/compose-org-active-contributors/thumbnail.png?activity=new&period=past_28_days&owner_id=108205785&repo_ids=507542208&image_size=2x3&color_scheme=light"
          />
        </picture>
      </td>
    </tr>
  </table>
</a>

## 致谢

非常感谢  https://github.com/tmm 给 Farm 团队捐献了 [farm](https://www.npmjs.com/package/farm) npm 包。

感谢:

- [SWC](https://github.com/swc-project/swc) 项目由[@kdy1](https://github.com/kdy1)创建, 为 Farm 的代码解析、转换和压缩提供了支持。

- [NAPI-RS](https://github.com/napi-rs/napi-rs) 项目由 [@Brooooooklyn](https://github.com/Brooooooklyn)创建, 为 Farm 的 node-binding 实现提供了支持。

- [Rollup](https://github.com/rollup/rollup) 项目由 [@lukastaegert](https://github.com/lukastaegert)创建, 启发了 Farm 的插件系统的实现。

- [Vite](https://github.com/vitejs/vite) 项目由 [Evan You](https://github.com/yyx990803)创建, 针对 Vite 插件生态的兼容性设计提供了启发。

## 作者 & 维护者

作者：

- [brightwu（吴明亮）](https://github.com/wre232114)，, 曾就职于字节跳动。[推特](https://twitter.com/brightwwu46799)

维护者：

- [ErKeLost](https://github.com/ErKeLost)
- [shulandmimi](https://github.com/shulandmimi)
