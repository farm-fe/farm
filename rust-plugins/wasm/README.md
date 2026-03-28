# @farmfe/plugin-wasm

A Farm plugin for seamless WebAssembly integration and support `wasm-pack` generated modules.

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

Pre-compiled `.wasm` files can be imported using the `?init` query. The default export is an initialization function, which returns a Promise resolving to a [`WebAssembly.Instance`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance) object:

```ts
import init from './example.wasm?init';
init().then((instance) => {
  instance.exports.test();
});
```

The `init` function can also take an import object as its second parameter, which is passed to [`WebAssembly.Instance`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance):

```ts
import init from './example.wasm?init';
init({
  imports: {
    someFunc: () => {
      /* ... */
    },
  },
}).then(() => {
  /* ... */
});
```

You can also directly import WASM packages like ESM modules without using the `?init` query:

```ts
import { someFunc } from 'wasm-project';
someFunc(); // Run this wasm function
```
