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

## 高级压缩选项

你可以向 `minify` 传递一个对象进行细粒度控制：

```ts title="farm.config.ts"
export default {
  compilation: {
    minify: {
      compress: true,
      mangle: true,
      include: ['src/**'],       // 仅压缩匹配这些模式的文件
      exclude: ['src/vendor/**'], // 跳过匹配这些模式的文件
    }
  },
};
```

### `include` / `exclude`
使用 `include` 和 `exclude`（正则表达式模式）来控制哪些模块被压缩。默认情况下，启用压缩时所有模块都会被压缩。

### `compress` 和 `mangle`
`compress` 和 `mangle` 都接受 `true`、`false` 或详细的 SWC 压缩器选项对象，用于高级自定义。

:::note

Farm 使用 swc minifier，有关详细选项，请参阅[compilation.minify](/zh/docs/config/compilation-options#minify)
:::
