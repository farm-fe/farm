# 懒编译
当涉及到一个大项目时，您可能希望将它们分成小块并按需加载。 这可以通过动态导入来实现。

````js
const page = React.lazy(() => import('./page')); // 延迟加载页面
````

默认情况下，Farm 会在开发时延迟编译这些动态导入，仅在模块真正执行时才编译它们。 延迟编译可以极大提速大型项目的编译。

:::note
对于生产构建，延迟编译始终被禁用。
:::

请注意，正确使用`动态导入`对于使`懒编译`更好地工作非常重要。 例如，如果你的一个页面有一个很大的依赖项，但是这个依赖项在这个页面渲染之前不会被使用，那么有必要确保这个大的依赖项是动态导入的，所以它不会被编译，直到页面执行。

## 配置延迟编译
使用`compilation.lazyCompilation`来启用或禁用它：

```ts title="farm.config.ts"
export default {
   compilation: {
     lazyCompilation: true,
   },
};
```

## 懒编译如何工作
当启用延迟编译时，Farm 将首先分析您的所有`动态导入`，例如：

```js
const page = React.lazy(() => import('./page'));
```
Farm 会将 `./page` 视为应该延迟编译的模块，并且不会编译它，相反，Farm 将为 `./page` 返回一个虚拟占位符模块，如下所示：

```ts
// ... other actions
const compilingModules = FarmModuleSystem.compilingModules;
// 返回一个promise，这个promise将在延迟编译完成后 resolve。
let promise = Promise.resolve();

// 模块已经在懒编译中
if (compilingModules.has(modulePath)) {
  promise = promise.then(() => compilingModules.get(modulePath));
} else {
 // 请求开发服务器进行延迟编译
  const url = '/__lazy_compile?paths=' + paths.join(',') + `&t=${Date.now()}`;
  promise = import(url).then((module: any) => {
      const result: LazyCompileResult = module.default;
      // ...
  });
  // ... more actions
}

export const __farm_async = true;
export default promise;
```

上面的例子说明了虚拟占位符模块的基本结构。 当占位符执行时，它将请求开发服务器编译该模块及其依赖项。 从开发服务器获取延迟编译结果后，占位符模块会将这些更改修补到 Farm 的运行时模块系统。