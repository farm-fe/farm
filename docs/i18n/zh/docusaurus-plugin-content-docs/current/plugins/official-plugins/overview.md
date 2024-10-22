# 插件概览

Farm官方提供了很多有用的插件，包括Rust插件和JS插件。 Rust 插件比 Js 插件快得多，我们建议尽可能使用 Rust 插件。

:::tip
关于如何在 Farm 中使用插件，请参阅[使用插件](/docs/using-plugins)。
:::

## Rust 插件

* **[`@farmfe/plugin-react`](./react)**：支持 React `jsx` 和 `react-refresh`。
* **[`@farmfe/plugin-sass`](./sass)**：支持编译`sass/scss`文件。
* **[`@farmfe/plugin-strip`](./strip)**：一个Farm的Rust插件，用于从你的代码中移除`debugger`语句和类似`assert.equal`、`console.log`这样的函数。
* **[`@farmfe/plugin-dsv`](./dsv)**：一个Farm插件，用于将`.csv`和`.tsv`文件转换为JavaScript模块。
* **[`@farmfe/plugin-yaml`](./yaml)**：一个Farm插件，用于将YAML文件转换为ES6模块。
* **[`@farmfe/plugin-virtual`](./virtual)**：一个方便在farm中使用虚拟模块的rust插件。
* **[`@farmfe/plugin-react-components`](./react-components)**：用于React的按需组件自动导入。

## Js 插件

* **[`@farmfe/js-plugin-postcss`](./js-postcss)**：支持 React `jsx` 和 `react-refresh`。
* **[`@farmfe/js-plugin-less`](./js-less)**：支持编译 `sass/scss` 文件。
* **[`@farmfe/js-plugin-svgr`](./js-svgr)**：支持编译`sass/scss`文件。
* **[`@farmfe/js-plugin-dts`](./js-dts)**：支持编译`sass/scss`文件。
* **[`@farmfe/js-plugin-sass`](./js-sass)**：支持编译`sass/scss`文件。

## 社区插件

如果官方插件不能满足您的需求，您可以尝试[社区插件](../community-plugins)。

当然也可以前往查看 [awesome-farm](https://github.com/farm-fe/awesome-farm) - 您也可以提交 PR，在那里列出您的插件。
