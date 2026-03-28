# Source Map
Farm 支持 Source Map，默认情况下自动启用。 可以通过选项启用或禁用 sourcemap。

:::note
Farm 默认不会为 node_modules 下的文件生成 sourcemap，如果你想为 node_modules 下的文件生成 sourcemap，请将 `compilation.sourcemap` 配置为`all`。
:::

使用`compilation.sourcemap`配置 sourcemap 生成：
```ts title="farm.config.ts"
export default {
   compilation: {
     sourcemap: 'all', // generate sourcemap for modules under node_modules
   },
};
```

所有选项如下：
* **`true`**：只为不在`node_modules`下的文件生成 sourcemap，并生成单独的 sourcemap 文件
* **`false`**：禁用源映射
* **`inline`**：只为不在`node_modules`下的文件生成 sourcemap，并将 sourcemap 内联到产物中，不生成单独的文件
* **`all`**：为所有文件生成 sourcemap，并生成单独的 sourcemap 文件
* **`all-inline`**：为所有文件生成 sourcemap，并将 sourcemap 内联到产品中，不生成单独的文件