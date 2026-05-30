# 插件概览

Farm官方提供了很多有用的插件，包括Rust插件和JS插件。 Rust 插件比 Js 插件快得多，我们建议尽可能使用 Rust 插件。

:::tip
关于如何在 Farm 中使用插件，请参阅[使用插件](/docs/using-plugins)。
:::

## Rust 插件 {#rust-plugins}

* **[`@farmfe/plugin-react`](./react)**：支持 React `jsx` 和 `react-refresh`。
* **[`@farmfe/plugin-vue`](./vue)**：使用 `fervid` Rust 编译器编译 Vue 3 单文件组件。
* **[`@farmfe/plugin-sass`](./sass)**：支持编译`sass/scss`文件。
* **[`@farmfe/plugin-auto-import`](./auto-import)**：从预设和本地导出中扫描并自动注入导入。
* **[`@farmfe/plugin-tailwindcss`](./tailwindcss)**：基于 Rust 的 TailwindCSS 集成。
* **[`@farmfe/plugin-svgr`](./svgr)**：将 SVG 文件转换为 React 组件。
* **[`@farmfe/plugin-wasm`](./wasm)**：支持 WebAssembly 和 `wasm-pack` 生成的模块。
* **[`@farmfe/plugin-worker`](./worker)**：支持通过构造函数和 worker 查询后缀使用 Web Worker。
* **[`@farmfe/plugin-url`](./url)**：将文件作为 data URI 或产物资源 URL 导入。
* **[`@farmfe/plugin-dts`](./dts)**：为匹配的 TypeScript 模块生成 `.d.ts` 文件。
* **[`@farmfe/plugin-icons`](./icons)**：按需使用 Iconify 图标作为框架组件。
* **[`@farmfe/plugin-image`](./image)**：将图片文件作为 base64 data URI 或 DOM `Image` 导出导入。
* **[`@farmfe/plugin-mdx`](./mdx)**：将 `.md` 和 `.mdx` 文件编译为 JSX。
* **[`@farmfe/plugin-compress`](./compress)**：使用 Brotli、Gzip、Deflate 或 DeflateRaw 压缩生成的资源。
* **[`@farmfe/plugin-modular-import`](./modular-import)**：将 UI 库的具名导入转换为按组件导入。
* **[`@farmfe/plugin-replace-dirname`](./replace-dirname)**：在脚本模块中替换 `__dirname`、`__filename` 和 `import.meta.url`。
* **[`@farmfe/plugin-strip`](./strip)**：一个Farm的Rust插件，用于从你的代码中移除`debugger`语句和类似`assert.equal`、`console.log`这样的函数。
* **[`@farmfe/plugin-dsv`](./dsv)**：一个Farm插件，用于将`.csv`和`.tsv`文件转换为JavaScript模块。
* **[`@farmfe/plugin-yaml`](./yaml)**：一个Farm插件，用于将YAML文件转换为ES6模块。
* **[`@farmfe/plugin-virtual`](./virtual)**：一个方便在farm中使用虚拟模块的rust插件。
* **[`@farmfe/plugin-react-components`](./react-components)**：用于React的按需组件自动导入。

## Js 插件 {#js-plugins}

* **[`@farmfe/js-plugin-postcss`](./js-postcss)**：支持在项目中使用 PostCSS。
* **[`@farmfe/js-plugin-babel`](./js-babel)**：对匹配的 Farm 模块运行 Babel。
* **[`@farmfe/js-plugin-copy`](./js-copy)**：在 Farm 构建结束时复制文件和 glob 匹配结果。
* **[`@farmfe/js-plugin-less`](./js-less)**：支持编译 `less` 文件。
* **[`@farmfe/js-plugin-svgr`](./js-svgr)**：支持将 SVG 文件编译为组件。
* **[`@farmfe/js-plugin-dts`](./js-dts)**：支持生成 `*.d.ts` 类型声明文件。
* **[`@farmfe/js-plugin-sass`](./js-sass)**：支持编译`sass/scss`文件。
* **[`@farmfe/js-plugin-tailwindcss`](./js-tailwindcss)**：支持 TailwindCSS 集成。
* **[`@farmfe/js-plugin-visualizer`](./js-visualizer)**：可视化分析产物大小与组成。
* **[`@farmfe/js-plugin-electron`](./js-electron)**：支持构建 Electron 应用。
* **[`@farmfe/js-plugin-qiankun`](./js-qiankun)**：将 Farm 应用暴露为 qiankun 微应用。
* **[`@farmfe/js-plugin-react-compiler`](./js-react-compiler)**：通过 Babel 为 JSX/TSX 模块运行 React Compiler。
* **[`@farmfe/js-plugin-vuetify`](./js-vuetify)**：支持 Vuetify 自动导入和样式解析。

## 社区插件

如果官方插件不能满足您的需求，您可以尝试[社区插件](../community-plugins)。

当然也可以前往查看 [awesome-farm](https://github.com/farm-fe/awesome-farm) - 您也可以提交 PR，在那里列出您的插件。
