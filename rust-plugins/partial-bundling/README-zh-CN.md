<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <a  href="./README.md">English</a> |
    <span>简体中文</span>  
</div>

---

# Webpack Partial Bundling Plugin

拆分 resource pot，类似 Webpack splitChunks 插件

> 这个模块仅在 `farm build` 时运行，其它情况使用默认 partial bundling

## 使用

下载 `@farmfe/plugin-webpack-partial-bundling`:

```bash
npm i @farmfe/plugin-webpack-partial-bundling --save-dev
```

配置 `farm.config.ts`:

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

## 配置

### name

Type: `string`

module bucket 名称

### test

Type: `(string | regex)[]`

Default: `[]`

匹配对应的 module

> 当 `test` 为空时，会匹配所有模块

### weight

Type: `number`

Default: `0`

决定该 module bucket 是否优先处理

### minSize

Type: number

Default: `undefined`

resource pot 最小值

### maxConcurrentRequests

Type: number

Default: `undefined`

拆分后的 resource pot 并发数量
