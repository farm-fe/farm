# Js Plugin Api
Farm Js Plugin 设计了类似 rollup 风格的设计插件系统，可以轻松地从 Rollup/Vite/Webpack 迁移您的插件/项目。


## 配置 Js 插件

通过 `plugins` 选项添加 JS 插件：

```ts title="farm.config.ts" {3,7}
import { defineConfig } from "@farmfe/core";
// import a js plugin
import farmPluginFoo from "farm-plugin-foo";

export default defineConfig({
  // configuring it in plugins
  plugins: [farmPluginFoo()],
});
```

## 编写Js插件
Farm Js 插件是一个普通的 javascript 对象，它公开了一组 `hook` 。 例如：

```ts title="my-farm-plugin.ts"
// 创建一个插件文件，导出一个返回 `JsPlugin` 对象的插件函数：
import type { JsPlugin } from '@farmfe/core';

// 插件选项
export interface PluginOptions {
  test: boolean;
}
// 导出插件函数
export default function MyPlugin(options: PluginOptions): JsPlugin {
  // 读取插件 options
  const { test } = options;

  // 返回一个暴露钩子的对象
  return {
    name: 'my-farm-plugin',
    // 使用load hook加载自定义模块
    load: {
      filters: {
        resolvedPaths: ['\\.test$'] // 过滤文件以提高性能
      },
      async executor({ resolvedPath }) {
        if (test && resolvedPath.endsWith('.test')) {
          return {
            content: 'test file',
            sourceMap: null
          }
        }
      }
    }
  }
}
```

:::note
* Farm提供`create-farm-plugin`工具来帮助您快速创建和开发您的js插件。 有关编写 JS 插件的更多详细信息，请参阅[编写 JS 插件](/docs/plugins/writing-plugins/js-plugin)
:::

