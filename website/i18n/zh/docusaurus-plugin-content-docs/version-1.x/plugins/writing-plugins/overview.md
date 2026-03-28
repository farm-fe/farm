
# 概览

Farm 采用完全插件化的形式，提供了多种类型的插件来干预 Farm 的几乎所有行为，Farm 支持的主要插件类型分为以下几类：
* **编译插件**：干预、增强 Farm 的编译能力，支持使用 Rust（推荐）以及 Js 编写插件
* **运行时插件**：干预、增强 Farm 的运行时能力，使用 Js 编写
* **Dev Server 插件**：干预、增强 Farm 的 Dev Server，例如挂载更多变量，注册 middleware 等

To use a Rust plugin, configuring `plugins` in `farm.config.ts`.

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';

defineFarmConfig({
  // ...
  plugins: [
    { /*..*/ }, // Js plugin, a object with hook defined
    '@farmfe/plugin-react', // rust plugin package name
  ]
})

```

Farm support both rust plugins and js plugins:

* [Rust Plugin](/docs/plugins/rust-plugin)
* [Js plugin](/docs/plugins/js-plugin)
