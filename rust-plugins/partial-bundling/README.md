<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <span>English</span> |
    <a href="./README-zh-CN.md">简体中文</a>  
</div>

---

# Webpack Partial Bundling Plugin for Farm

support split resource like webpack splitChunk

> This plugin is only running when `farm build`, in other cases, use the default partial bundling

## Usage

Install `@farmfe/plugin-webpack-partial-bundling` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/plugin-webpack-partial-bundling --save-dev
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  plugins: ['@farmfe/plugin-webpack-partial-bundling']

  // with options
  plugins: [
    '@farmfe/plugin-webpack-partial-bundling',
    {
      module_bucket: [
        {
          name: 'node_modules_vendor',
          test: 'node_modules',
          weight: 100,
          minSize: 1024,
          maxConcurrentRequests: 10
        }
      ]
    }
  ]
});
```

## Options

### name

Type: `string`

module bucket name

### test

Type: `(string | regex)[]`

Default: `[]`

Match the corresponding module

> When `test` is empty, all modules are matched

### weight

Type: `number`

Default: `0`

Determine whether the module bucket is given priority

### minSize

Type: number

Default: `undefined`

The minimum value of resource pot

### maxConcurrentRequests

Type: number

Default: `undefined`

concurrent resource pot transactions after splitting