## Plugin Hook Overview
Js 插件 Hook 与 Rust 插件相同，请参阅 [Rust 插件 Hook 概述](/docs/api/rust-plugin-api#plugin-hooks-overview)。

:::note
并非所有钩子都暴露给 Js 插件，只有本文档中列出的钩子可用。
:::

## hooks
### name
- **type: `string`**
- **required: `true`**

该插件的名称，不能为空。
```ts
export default function MyPlugin() {
  return {
    name: 'my-plugin',
    // ...
  }
}
```

### priority
- **type: `number`**
- **required: `false`**
- **default: `100`**

该插件的优先级，默认为 `100` 。  `priority` 控制插件的执行顺序，值越大，插件越早执行。

```ts
export default function MyPlugin() {
  return {
    name: 'my-plugin',
    priority: 1000, // // 使该插件先于所有其他插件执行
    // ...
  }
}
```
:::note
请注意，大多数 Farm 内部插件（如 `plugin-script` 、 `plugin-resolve` ）的优先级是 `99` ，这意味着您的插件始终在内部插件之前执行。 如果您想让您的插件在农场内部插件之后执行，请将 `priority` 设置为小于 `99` 的值，例如： `98` 。 优先级值也可以为负数，您可以将其设置为 `-9999` 以确保它始终最后执行。
:::

### config
- **type: `config?: (config: UserConfig) => UserConfig | Promise<UserConfig>;`**
- **hook type: `serial`**
- **required: `false`**

在`config`钩子中修改[Farm config](/docs/config/configuring-farm)，返回（部分）`修改后的配置`，返回的配置将深度合并到从cli和配置文件解析的配置中。 您也可以直接更改配置。

示例:
```ts
const resolveConfigPlugin = () => ({
  name: 'return-resolve-config-plugin',
  config: (_config) => ({
    compilation: {
      resolve: {
        alias: {
          foo: 'bar'
        }
      }
    }
  })
});
```

:::note
在解析所有 `用户插件` 后，会调用 `config` 钩子，因此在 config 钩子中将新插件添加到配置中无效。
:::


### configResolved
- **type: `configResolved?: (config: ResolvedUserConfig) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

当配置解析时调用（在调用所有插件的 `config`  钩子之后）。 当您想要获得插件的最终解析配置时很有用。

示例:
```ts
const myPlugin = () => {
  let farmConfig;

  return {
    name: 'my-plugin',
    configResolved(resolvedConfig) {
      // get resolved config
      resolvedConfig = farmConfig;
    },
    transform: {
      filters: {
        moduleTypes: ['js']
      },
      async executor(param) {
        if (farmConfig.xxx) {
          // ...
        }
      }
    }
  }
}
```

### configureDevServer
- **type: `configureDevServer?: (server: Server) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

:::note
请注意，该钩子仅在开发模式下运行。
:::

当 `Dev Server` 准备就绪时调用，您可以获得开发服务器实例。

示例:
```ts
const myPlugin = () => {
  let devServer;

  return {
    name: 'my-plugin',
    configureDevServer(server) {
      devServer = server;
    }
  }
}
```

:::note
`js plugin` 的 `config` 和 `configResolved` 钩子都会在 `rust plugin` 的 `config` 钩子之前被调用。
:::

### configureCompiler
- **type: `configureCompiler?: (compiler: Compiler) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

当 `Rust Compiler` 准备好时调用，该钩子在开发和生产中运行。 您可以在此处获取 `Compiler` 实例

示例:
```ts
const myPlugin = () => {
  let farmCompiler;

  return {
    name: 'my-plugin',
    configureCompiler(compiler) {
      farmCompiler = compiler;
    }
  }
}
```

### buildStart
- **type: `buildStart?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

在编译开始之前调用。 你可以在这里做一些初始化工作。

Example:
```ts
const myPlugin = () => {
  // 定义插件操作
  let myPluginContext = createMyPluginContext();

  return {
    name: 'my-plugin',
    buildStart: {
      async executor() {
        // 在编译之前初始化插件上下文
        myPluginContext.setup();
      }
    }
  }
}
```
:::note
`buildStart` 仅在第一次编译时调用一次。 后期编译如 `Lazy Compilation` 和 `HMR Update` 不会触发 `buildStart` 。
:::

### resolve
- **required: `false`**
- **hook type: `first`**
- **type:**
```ts
type ResolveHook = { 
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginResolveHookParam, PluginResolveHookResult> 
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> }
) => Promise<R | null | undefined>;

/// resolve 钩子的参数
export interface PluginResolveHookParam {
  /// 解析 `source` 的起始位置，如果 resolve 入口或 resolve hmr 更新，则为 [None]。
  /// 值为父模块的id，例如：`src/index.ts` 或 `src/index.vue?vue&type=xxx`
  importer: string | null;
  /// 例如，[ResolveKind::Import] 用于静态导入 (`import a from './a'`)
  kind: ResolveKind;
  /// 导入来源。 例如在index.ts中（import App from "./App.vue"）
  /// 源应该是 './App.vue'
  source: string;
}
/// resolve 钩子的解析结果
export interface PluginResolveHookResult {
  /// 解析路径，通常是绝对路径。 您还可以返回虚拟路径，并使用 [PluginLoadHookResult] 提供虚拟路径的内容
  resolvedPath: string;
  /// 该模块是否应该被 external，如果为 true，则该模块不会出现在最终结果中
  external: boolean;
  /// 该模块是否有副作用，影响tree shake
  sideEffects: boolean;
  /// 从说明符解析的查询，例如，如果说明符是`./a.png?inline`，查询应该是`{ inline: true }`
  query: [string, string][] | null;
  /// 模块的元数据，将传递给 [PluginLoadHookParam] 和 [PluginTransformHookParam]
  meta: Record<string, string> | null;
}
```

:::note
解析钩子的所有过滤器 `sources` 和 `importers` 都是 `正则字符串` 。
:::

从 `importer` 解析自定义 `source` ，例如从 `a.ts`  resolve  `./b` ：
```ts title="a.ts"
import b from './b?raw';
// ...
```
那么 resolve 参数将是：
```ts
const param = {
  source: "./b",
  importer: { relative_path: "a.ts", query_string: "" },
  kind: 'import'
}
```
默认的 resolve 结果为：
```rust
const resolve_result = {
  resolved_path: "/root/b.ts",   // 解析后的模块绝对路径
  external: false, // 该模块应该包含在最终编译的资源中，并且不应该被 external
  side_effects: false, // 不包含副作用，可以被 tree shake
  query: [["raw", ""]], // query 参数
  meta: {}
}
```

`HookContext` 用于在您可以递归挂钩时传递状态，例如，您的插件在 `resolve hook` 中调用 `context.resolve`：
```ts
const myPlugin = () => ({
  name: 'my-plugin',
  resolve: {
    filters: {
      sources: ['^.+foo.+$'],
      importers: ['^src/index.ts$']
    },
    executor: async (param, context, hookContext) => {
      console.log(param);
      if (hookContext.caller === 'my-plugin') {
        return null;
      }
      // 替换原来的源并解析新的源
      const newSource = param.source.replace('foo', 'bar');
      return context.resolve({
        ...param,
        source: newSource
      }, {
        caller: 'my-plugin',
        meta: {}
      });
    }
  }
});
```

在上面的例子中，我们调用 `context.resolve` 并传递 `caller` 作为参数，然后我们应该添加一个类似 `if (hookContext.caller === 'my-plugin') {` 的保护以避免无限循环。

注意：
* 默认情况下，您的`resolve hook`在Farm内部默认解析器**之后**执行，只有内部解析器无法解析的源才会传递给您的插件，这意味着如果您想覆盖默认解析器 ，您需要将**插件的优先级设置为大于**`101`。
* 通常 `resolved_path` 是指向文件的真实绝对路径。 但是您仍然可以返回一个 `虚拟模块 id` ，例如 `virtual:my-module` ，但是对于虚拟模块，您需要实现 `load` 钩子来自定义如何加载虚拟模块。 在 Farm 中，`resolved_path + query = module_id`。
* `ResolveKind` 表示 `import type`，示例值：`require`（由 commonjs require 导入）、`cssImport`（由 css 的 import 语句导入）等。
* `meta` 可以在插件和钩子之间共享，您可以从任何插件中的 `load`、`transform` 和 `parse` 钩子的参数中获取 `meta`。

### load
- **required: `false`**
- **hook type: `first`**
- **type:**
```ts
type LoadHook = { 
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginLoadHookParam, PluginLoadHookResult> 
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> }
) => Promise<R | null | undefined>;

export interface PluginLoadHookParam {
  moduleId: string;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
}

export interface PluginLoadHookResult {
  /// 模块的内容
  content: string;
  /// 模块的类型，例如[ModuleType::Js]代表普通的javascript文件，
  /// 通常以 `.js` 扩展名结尾
  moduleType: ModuleType;
  sourceMap?: string | null;
}
```

自定义如何从已解析的模块路径或模块 ID 加载模块。 例如加载一个虚拟模块：
```ts
const myPlugin = () => ({
  name: 'my-plugin',
  load: {
    filters: {
      resolvedPaths: ['^virtual:my-plugin$'],
    },
    executor: async (param, context, hookContext) => {
      if (param.resolvedPath === 'virutal:my-plugin') {
        return {
          content: 'export default "foo"',
          moduleType: 'js'
        };
      }
    }
  }
});
```

在 `load` 挂钩中加载模块时需要返回 `module_type` 和 `content` 。 `source_map` 是可选的，如果您在 `load` 钩子中进行转换（不推荐，我们建议在这种情况下使用 `transform` 钩子）或者从其他位置加载原始源地图，则可以返回源地图。


`load hook` 的 `filters.resolvedPath` 为 `resolvedPath + query`，例如：`/root/src/index.vue?vue&type=style&lang=css`。 如果你想在过滤模块时忽略查询，可以使用 `$`: `src/index\\.vue$`; 如果你想通过查询来过滤模块，例如过滤 `lang=css`，可以使用`src/index.vue\\.+\\?vue&.+lang=css`。

### transform
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type TransformHook = { 
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginTransformHookParam, PluginTransformHookResult> 
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> }
) => Promise<R | null | undefined>;

