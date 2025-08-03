<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/less/README.md">English</a> |
    <span>简体中文</span>
</div>

---

# Less Plugin for Farm

本文介绍如何在Farm中使用Less插件进行编译Less文件。

## 开始使用

首先需要安装`less`和`@farmfe/js-plugin-less`依赖：

```console
npm install less @farmfe/js-plugin-less --save-dev
```

or

```console
yarn add -D less @farmfe/js-plugin-less
```

or

```console
pnpm add -D less @farmfe/js-plugin-less
```

然后在`farm.config.ts`中配置插件：

```ts
import { defineFarmConfig } from "farm/dist/config";
//引入该插件
import Less from "@farmfe/js-plugin-less";

export default defineFarmConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
    },
  },
  plugins: [
    Less({
      //在这里进行配置
    }),
  ],
});
```

## Options

- **[`lessOptions`](#lessoptions)**
- **[`additionalData`](#additionalData)**
- **[`sourceMap`](#sourcemap)**
- **[`implementation`](#implementation)**

### lessOptions

Type:

```ts
type lessOptions = import('less').options | ((loaderContext: LoaderContext) => import('less').options})
```

Default: `{ relativeUrls: true }`

在这里，您可以将任何Less选项传递给`@farm/js-plugin-less`。可以参考[Less options](https://lesscss.org/usage/#less-options) 以了解您需要的任何可用选项。

### additionalData

Type:

```ts
type additionalData =
  | string
  | ((content: string, resolvePath: string) => string);
```

Default: `undefined`

将`Less`代码附加到实际入口文件。在这种情况下，`@farm/js-plugin-less`不会覆盖源代码，而只是将代码**前置**到入口文件的内容中。

在实际开发中，这很有用，因为我们不再需要添加新的文件。

> 由于您正在注入代码，因此这将破坏您入口文件中的源映射。通常有比这更简单的解决方案，例如多个Less入口文件。

#### `string`

```ts
export default defineFarmConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
    },
  },
  plugins: [
    // use the less plugin.
    Less({
      additionalData: `@hoverColor: #f10215;`,
    }),
  ],
});
```

#### `function`

```ts
export default defineFarmConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
    },
  },
  plugins: [
    farmLessPlugin({
      additionalData: (content: string, resolvePath: string) => {
        if (path.basename(resolvePath, ".less") === "index") {
          return `@hoverColor: #f10215;` + content;
        }
      },
    }),
  ],
});
```

### sourceMap

Type: `boolean`

Default: `false`

是否生成 sourceMap

> 如果没有设置，则会读取 farm 配置中的 compilation.sourcemap 配置

### implementation

Type: `string | undefined`

Default: `undefined`

> `@farm/js-plugin-less` 兼容 Less 3 和 4 版本

特殊的`implementation`选项确定要使用的 Less 实现。如果你没有配置，它会在本地的`node_modules`中查找 `less`
