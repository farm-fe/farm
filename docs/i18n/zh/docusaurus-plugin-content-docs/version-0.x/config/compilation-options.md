# 配置参考

Farm 默认从项目根目录的 `farm.config.ts|js|mjs` 文件中读取配置，配置文件示例:

```ts title="farm.config.ts"
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  root: process.cwd(), // 编译的根目录
  // 编译选项
  compilation: {
    // ...
  },
  // Dev Server 选项
  server: {
    hmr: true,
    // ...
  },
  // 插件配置
  plugins: [],
});
```

## 编译选项 - compilation

所有与编译相关的配置都在 `compilation` 字段下。

### input

- **type**: `Record<string, string>`

项目的入口点。 Input 的文件可以是`html`、`ts/js/tsx/jsx`、`css` 或通过插件支持的其他文件。

```tsx
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
      about: "./about.html",
    },
  },
  // ..
};
```

### output

- **type**: `OutputOptions`

```ts
interface OutputOptions {
  // 局部打包后，入口文件所在资源的文件名配置
  entryFilename?: string;
  // 局部打包后，除入口资源外的其他资源输入文件名配置
  filename?: string;
  // 输入目录
  path?: string;
  // public path：资源加载前缀
  publicPath?: string;
  // 静态资源文件名配置
  assetsFilename?: string;
  // 目标执行环境，浏览器或者 Node
  targetEnv?: "browser" | "node";
  // 输出模块格式
  format?: "cjs" | "esm";
}
```

:::note
我们称编译结果为 `资源(resource)`
:::

#### `output.entryFilename`

- **默认值**: `"[entryName].[ext]"`

配置入口资源的文件名：您可以使用 `[entryName]` 等占位符。 所有占位符如下：

- `[entryName]`：入口名，例如对于 `input: { index: "./index.html" }`，`[entryName]` 为 `index`
- `[resourceName]`：资源的名称，一般是 Farm 内部生成的一个独特哈希值。
- `[contentHash]`：该资源的内容哈希。
- `[ext]`：该资源的扩展名，对于 `js/jsx/ts/tsx` 为 `js`，对于 `css/scss/less` 为 `css`。

#### `output.filename`

- **默认值**: `"[resourceName].[ext]"`

局部打包后，除 `entryFilename` 配置的资源外的其他资源文件名. 所有占位符如下：

- `[resourceName]`：资源的名称，一般是 Farm 内部生成的一个独特哈希值。
- `[contentHash]`：该资源的内容哈希。
- `[ext]`：该资源的扩展名，对于 `js/jsx/ts/tsx` 为 `js`，对于 `css/scss/less` 为 `css`。

#### `output.path`

- **默认值**: `"dist"`

输出资源的目录

#### `output.publicPath`

- **默认值**: `"/"`

资源 url 加载的前缀. 比如 URL `https://xxxx`，或者一个路径 `/xxx`.

#### `output.assetsFileName`

- **默认值**: `"[resourceName].[ext]"`

静态资源输出的文件名配置，占位符和 `output.filename` 相同。

#### `output.targetEnv`

- **默认值**: `"browser"`

配置产物的执行环境，可以是 `"browser"` 或者 `"node"`.

#### `output.format`

- **默认值**: `"esm"`

配置产物的格式，可以是 `"esm"` 或者 `"cjs"`.

:::note
该选项只对 Js 产物有效
:::

### resolve

- **type**: `ResolveOptions`

```ts
interface ResolveOptions {
  extensions?: string[];
  alias?: Record<string, string>;
  mainFields?: string[];
  conditions?: string[];
  symlinks?: boolean;
  strictExports?: boolean;
}
```

#### `resolve.extensions`

- **默认值**: `["tsx", "ts", "jsx", "js", "mjs", "json", "html", "css"]`

配置解析依赖时的后缀，例如解析 `./index` 时，如果没有解析到，则会自动加上后缀解析，如尝试 `./index.tsx`, `./index.css` 等。

#### `resolve.alias`

- **默认值**: `{}`

配置解析别名，示例：

```ts
export default defineConfig({
  compilation: {
    resolve: {
      alias: {
        "/@": path.join(process.cwd(), "src"),
        stream$: "readable-stream",
         "$__farm_regex:^/(utils)$": path.join(process.cwd(), "src/$1"),
      },
    },
  },
});
```

alias 为前缀替换，对于上述例子 `/@/pages` 将会被替换为，`/root/src/pages`。

