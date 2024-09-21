# 2. 使用 Farm 开发项目
在本章中，我们将介绍**常用的配置和插件**来帮助您使用 Farm 构建复杂的生产就绪的 Web 项目。

:::note
本章重用我们在第 1 章中创建的项目
:::

我们将逐步设置我们的项目：
1.引入流行的组件库antd，并为其配置必要的插件
2.介绍postcss、svgr、less等常用插件。
3. 配置代理和其他有用的开发服务器选项

## 引入组件库
开发 Web 项目时常常需要用到组件库，本节我们将使用`ant-design`作为 demo 来展示如何在 Farm 中添加组件库。

> 我们这里使用`ant design`只是为了说明，你可以引入任何组件库。 对于组件库选择，Farm 没有任何倾向。

首先我们需要将 ant-design 安装到我们的项目中：
````bash
pnpm add antd # 在项目根目录下执行
````

Ant Design需要Sass，所以我们还需要安装编译 scss 的插件。 我们可以使用 Farm 官方提供的 Rust 插件 `@farmfe/plugin-sass`：
````bash
pnpm add @farmfe/plugin-sass -D
````

然后将此插件添加到`plugins`中：
```ts title="farm.config.ts" {7}
// ...

export default defineConfig({
  // ... ignore other fields
  plugins: [
    '@farmfe/plugin-react',
    '@farmfe/plugin-sass'
  ]
});
```

现在 Antd 已经准备好了，将其添加到我们的项目中：
```tsx {4,12}
import React from 'react';
import { createRoot } from 'react-dom/client';

import { DatePicker } from 'antd';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <div>
    A React Page compiled by Farm. 
    antd DatePicker: <DatePicker />
  </div>
);
```
然后执行`npm start`并在浏览器中打开`http://localhost:9000`：

<img src="/img/2023-10-10-21-41-45.png" width="500" /> &nbsp;<img src="/img/2023-10-10-21-34-33.png" width="580" /> 

## 给项目添加 CSS 样式
现在我们已经成功地将组件库引入到我们的项目中。 接下来我们将学习如何给项目添加样式。

### 创建基本的管理站点布局
首先，我们在`index.tsx`旁边创建一个新的`app.tsx`：
```text {7}
.
├── farm.config.ts
├── package.json
├── pnpm-lock.yaml
└── src
    ├── index.html
    ├── app.tsx
    └── index.tsx
```
`app.tsx`的内容（来自Antd官网的演示代码）：
```tsx title="app.tsx"
import React from 'react';
import { Breadcrumb, Layout, Menu, theme } from 'antd';

const { Header, Content, Footer } = Layout;

const App: React.FC = () => {
  const {
    token: { colorBgContainer },
  } = theme.useToken();

  return (
    <Layout className="layout">
      <Header style={{ display: 'flex', alignItems: 'center' }}>
        <div className="demo-logo" />
        <Menu theme="dark" mode="horizontal" defaultSelectedKeys={['2']}
          items={new Array(15).fill(null).map((_, index) => {
            const key = index + 1;
            return {
              key,
              label: `nav ${key}`,
            };
          })}
        />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Breadcrumb style={{ margin: '16px 0' }}>
          <Breadcrumb.Item>Home</Breadcrumb.Item>
          <Breadcrumb.Item>List</Breadcrumb.Item>
          <Breadcrumb.Item>App</Breadcrumb.Item>
        </Breadcrumb>
        <div className="site-layout-content" style={{ background: colorBgContainer }}>
          Content
        </div>
      </Content>
      <Footer style={{ textAlign: 'center' }}>Ant Design ©2023 Created by Ant UED</Footer>
    </Layout>
  );
};

export default App;
```
然后将 `index.tsx` 修改为：
```tsx {4,13} title="index.tsx"
import React from 'react';
import { createRoot } from 'react-dom/client';

import App from './app';
// import { DatePicker } from 'antd';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <div>
    A React Page compiled by Farm. 
    <App />
    {/* antd DatePicker: <DatePicker /> */}
  </div>
);
  
```
然后我们得到一个基本的管理站点布局：
<img src="/img/2023-10-12-22-16-48.png" width="800" />

