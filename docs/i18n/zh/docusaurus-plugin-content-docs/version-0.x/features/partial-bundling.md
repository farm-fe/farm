# Partial Bundling
`Partial Bundling` 是 Farm 用于打包模块的策略，与其他 bundler的做法类似，但 Farm 的 `Partial Bundling` 的目标不同。

与其他 bundler 不同，Farm 不会尝试将所有内容打包在一起，然后使用`splitChunks`等优化将它们分开，相反，Farm 会将项目直接打包到多个输出文件中。 例如，如果启动一个 html 页面需要数百个模块，Farm 将尝试直接将它们打包到 20-30 个输出文件中。 Farm 将这种行为称为`Partial Bundling`。

Farm Partial Bundling 的目标是：
1. **减少请求数量和请求层次**：使数百或数千个模块请求减少到20-30个请求，并避免由于依赖层次而逐个加载模块，这将使资源加载更快。
2. **提高缓存命中率**：当模块更改时，确保只有少数输出文件受到影响，以便更多缓存可以用于在线项目。

对于传统的 bundler，我们可能很难配置复杂的`splitChunks`或`manualChunks`来实现上述目标，但在 Farm 中，通过`Partial Bundling`原生支持它。

:::tip
请参阅 [RFC-003 局部打包](https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md) 以获取更多技术细节。
:::

## 设计动机
现在 Web 构建工具中处理模块的方法主要有两种：打包或使用原生 ESM。 但它们都有缺点：
* 对于打包，bundler 的目标是将所有内容打包在一起，然后将它们拆分出来进行优化，但拆分通常很难配置，并且很难手动平衡资源加载性能和缓存命中率。
* 对于原生esm，每个模块都可以单独编译、缓存，但是当有数百个模块请求时，加载性能受到严重影响。

所以我一直在想，如果有一种策略可以避免这两个极端——也许我们可以进行局部打包？ 我们可以直接将项目自动打包到几个有限的、大小平衡的资源中。 我将这种想法命名为 `Module Merging` - 在打包和不打包之间找到平衡，仅打包一些相关模块以提高加载性能而不损失缓存粒度。

> 后来我将`Module Merging`改名为`Partial Bundling`，因为我认为`Partial Bundling`可以更准确地表达我的想法。

## Partial Bundling 规则
> 在本节中，我们将通过示例介绍`Partial Bundling`使用的基本规则。

首先我们研究一个基本的 React 项目示例。 对于像下面这样的基本 react 项目，我们在入口脚本中导入 react 和 react-dom：
```tsx title="index.tsx"
import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <>
    <div>Index page</div>
  </>
);
```
打包结果将如下所示：
```text
./dist/
├── index_9c07.49b83356.js    # 包含react-dom
├── index_a35f.0ac21082.js    # 包含./index.tsx
├── index_b7e0.7ab9ca2d.js    # 包含react及其依赖项
├── index_ce26.7f833381.css   # 包含./index.scss
└── index.html                # 包含./index.html
```
默认情况下，Farm 会将项目打包为 5 个文件：
* `2 个 js 文件`来自 `node_modules`，包含 `react`、`react-dom` 及其依赖项。
* `1 个js文件`来自`./index.tsx`
* `1 个 css 文件`来自`./index.scss`；
* `1个html文件`来自`./index.html`;

Farm 使用以下规则来获得上述结果：
1. **可变和不可变模块应始终位于不同的输出文件中**：默认情况下，Farm 认为 `node_modules` 下的所有模块都是不可变的，否则是可变的。 所以 `./index.tsx` 位于一个单独的文件中，因为它是一个可变模块，所以它永远不会与 `react` 和 `react-dom` 位于同一个输出文件中。
2. **不同类型的模块始终位于不同的输出文件中**：因此 `./index.scss` 位于单独的文件中。
3. **同一包中的模块应该位于同一个输出文件中**：因此所有`react`模块始终位于同一个输出文件中，`react-dom`也是如此。
4. **默认情况下，资源加载的目标并发请求数应在 20-30 之间**：因此有 3 个 js 输出文件，而不是 1 个 js 包。
5. **输出文件大小应相似，默认最小资源大小应大于20KB**：因为`react-dom`最大，超过100KB，所以它在一个单独的文件中，而 `react` 的依赖项小于`20KB`，被合并到同一个输出文件中。

现在我们已经熟悉了`Partial Bundling`的基本规则，如果遇到部分打包的问题，可以使用上述规则来调试您的项目。 接下来我们将介绍如何配置 `Partial Bundling`。

## 配置 Partial Bundling
`Partial Bundling` 支持很多选项，让用户自定义其行为。 所有选项如下：

