<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <span>English</span> |
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/svgr/README-zh-CN.md">简体中文</a>  
</div>

---

# Svgr Plugin for Farm

Support compiling Svg as React components in Farm.

## Getting Started

To begin, you'll need to `@farmfe/js-plugin-svgr`:

```bash
npm install @farmfe/js-plugin-svgr --save-dev
```

or

```bash
yarn add -D @farmfe/js-plugin-svgr
```

or

```bash
pnpm add -D @farmfe/js-plugin-svgr
```

Configuring the plugin in `farm.config.ts`:

```ts
import { UserConfig } from '@farmfe/core';
import svgr from '@farmfe/js-plugin-svgr'; //  import the plugin

function defineConfig(config: UserConfig) {
  return config;
}

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  plugins: [
    // use the svgr plugin.
    svgr({
      // custom options here
    })
  ]
});
```

## Options

- **[`svgrOptions`](#svgroptions)**

### svgrOptions

refer to https://react-svgr.com/docs/options
