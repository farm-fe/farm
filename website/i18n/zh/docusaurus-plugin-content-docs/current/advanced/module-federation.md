---
title: Module Federation（实验性）
description: 使用 Farm 原生 Module Federation 插件配置跨应用加载、共享 React 和远端资源。
---

# Module Federation（实验性）

:::warning 本地实验实现
`@farmfe/plugin-module-federation` 是尚未发布的私有 workspace 包，没有可安装的正式版本号。需要使用包含插件、配套核心改动和示例的源码工作区；旧版已发布 Farm core 不包含异步 CommonJS 修复。本文描述已验证的本地实现，不代表完整 Federation 协议兼容性。
:::

## 实现方式

Rust 编译插件将 remote/shared 请求纳入 Farm 模块图并生成容器；薄适配层连接 Farm 模块系统和官方 Module Federation runtime 2.9.2，由官方处理加载与共享版本选择。业务模块注册、缓存、执行、动态 JS 分包和 CSS 仍由 Farm 管理。适配层通过模块图加载，不依赖异步 bootstrap 钩子。

默认容器是普通 script，在独立全局名称上暴露标准 `get`/`init`，不需要浏览器原生 ESM 来加载远端入口。可选 ESM 容器保留相同标准接口，其他构建器不必理解 Farm 模块 ID。

## 运行已验证示例

按[参与贡献](../contribution.mdx)构建匹配的源码包，再执行：

```bash
# From the repository root, with the matching core JS/native packages built:
pnpm install --frozen-lockfile
node rust-plugins/module-federation/scripts/build-native.mjs
pnpm exec playwright install chromium
pnpm run test-e2e -- --example module-federation-native-probe
```

原生构建命令将当前机器的动态库放入 `npm/<platform>-<arch>/index.farm`，可追加 `--target-dir` 等 Cargo 参数；它不是交叉编译安装器。当前执行验收在 macOS arm64 上完成，其他系统需要自行构建并验收。在 cows 工作区中，pnpm、Cargo 和浏览器缓存应继续使用 overlay。

示例位于 `examples/module-federation-native-probe`。其 `config.mjs` 定义 host 4100、remote A 4101 的 `/deploy/a/` 和 remote B 4102 的 `/deploy/b/`。README 记录了原生产物前置条件、保留端口、日志和清理方式。准备好产物后运行 `pnpm --filter @farmfe-examples/module-federation-native-probe start`，浏览器打开 `http://127.0.0.1:4100`。

仓库 E2E 命令会自行构建并启动独立生产静态站点和真实开发服务器，渲染远端 React hooks 组件，验证 singleton 身份、资源、失败后刷新恢复以及 Rspack/Webpack 矩阵。它设置了超时并清理自己启动的进程，无需手动提前启动服务。先安装 Chromium，或将 `PLAYWRIGHT_BROWSERS_PATH` 指向已有的匹配版本。

## 最小 host 与 remote

把下列文件放在同一个本地 workspace 示例包中；示例包依赖 `@farmfe/core`、`@farmfe/cli`、`@farmfe/plugin-module-federation` 的 `workspace:*`，以及 **18.3.1** 的 React/ReactDOM。原生插件自身依赖固定版本的官方 runtime。`shared.mjs` 与 `host/`、`remote/` 同级。示例直接调用 JSX runtime，无需额外 JSX 转换插件。

```js title="shared.mjs"
import { fileURLToPath } from 'node:url';

export const shared = Object.fromEntries(
  ['react', 'react/jsx-runtime'].map((key) => [key, {
    version: '18.3.1',
    requiredVersion: '^18.3.1',
    singleton: true,
    strictVersion: true,
    fallback: fileURLToPath(import.meta.resolve(key)),
  }]),
);
```

```js title="remote/farm.config.mjs"
import { fileURLToPath } from 'node:url';
import federation from '@farmfe/plugin-module-federation';
import { shared } from '../shared.mjs';

export default {
  root: fileURLToPath(new URL('.', import.meta.url)),
  server: { host: '127.0.0.1', port: 4101, strictPort: true,
    origin: 'http://127.0.0.1:4101', cors: true, hmr: false },
  compilation: {
    input: { remoteEntry: 'farm-mf-native:container' },
    output: { targetEnv: 'browser-esnext', publicPath: '/deploy/a/' },
    sourcemap: false,
    lazyCompilation: false,
    css: { transformToScript: false },
  },
  plugins: [federation({
    name: 'providerA',
    globalName: 'remoteA',
    publicPath: 'http://127.0.0.1:4101/deploy/a/',
    format: 'global',
    filename: 'remoteEntry.js',
    exposes: { './Button': './Button.js' },
    shared,
  })],
};
```

