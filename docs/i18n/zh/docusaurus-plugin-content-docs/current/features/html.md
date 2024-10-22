---
sidebar_position: 1
---

# Html

## 基本用法

Farm 支持开箱即用地编译 Html，并且在构建 Web 项目时应该使用 Html 作为入口，例如：

```ts title="farm.config.ts"
import type { UserConfig } from "@farmfe/core";

export default defineConfig({
  input: {
    index: "./index.html", // using ./index.html as entry
  },
});
```

:::note
如果未指定 `input`，则默认为 `{index: './index.html'}`。
:::

在`./index.html`中，应该使用`<script src="./xxx">`来引用您的入口 `Js/Ts/Jsx/Tsx` 文件。

```html title="./index.html"
<html>
  <!-- ... -->
  <body>
    <div id="root"></div>
    <!-- index.ts is the script entry -->
    <script src="./index.ts"></script>
  </body>
</html>
```

你也可以使用`<link href="./xxx">`来引用你的全局 CSS。

Farm 在编译时会将这些 `script` 和 `link` 转化为最终的生产可用的产物。请注意，当您想引用本地模块时，必须使用 `相对路径`，例如 `<script src="./index.tsx"></script>` 将引用本地模块并编译它， 但 `<script src="/index.tsx"></script>` 或 `<script src="https://xxx.com/index.tsx"></script>` 则不会。

:::note
`script` 和 `link` 可以引用 farm 支持的任何模块类型，例如，`js`、`jsx`、`ts`、`tsx` 或插件支持的其他模块类型。 您可以根据需要使用任意数量的 `script` 或 `link`。
:::

## 多页面应用程序 - MPA

如果您正在构建多页面应用程序，只需配置多个 html，例如：

```ts title="farm.config.ts"
import type { UserConfig } from '@farmfe/core';

export default defineConfig({
  input: {
    home: './index.html', // Home Page
    about: './about.html', // About Page
    // ... more pages
  }
})
```

Farm 将并行编译这些页面。

## 继承 html 模板

Farm 支持通过使用 `html.base` 配置继承 html 模板，这在构建共享 html 的多页面应用程序时很有帮助。

```ts title="farm.config.ts"
import type { UserConfig } from "@farmfe/core";

export function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // ...
  compilation: {
    input: {
      home: "./index.html", // Home Page
      about: "./about.html", // About Page
      // ... more pages
    },
    html: {
      base: "./base.html",
    },
  },
});
```

然后添加一个`base.html`，占位符`{{children}}`将被替换为子 html 的内容。

```html title="./base.html"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body>
    <div id="root"></div>
    <!-- 占位符将会在编译时替换成对应的子 html 的内容 -->
    {{children}}
  </body>
</html>
```

继承`./base.html`：

```html title="./src/home.html"
<!-- 其他字段继承自../base.html -->
<script src="./index.tsx"></script>
```