### 使用 CSS Modules
Farm 开箱即用地支持`css modules`，默认情况下，Farm 会将任何`.module.(css|scss|less)`视为`css 模块`。 首先我们创建一个`app.module.scss`：
```text {8}
.
├── farm.config.ts
├── package.json
├── pnpm-lock.yaml
└── src
    ├── index.html
    ├── app.tsx
    ├── app.module.scss
    └── index.tsx
```
Content of `app.module.scss`:
```scss title="app.module.scss"
$primary-color: #1890ff;

.site-layout-content {
  min-height: 200px;
  padding: 24px;
  font-size: 24px;
  color: $primary-color;
}
```
然后在`app.tsx`中导入`app.module.scss`并保存：
```tsx
import styles from './app.module.scss';
// ...
```
然后你的页面应该更新成如下：
<img src="/img/2023-10-14-21-24-40.png" width="800" />

### 使用 CSS 预处理器
Farm 为 `postcss`(`@farmfe/js-plugin-postcss`) 和 `less`(`@farmfe/js-plugin-less`) 提供了官方 js 插件（在上文中，我们已经安装了 `sass` 插件（`@farmfe/plugin-sass`））。

要使用postcss，首先我们需要安装插件：
```bash
pnpm add -D @farmfe/js-plugin-postcss
```
然后在`farm.config.ts`的`plugins`中配置它：
```ts title="farm.config.ts" {7}
// ...
import farmPluginPostcss from '@farmfe/js-plugin-postcss';

export default defineConfig({
  // ... ignore other fields
  plugins: [
    '@farmfe/plugin-react',
    '@farmfe/plugin-sass',
    farmPluginPostcss()
  ]
});
```
现在 Farm 完全支持 postcss，我们不会在这里介绍 postcss 细节，请参阅 postcss 文档以获取更多详细信息。
:::tip
请参阅 [使用 Farm 插件](/docs/using-plugins) 了解有关 Farm 插件的更多信息。
:::

## 配置 Alias 以及 Externals
Alias 和 externals 是最常用的配置之一, 在 Farm 中，可以使用 `compilation.resolve.alias` 和 `compilation.externals` 配置项:

```ts title="farm.config.ts"
// ...

export default defineConfig({
  compilation: {
    resolve: {
      alias: {
        '@/': path.join(process.cwd(), 'src')
      },
      externals: [
        'node:fs'
      ]
    }
  }
  // ...
});
```

## 配置开发服务器
您可以在[Farm Dev Server Config](/docs/config/farm-config#devserver-options---server)中找到服务器配置。

### 常用配置
配置示例：
```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
   return config;
}

export default defineConfig({
   // 所有开发服务器选项都在 server 下
   server: {
     open: true,
     port: 9001,
     hmr: {
       // 配置Websocket的监听端口
       port: 9801
       host: 'localhost',
       // 配置文件监听时要忽略的文件
       ignores: ['auto_generated/*']
     }
     //...
   }
});
```
对于上面的示例，我们使用了以下选项：
* **打开**：自动打开指定端口的浏览器
* **端口**：将开发服务器端口设置为“9001”
* **hmr**：设置 hmr 端口和监视文件，我们忽略 `auto_generate` 目录下的文件更改。


### Setup Proxy
配置服务器代理。基于 [http-proxy](https://github.com/http-party/node-http-proxy?tab=readme-ov-file#options) 实现，具体选项参考其文档，示例：

```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
   return config;
}

export default defineConfig({
    server: {
     proxy: {
       '/api': {
         target: 'https://music-erkelost.vercel.app/banner',
         changeOrigin: true,
         pathRewrite: (path: any) => path.replace(/^\/api/, ''),
       },
     },
   },
});

```