如果希望精确匹配，可以加上 `$`，例如 `stream$` 只会替换 `stream`，而不会替换 `stream/xxx`。

当然也支持使用正则表达式，例如 `$__farm_regex:^/(utils)$`，将会匹配 `/utils`，并替换为 `/root/src/utils`。
#### `resolve.mainFields`

- **默认值**: `["exports", "browser", "module", "main"]`

解析 node_modules 下依赖时，从 package.json 中将会按照 `mainFields` 中配置的字段和顺序进行解析。对于 `package.json`

```json
{
  "name": "package-a",
  "module": "es/index.js",
  "main": "lib/index.js"
}
```

将会优先使用 `es/index.js`（如果路径存在），不存在则会继续向后搜索。

#### `resolve.conditions`

暂不支持配置。

#### `resolve.symlinks`

- **默认值**: `true`

解析文件时，是否追踪 symlink 对应的真实目录，并从真实目录开始解析下一个依赖。如果使用 pnpm 管理依赖，该选项必须配置为 true。

#### `resolve.strictExports`

- **默认值**: `false`

是否严格遵循 `package.json` 中 `exports` 中定义的导出。如果设置为 true，当 `package.json` 中定义了 `exports`，但是 `exports` 没有定义对应导出时，会直接报错。如果设置为 true，会按照 mainFields 继续尝试其他入口。

### define

- **默认值**: `{}`

全局变量注入，配置的变量名和值将会在编译时注入到产物中。Farm 默认注入 `process.env.NODE_ENV` 以及部分 Farm 自身使用的变量比如 `FARM_HMR_PORT`

```ts
export default defineConfig({
  compilation: {
    define: {
      MY_VAR: 123,
    },
  },
});
```

:::note

1. define 为了强化性能，使用的是全局变量的注入形式，这意味着，对象形式的变量无法注入，例如 `process.env.XXX` 形式的变量无法注入，只能配置 `XXX` 形式的变量。
2. 如果在同一个 window 下挂载多个 Farm 项目，多个项目同名的 define 会相互覆盖。
3. 注入的是字符串，如果需要转为其他类型，需要在运行时代码中手动转换，例如 `Number(MY_VAR)`
   :::

### external

- **默认值**: `[]`

配置被 external 的导入，被 external 的导入不会出现在编译产物中。但是对应 import 语句不会删除，需要自定义 external 后如何处理，否则运行时会报错，对于 targetEnv 是 node 下的 external 模块，会自动尝试 require 该模块。

需要使用正则方式配置，例如：

```ts
export default defineConfig({
  compilation: {
    external: ["^stream$"],
  },
});
```

### mode

- **默认值**: 对于 start、watch 命令是 `development`，对于 build 命令是 `production`

配置编译模式，为了优化开发时性能，在没有手动配置生产优化相关选项（minify，tree shake 等）时，默认在 `development` 下会禁用生产环境优化比如压缩和 tree shake，在 `production` 模式下启用。

### root

- **默认值**: `process.cwd()`

配置项目编译的 root 目录，该选项会影响默认配置文件的查找路径，编译模块依赖的查找等。

### runtime

配置 Farm 运行时能力。类型如下：

```ts
interface FarmRuntimeOptions {
  runtime?: {
    path: string;
    plugins?: string[];
    namespace?: string;
  };
}
```

#### `runtime.path`

- **默认值**: Farm 内置 runtime 的路径

自定义一个 Runtime 替换 Farm 内置的 Runtime。

:::warning
正常情况下不建议配置该选项，因为一旦配置了该选项，指向的 runtime 需要所有实现 Farm Runtime 已有的能力，例如模块系统、HMR、动态资源加载等。
:::

#### `runtime.plugins`

- **默认值**: Farm 内置 runtime-plugin-hmr 的路径

配置 Runtime 插件，通过 Runtime 插件，可以干预 Runtime 行为，如模块加载，资源加载等。具体可以参考：WIP。

#### `runtime.namespace`

- **默认值**: 项目 package.json 的 name 字段

配置 Farm Runtime 的命名空间，保证在同一个 window 或者 global 下不同产物的执行能够相互隔离。默认使用项目 package.json 的 name 字段作为 namespace。

### assets

#### `assets.include`

- **默认值**: `[]`

额外视为静态资源的文件后缀，例如下述示例，`txt` 将会被视为姿态资源，引入 txt 文件时当作静态资源处理：