export interface PluginTransformHookParam {
  moduleId: string;
  /// 加载后的源内容或上一个插件转换后的结果
  content: string;
  /// 加载后的模块类型
  moduleType: ModuleType; // Module Type is 'js' | 'jsx' | 'ts' | 'tsx' | 'css' | 'html'...
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
  sourceMapChain: string[];
}

export interface PluginTransformHookResult {
  /// 转换后的源内容，将传递给下一个插件。
  content: string;
  /// 您可以在转换后更改模块类型。
  moduleType?: ModuleType;
  /// 转换后的源映射，所有插件转换后的源映射将存储为源映射链。
  sourceMap?: string | null;
  // 忽略之前的 source map。 如果为 true，则source map链将被清除。 这个结果应该返回一个新的source map，它结合了所有以前的 source map。
  ignorePreviousSourceMap?: boolean;
}
```

根据**`模块内容`**和**`模块类型`**进行转换。 将 `sass` 转换为 `css` 的示例：

```ts
export default function farmSassPlugin(
  options: SassPluginOptions = {}
): JsPlugin {
  return {
    name: pluginName,
    load: {
      filters: { resolvedPaths: ['\\.(scss|sass)$'] },
      async executor(param) {
        if (param.query.length === 0 && existsSync(param.resolvedPath)) {
          const data = await readFile(param.resolvedPath);
          return {
            content: data,
            moduleType: 'sass'
          };
        }

        return null;
      }
    },
    transform: {
      filters: {
        moduleTypes: ['sass']
      },
      async executor(param, ctx) {
        const { css: compiledCss, map } = compileSass(param.content);
        return {
          content: compiledCss,
          moduleType: 'css' // transformed sass to css,
          sourceMap: JSON.stringify(map)
          ignorePreviousSourceMap: false,
        }
      }
    }
  }
}
```

编写 `transform hook` 的正常步骤：
1. 添加基于 `moduleType` 或 `resolvedPath` 或 `moduleId` 的 `if` 保护
2. 对 `内容` 进行转换
3.返回转换后的`content`、`sourceMap`和`moduleType`

对于 `ignorePreviousSourceMap` ，如果您处理了 `param.sourceMapChain` 并折叠了 `transform hook` 中以前插件的源映射。 您应该将 `ignorePreviousSourceMap` 设置为 `true` 以确保源映射正确。 否则，您应该始终将此选项设置为 `false` 并让 Farm 处理源映射链。

对于 filters：
* 当同时指定 `resolvedPaths` 和 `moduleTypes` 时，取并集。
* `filters.resolvedPaths` 是 `resolvedPath + query`，例如：`/root/src/index.vue?vue&type=style&lang=css`。 如果你想在过滤模块时忽略查询，可以使用 `$`: `src/index\\.vue$`; 如果你想通过查询来过滤模块，例如过滤 `lang=css`，可以使用`src/index.vue\\.+\\?vue&.+lang=css`。
* `filters.moduleTypes` 不是 ** `regex`，它必须与 `ModuleType` 完全匹配，如 `css`、`js`、`tsx` 等。

:::note
`transform` 钩子是**内容到内容**。 有一个类似的钩子叫做 `process_module` ， `process_module` 是**ast 到 ast**。 由于性能问题，Js 插件不支持 `process_module` 钩子，如果您想要 **ast 到 ast** 转换，请尝试使用 [`Rust Plugin`](/docs/plugins/writing-plugins/rust-plugin)。
:::

### buildEnd
- **type: `buildEnd?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

