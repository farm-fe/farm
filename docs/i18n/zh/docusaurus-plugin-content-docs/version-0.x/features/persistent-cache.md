# 增量构建
:::tip
自`v0.14.0`起，Farm 支持通过持久缓存进行增量构建
:::

从 v0.14.0 开始，Farm 支持将编译结果缓存到磁盘，这可以大大加快热启动/热构建的编译速度。 当启用`persistentCache`时，编译时间可以减少**高达`80%`**。

使用 [examples/argo-pro](https://github.com/farm-fe/farm/tree/main/examples/arco-pro) 进行冷启动（无缓存）和热启动（有缓存）的性能比较：

||冷（无缓存）|热（有缓存）| 差异|
|---|---|---|---|
|开始|1519ms|371ms|减少 75%|
|构建|3582ms|562ms|减少 84%|

## 使用缓存
使用`compilation.persistentCache`启用/禁用缓存：

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: true
  }
})
```
:::note
`persistentCache: true` 等于：
```js
({
  persistentCache: {
    // Directory that cache is stored
    cacheDir: 'node_modules/.farm/cache',
    // namespace of the cache
    namespace: 'farm-cache',
    buildDependencies: [
      'farm.config.ts',
      '@farmfe/core',
      '@farmfe/plugin-react'
      // ... all other dependencies
    ],
    moduleCacheKeyStrategy: {
      timestamp: true,
      hash: true,
    }
  }
})
```
:::

将`persistentCache`配置为`false`以禁用缓存。

## 缓存验证
当以下条件尝试重用缓存时，缓存将被验证，如果以下任何条件发生变化，所有缓存将失效：
* **Env Object**：由`persistentCache.envs`配置，默认为`Farm Env Mode`(`process.env.NODE_ENV`, `process.env.DEV`, `process.env.PROD`)，参见 `Farm Env`(/docs/config/farm-config#environment-variable)。
* **lockfile**：如果您的`npm-lock.json`或者`yarn.lock`或者`pnpm-lock.yaml`发生更改，则意味着依赖项发生更改，缓存将失效。
* **构建依赖项**：通过`persistentCache.buildDependency`配置，如果任何一个buildDependency发生变化，所有缓存都将失效。
* **缓存命名空间**：通过`persistentCache.namespace`配置，不同命名空间下的缓存不会被重用。 如果想让所有缓存失效，可以配置不同的命名空间。
* **内部缓存版本**：Farm内部维护了一个缓存版本，如果Farm本身发生变化，例如影响Farm版本之间输出的渲染优化，Farm会改变缓存版本，所有缓存都会失效。

如果您的缓存不起作用，请检查上述情况以找出原因。 如果缓存损坏，您还可以删除`node_modules/.farm/cache`来手动删除缓存。

## 构建依赖
构建依赖项是可以影响编译过程或编译输出的依赖项，例如插件或配置文件。 如果这些依赖项中的任何一个发生更改，所有缓存都将失效。

构建依赖项可以是包名称的文件路径，例如：

```ts
import { defineConfig } from '@farmfe/core';
import path from 'node:path';

export default defineConfig({
  persistentCache: {
    buildDependencies: [
      // a file path
      path.resolve(process.cwd(), './plugins/my-plugin.js'),
      // a package name, note that this package must expose package.json
      'farm-plugin-custom-xxx'
    ]
  }
})
```

:::note
默认情况下，包含所有配置文件及其依赖项。 但是如果你想添加一些额外的文件或依赖项来使缓存失效，你可以使用`buildDependencies`，一旦这些文件发生更改，所有缓存都将失效。
:::

## 模块缓存关键策略
Farm提供了2种策略来控制如何生成模块缓存密钥：

* `timestamp`: 是否检查模块的时间戳，如果更新时间戳没有改变，则跳过该模块的构建，性能最佳。
* `hash`: 加载和转换后是否检查内容哈希，如果内容没有改变，则跳过该模块的左侧构建。

默认情况下，`timestamp`和`hash`均已启用。

## 插件注意事项
当启用`timestamp`时，所有构建阶段 hook（如`load`和`transform`）都不会被调用。 因此，如果插件依赖于`load`和`transform`，并且没有实现`plugin_cache_loaded`和`write_plugin_cache`挂钩，则它可能无法按预期工作。 例如，如果一个插件在`load`和`transform`中收集信息，并在`finish`钩子上发出它们，那么它应该实现`plugin_cache_loaded`和`write_plugin_cache`钩子来加载和写入缓存，否则它将无法按预期工作 。

当`output.targetEnv`为`node`时，Farm 会将`timestamp`设置为`false`。

<!-- ## 深入持久化 -->