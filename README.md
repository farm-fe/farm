<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="./assets/logo.png" width="550" />
  </a>
  <h3>Extremely fast Vite-compatible web building tool written in Rust</h3>
  <p>
    <span>English</span> |
    <a href="https://github.com/farm-fe/farm/blob/main/README.zh-CN.md">简体中文</a>  
  </p>
  <p align="center">
    <a href="https://discord.gg/mDErq9aFnF">
      <img src="https://img.shields.io/badge/chat-discord-blueviolet?style=flat&logo=discord&colorA=ffe3f5&colorB=711a5f" alt="discord chat" />
    </a>
    <a href="https://twitter.com/fe_farm" > 
      <img src="https://img.shields.io/twitter/url.svg?label=@fe_farm&style=social&url=https://twitter.com/fe_farm"/> 
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

## Intro

Farm is a extremely fast vite-compatible web-building tool written in Rust. It's designed to be fast, powerful and consistent, aims to provide best experience for web development, which is the real next generation build tool.

## Online experience

[![Edit Farm](https://codesandbox.io/static/img/play-codesandbox.svg)](https://codesandbox.io/p/github/ErKeLost/react/main)

## Why Farm?

> See [Why Farm](https://farmfe.org/docs/why-farm) for details.

In short, tools like webpack are too slow, but new tools like Vite are not perfect, Vite has a lot of drawbacks when comes to a large project:

- **A huge number of requests during development**：when there are hundreds or thousands modules per page, loading performance severely degraded, it may takes seconds or more when refresh the page.
- **Inconsistency between development and production**: Using different strategy and tools in development and production, it's really inconsistent and it's hard to debug online issues.
- **Inflexible Code Splitting**: It's hard to control the output of your bundles.

Farm can solve these problems perfectly, and it's really fast cause it's written in Rust. Farm aims to be fast, consistent, flexible, which is the real next generation build tool.

## Features

> [!NOTE]
>
> - Since Farm v0.13, Vite plugins can be used directly in Farm. Refer to [Using vite plugins in Farm](https://farmfe.org/docs/using-plugins#using-vite-plugins-in-farm)
> - Since Farm v0.14, persistent disk cache enabled by default. Refer to [Incremental Building](https://farmfe.org/docs/advanced/persistent-cache)
> - Now Farm is **1.0 stable** and **production ready!**. See [Farm official website](https://farmfe.org/) to get started.

- ⚡ **Extremely Fast**: Written in Rust, start a React / Vue project in milliseconds and perform an HMR update within 20ms for most situations.
- ⚡ **Incremental Building**: Support persistent cache, module level cache enabled by default, any module won't be compiled twice until it's changed!
- 🧰 **Fully Pluggable and Vite Compatible**: Everything inside Farm is powered by plugins, Support Vite Plugins out of box. Supports Farm compilation plugins(both Rust and JavaScript plugins, and SWC plugins), Farm runtime plugins and Farm server plugin.
- ⚙️ **Powerful**: Compiles JS/TS/JSX/TSX, CSS, Css Modules, HTML, and static assets out of the box. Support official compilation plugins for Popular frameworks/tools like React, Vue, SolidJs, Sass, Less, Postcss and so on.
- ⏱️ **Lazy Compilation**: Dynamically imported resources are compiled only when requested, speed up compilation for large scale project. Just write a `dynamic import` and the imported module won't be compiled when it is executed.
- 📦 **Partial Bundling**: Bundle your project into a few reasonable bundles automatically, speeding up resource loading without losing caching granularity. Refer to [RFC-003 Partial Bundling](https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md) for details.
- 🔒 **Consistency**: What you see in development will be the same as what you get in production.
- 🌳 **Compatibility**: Supports both legacy (ES5) and modern browsers.

<br/>

> Farm has implemented all features of a web build tool, including production optimization like tree shake and minification. It's now 1.0 stable. We have already migrated enterprise projects to Farm, and it works great!

See [RFC-001 Architecture](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation) for design motivation and architecture.

<br/>

## Getting Started

Create a new Farm(support both React and Vue) project with your favorite package manager:

```bash
# with npm
npm create farm@latest
# with yarn
yarn create farm@latest
# with pnpm
pnpm create farm@latest
```

Visit [Farm Documentation](https://farmfe.org) to learn more about Farm.

## Benchmark

Farm is much faster than similar tool， **20x** faster than webpack and **10x** faster than Vite in the benchmark:

![benchmark](./assets/benchmark-full.png)

See [Benchmark](https://github.com/farm-fe/performance-compare) for details.

## Contribution

See [Contributing Guide](https://github.com/farm-fe/farm/blob/main/CONTRIBUTING.md).

## Chat With Us

- [Author Twitter](https://twitter.com/brightwwu46799), [Official Twitter](https://twitter.com/fe_farm)

- With [Discord](https://discord.gg/mDErq9aFnF)

- Wechat group

<br><img src="https://github.com/farm-fe/farm/assets/66500121/98227b2b-91d0-4ad0-b46d-96e231a425fe" width="30%" />

## Contributors

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

## Credits

Thanks to:

- The [SWC](https://github.com/swc-project/swc) project created by [@kdy1](https://github.com/kdy1), which powers Farm's code parsing, transformation and minification.

- The [NAPI-RS](https://github.com/napi-rs/napi-rs) project created by [@Brooooooklyn](https://github.com/Brooooooklyn), which powers Farm's node-binding implementation.

- The [Rollup](https://github.com/rollup/rollup) project created by [@lukastaegert](https://github.com/lukastaegert), which inspired Farm's plugin system implementation.

- The [Vite](https://github.com/vitejs/vite) project created by [Evan You](https://github.com/yyx990803), which inspired Farm's compatibility design of ecosystem.

## Author & Maintainer

Author:

- [brightwu（吴明亮）](https://github.com/wre232114)，worked at bytedance. [Twitter](https://twitter.com/brightwwu46799)

Maintainer:

- [ErKeLost](https://github.com/ErKeLost)
- [shulandmimi](https://github.com/shulandmimi)
