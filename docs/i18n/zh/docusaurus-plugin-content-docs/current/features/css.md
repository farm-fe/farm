# Css/Sass/Less
Farm 支持开箱即用的 CSS 编译，例如：

```tsx
import './index.css';
```

然后 Farm 会自动为 css 模块启用 HMR，并自动打包 Css。

## CSS Modules
Farm 默认支持 css modules，以 `.module.css|less|scss|sass` 结尾的模块默认将被视为 `Css Modules`。

```tsx title="comp.tsx"
import styles from './index.module.css'

export function Comp() {
  return <div className={styles.main}>Main</div>
}
```
```css title="index.module.css"
.main {
  color: red;
}
```

您可以通过[`css.modules`](/docs/config/farm-config#cssmodules)配置CSS模块。 例如，您可以将 `css.modules.paths` 设置为 `['.css|sass|less|scss']` 那么所有 css 文件将被视为 css 模块。

## CSS 预处理器
Farm 官方提供了 sass、less、postcss 插件。

### Sass
Farm Sass 插件是一个 Rust 插件，使用 `sass-embeded`（后面我们可能会迁移到纯 Rust 编写的 [`grass`](https://github.com/connorskees/grass)）。

在 Farm 中编译 `sass/scss` 模块的步骤如下：

1. 安装依赖
```sh
# npm 或者 yarn 或者 pnpm，使用任意你喜欢的包管理器 
npm install @farmfe/plugin-sass
```

2. 配置插件
```ts
import type { UserConfig } from '@farmfe/core';

export default <UserConfig> {
  // ...
  plugins: ['@farmfe/plugin-sass'] // 配置 Rust 插件的包名即可引入和使用该插件
  // 如果你希望配置 plugin-sass 的参数，可以使用如下形式的配置
  // plugins: [
  //   ['@farmfe/plugin-sass', { sourceMap: false }]
  // ]
};
```

3. 导入sass模块
```ts
import './index.scss';
```

如果要将 `sass` 与 `css modules` 一起使用，请将文件名从 `index.scss` 更改为 `index.module.scss`，请参阅 [css modules](#css-modules)。

`@farmfe/plugin-sass` 支持很多选项，使用 plugins 的数组配置指定插件 sass 的选项：

```ts
import type { UserConfig } from '@farmfe/core';

export default <UserConfig> {
  plugins: [
    // 通过数组语法指定插件以及配置
    [
      '@farmfe/plugin-sass',
      // 所有支持的选项如下
      {
        sourceMap: true // bool
        sourceMapIncludeSources: true, // bool
        alertAscii: true, // bool
        alertColor: true, // bool
        charset: true, // bool
        quietDeps: true, // bool
        verbose: false, // bool
        style: 'expanded' | 'compressed' // output code style
      }
    ]
  ]
};
```

### Less
Farm less 插件是一个 Js 插件。 在 Farm 中编译 `less` 模块的步骤如下：

1. 安装依赖
```sh
# npm or yarn or pnpm, choose your favorite package manager
npm install @farmfe/js-plugin-less
```

2. 配置插件
```ts
import type { UserConfig } from '@farmfe/core';
import less from '@farmfe/js-plugin-less';

export default <UserConfig> {
  // ...
  plugins: [less()] // pass argument to the less function like `less({ /* your options */ })` to specify less options
};
```

3. 导入 Less 模块
```ts
import './index.less';
```

要将 `less` 与 `css modules` 一起使用，请将文件名从 `index.less` 更改为 `index.module.less`，参考 [css modules](#css-modules)

### Postcss
Farm postcss 插件是一个 JS 插件，在 Farm 中引入 postcss 的步骤如下：

1. 安装依赖
```sh
# npm or yarn or pnpm, choose your favorite package manager
npm install @farmfe/js-plugin-postcss
```

2. 配置插件
```ts
import type { UserConfig } from '@farmfe/core';
import postcss from '@farmfe/js-plugin-postcss';

export default <UserConfig> {
  // ...
  plugins: [postcss()] // pass argument to the less function like `less({ /* your options */ })` to specify less options
};
```

3. 配置 `postcss.config.js`，引入需要的 postcss 插件

```js title=postcss.config.js
module.exports = {
  plugins: [
    require('postcss-pxtorem')({
      rootValue: 16,
      propList: ['*'],
    }),
    require('tailwindcss'),
  ]
}
```

## Css Prefixer
Farm 支持开箱即用的 css prefixer，您可以使用`compilation.css.prefixer`对其进行配置。

```ts title="farm.config.ts"
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
   return config;
}

export default defineConfig({
   compilation: {
     css: {
       prefix: {
        targets: ['ie >= 10']
       }
     },
   },
});
```
对于输入代码
```css
div {
  display: flex;
}
```
输出
```css
div{display:-ms-flexbox;display:flex}
```
