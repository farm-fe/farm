# 语法降级和 Polyfill
默认情况下，Farm 将降级到`ES5`并在生产模式下自动注入`polyfills`。

:::note
默认情况下，Farm 不会对 `node_modules/` 下的模块进行转换并注入 polyfill，如果您需要为 `node_modules/` 降级语法并注入 polyfill，您可以使用 `compilation.presetEnv.include`。
:::

## 配置 `presetEnv`
您可以使用`compilation.presetEnv`来自定义语法降级和 polyfill。 使用 include 添加需要注入 polyfill 的额外模块

```ts title="farm.config.ts"
export default {
   compilation: {
     presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: "Chrome >= 48"
      }
     }
   },
};
```

默认情况下，Farm 会将目标设置为`> 0.25%, not dead`。 如果你的项目不需要浏览器兼容性，你可以为`targets`设置一个更宽松的值，那么注入的 polyfills 就会更少，输出的资源大小也会更小。

更多选项，请参阅 [compilation.presetEnv](/docs/config/farm-config#presetenv)。

## 使用 `script.target`
`script.target` 也可以在生成代码时控制目标环境。 如果您想将项目降级到`ES5`，您应该同时设置：

```ts title="farm.config.ts"
export default {
   compilation: {
     script: {
      target: 'ES5'
     },
     presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: "> 0.25%, not dead"
      }
     }
   },
};
```