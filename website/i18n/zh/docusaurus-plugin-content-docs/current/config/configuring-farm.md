# 配置 Farm

## 配置文件规范
默认情况下，Farm 从项目根目录下的“farm.config.ts|js|mjs”文件中读取配置，示例配置文件：

```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  root: process.cwd(), // // 编译的根目录
  // 编译选项
  compilation: {
    //...
  },
  // 开发服务器选项
  server: {
    hmr: true,
    //...
  },
  // 插件配置
  plugins: [],
});
```

有关配置选项的详细信息，请参阅：
* [`编译器选项`](/docs/config/compilation-options): 配置编译器选项(`compilation`字段)，如`input`、`output`、`css 编译`、`打包配置`等。
* [`开发服务器选项`](/docs/config/dev-server): 配置开发服务器选项（`server`字段），如`port`、`host`、`protocol`等。
* [`共享选项`](/docs/config/shared): 配置共享选项，如 `root`、`env` 等。

:::note
您还可以使用“farm start/build -c my-config.ts”将自定义文件用作配置文件。
:::

## 加载Ts配置文件
Farm 支持开箱即用加载 ts 配置文件，如“farm.config.ts”。 Farm 将首先将“farm.config.ts”及其本地 ts 依赖项打包到“farm-config.xxx.mjs”文件中，然后从磁盘加载它。 由于 Farm 将 `farm.config.ts` 编译为 `mjs` 文件，因此您 **不能** 在 `farm.config.ts` 中使用 `__dirname` 或 `__filename`，请使用 `import.meta.url` 作为替代。

或者您可以使用“farm.config.mjs”或“farm.config.cjs”与“@type”来支持类型，避免打包“farm.config.ts”：

```js title="farm.config.mjs"
/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  // ...
}
```

## 示例
### 输入和输出
```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // compile options
  compilation: {
    input: {
      index: './src/index.html',
      about: './src/about.html',
    },
    output: {
      path: 'build',
      publicPath: process.env.NODE_ENV === 'production' ? 'https://my-cdn.com' : '/'
    }
  },
});
```

In above example, we configured `./src/index.html` and `./src/about.html` as input, then output the compiled resources to `build` dir.

### 开发服务器端口

```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  server: {
    port: 9801
  }
});
```

### 仅用默认优化策略
```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // compile options
  compilation: {
    lazyCompilation: false,
    persistentCache: false,
    minify: false,
    treeShake: false
  },
});
```

