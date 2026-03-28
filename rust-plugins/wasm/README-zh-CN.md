# @farmfe/plugin-wasm

A farm plugin which imports Wasm modules.

## Installation

```bash
npm i -D @farmfe/plugin-wasm
```

## Usage

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```ts
import { defineConfig } from '@farmfe/core';
import wasm from '@farmfe/plugin-wasm';
export default defineConfig({
  plugins: [
    wasm(),
  ],
});
```

## WebAssembly

预编译的 `.wasm` 文件可以通过`?init`来导入。 默认导出一个初始化函数，返回值为所导出 [`WebAssembly.Instance`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance) 实例对象的 Promise：

```ts
import init from './example.wasm?init'
init().then((instance) => {
  instance.exports.test()
})
```

`init` 函数还可以将传递给 [`WebAssembly.Instance`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance) 的导入对象作为其第二个参数：

```ts
import init from './example.wasm?init'
init({
  imports: {
    someFunc: () => {
      /* ... */
    },
  },
}).then(() => {
  /* ... */
})
```

在生产构建当中，体积小于 assetInlineLimit 的 .wasm 文件将会被内联为 base64 字符串。否则，它们将被视为 静态资源 ，并按需获取。