在 `ModuleGraph` 构建之后、资源渲染和生成开始之前调用。 您可以在此处进行一些状态更新或完成工作。

示例:
```ts
const myPlugin = () => {
  // 定义插件上下文
  let myPluginContext = createMyPluginContext();

  return {
    name: 'my-plugin',
    buildEnd: {
      async executor() {
        // 更新插件状态
        myPluginContext.updateStatus('module-graph-built');
      }
    }
  }
}
```
:::note
`buildEnd` 仅在第一次编译时调用一次。 稍后编译如`Lazy Compilation`和`HMR Update`不会触发`buildEnd`。
:::

### renderStart
- **type: `renderStart?: { executor: Callback<Config['config'], void>; };`**
- **hook type: `parallel`**
- **required: `false`**

在资源渲染开始之前调用。

示例:
```ts
const myPlugin = () => {
  // 定义插件上下文
  let myPluginContext = createMyPluginContext();

  return {
    name: 'my-plugin',
    renderStart: {
      async executor() {
        // 更新插件状态
        myPluginContext.updateStatus('render-start');
      }
    }
  }
}
```
:::note
`renderStart` 仅在第一次编译时调用一次。 稍后编译如 `Lazy Compilation` 和 `HMR Update` 将不会触发 `renderStart` 。
:::

