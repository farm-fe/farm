# Qiankun Js Plugin for Farm

支持在 Farm 构建工具中使用 qiankun 微前端框架。

## 使用方法

在 `farm.config.ts` 中配置插件：

```ts
import { defineConfig } from '@farmfe/core';
import { qiankunFarmPlugin } from '@farmfe/js-plugin-qiankun';

export default defineConfig({
  ...,
  plugins: [
    qiankunFarmPlugin({
      appName: 'my-micro-app', // 微应用名称，必填
      devMode: true // 是否开启开发模式，可选，默认为 false
    })
  ]
});
```

## 完整示例

### React 应用示例

```ts
// main.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { injectQiankun } from '@farmfe/js-plugin-qiankun/helper';
import App from './App';

let root: ReactDOM.Root | null = null;

injectQiankun({
  async bootstrap() {
    console.log('React 微应用启动');
  },
  
  async mount(props) {
    console.log('React 微应用挂载', props);
    const container = document.getElementById('root');
    if (container) {
      root = ReactDOM.createRoot(container);
      root.render(<App />);
    }
  },
  
  async unmount() {
    console.log('React 微应用卸载');
    if (root) {
      root.unmount();
      root = null;
    }
  }
});
```

### Vue 应用示例

```ts
// main.ts
import { createApp } from 'vue';
import { injectQiankun } from '@farmfe/js-plugin-qiankun/helper';
import App from './App.vue';

let app: ReturnType<typeof createApp> | null = null;

injectQiankun({
  async bootstrap() {
    console.log('Vue 微应用启动');
  },
  
  async mount(props) {
    console.log('Vue 微应用挂载', props);
    app = createApp(App);
    app.mount('#app');
  },
  
  async unmount() {
    console.log('Vue 微应用卸载');
    if (app) {
      app.unmount();
      app = null;
    }
  }
});
```

## 选项说明

### PluginOptions

#### appName

- 类型: `string`
- 必填: 是
- 说明: 微应用的名称，用于在全局对象上注册生命周期函数

#### devMode

- 类型: `boolean`
- 默认值: `false`
- 说明: 是否开启开发模式。开启后会在控制台输出调试信息

### InjectOptions

#### bootstrap

- 类型: `() => Promise<void>`
- 必填: 是
- 说明: 应用启动时的生命周期钩子

#### mount

- 类型: `(props: unknown) => Promise<void>`
- 必填: 是
- 说明: 应用挂载时的生命周期钩子，`props` 为主应用传递的属性

#### unmount

- 类型: `() => Promise<void>`
- 必填: 是
- 说明: 应用卸载时的生命周期钩子

#### update

- 类型: `(props: unknown) => Promise<void>`
- 必填: 否
- 说明: 应用更新时的生命周期钩子，`props` 为主应用传递的属性

## 参考项目

- [vite-plugin-qiankun](https://github.com/tengmaoqing/vite-plugin-qiankun) - 帮助应用快速接入乾坤的 vite 插件