```js title="remote/Button.js"
import react from 'react';
import { jsx } from 'react/jsx-runtime';
import './button.css';

export const loadLabel = async () => (await import('./lazy.js')).label;

export function Counter() {
  const [label, setLabel] = react.useState('Remote');
  return jsx('button', {
    className: 'remote-button',
    onClick: async () => setLabel(await loadLabel()),
    children: label,
  });
}
```

```css title="remote/button.css"
.remote-button { color: rgb(17, 85, 153); }
```

```js title="remote/lazy.js"
export const label = 'Loaded from the remote chunk';
```

```js title="host/farm.config.mjs"
import { fileURLToPath } from 'node:url';
import federation from '@farmfe/plugin-module-federation';
import { shared } from '../shared.mjs';

export default {
  root: fileURLToPath(new URL('.', import.meta.url)),
  server: { host: '127.0.0.1', port: 4100, strictPort: true, hmr: false },
  compilation: {
    input: { index: './index.html' },
    output: { targetEnv: 'browser-esnext' },
    sourcemap: false,
    lazyCompilation: false,
  },
  plugins: [federation({
    name: 'consumer',
    remotes: {
      remote: { name: 'providerA', globalName: 'remoteA', format: 'global',
        entry: 'http://127.0.0.1:4101/deploy/a/remoteEntry.js' },
    },
    shared,
  })],
};
```

```html title="host/index.html"
<!doctype html>
<div id="app"></div>
<script type="module" src="./main.js"></script>
```

```js title="host/main.js"
import { createRoot } from 'react-dom/client';
import { jsx } from 'react/jsx-runtime';
import { Counter } from 'remote/Button';

createRoot(document.querySelector('#app')).render(jsx(Counter, {}));

const remote = await import('remote/Button');
console.log(await remote.loadLabel());
```

在示例根目录的两个终端分别执行 `pnpm exec farm start --config remote/farm.config.mjs` 和 `pnpm exec farm start --config host/farm.config.mjs`。remote 必须可访问，host 才能完成静态远端导入。生产模式使用相应的 `farm build --config ...` 命令，将两个应用各自的 `dist` 部署到所配置的来源和子路径。正确设置 CORS 与 MIME 类型，尤其是 ESM 入口。自动化示例使用明确的 loopback 绑定和受控静态服务器。

## 配置字段

| 选项 | 默认值 | 含义 |
| --- | --- | --- |
| `name` | 必填 | 唯一应用名，须为 JavaScript 标识符；Farm namespace 为 `farm_mf_<name>`。 |
| `hot` | `false` | 显式开启开发态 Farm↔Farm HMR，消费者须声明接受边界。 |
| `globalName` | `name` | 容器全局名称，须为 JavaScript 标识符。 |
| `format` | `'global'` | 生产者容器格式：`'global'` 或 `'esm'`。 |
| `filename` | `'remoteEntry.js'` | 不含目录分隔符的 `.js` 文件名。 |
| `publicPath` | `compilation.output.publicPath` | remote JS/CSS 的绝对 HTTP(S) 基址，包含部署子路径与末尾斜杠；有 exposes 时必须得到绝对地址。 |
| `exposes` | `{}` | 以 `./` 开头的公开模块名到源码路径的映射，源码路径相对应用根目录解析。 |
| `remotes` | `{}` | 导入别名到远端描述对象的映射，见下文。 |
| `shared` | `{}` | 精确包请求到共享描述对象的映射，见下文。 |

远端描述对象必填 `name`（生产者身份）和 `entry`（HTTP(S) 入口 URL）；`globalName` 默认等于该对象的 `name`，`format` 默认 `'global'`。外层键才是导入别名：`remote` 将 `remote/Button` 映射到生产者的 `./Button`。别名与名称应明确区分，避免官方 runtime 中别名 `remote` 与另一个名称 `remoteB` 这样的前缀冲突。

共享描述对象必填 `version`、`requiredVersion`、`fallback`（本地模块绝对路径）。`singleton` 和 `strictVersion` 默认均为 `false`。目前不会自动推断依赖版本，不支持通配符子路径或 `fallback: false`；`react/jsx-runtime` 要独立于 `react` 配置。包装器会填写 `runtimePath`，应用无需设置。Rust 配置会拒绝未知字段。