```ts
export default defineConfig({
  compilation: {
    assets: {
      include: ["txt"],
    },
  },
});
```

### script

#### `script.target`

- **默认值**: `esnext`（根据 Farm 的迭代动态调整）

配置 Farm 解析 `js/jsx/ts/tsx` 的 AST 以及生成代码时支持的 ES 语法版本。 可选值：`es5`, `es6`, `es2015` - `es2023`, `esnext`

#### `script.parser`

- **默认值**: 与 SWC 相同

配置 SWC 解析 AST 时的行为，配置项参考：https://swc.rs/docs/configuration/compilation#jscparser

#### `script.plugins`

- **默认值**: `[]`

配置 swc 插件数组，数组每一项包含三个字段：

- **name**：swc 插件的包名
- **options**: 传给 swc 插件的配置项
- **filters**: 对哪些模块执行该插件，必须配置，支持 `resolvedPaths` 和 `moduleTypes` 这两个过滤项，两者如果同时指定，取并集。

对于 Vue 项目支持 JSX 的配置示例如下：

```ts
import jsPluginVue from "@farmfe/js-plugin-vue";

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    script: {
      plugins: [
        {
          name: "swc-plugin-vue-jsx",
          options: {
            transformOn: true,
            optimize: true,
          },
          filters: {
            // resolvedPaths: [".+"]
            moduleTypes: ["tsx", "jsx"],
          },
        },
      ],
    },
  },
  plugins: [jsPluginVue()],
};
```

#### `script.decorators`

```ts
export interface DecoratorsConfig {
  legacyDecorator: boolean;
  decoratorMetadata: boolean;
  /**
   * 装饰器版本： 2021-12 或者 2022-03
   * @default 2021-12
   */
  decoratorVersion: "2021-12" | "2022-03" | null;
  /**
   * @default []
   */
  includes: string[];
  /**
   * @default ["node_modules/"]
   */
  excludes: string[];
}
```

建议使用 Farm 默认的装饰器配置，除非你想提高性能，可以设置`includes`和`excludes`。

选项：

- **legacyDecorator**：默认为`true`。使用遗留装饰器提案。
- **decoratorMetadata**：默认为`false`。如果您想将`legacyDecorator`设置为`true`，则必须将其设置为`false`。
- **decoratorVersion**：默认为`2021-12`，提案版本。该值为 2021-12 或 2022-03。
- **包括**：默认为`[]`。如果要包含排除的模块，可以设置此选项。支持正则表达式。
- **排除**：默认为`['node_modules/']`。变换装饰器时，这些路径下的模块将被忽略。支持正则表达式

### css

#### `css.modules`

配置 Farm CSS Modules。

```ts
interface FarmCssModulesConfig {
  // 配置哪些路径会被处理为 css modules，使用正则字符串
  // 默认为 `.module.css` 或者 `.module.scss` 或者 `.module.less`
  paths?: string[];
  // 配置生成的 css 类名，默认为 `[name]-[hash]`
  indentName?: string;
}
```

##### `css.modules.paths`

- **默认值**: `["\\.module\\.(css|scss|sass|less)"]`

配置哪些路径对应的模块会被视为 CSS Modules。需要配置正则字符串。默认是以 `.module.(css|scss|sass|less)` 结尾的文件。

##### `css.modules.identName`

- **默认值**: `[name]-[hash]`

配置生成的 CSS Modules 类名，默认是 `[name]-[hash]`，`[name]`, `[hash]` 为占位符（也是目前支持的所有占位符）。`[name]` 表示原始类名，`[hash]` 表示改 css 文件 id 的 hash。

#### `css.prefixer`

配置 CSS 的兼容性前缀，例如 `-webkit-`。

```ts
interface FarmCssPrefixer {
  targets?: string[] | string | BrowserTargetsRecord;
}

type BrowserTargetsRecord = Partial<
  Record<
    | "chrome"
    | "opera"
    | "edge"
    | "firefox"
    | "safari"
    | "ie"
    | "ios"
    | "android"
    | "node"
    | "electron",
    string
  >
> & { [key: string]: string };
```

##### `css.prefixer.targets`

- **默认值**: `undefined`

