# 压缩
Farm 支持开箱即用的生产环境压缩，默认情况下会自动在`production`模式下启用。 可以通过`compilation.minify`选项启用或禁用。

使用`compilation.minify`进行配置：
```ts title="farm.config.ts"
export default {
   compilation: {
     minify: true
   },
};
```

如果启用了压缩：
* 对于js/ts模块，它将被压缩以及混淆，所有空白字符将被删除，变量等将被压缩。
* 对于 css 和 html 模块，所有空格都将被删除。

:::note
Farm 底层使用swc minifier，swc minifier 的所有选项都可以在 Farm 中使用。
:::