当 `exposes` 非空时，插件目前执行专门的容器构建：替换 input 映射，以 `filename` 作为入口名。上面的虚拟 input 可避免 JavaScript 配置归一化阶段要求 `index.html`。如果 remote 同时需要普通应用页面，请使用独立构建。插件仅支持浏览器目标。

## 共享 React 与初始化

host 和 remote 配置相同的 React 根包与 JSX 子路径。host 的 ReactDOM 会实际渲染远端组件；`singleton: true` 避免选择第二个 React 实例，`strictVersion: true` 拒绝不满足要求的已加载提供者。官方 runtime 根据 `requiredVersion` 协商版本；`fallback` 是没有合适提供者时实际加载的本地包，不是只注册一份配置。

当前验证的是 **loaded-first** 策略与单一 share scope。容器先接入 host scope，再提供模块；同一 scope 的并发 init 共用 promise，不同 scope 会报错。静态 remote 导入和 shared 代理属于异步图节点，依赖它们的应用代码会等待就绪。不同 Farm runtime 创建的 namespace 包装对象不一定相同，验证 singleton 时应比较实际 React 默认导出或 `jsx` 函数。

## CSS、分包与部署路径

remote CSS 和动态 JS 由 remote 自身隔离的 Farm runtime 加载，不能按 host 地址解析。各应用（包括独立部署）必须使用唯一 `name`，`publicPath` 必须对应远端实际资源地址。开发时相对的 `compilation.output.publicPath` 用于服务路由，`server.origin` 提供来源，插件的绝对 `publicPath` 初始化远端资源加载。生产部署也必须在该绝对地址提供全部分包与 CSS，而不只是 `remoteEntry.js`。

Federation 需要显式启用，但配套的编译器修改也用于非 Federation 构建：异步 CommonJS 求值保留源码顺序，注入 CSS 的样式标识和缓存键包含 runtime namespace。懒编译必须只匹配当前 namespace 的样式，再按原始模块 ID 失效缓存。公开的编译器/插件接口和 HMR runtime API 没有改变。下方 Federation 浏览器矩阵关闭了 `lazyCompilation`，不能据此声称所有 Federation 与懒编译组合都已验证。

未开启 `hot: true` 时，远端修改仍采用整页刷新。显式开启的跨应用 HMR 见下节。静态 remote 加载失败会阻止应用启动；恢复入口后重载整个文档是已验证的恢复边界，不承诺同页面自动重试。

## 显式开启跨应用 HMR

在 host 和 remote 的 Federation 配置中设置 `hot: true`，并开启各应用原有的 Farm HMR 服务。remote 必须配置自己的 HMR host/port，并通过 `server.allowedHosts` 允许准确的 host 页面来源。例如 remote 使用 4101、host 使用 4100：

```js
server: {
  host: '127.0.0.1',
  port: 4101,
  hmr: { host: '127.0.0.1', port: 4101 },
  allowedHosts: ['http://127.0.0.1:4100'],
},
compilation: { css: { transformToScript: true } },
// 在两端现有 federation({...}) 配置中加入 hot: true。
```

host 需要显式接受每个使用中的 remote 边界：

```js
import { bridge } from 'farm:module-federation';
const remote = await import('remote/Button');
const unsubscribe = bridge.acceptRemote('remote/Button', updated => {
  // 用新导出更新应用自己的稳定渲染边界。
  renderRemote(updated);
});
import.meta.hot?.dispose(unsubscribe);
```

Rust 为 exposes 生成接受边界；remote 的 Farm HMR 客户端负责更新自身模块缓存。薄适配层更新 host 缓存代理背后的稳定 namespace，并等待消费者回调完成。共享协商仍由官方 MF runtime 处理。CSS 的 DOM 标识加入 runtime namespace，避免两个 remote 的同名 `button.css` 互相覆盖。热更新 remote 开发构建若显式设置 `transformToScript: false`，会给出编译诊断。

Mac 真实浏览器 E2E 已覆盖 global/ESM：代码与 CSS 更新、双 remote 同时更新、React singleton 一致、host 与 remote 状态保留、编译错误、执行失败回滚及修复。文档 token 与导航次数证明这些更新没有刷新 host。React 夹具采用显式稳定组件边界，并保持 Hooks 结构不变；这不是通用 React Fast Refresh/signature 实现。消费者回调必须负责自身渲染和恢复，任意外部副作用不会自动回滚。

