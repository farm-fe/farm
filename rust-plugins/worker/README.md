# @farmfe/plugin-worker

A web worker script can be imported using `new Worker()` and `new SharedWorker()` (inspired by [vite](https://github.com/vitejs/vite)).

## Installation

```bash
npm i -D @farmfe/plugin-worker
```

## Usage

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```ts
import { defineConfig } from '@farmfe/core';
import worker from '@farmfe/plugin-worker';
export default defineConfig({
  plugins: [
    worker(),
  ],
});
```

## Import via Constructor

A Web Worker can be imported using [`new Worker()`](https://developer.mozilla.org/zh-CN/docs/Web/API/Web_Workers_API/Using_web_workers) and [`new SharedWorker()`](https://developer.mozilla.org/zh-CN/docs/Web/API/SharedWorker). This syntax is closer to the standard compared to the worker suffix and is the recommended way to create workers.

```ts
const worker = new Worker(new URL('./worker.js', import.meta.url));
```

The Worker constructor accepts options that can be used to create a "module" worker:

```ts
const worker = new Worker(new URL('./worker.js', import.meta.url), {
  type: 'module',
});
```

## Import with Query Suffix

You can directly import a web worker script by adding the `?worker` or `?sharedworker` query parameter to the import request. The default export will be a custom worker constructor:

```ts
import MyWorker from './worker?worker';

const worker = new MyWorker();
```

This worker script can also use ESM import statements instead of `importScripts()`. Note: During development, this relies on native browser support, but in production builds, it will be compiled away.

By default, the worker script will compile into a separate chunk in production builds. If you want to inline the worker as a base64 string, please add the `inline` query parameter:

```ts
import MyWorker from './worker?worker&inline'
```

If you want to read the worker as a URL, add the `url` query:

```ts
import MyWorker from './worker?worker&url'
```
