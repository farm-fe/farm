<div align="center">
  <a href="./README.md">English</a> | <a href="./README.zh-CN.md">简体中文</a>
</div>

# @farmfe/plugin-svgr [![npm version](https://badgen.net/npm/v/@farmfe/plugin-svgr)](https://npm.im/@farmfe/plugin-svgr)

---

一个用于将 SVG 文件转换为 React 组件的 Farm 插件。

## 特性

- 将 SVG 文件转换为 React 组件
- 支持 SVG 优化
- 保持 SVG 属性作为 React props

## 安装

```bash
npm i -D @farmfe/plugin-svgr
```

## 使用方法

创建一个 `farm.config.ts` [配置文件](https://www.farmfe.org/docs/config/configuring-farm) 并导入插件：

```ts
import { defineConfig } from "@farmfe/core";
import svgr from "@farmfe/plugin-svgr";

export default defineConfig({
  plugins: [
    svgr({
      // 插件配置选项
      include: ["src/**/*.svg"], // 可选：SVG 文件包含模式
      exclude: ["src/icons/*.svg"], // 可选：SVG 文件排除模式
      defaultStyle: { fill: "currentColor" }, // 可选：SVG 的默认样式
      defaultClass: "svg-icon", // 可选：SVG 的默认类名
    }),
  ],
});
```

## 示例

基础用法：

```jsx
import Logo from "./logo.svg";

function App() {
  return (
    <div>
      <Logo width={50} height={50} />
    </div>
  );
}
```

## 文档

有关插件更详细的文档，请访问 [Farm 官方文档](https://www.farmfe.org/docs/plugins/official-plugins/overview)。

## 开源协议

MIT
