# Sass Plugin for Farm

Support compiling Sass/Scss in Farm.

## Usage

Install `@farmfe/js-plugin-sass` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-sass --save-dev # or pnpm/yarn add @farmfe/js-plugin-sass -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';
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

## Options
### implementation

类型: `string`

默认值: `''`

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

类型: `string`

默认值: `''`

它与 `globals` 的作用一致, 但它可以很方便的使用, 通常注入一些简单的 sass/scss 内容

### sourceMap

类型: `boolean`

默认值: `false`

是否生成 sourceMap

> 在没有设置的情况下, 它会去读取 farm 配置中 `compilation.sourcemap` 的配置

### sassOption

类型: `StringOptions<'async'>`

默认值: `{}`
