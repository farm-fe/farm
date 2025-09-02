# 局部打包
局部打包（`Partial Bundling`）是 Farm 用来打包模块的策略，类似于其他打包工具，但 Farm 的 `局部打包` 目标不同。

与其它打包工具不同，Farm不会尝试将所有内容打包在一起，而是使用像 `splitChunks` 之类的优化策略将其拆分出来，相反，Farm会将项目直接捆打包成多个输出文件。例如，如果需要数百个模块来启动一个html页面，Farm将尝试将它们直接打包成20到30个输出文件。Farm将这种行为称为`局部打包`。

Farm局部打包的目标是:
1. **减少请求数量和请求层次**: 将数百上千个模块请求减少到20-30个请求，避免由于依赖层次结构而逐个加载模块，从而加快资源的加载。
2. **提高缓存命中率**: 当模块发生更改时，确保只有少数输出文件受到影响，因此可以为项目提高缓存命中率。

对于传统打包工具，我们可能很难通过复杂的 `splitChunks` 或 `manualChunks` 配置来实现上述目标，但是 Farm 原生支持`局部打包`。

请注意，默认的打包策略是为浏览器设计的，但它也适用于 Node.js。 如果想要更改 Node.js 的打包策略，请尝试[配置局部打包](#configuring-partial-bundling)。

:::tip
请参考 [RFC-003 Partial Bundling](https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md) 局部打包以获取更多技术细节。
:::

## 动机
目前，Web构建工具处理模块的主要方法有两种：完全打包或原生ESM。但它们都有缺点：
* 对于完全打包，打包工具旨在将所有内容打包在一起，然后拆分出来进行优化，但拆分通常难以配置，手动平衡资源加载性能和缓存命中率很难。 
* 对于原生ESM，每个模块都可以单独编译和缓存，但当有数百个模块请求时，会严重影响加载性能。

因此，我一直在思考是否有一种策略可以避免这两种极端情况 - 也许我们可以进行局部打包？我们可以直接将项目打包成几个有限、大小平衡的资源，并且自动进行。我将这种思考命名为`模块合并` （ `Module Merging` ）- 在全量打包和非打包之间找到平衡，只打包几个相关的模块以提高加载性能，同时不失去缓存颗粒度。

> 后来，我将`模块合并`更名为`局部打包`，因为我认为`局部打包`更能准确地表达我的想法。

## 局部打包规则
> 在这一节中，我们将通过示例介绍`局部打包`的基本规则。

首先，我们来看一个基本的React项目示例。对于一个基本的React项目，我们只在入口文件中导入react和react-dom：

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
├── index_9c07.49b83356.js      # contains react-dom
├── index_a35f.0ac21082.js      # contains ./index.tsx
├── index_b7e0.7ab9ca2d.js      # contains react and its dependencies
├── index_ce26.7f833381.css     $ contains ./index.scss
└── index.html                  # contains ./index.html
```

默认情况下，Farm会将你的项目打包成5个文件：

* 2个js文件来自 `node_modules` ，包含 `react` 、 `react-dom` 其依赖项。
* 1个js文件来自 `./index.tsx`
* 1个css文件来自 `./index.scss`;
* 1个html文件来自 `./index.html`;

Farm使用以下规则来获得上述结果：

1. **可变和不可变模块应始终位于不同的输出文件中**: 默认情况下，Farm 将 `node_modules` 下的所有模块视为不可变的，否则它们是可变的。因此  `./index.tsx` 位于单独的文件中，因为它是一个可变模块，因此它永远不会与 `react` 和 `react-dom` 位于同一输出文件中。
2. **不同类型的模块应始终位于不同的输出文件中**: 因此 `./index.scss` 位于单独的文件中。
3. **同一包中的模块应位于同一输出文件中**: 因此，所有 `react` 模块始终位于同一输出文件中， `react-dom` 也是如此。
4. **资源加载的目标并发请求应默认在20-30之间**: 因此有3个js输出文件，而不是1个js输出文件。
5. **输出文件应具有相似的大小，最小资源大小应默认大于20KB**: 因为 `react-dom` 是最大的，超过100KB，所以它位于单独的文件中，而 `react` 及其依赖项小于20KB，因此被合并到同一输出文件中。

现在我们已经熟悉了`局部打包`的基本规则，如果遇到局部打包问题，请使用上述规则调试您的项目。接下来，我们将介绍如何配置局部打包。

## 配置局部打包
### 两种配置方法

有两种不同的方式来控制打包：
* **`groups`**: 告诉Farm您希望将这些模块尽可能地打包在一起，但由于Farm的优化策略，这并不是强制执行的。请参阅[模块分组](#模块分组)以了解此方法。
* **`enforceResources`**: 告诉Farm您希望这些模块始终打包在一起，忽略所有其他优化策略约束。请参阅使用 [`enforceResources`](#使用-enforceresources) 以了解此方法。

### 局部打包选项

`局部打包`支持许多选项，使用户可以自定义其行为。所有选项如下：

1. **`targetConcurrentRequests`**: Farm尝试为初始资源加载或动态资源加载生成尽可能接近此配置值的资源数量。
2. **`targetMinSize`**: 生成资源的最小大小，在压缩和gzip之前。请注意，如果 `ModuleBucket的大小` 小于 `targetMinSize`， `ModuleBucket` 将优先考虑，这时候大小限制不一定会被强制保证。可以使用配置 `enforceTargetMinSize` 来强制保证大小，但是这样可能会导致一些共享模块的优化策略失效。
3. **`targetMaxSize`**: 类似 `targetMinSize`，生成资源的最大大小，在压缩和gzip之前。
4. **`groups`**: 一组应该放在一起的模块。请注意，此组配置只是告诉编译器这些模块应该放在一起，它可能会产生多个资源，如果您想强制将模块放在同一资源中，应该使用 `enforceResources`。
    * **name**: 这组资源的名称.
    * **test**: 匹配属于该组的模块的正则表达式数组。
    * **groupType**: `mutable` 或 `immutable` ，此组仅用于指定模块的类型。
    * **resourceType**: `all`、 `initial` 或 `async`，此组仅用于指定资源的类型。
5. **`enforceResources`**: 匹配应该始终位于同一输出资源中的模块的数组，忽略所有其他约束。
    * **name**: 资源的名称.
    * **test**: 匹配属于该资源的模块的正则表达式数组。
6. **`enforceTargetConcurrentRequests`**: 强制目标并发请求对于每个资源加载，当为true时，较小的资源将被合并到较大的资源中以满足目标并发请求。这可能会导致css资源出现问题，请谨慎使用此选项。
7. **`enforceTargetMinSize`**: 强制设置对于每个资源的目标最小大小，当为true时，较小的资源将被合并到较大的资源中以满足目标并发请求。这可能会导致css资源出现问题，请谨慎使用此选项。
8. **`immutableModules`**: 匹配不可变模块的正则表达式数组。
9. **`immutableModulesWeight`**: 默认为0.8，不可变模块将具有80%的请求数量。例如，如果 `targetConcurrentRequest` 为25，则不可变资源将默认为 `25 * 80% = 20` 。此选项是为了确保可变和不可变模块是隔离的，如果您更改了业务代码，node_modules下的代码将不会受到影响。

:::note
通常，您可以使用 `targetConcurrentRequests` 、 `targetMinSize` 和 `targetMaxSize` 来控制局部打包的默认行为。Farm设置的默认值基于最佳实践，因此请确认是否必须修改默认值。
:::

### 模块分组
您可以使用 `groups` 将模块分组在一起。对于上述基本React项目示例，可以使用以下配置将 `node_modules` 下的模块打包在一起：
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
我们添加了一个 `group item` ，其中包含 `name` 和 `test` ，以将 `react` 和 `react-dom` 分组在一起。打包结果如下：
```
./dist/
├── index_499e.72cf733c.js    # contains `react`, `react-dom` and all other files under node_modules
├── index_a35f.0ac21082.js    # contains `./index.tsx`
├── index_ce26.7f833381.css   # contains `./index.scss`
└── index.html                # contains `./index.html`
```

现在， `node_modules` 下的所有模块都打包到 `index_499e.72cf733c.js` 中。请注意，groups并不强制所有匹配该组的模块都打包在一起，一个 `group`可以产生多个 `output file` ，因为：
1. 可变和不可变模块始终位于不同的输出文件中。当可变和不可变模块都匹配到这个 `group` 时，它们将位于不同的输出中。
2. 对于多页面应用或动态导入的入口，可能存在共享模块，这些模块应始终位于不同的输出文件中。

如果您需要强制将模块放在同一输出文件中，可以使用 `enforceResources`

### 使用 `enforceResources`
要将所有模块分组在一起并忽略所有其他条件，可以使用 `enforceResources` ，例如：
```ts title="farm.config.ts"
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

打包结果:

```
./dist/
├── index.7f833381.css  # all css modules are bundled together
├── index.ba5550d9.js   # all script modules are bundled together
└── index.html
```

:::warning
`enforceResources` 将忽略Farm的所有内部优化，使用时请小心。
:::

### 配置 `immutable modules`

使用 `immutableModules` 配置不可变模块，默认情况下，Farm将其设置为 `node_modules/` 。

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    partialBundling: {
      immutableModules: ['node_modules/', '/global-constants']
    },
  },
});
```

不可变模块会影响打包和传入的持久化缓存，如果您想修改它，请小心。

## 示例
:::note
通常您不需要手动配置打包，如果您想手动配置打包，请确保您确实需要它。这些示例仅用于帮助您轻松学习如何配置打包策略。
:::

### 将同一目录下的文件分组

将 `src/components` 下的 `modules` 分组，并**尽可能**将它们输出到同一资源中。

```ts title="farm.config.ts"
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      groups: [
        {
          name: 'components',
          test: ['./src/components'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

### 配置打包的数量和大小
```ts title="farm.config.ts"
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      targetConcurrentRequests: 15,
      targetMinSize: 200 * 1024 // 200 KB
      // c-highlight-end
    },
  },
});
```

在上面的示例中，Farm将尝试**尽可能**地将您的项目打包到 `15` 个文件中，每个文件的最小大小**尽可能**大于 `200KB` 。

### 将所有模块打包在一起
```ts
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

在上面的示例中，我们强制将所有模块打包在一起，并忽略所有其他约束（例如，请求数量、文件大小）。您也可以使用 `enforceResources` 强制将某些模块打包在一起：

```ts
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['\\./src/components/.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

我们强制将 `src/components` 目录下的所有模块打包在一起。

:::note
`enforceResources` 会破坏打包的内部优化，使用时请小心。
:::