配置对于哪些目标浏览器或者浏览器版本开启，示例：

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    css: {
      prefixer: {
        targets: ["last 2 versions", "Firefox ESR", "> 1%", "ie >= 11"],
      },
    },
  },
});
```

### html

#### `html.base`

- **默认值**: `undefined`

所有的 HTML 入口会继承 `html.base`，详情参考 [指南 - HTML](/docs/features/html)

### sourcemap

- **默认值**: `true`

配置是否启用 sourcemap，可选配置项及说明如下：

- **`true`**：仅对非 `node_modules` 下的文件生成 sourcemap，并且生成单独的 sourcemap 文件
- **`false`**: 关闭 sourcemap
- **`inline`**：仅对非 `node_modules` 下的文件生成 sourcemap，并且内联 sourcemap 到产物中，不生成单独的文件
- **`all`**：对所有文件生成 sourcemap，并且生成单独的 sourcemap 文件
- **`all-inline`**: 对所有的文件生成 sourcemap，并且内联 sourcemap 到产物中，不生成单独的文件

### partialBundling

配置 Farm 局部打包的行为，详情可以参考 [局部打包](/docs/features/partial-bundling)

```ts
export interface FarmPartialBundlingConfig {
  targetConcurrentRequests?: number;
  targetMinSize?: number;
  targetMaxSize?: number;
  groups?: {
    name: string;
    test: string[];
    groupType?: "mutable" | "immutable";
    resourceType?: "all" | "initial" | "async";
  }[];
  enforceResources?: {
    name: string;
    test: string[];
  }[];
  enforceTargetConcurrentRequests?: boolean;
  enforceTargetMinSize?: boolean;
  immutableModules?: string[];
}
```

#### `partialBundling.targetConcurrentRequests`

- **default**: `25`

Farm 尝试生成尽可能接近此配置值的资源数量，控制初始资源加载或动态资源加载的并发请求数量。

#### `partialBundling.targetMinSize`

- **default**: `20 * 1024` bytes, 20 KB

minify 和 gzip 之前生成的资源的最小大小。 请注意，`targetMinSize` 并不一定保证满足，可以配置`enforceTargetMinSize`可用于强制限制最小的大小。

#### `partialBundling.targetMaxSize`

- **default**: `1500 * 1024` bytes, 1500 KB

minify 和 gzip 之前生成的资源的最大大小。

#### `partialBundling.groups`

- **default**: `[]`

一组应该放在一起的模块。 请注意，此组配置只是对编译器的打击，即这些模块应该放置在一起，它可能会产生多个资源，如果您想强制打包模块到同一个资源中，使用`enforceResources`。

数组每一项的配置选项如下:

- **name**: 该组的名称。
- **test**: 匹配该组中的模块路径的正则表达式数组。.
- **groupType**: `mutable` 或 `immutable`，限制该组仅适用于指定类型的模块。
- **resourceType**: `all`、`initial` 或 `async`，限制该组仅适用于指定类型的资源。

```ts title="farm.config.ts" {4-9}
export default defineConfig({
  compilation: {
    partialBundling: {
      groups: [
        {
          name: "vendor-react",
          test: ["node_modules/"],
        },
      ],
    },
  },
});
```

#### `partialBundling.enforceResources`

- **default**: `[]`

Array to match the modules that should always be in the same output resource, ignore all other constraints.

Options for each item:

- **name**: Name of this resource.
- **test**: Regex array to match the modules which are in this resource.

```ts title="farm.config.ts" {4-9}
export default defineConfig({
  compilation: {
    partialBundling: {
      enforceResources: [
        {
          name: "index",
          test: [".+"],
        },
      ],
    },
  },
});
```

:::warning
`enforceResources` will ignore all Farm's internal optimization, be careful when you use it.
:::

#### `partialBundling.enforceTargetConcurrentRequests`

- **default**: `false`

对每个资源加载强制执行目标并发请求数量，当为 true 时，较小的资源将合并为较大的资源以满足目标并发请求。 这可能会导致 css 资源出现问题，请小心使用此选项

#### `partialBundling.enforceTargetMinSize`

- **default**: `false`

为每个资源强制执行目标最小大小限制，如果为真，较小的资源将合并为较大的资源以满足目标并发请求。 这可能会导致 css 资源出现问题，请小心使用此选项

#### `partialBundling.immutableModules`

- **default**: `['node_modules']`

匹配不可变模块的正则表达式数组

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    partialBundling: {
      immutableModules: ["node_modules/", "/global-constants"],
    },
  },
});
```

不可变模块会影响打包和持久缓存，如果要更改它，请小心。

#### `partialBundling.immutableModulesWeight`

- **default**: `0.8`

