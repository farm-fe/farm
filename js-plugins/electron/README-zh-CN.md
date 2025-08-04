<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <a href="./README.md">English</a> |
    <span>简体中文</span>
</div>

---

# Farm electron 插件

支持使用 Farm 开发 Electron 应用。

## 入门

首先，您需要 `@farmfe/js-plugin-electron`：

```bash
npm install @farmfe/js-plugin-electron --save-dev
```

或者

```bash
纱线添加-D @farmfe/js-plugin-electron
```

或者

```bash
pnpm add -D @farmfe/js-plugin-electron
```

在 `farm.config.ts` 中配置插件：

```ts
import { defineConfig } from "farm";
import electron from "@farmfe/js-plugin-electron";

import { defineConfig } from "farm";
import electron from "./farm-plugin-electron";

export default defineConfig({
  plugins: [
    electron({
      main: {
        input: "electron/main.ts",
      },
      preload: {
        input: "electron/preload.ts",
      },
    }),
  ],
});
```

## 选项

```ts
import type { UserConfig } from "farm";

export interface BuildOptions {
  /**
   * `compilation.input` 的别名
   */
  input: string | Record<string, string>;
  farm?: UserConfig;
}

export interface ElectronPluginOptions {
  main: BuildOptions;
  preload?: BuildOptions;
}
```
