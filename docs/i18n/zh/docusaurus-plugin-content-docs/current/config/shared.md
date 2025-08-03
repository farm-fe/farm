# 通用配置

为 Farm 的 DevServer 和编译器配置共享选项。 例如：

```ts
import { defineConfig } from "farm";

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
import { defineConfig } from "@farmfe/core";
import { resolve } from "path";
export default defineConfig({
  envPrefix: ["FARM_", "CUSTOM_PREFIX_", "NEW_"],
  envDir: resolve(process.cwd(), "./env"),
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