Default to `0.8`, immutable module will have 80% request numbers. For example, if `targetConcurrentRequest` is 25, then immutable resources will take `25 * 80% = 20` by default. This option is to make sure that mutable and immutable modules are isolate, if change your business code, code under node_modules won't be affected.

### lazyCompilation

- **默认值**: 在开发模式是 `true`，构建模式是 `false`

是否启用懒编译，配置为 false 关闭。参考 [懒编译](/docs/features/lazy-compilation)。

### treeShaking

- **默认值**: 在开发模式是 `false`，构建模式是 `true`

是否启用 tree shake，配置为 false 关闭。参考 [Tree Shake](/docs/features/tree-shake)。

### minify

- **默认值**: 在开发模式是 `false`，构建模式是 `true`

是否启用压缩，开启后将会对产物进行压缩和混淆。参考 [压缩](/docs/features/tree-shake)。

### presetEnv

- **默认值**: 在开发模式是 `false`，构建模式是 `true`

```ts
type FarmPresetEnvConfig =
  | boolean
  | {
      include?: string[];
      exclude?: string[];
      // TODO using swc's config
      options?: any;
      assumptions?: any;
    };
```

默认不会对 node_modules 下的模块注入 polyfill，如果需要，请使用 `include` 添加 polyfill。

#### `presetEnv.include`

- **默认值**: `[]`

额外包含哪些需要 polyfill 的模块，配置正则字符串，例如 `include: ['node_modules/(es6-package|my-package)/']`

#### `presetEnv.exclude`

- **默认值**: `['node_modules/']`

配置哪些不需要 polyfill 的模块，配置正则字符串，例如 `exclude: ['custom-path/(es5-package|my-package)/']`。默认 node_modules 被排除，如果需要包含被排除的模块，建议使用 `include`

#### `presetEnv.options`

- **默认值**: `降级到 ES5`

传递给 swc preset env 的选项，参考 https://swc.rs/docs/configuration/compilation#env。

### persistentCache

- **default**: `true`

[增量构建](/docs/features/persistent-cache) 的缓存配置选项. 配置成 `false` 来禁用缓存.

```ts
export type PersistentCache =
  | boolean
  | {
      namespace?: string;
      cacheDir?: string;
      buildDependencies?: string[];
      moduleCacheKeyStrategy?: {
        timestamp?: boolean;
        hash?: boolean;
      };
    };
```

#### `persistentCache.namespace`

- **default**: `farm-cache`

缓存的命名空间，不同空间下的缓存会相互隔离，不会复用。

#### `persistentCache.cacheDir`

- **default**: `node_modules/.farm/cache`

缓存文件的存放目录。

#### `persistentCache.buildDependencies`

- **default**: `farm.config.ts and all its deep dependencies`

所有配置文件、插件等构建依赖的路径，默认包含 `farm.config.ts/js/mjs` 的所有依赖以及配置的所有 rust 和 js 插件。如果任意一个构建依赖变更了，所有缓存将会失效。

配置项可以是一个路径或者一个包名, 例如:

```ts
import { defineConfig } from "@farmfe/core";
import path from "node:path";

export default defineConfig({
  persistentCache: {
    buildDependencies: [
      // a file path
      path.resolve(process.cwd(), "./plugins/my-plugin.js"),
      // a package name, note that this package must expose package.json
      "farm-plugin-custom-xxx",
    ],
  },
});
```

#### `persistentCache.moduleCacheKeyStrategy`

- **default**: `{ timestamp: true, hash: true }`

控制复用缓存时，如何生成缓存的键。如果 `timestamp` 被设置为 true，并且模块没有倍改过，那么该模块所有的构建步骤将会被跳过（如`load`, `transform` 等钩子），缓存的模块将会被复用。如果`hash`设置成 true，并且 timestamp 没有命中，那么会调用 `load` 以及 `transform` 钩子来获取模块的内容，如果模块内容没有变更，那么缓存将会被复用，剩余构建步骤会被跳过。

- `timestamp`: 是否检查模块的 timestamp，性能最优，但是如果某些插件依赖前一次的构建状态，可能存在问题，见[注意事项](/docs/features/persistent-cache#caveats-for-plugins).
- `hash`: 是否检查 load 和 transform 后的内容。

#### `persistentCache.envs`

- **default**: [Farm Env](/docs/config/farm-config#environment-variable)

可能影响构建过程的环境变量，如果任意一个环境变化了，缓存将会过期。

<!-- #### `presetEnv.assuptions` -->
