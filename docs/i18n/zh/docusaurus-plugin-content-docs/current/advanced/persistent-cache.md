# 增量构建

:::tip
Farm 从`v0.14.0`开始支持通过持久缓存的增量构建
:::

从`v0.14.0`开始，Farm 支持将编译结果缓存到磁盘，这可以大大加快热启动/热构建的编译速度。当启用`persistentCache`时，编译时间可以减少**高达`80%`**。

冷启动（无缓存）和热启动（有缓存）的性能比较使用 [examples/argo-pro](https://github.com/farm-fe/farm/tree/main/examples/arco-pro):

|       | Cold(without cache) | Hot(with cache) | diff        |
| ----- | ------------------- | --------------- | ----------- |
| start | 1519ms              | 371ms           | reduced 75% |
| build | 3582ms              | 562ms           | reduced 84% |

## 使用缓存

使用[`compilation.persistentCache`](/zh/docs/config/compilation-options#persistentcache) 来`启用/禁用`缓存：

```ts
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    persistentCache: true,
  },
});
```

:::note
`persistentCache: true` 相当于:

```js
({
  persistentCache: {
    // Directory that cache is stored
    cacheDir: "node_modules/.farm/cache",
    // namespace of the cache
    namespace: "farm-cache",
    buildDependencies: [
      "farm.config.ts",
      "farm",
      "@farmfe/plugin-react",
      // ... all other dependencies
    ],
    moduleCacheKeyStrategy: {
      timestamp: true,
      hash: true,
    },
  },
});
```

:::
将`persistentCache`为`false`以禁用缓存

## 缓存验证

缓存在尝试重用时会通过以下条件进行验证，如果以下任何条件发生变化，所有缓存都将失效

- **Env Object**：由`persistentCache.envs`配置，默认为`Farm Env Mode`(`process.env.NODE_ENV`, `process.env.DEV`, `process.env.PROD`)，参见 **[`环境变量和模式`](/zh/docs/features/env)**。

- **lockfile**:如果你的 lockfile (例如 pnpm-lock.yaml) 改变了，意味着有依赖项改变，缓存将失效。

- **构建依赖项**：由`persistentCache.buildDependencies`配置，如果任何 buildDependency 更改，所有缓存将失效。

- **Cache 命名空间**：由`persistentCache.namespace`配置，不同命名空间下的缓存不会重复使用。如果要使所有缓存失效，可以配置不同的命名空间。

- **内部缓存版本**：Farm 内部维护一个缓存版本，如果 Farm 本身发生了变化，例如，影响 Farm 版本之间输出的渲染优化，Farm 将碰撞缓存版本，所有缓存将失效。

如果您的缓存不起作用，请查看上述条件以找出原因。如果缓存损坏，您还可以删除 `node_modules/.farm/cache` 以手动删除缓存。

## 构建依赖项

构建依赖是可以影响编译过程或编译输出的依赖，例如插件或 config 文件。如果这些依赖中的任何一个发生了变化，所有缓存都将失效

构建依赖项可以是包名的文件路径，例如:

```ts
import { defineConfig } from "farm";
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

:::note
默认情况下，所有配置文件及其依赖项都包含在内。但是，如果您想添加一些额外的文件或依赖项来使缓存失效，您可以使用`buildDependencies`一旦这些文件更改，所有缓存都将失效
:::

## 模块缓存密钥策略

Farm 提供了 2 种策略来控制如何生成模块缓存键:

- `timestamp`: 检查模块的时间戳，如果更新时间戳没有变化，则跳过此模块的构建，具有最佳性能.
- `hash`: 加载和转换后检查 content hash，如果内容没有变化，将跳过此模块的剩余构建.

默认情况下，`timestamp`和`hash`都是启用的

## 插件注意事项

当启用`timestamp`时，不会调用所有构建阶段钩子，如`load`和`transform`。因此，如果插件依赖于`load`和`transform`，并且没有实现 `plugin_cache_loaded` 和 `write_plugin_cache` 钩子，它可能无法按预期工作。例如，如果插件在`load`和`transform`中收集信息，所有在`finish` hook 时发出它们，它应该实现 `plugin_cache_loaded` 和 `write_plugin_cache` hook 来加载和写入缓存，否则它将无法按预期工作。

Farm 将设置`timestamp`为`false` 当 `output.targetEnv`是 `node`

<!-- ## Dive deep into Persistent  -->
