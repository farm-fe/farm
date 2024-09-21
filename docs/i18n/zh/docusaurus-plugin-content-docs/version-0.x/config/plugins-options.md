## Plugins Options

配置 Farm 的插件，支持 Rust 插件或者 Js 插件，示例如下：

```ts
import { defineConfig } from "@farmfe/core";
import less from "@farmfe/js-plugin-less";
export default defineConfig({
  plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass", less()],
});
```

### Rust Plugins

- **默认值**: `[]`

Rust 插件通过 `插件名`或者 `[插件名, 配置项选项]` 方式配置，如下：

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  plugins: [
    [
      "@farmfe/plugin-react",
      {
        /* options here */
      },
    ],
    "@farmfe/plugin-sass",
  ],
});
```

### Js Plugins

- **默认值**: `[]`

Js 插件就是一个对象，具体可以参考 [Js 插件](/docs/plugins/js-plugin)