1. **`targetConcurrentRequests`**: Farm 尝试生成尽可能接近此配置值的资源数量，控制初始资源加载或动态资源加载的并发请求数量。
2. **`targetMinSize`**: minify 和 gzip 之前生成的资源的最小大小。 请注意，`targetMinSize` 并不一定保证满足，可以配置`enforceTargetMinSize`可用于强制限制最小的大小。
3. **`targetMaxSize`**: minify 和gzip 之前生成的资源的最大大小。
4. **`groups`**: 一组应该放在一起的模块。 请注意，此组配置只是对编译器的打击，即这些模块应该放置在一起，它可能会产生多个资源，如果您想强制打包模块到同一个资源中，使用`enforceResources`。
    * **name**: 该组的名称。
    * **test**: 匹配该组中的模块路径的正则表达式数组。
    * **groupType**: `mutable` 或 `immutable`，限制该组仅适用于指定类型的模块。
    * **resourceType**: `all`、`initial` 或 `async`，限制该组仅适用于指定类型的资源。
5. **`enforceResources`**: 忽略所有其他约束，强制匹配的模块打包到一起。
    * **name**: 该组的名称。
    * **test**: 匹配该组中的模块路径的正则表达式数组。
6. **`enforceTargetConcurrentRequests`**: 对每个资源加载强制执行目标并发请求数量，当为 true 时，较小的资源将合并为较大的资源以满足目标并发请求。 这可能会导致 css 资源出现问题，请小心使用此选项
7. **`enforceTargetMinSize`**: 为每个资源强制执行目标最小大小限制，如果为真，较小的资源将合并为较大的资源以满足目标并发请求。 这可能会导致 css 资源出现问题，请小心使用此选项
8. **`immutableModules`**: 匹配不可变模块的正则表达式数组
9. **`immutableModulesWeight`**: 默认为`0.8`，不可变模块将拥有80%的请求数。 例如，如果`targetConcurrentRequest`为 25，则默认情况下不可变资源将采用`25 * 80% = 20`。 该选项是为了确保可变模块和不可变模块是隔离的，如果更改您的业务代码，node_modules下的代码不会受到影响。

:::note
您可以使用`targetConcurrentRequests`、`targetMinSize`和`targetMaxSize`来控制 Partial Bundling 的默认行为。 Farm 设置的默认值基于最佳实践，因此当您想要更改默认值时请确保有必要。
:::

### Grouping Modules
您可以使用`groups`将模块分组在一起，对于上面的基本React项目示例，使用以下配置将`node_modules`下的模块打包在一起：

```ts title="farm.config.ts" {4-9}
export default defineConfig({
  compilation: {
    partialBundling: {
      groups: [
        {
          name: 'vendor-react',
          test: ['node_modules/'],
        }
      ]
    },
  },
});
```
我们添加一个带有`name`和`test`的`group item`，将`react`和`react-dom`分组在一起。 打包结果是：
```
./dist/
├── index_499e.72cf733c.js      # 包含`react`、`react-dom`以及node_modules下的所有其他文件
├── index_a35f.0ac21082.js      # 包含 `./index.tsx`
├── index_ce26.7f833381.css     # 包含 `./index.scss`
└── index.html                  #包含`./index.html`
```

现在 `node_modules` 下的所有模块都打包到 `index_499e.72cf733c.js` 中。 请注意，`groups`并不强制打包所有与该组匹配的模块，一个`group`会生成多个`output file`，因为：
1. 可变和不可变模块始终位于不同的输出文件中。 当可变模块和不可变模块都命中这个`组`时，它们将处于不同的输出中。
2. 当涉及多页面应用程序或 dynamic import 时，可能存在共享模块，这些模块会始终位于不同的输出文件中。

如果需要强制打包指定的模块到一个文件中，可以使用`enforceResources`

### Using `enforceResources`
要将所有模块分组在一起并忽略所有其他条件，您可以使用`enforceResources`，例如：
```ts title="farm.config.ts" {4-9}
export default defineConfig({
  compilation: {
    partialBundling: {
      enforceResources: [
        {
          name: 'index',
          test: ['.+'],
        }
      ]
    },
  },
});
```
will produce:
```
./dist/
├── index.7f833381.css # 所有css模块都打包在一起
├── index.ba5550d9.js  # 所有脚本模块都打包在一起
└── index.html
```

:::warning
`enforceResources` 会忽略 Farm 的所有内部优化，使用时要小心。
:::

### Configuring `immutable modules`
使用`immutableModules`配置不可变模块，默认情况下，Farm 将其设置为`node_modules/`。

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    partialBundling: {
      immutableModules: ['node_modules/', '/global-constants']
    },
  },
});
```
不可变模块会影响打包和传入的持久缓存，如果要更改它，请小心。