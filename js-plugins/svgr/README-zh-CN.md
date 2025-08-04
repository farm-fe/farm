<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/svgr/README.md">English</a> |
    <span>简体中文</span>
</div>

---

# Farm Svgr 插件

支持在 Farm 中将 Svg 编译为 React 组件。

## 入门

首先，您需要 `@farmfe/js-plugin-svgr`：

```bash
npm install @farmfe/js-plugin-svgr --save-dev
```

或者

```bash
纱线添加-D @farmfe/js-plugin-svgr
```

或者

```bash
pnpm add -D @farmfe/js-plugin-svgr
```

在 `farm.config.ts` 中配置插件：

```ts
import { UserConfig } from "farm";
import svgr from "@farmfe/js-plugin-svgr"; //  import the plugin

function defineConfig(config: UserConfig) {
  return config;
}

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
    // 配置 svgr 插件
    svgr({
      // 指定插件参数
    }),
  ],
});
```

## 选项

- **[`svgrOptions`](#svgroptions)**

### svgr 选项

参考 https://react-svgr.com/docs/options
