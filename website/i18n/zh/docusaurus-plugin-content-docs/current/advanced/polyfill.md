# 语法降级和 Polyfill

默认情况下，Farm 将降级到 ES2017(原生支持 async/await)，并在生产模式下自动注入必要的`polyfills`

:::note
默认情况下，Farm 不会为 `node_modules/` 下的模块进行转换和注入 polyfills ，如果您需要降级语法并为 `node_modules/` 注入 polyfills ，您可以使用 `compilation.presetEnv.include`
:::

## 配置 `targetEnv`

Farm 提供一个规范化的[`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv)选项来配置你的应用程序的目标执行环境。Farm 会自动为你的目标环境执行正确的`syntax downgrade`和`polyfill injection`。例如：

```ts title="farm.config.ts"
export default {
  compilation: {
    output: {
      targetEnv: 'browser-legacy',
    },
  },
};
```

Farm 会将您的应用程序编译为旧版浏览器（ES5）

- 将所有的 `Js/Jsx/Ts/Tsx`模块编译到 ES5，并注入所有的 polyfills（Promise，regenerator-runtime 等）
- 为所有 `css/scss/less` 模块添加前缀，例如`--webkit-`

Farm 支持许多规范化的`targetEnv`选项，如`browser-modern`，`browser-es2017`，`browser-es2015`，`node16`，`node-legacy`等。**默认情况下，`targetEnv`是`browser-es2017`**。请参阅[`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv)

:::note
您可能需要手动安装 `core-js@3`或`regeneration-runtime`，如果需要 polyfill 。尝试运行`pnpm add core-js` 如果你遇到了一些错误例如：`can not resolve 'core-js/modules/xxx'`
:::

## 分别配置语法和 Polyfill

在内部，`targetEnv`只是 `presetEnv`、`script.target` 和 `css.prefixer`的预设。如果需要，您可以更精确地配置它们

### 配置 `presetEnv`

您可以使用 `compilation.presetEnv` 自定义语法降级和 polyfill injection。默认情况下，`node_modules`下的所有模块都将被忽略。使用`include`添加需要被 polyfill 的额外模块。

```ts title="farm.config.ts"
export default {
  compilation: {
    presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: 'Chrome >= 48',
      },
    },
  },
};
```

请注意，如果您的项目不需要浏览器兼容性，您可以为 `targets` 设置一个较宽松的值，那么注入的 polyfills 会更少，产物体积也会更小

有关更多选项，请参阅[compilation.presetEnv](/docs/config/compilation-options#presetenv)

### 配置 `script.target`

`script.target` 用于在生成代码时控制目标环境。如果要将项目降级为 `ES5`，则应同时设置：

```ts title="farm.config.ts"
export default {
  compilation: {
    script: {
      target: 'ES5',
    },
    presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: '> 0.25%, not dead',
      },
    },
  },
};
```
