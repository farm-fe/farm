# 语法降级和 Polyfill

在生产模式下选择带版本的浏览器 [`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv)（例如 `browser-es2017`、`browser-es2015` 或 `browser-legacy`）时，Farm 可以自动降级语法并注入 polyfill。

:::note
默认情况下，Farm 不会为 `node_modules/` 下的模块进行转换和注入 polyfills ，如果您需要降级语法并为 `node_modules/` 注入 polyfills ，您可以使用 `compilation.presetEnv.include`
:::

## 配置 `targetEnv`

Farm 提供一个规范化的 [`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv) 选项来配置应用程序的目标执行环境。带版本的浏览器目标会启用对应的语法降级和 polyfill 注入。例如：

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
Farm 支持许多规范化的 `targetEnv` 选项，如 `browser-esnext`、`browser-es2017`、`browser-es2015`、`browser-legacy`、`node16`、`node-legacy` 和 `node-next`。需要特定兼容性目标时请显式设置 `targetEnv`。请参阅 [`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv)。

:::note
您可能需要手动安装 `core-js@3`或`regeneration-runtime`，如果需要 polyfill 。尝试运行`pnpm add core-js` 如果你遇到了一些错误例如：`can not resolve 'core-js/modules/xxx'`
:::

## 分别配置语法和 Polyfill

在内部，`targetEnv` 会选择生产兼容性默认值，例如 `presetEnv`，以及带版本浏览器目标对应的 `script.target`。如果需要，您可以更精确地配置这些选项。

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
