<div align="center">
  <a href="./README.md">English</a> | <a href="./README.zh-CN.md">简体中文</a>
</div>

# @farmfe/plugin-svgr [![npm version](https://badgen.net/npm/v/@farmfe/plugin-svgr)](https://npm.im/@farmfe/plugin-svgr)

---

A Farm plugin that transforms SVG files into React components.

## Features

- Convert SVG files to React components
- Support for SVG optimization
- Maintain SVG attributes as React props

## Installation

```bash
npm i -D @farmfe/plugin-svgr
```

## Usage

Create a `farm.config.ts` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```ts
import { defineConfig } from "@farmfe/core";
import svgr from "@farmfe/plugin-svgr";

export default defineConfig({
  plugins: [
    svgr({
      // Plugin options
      include: ["src/**/*.svg"], // Optional: Include patterns for SVG files
      exclude: ["src/icons/*.svg"], // Optional: Exclude patterns for SVG files
      defaultStyle: { fill: "currentColor" }, // Optional: Default style for SVG
      defaultClass: "svg-icon", // Optional: Default class name for SVG
    }),
  ],
});
```

## Example

Basic usage:

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

## Documentation

For more Plugins detailed documentation, please visit [Farm official documentation](https://www.farmfe.org/docs/plugins/official-plugins/overview).

## License

MIT
