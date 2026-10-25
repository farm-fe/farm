# 通用配置 

为 Farm 的 DevServer 和编译器配置共享选项。 例如：

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // All dev server options are under server
  root: process.cwd(),
});
```

类型：
```ts
export interface UserConfig {
  /** 该项目的当前根目录，默认为当前工作目录 */
  root?: string;
  envDir?: string;
  envPrefix?: string | string[];
  /** 该目录下的文件将始终被视为静态资产。 在dev中提供它，并在构建时将其复制到output.path */
  publicDir?: string;
  /** js 插件（这是一个 javascript 对象）和 rust 插件（这是引用 .farm 文件或包的字符串） */
  plugins?: (RustPlugin | JsPlugin | JsPlugin[])[];
  /** vite 插件 */
  vitePlugins?: (object | (() => { vitePlugin: any; filters: string[] }))[];
  // compilation?: Pick<InternalConfig, AvailableUserConfigKeys>;
  // server?: UserServerConfig;
}
```
## root

- **default**: `process.cwd()`

配置项目编译的根目录。 所有相对路径在编译期间都是相对于 `root` 的。

## clearScreen
- **default**: `true`

开始编译时是否清屏。

## envDir
- **default**: `<root>`

配置目录以加载 `.env`、`.env.development`、`.env.Production` 文件。 默认情况下它与 root 相同。

```ts
import { defineConfig } from '@farmfe/core';
import { resolve } from 'path';
export default defineConfig({
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_', 'NEW_'],
  envDir: resolve(process.cwd(), './env'),
});
```
在上面的示例中，将从 `<root>/env` 目录加载 `.env`、`.env.development`、`.env.Production` 文件。

## envPrefix
- **default**: `['FARM_', 'VITE_']`

以 `envPrefix` 开头的环境变量将自动注入 [`define`](/docs/config/compilation-options#define)。

## publicDir
- **default**: `public`

`publicDir` 下的文件将始终被视为静态资源。 在 dev 时可以通过 dev server 直接访问，在构建时会将其复制到 [`output.path`](/docs/config/compilation-options#outputpath)。

例如，您可以将字体等静态资源添加到 `public` 目录，并将它们用作 `/xxx.ttf` 。

## plugins
- **default**: `[]`

配置 Farm 插件。 参考[使用 Farm 插件](/docs/using-plugins#farm-compilation-plugins)

## vitePlugins
- **default**: `[]`

配置 Vite/Rollup/Unplugin 插件。 参见 [使用Vite插件](/docs/using-plugins#using-viterollupunplugin-plugins-in-farm)

## workflow

- **默认值**：`undefined`

框架可配置受信任的本地命令编排模块，协调多次 Farm 编译和静态页面渲染。workflow 属于命令编排，不是 JavaScript 编译插件；源码转换仍由编译器插件负责。

```ts
export default defineConfig({
  workflow: {
    module: './site-workflow.mjs',
    options: { title: '我的文档站' },
  },
  compilation: { input: { index: './index.html' } },
});
```

模块导出 `async function run({ command, config, options, signal })`。Farm 完成配置解析后，在创建编译器或服务器之前分派 `dev`、`build`、`watch`、`preview`。传入的 `config` 为移除了 workflow 的已解析配置，防止编排器再次调用 Farm API 时递归分派自身；编排器错误会传递给调用方。

`module` 支持相对于项目根目录的本地路径、绝对路径、file URL 或本地可解析的包入口。仅使用受信任的项目模块。不配置 workflow 时沿用 Farm 标准命令流程。分派前仍会执行普通配置解析，包括编译入口默认值的检查。


运行时 signal 通过 build(config, { signal }) 传递，不进入序列化配置。build workflow 还会接收 SIGINT/SIGTERM 协作取消，runner 结束后移除监听；runner 负责在安全边界响应并完成已开始的提交或恢复。不会自动中断原生编译。

受信任的框架编排器可使用 @farmfe/core 导出的 OutputDirectoryLock，对绝对路径的普通文件获取操作系统排他锁。对象需保留到 release()，并在 finally 中释放；竞争返回 DOCS_OUTPUT_LOCKED。不要删除或替换锁文件，否则不同进程可能锁住不同文件身份。这个底层桥接不替框架校验输出路径或实现恢复协议。