### renderResourcePot
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type RenderResourcePotHook = JsPluginHook<
  {
    resourcePotTypes?: ResourcePotType[];
    moduleIds?: string[];
  },
  RenderResourcePotParams,
  RenderResourcePotResult
>;

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
) => Promise<R | null | undefined>;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };

export interface RenderResourcePotParams {
  content: string;
  sourceMapChain: string[];
  resourcePotInfo: {
    id: string;
    name: string;
    resourcePotType: ResourcePotType;
    map?: string;
    modules: Record<ModuleId, RenderedModule>;
    moduleIds: ModuleId[];
    data: JsResourcePotInfoData;
    custom: Record<string, string>;
  };
}
export interface RenderResourcePotResult {
  content: string;
  sourceMap?: string;
}
```

`Resource Pot` 是最终输出的打包后的文件的抽象表示，您可以返回转换后的 `resourcePot content` 来改变最终的包。 例如渲染CSS：

```ts
const myPlugin = () => ({
  name: 'test-render-resource-pot',
  renderResourcePot: {
    filters: {
      moduleIds: ['^index.ts\\?foo=bar$'],
      resourcePotTypes: ['css']
    },
    executor: async (param) => {
      return {
        content: param.content.replace(
          '<--layer-->',
          cssCode
        ),
        sourceMap
      };
    }
  }
})
```
我们将 css 资源罐中的所有 `<--layer-->` 进行转换，并将其替换为真正的 `css 代码`。

:::note
当同时指定了 `filters.moduleIds` 和 `filters.resourcePotTypes` 时，取并集。
:::

### augmentResourceHash
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type AugmentResourceHash = JsPluginHook<
  {
    resourcePotTypes?: ResourcePotType[];
    moduleIds?: string[];
  },
  {
    id: string;
    name: string;
    resourcePotType: ResourcePotType;
    map?: string;
    modules: Record<ModuleId, RenderedModule>;
    moduleIds: ModuleId[];
    data: JsResourcePotInfoData;
    custom: Record<string, string>;
  },
  string
>;

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
) => Promise<R | null | undefined>;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };
```

为给定资源罐附加资源哈希。 如果您想在生成资源哈希时添加附加条件，则非常有用。

```ts
const myPlugin = () => ({
  name: 'test-augment-resource-pot',
  renderResourcePot: {
    filters: {
      moduleIds: ['^index.ts\\?foo=bar$'],
      resourcePotTypes: ['css']
    },
    executor: async (param) => {
      return 'my-hash-args';
    }
  }
})
```

:::note
当同时指定了 `filters.moduleIds` 和 `filters.resourcePotTypes` 时，取并集。
:::

### finalizeResources
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type FinalizeResourcesHook = {
  executor: Callback<
    FinalizeResourcesHookParams,
    FinalizeResourcesHookParams['resourcesMap']
  >;
};

export type FinalizeResourcesHookParams = {
  resourcesMap: Record<string, Resource>;
  config: Config['config'];
};

export interface Resource {
  name: string;
  bytes: number[];
  emitted: boolean;
  resourceType: string;
  origin: { type: 'ResourcePot' | 'Module'; value: string };
  info?: ResourcePotInfo;
}
```

对所有生成的资源进行一些转换，返回 `转换后的resourcesMap` 。 您可以在此钩子中 `添加` 、 `删除` 、 `修改` 最终生成的资源。

注意：
* `bytes` 是最终输出的二进制，对于 `js/css/html` 代码，可以使用 `Buffer.from(bytes).toString()` 来获取代码。
* `name` 是最终的文件名。
* `origin` 代表这个 `Resource` 的来源，`ResourcePot` 表示它是从 `ResourcePot` 生成的，而 `ResourcePot` 是一个模块包；  `Module` 表示它来自 `Module` ，例如 `.png/.jpg` 等静态文件来自 `Module` 。

### transformHtml
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type TransformHtmlHook = {
  order?: 0 | 1 | 2；
  executor: Callback<{ htmlResource: Resource }, Resource>;
};
```

