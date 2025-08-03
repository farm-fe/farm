# 编写 Runtime 插件

Farm 运行时插件是一个纯 JavaScript 对象，它定义了一组钩子来增强 Farm 运行时。 例子：

```ts
/**
 * HMR 客户端作为 Farm 运行时插件
 */
import type { Plugin } from "@farmfe/runtime";
import { createHotContext } from "./hot-module-state";
import { HmrClient } from "./hmr-client";

let hmrClient: HmrClient;
// 导出 Farm 运行时插件对象
export default <Plugin>{
  name: "farm-runtime-hmr-client-plugin",
  // 定义钩子
  bootstrap(moduleSystem) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
  moduleCreated(module) {
    // 为每个模块创建一个 hot 上下文
    module.meta.hot = createHotContext(module.id, hmrClient);
  },
};
```

上面是一个支持 Farm 的 HMR 的运行时插件。 要点：

- 运行时插件入口文件应该 **`导出`** 定义一组钩子的默认对象。 例如 `导出默认 <Plugin>{/*...*/}`
- 需要`name`来标识插件，确保`name`是唯一的
- `hook` 是在导出对象中定义的方法。

:::note
有关上述示例的完整实现，请参阅 [@farmfe/runtime-plugin-hmr](https://github.com/farm-fe/farm/tree/main/packages/runtime-plugin-hmr)。
:::

## 注意事项

您应该使您的运行时插件尽可能**简单**。 你**不应该**：

- 使用node_modules中的**大依赖**，这会让你的 Farm 运行时插件非常大，可能会严重影响运行时性能。
- 使用 `top level await` 等新功能，因为这些与运行时相关的功能很难针对低级别运行时进行 polyfill。

强烈建议确保您的运行时插件**尽可能小且简单**。

:::tip
`import.meta.xxx` 将被编译为 `module.meta.xxx`，您可以在运行时插件中向 `module.meta` 添加值来增强 `import.meta`。 例如， `module.meta.hot = createHotContext(module.id, hmrClient)` 使 `import.meta.hot` 可用。
:::

## 惯例

Farm 运行时插件名称应以 `farm-runtime-plugin` 为前缀，例如 `farm-runtime-plugin-xxx` 。

:::note
`plugin.name` 和 `package name`（仅当您将插件发布为包时）都应该加上前缀。
:::

## 使用 Runtime 插件

使用 `compilation.runtime.plugins` 为您的项目配置运行时插件：

```ts
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    runtime: {
      plugins: [
        // relative path
        "./src/my-plugin1.ts",
        // absolute path
        "/root/project/src/my-plugin2.ts",
        // package name
        "@scope/plugin-package-from-node-modules",
      ],
    },
  },
});
```

您可以通过 3 种方式配置运行时插件项：

- **`相对路径`**：相对于`root`的路径，例如`./src/my-plugin1.ts`将尝试从`<root>/src/my-plugin1.ts`加载插件。
- **`绝对路径`**：例如`/root/project/src/my-plugin2.ts`。 （在 Windows 上绝对路径应为 `C:\project\src\my-plugin2.ts` ）。
- **`包名称`**：Farm将尝试从`node_modules`加载此包，例如`@scope/plugin-package-from-node-modules`。

## 编写 Runtime 插件

:::tip
Farm支持直接加载`.ts`文件，因此您可以直接在`runtime.plugins`中配置一个`.ts`文件（或条目为`ts`文件的包）。

```ts
export default defineConfig({
  compilation: {
    runtime: {
      plugins: [
        // configuring ts file directly
        "./src/my-plugin.ts",
      ],
    },
  },
});
```

:::

### 创建插件

正如我们上面提到的，Farm 运行时插件是一个纯 JavaScript 对象，它定义了一组钩子，您只需创建一个 ts 文件，例如：

```ts title="./plugins/runtime.ts"
import type { Plugin } from "@farmfe/runtime";

export default <Plugin>{
  name: "my-plugin",
  // ...
};
```

然后在导出的对象中定义您需要的[hooks](#runtime-plugin-hooks)：

```ts title="./plugins/runtime.ts"
import type { Plugin } from "@farmfe/runtime";

export default <Plugin>{
  name: "my-plugin",
  moduleCreated(module) {
    // ...
  },
  readModuleCache(module) {
    // ...
  },
  loadResource(resource, targetEnv) {
    // ...
  },
  // ... more hooks as long as you need
};
```

### 定义插件

配置您在 `runtime.plugins` 中创建的插件：

```ts
export default defineConfig({
  compilation: {
    runtime: {
      plugins: ["./plugins/runtime.ts"],
    },
  },
});
```

然后启动Farm项目，这个插件会在编译时注入输出资源的 runtime 中。

### 发布插件（可选）

您可以将运行时插件发布到 npm 注册表以共享您的 Farm 运行时插件。 只需创建一个 `package.json` ，例如：

```json
{
  "name": "@farmfe/runtime-plugin-hmr",
  "version": "3.4.2",
  "description": "Runtime hmr plugin of Farm",
  // c-highlight-start
  "main": "src/index.ts"
  // c-highlight-end
  // ... ignore other fields
}
```

You can just export `ts` file using `"main": "src/index.ts"`.

## 运行时插件钩子

请参阅[运行时插件 API](/docs/api/runtime-plugin-api)
