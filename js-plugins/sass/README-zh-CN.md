<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/sass/README.md">English</a> |
    <span>简体中文</span>
</div>

---

# Sass Plugin for Farm

本文介绍如何在Farm中使用Sass插件进行编译Sa(c)ss文件。

## 使用方法


通过你最喜欢的包管理器（npm、yarn、pnpm 等）安装`@farmfe/js-plugin-sass`：

```bash
npm i @farmfe/js-plugin-sass --save-dev # or pnpm/yarn add @farmfe/js-plugin-sass -D
```

在`farm.config.ts`中配置插件：

```ts
import { defineFarmConfig } from 'farm/dist/config';
import Sass from '@farmfe/js-plugin-sass'; //  import the plugin

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
  plugins: [
    // use the sass plugin.
    Sass({
      // custom options here
    }),
  ],
});
```

## 配置项
### implementation

类型: `string | undefined`

默认值: `undefined`

指定sass文件的执行器(如sass,sass-embedded),如果未定义，则默认查找node_module中的文件

### match

类型: `string[]`

默认值: `["\\.s[ac]ss$"]`

指定匹配的文件

### globals

类型: `string[]`

默认值: `[]`

将读取文件中的内容并注入到每个 sass/scss 文件中, 它通常用来注入一些全局变量

> 注意，该文件中不要写入正常的 css, 否则它会将它们重复的注入到各个编译后的 css 文件中

### content

类型: `string | undefined`

默认值: `undefined`

它与 `globals` 的作用一致, 但它可以很方便的使用, 通常注入一些简单的 sass/scss 内容

### sourceMap

类型: `boolean`

默认值: `false`

是否生成 sourceMap

> 在没有设置的情况下, 它会去读取 farm 配置中 `compilation.sourcemap` 的配置

### sassOption

类型: `StringOptions<'async'>`

默认值: `{}`
