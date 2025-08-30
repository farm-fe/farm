# 产物压缩

Farm 支持开箱即用的生产压缩，默认情况下在生产中自动启用，可以通过[compilation.minify](/zh/docs/config/compilation-options#minify) 选项启用或禁用。

```ts title="farm.config.ts"
export default {
  compilation: {
    // enable minification for both development and production
    minify: true,
  },
};
```

如果启用压缩:

- 对于 js/ts 模块，代码将被`compressed`和 `mangled`，所有空白字符将被删除.
- 对于css和html模块，所有空格都将被删除

:::note

Farm 使用 swc minifier，有关详细选项，请参阅[compilation.minify](/zh/docs/config/compilation-options#minify)
:::