浏览器 `farm:mf:hot-status` 事件报告 `applied`、`compile-error`、`rolled-back`、`reload-required`。导出形态不兼容或缺少消费者接受边界，不会被静默当作成功；兼容源码修复可以在回滚后恢复，否则需主动刷新。remote 连接丢失后，沿用 Farm 的重连与整页刷新策略，避免漏掉更新；该降级会重置状态，测试单独断言，并验证恢复后的下一次修改仍可无刷新热更新。共享库或配置变更应整页刷新。

目前 HMR 范围为具有显式接受边界的 Farm↔Farm。下方 Rspack/Webpack 矩阵验证加载，不代表跨构建器 HMR。该路径复用现有 Farm 开发 HMR 传输与 CSS 注入；生产环境严格 CSP 验收不代表 HMR 已兼容同样的 CSP。本轮未放宽浏览器或系统安全策略。生产模式始终关闭 `hot`。

## global 与 ESM 互通

启用 ESM 时，同时把生产者 `format` 和消费者远端描述对象的 `format` 改为 `'esm'`，不能混淆 script 入口与 ESM 入口。当前 ESM 输出除导出 `get`/`init` 外，也会发布命名全局对象。global 输出在执行前检查容器名称和 Farm namespace 冲突；ESM 的依赖可能先于容器检查执行，因此仍须确保名称唯一。

| 生产者 → 消费者 | global/script | ESM |
| --- | --- | --- |
| Farm → Farm（生产与开发） | 已验证 | 已验证 |
| Rspack → Farm / Farm → Rspack | 已验证 | 已验证 |
| Webpack → Farm / Farm → Webpack | 已验证 | 已验证 |

矩阵采用真实浏览器，包含远端组件渲染、React/JSX 引用一致、CSS 与动态分包。版本为 React/ReactDOM 18.3.1、MF runtime/enhanced 2.9.2、Rspack 2.3.0、Webpack 5.112.2、Playwright Chromium 1.55.0；Webpack CSS 使用 css-loader 7.1.5 和 mini-css-extract-plugin 2.10.2。生产测试的 CSP 不允许 `unsafe-eval` 或 `unsafe-inline`。

Rspack 使用 MF enhanced 适配器。Webpack 使用 **Webpack 内置 ModuleFederationPlugin**：enhanced 2.9.2 引用了 Webpack 5.112.2 已不存在的 `webpack/lib/util/extractUrlAndGlobal`，因此该 enhanced/Webpack 组合不属于通过范围。script remotes 使用 `globalName@URL`；module externals 使用纯 URL，并要求支持动态 import 的编译目标（测试为 web + ES2020）。准确配置见本地示例 `interop-build.mjs`。同一应用混用不同格式的多个 remote 尚未独立验收。

## 错误排查

| 现象 | 检查项 |
| --- | --- |
| 缺少 `index.farm` 或 core 不匹配 | 构建当前平台插件与匹配的 workspace core，不要替换为不相关二进制。 |
| remote 构建要求 `index.html` | 设置 `input: { remoteEntry: 'farm-mf-native:container' }`。 |
| 找不到远端或 expose | 核对 alias/name/entry/globalName，区分 `./Button` 与 `remote/Button`。 |
| 入口成功但 CSS/分包失败 | 检查 remote publicPath、部署子路径、CORS、资源状态和 MIME 类型。 |
| 容器全局名称或 namespace 冲突 | 使用唯一名称；global 容器拒绝覆盖既有 runtime。 |
| 共享版本错误或 invalid hook call | 检查 version/requiredVersion，两端均配置 singleton，精确共享 React 与 JSX 请求。 |
| Different share scope | 复用同一 scope 对象；不支持通过另一个 scope 重新初始化。 |
| 同步 require 异步模块的诊断 | 使用明确的异步 import 边界，不要通过引入第二份未共享 React 掩盖问题。 |

## 当前限制

global 入口组装要求 `sourcemap: false`，只接受生成的 eager side-effect import 形态，并会保留冗余的 eager 资源文件；完整 global source map 与入口产物优化尚未完成。不支持 SSR、manifest、类型生成、自动跨框架 HMR、多 share scope、version-first 策略、eager shares 或通配符共享子路径。

普通同步 CJS 循环与延迟 remote A→B→A 已测试。静态异步 CJS 循环、普通同步函数或 class initializer 内 require 异步依赖会产生编译诊断，不支持任意异步循环求值。套件同时保留非 MF 的 CJS/ESM 回归。底层概念可参考 [runtime 插件](../api/runtime-plugin-api.md)和[开发服务器配置](../config/dev-server.md)。