`order` 控制 `transformHtml` 执行时机:
* `0`: 代表 `pre`, 在 parse 之前执行，在这里可以转换原始的 html。
* `1` and `2`: 代表 `normal` and `post`, 在 parse 和 generate resources 之后执行. 在这个阶段, 所有的 `<script>`, `<link>` 标签都已经被注入。

转换最终生成的html（注入所有`<script>`、`<link>`标签后）。

```ts
const myPlugin = () => ({
  name: 'my-plugin',
  transformHtml: {
    order: 2,
    async executor({ htmlResource }) {
      const htmlCode = Buffer.from(htmlResource).toString();
  
      const newHtmlCode = htmlCode.replace('my-app-data', data);
      htmlResource.bytes = [...Buffer.from(newHtmlCode)];

      return htmlResource;
    }
  }
});
```

:::note
您应该修改 `htmlResource` 的 `bytes` 字段并返回更新后的 `htmlResource` ，改变任何其他字段不会产生任何影响
:::

### writeResources
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type WriteResourcesHook = {
  executor: (param: FinalizeResourcesHookParams) => void | Promise<void>;
};
```

在所有资源写入磁盘**之后**调用。

### pluginCacheLoaded
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type PluginCacheLoadedHook = {
  executor: Callback<number[], undefined | null | void>;
};
```

扩展插件的持久缓存加载。

当启用 `持久缓存` 时，在命中缓存时可能会跳过 `load` 和 `transform` 钩子。 如果您的插件依赖于以前的编译结果（例如，基于现有模块加载虚拟模块），您可能需要实现此钩子来加载插件的缓存信息，以确保缓存按预期工作。

示例：
```ts
const myPlugin = () => {
  let cachedData;

  return {
    name: 'my-plugin',
    pluginCacheLoaded: {
      async executor(bytes) {
        const str = Buffer.from(bytes).toString();
        cachedData = JSON.parse(str);
      }
    }
  }
}
```
:::note
您必须决定如何在插件中将缓存 `序列化/反序列化` 为 `字节` 。 作为一个基本示例，您可以通过 `[...Buffer.from(JSON.stringify(data))]` 反序列化数据
:::

### writePluginCache
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type WritePluginCacheHook = {
  executor: Callback<undefined, number[]>;
};
```

扩展插件的持久缓存写入。 `writePluginCache` 通常与 [pluginCacheLoaded](#plugincacheloaded) 一起使用来读写插件的持久缓存。 返回数据的序列化字节。

示例：
```ts
const myPlugin = () => {
  let cachedData = { foo: 'bar' };

  return {
    name: 'my-plugin',
    writePluginCache: {
      async executor() {
        const bytes = [...Buffer.from(JSON.stringify(data))];
        return bytes;
      }
    }
  }
}
```

:::note
您必须决定如何在插件中将缓存 `序列化/反序列化` 为 `字节` 。 作为一个基本示例，您可以通过 `[...Buffer.from(JSON.stringify(data))]` 反序列化数据
:::

### finish
- **type: `finish?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

在资源渲染开始之前调用。

例子：
```ts
const myPlugin = () => {
  // 设置插件上下文
  let myPluginContext = createMyPluginContext();

  return {
    name: 'my-plugin',
    finish: {
      async executor() {
        // 更新插件的状态
        myPluginContext.updateStatus('finish');
      }
    }
  }
}
```
:::note
 `finish` 仅在第一次编译时调用一次。 稍后编译，如 `Lazy Compilation` 和 `HMR Update` ，不会触发 `finish` 。
:::

### updateModules
- **required: `false`**
- **hook type: `serial`**
- **type:**
```ts
type UpdateModulesHook = {
  executor: Callback<
    { paths: [string, string][] },
    string[] | undefined | null | void
  >;
};
```

调用compiler.update(module_paths)时调用。 对于执行 HMR 时执行一些操作（例如清除以前的状态或忽略某些文件）很有用。

* `paths` 是将为此更新重新编译的路径
* 返回新的`paths`，后续的编译将更新返回的新路径。
