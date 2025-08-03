# 服务端渲染 (SSR)

Server-Side Rendering（SSR）意味着在Node.js（服务器端）中将前端框架（例如React、Vue、Solid等）渲染为 `html` ，并在客户端对已经渲染好的HTML（ `rendered html` ）进行注水 (hydrate)。

:::note
本文档描述了如何从头开始在 Farm 上构建 SSR 应用程序。
:::

## 示例项目

Farm为流行的框架提供了 SSR [示例](https://github.com/farm-fe/farm/tree/main/examples)：

- **[React](https://github.com/farm-fe/farm/tree/main/examples/react-ssr)**
- **[Vue](https://github.com/farm-fe/farm/tree/main/examples/vue-ssr)**
- **[Solid](https://github.com/farm-fe/farm/tree/main/examples/solid-ssr)**

## Project Structure

一个[典型的SSR应用程序](https://github.com/farm-fe/farm/tree/main/examples)通常具有以下源文件结构：

```
.
├── index.html
├── farm.config.ts
├── farm.config.server.ts
├── server.js
└── src
    ├── index-client.tsx
    ├── index-server.tsx
    └── main.tsx
```

- **`index.html`**: 应用程序运行在客户端（浏览器）上的入口HTML
- **`farm.config.ts`**: 构建项目到客户端的farm配置
- **`farm.config.server.ts`**: 构建项目到Node.js（服务端）的farm配置
- **`server.js`**: 应该部署到生产环境的服务端脚本
- **`src/index-client.tsx`**: 客户端入口脚本
- **`src/index-server.tsx`**: 服务端入口脚本
- **`src/main.tsx`**: 客户端和服务器共享的应用程序代码

`index.html` 需要引用 `index-client.tsx` 并包含一个占位符，其中应注入服务器渲染的标记（`markup`）：

```html
<div id="root"><div>app-html-to-replace</div></div>
<script src="./src/index-client.tsx"></script>
```

你应该将 `<div>app-html-to-replace</div>` 替换为服务器渲染的`markup`。

:::tip
我们必须为客户端（浏览器）和服务端（Node.js）分别构建SSR应用程序**共两次**。因此，需要 `farm.config.ts` 和 `farm.config.server.ts` ，我们将在后面的章节中讨论详细信息。
:::

## 设置开发服务器

对于上述示例， `farm.config.ts` 用于**构建浏览器端项目**并设置开发服务器进行服务器渲染。 `farm.config.ts` 的通常这样写：

```ts title="farm.config.ts"
import path from "path";
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    input: {
      index_client: "./index.html",
    },
    output: {
      path: "./build",
    },
  },
  server: {
    hmr: true,
    cors: true,
    middlewares: [
      // 注册一个中间件，在服务端渲染应用，
      // 然后注入到服务器渲染的标记并返回最终的index.html
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();

          // 处理index.html或单页面应用路由设置
          if (ctx.path === "/" || ctx.status === 404) {
            // 加载服务端入口，并通过ctx.path渲染
            const render = await import(
              path.join(process.cwd(), "dist", "index.js")
            ).then((m) => m.default);
            const renderedHtml = render(ctx.path);

            // 通过server.getCompiler()获取编译的index.html内容
            // 这里的html经过编译并注入了所有客户端bundles文件
            const template = server
              .getCompiler()
              .resource("index_client.html")
              .toString();

            // 将占位符替换为渲染好的内容，并将其作为HTML返回
            const html = template.replace(
              "<div>app-html-to-replace</div>",
              renderedHtml,
            );
            ctx.body = html;
            ctx.type = "text/html";
            ctx.status = 200;
          }

          console.log("ctx.path outer", ctx.path);
        });
      },
    ],
  },
  plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass"],
});
```

在上面的示例中，需要一个中间件（`middleware`）来将应用程序渲染为标记并将其作为HTML提供。中间件中SSR的正常工作流程：

- **加载编译后的服务端入口:** 需要一个导出 `render` 函数的index-server入口，然后通过 `import(server_entry_path)` 来获取这个 `render` 函数。
- **获取编译后的客户端index.html:** 所有客户端打包代码和Farm运行时都注入到 `index.html`中，用于在客户端进行水合作用（`hydrate`）。
- **将占位符替换为渲染后的代码:** 替换占位符并返回最终的html代码（`final html`）。

:::note
在这个示例中，我们使用 `if (ctx.path === '/' || ctx.status === 404) {` 来构建一个 `SPA` SSR应用程序，如果你需要构建一个 `MPA` SSR应用程序，请将 `ctx.path` 传递到你的页面。
:::

## 构建 Node.js 服务端产物

`farm.config.server.ts` 用于**构建 Node.js 端产物**，生成编译后的服务端入口，可用于在服务端将应用渲染为标记（`markup`）。

```ts title="farm.config.server.ts"
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    // c-highlight-start
    input: {
      index: "./src/index-server.tsx",
    },
    output: {
      path: "./dist",
      targetEnv: "node",
    },
    // c-highlight-end
  },
  plugins: [
    [
      "@farmfe/plugin-react",
      {
        refresh: false,
        development: false,
      },
    ],
    "@farmfe/plugin-sass",
  ],
});
```

对于 `farm.config.server.ts` ，我们将 `input` 设置为**服务端入口**，并将 [`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv) 设置为 `node` 。

:::note
默认情况下，Farm将服务端入口脚本编译为 `esm` ，如果你想要将其编译为cjs，请尝试设置 [`output.format`](/zh/docs/config/compilation-options#output-format)。
:::

## 开发SSR项目

你需要为客户端和服务端启动编译，例如，你可能会在package.json中有以下脚本：

```json title="package.json"
{
  "name": "@farmfe-examples/react-ssr",
  "scripts": {
    // c-highlight-start
    "start": "farm start",
    "start:server": "farm watch --config farm.config.server.mjs"
    // c-highlight-end
  }
}
```

当你开发SSR项目时，你需要在不同的终端中运行 `npm run start` 和 `npm run start:server` 。同时监听 server 和 client 的变动并重新编译。

## 生产环境构建

你需要同时为客户端和服务器构建项目，例如，你可能需要在 `scripts` 中添加以下命令：

```json title="package.json"
{
  "name": "@farmfe-examples/react-ssr",
  "scripts": {
    "start": "farm start",
    "start:server": "farm watch --config farm.config.server.mjs",
    // c-highlight-start
    "build": "farm build",
    "build:server": "farm build --config farm.config.server.mjs"
    // c-highlight-end
  }
}
```

打包构建时，你需要运行 `npm run build` 和 `npm run build:server`，客户端打包将被输出到 `build` 目录，服务端打包将被输出到 `dist` 目录。

对于生产环境，你需要一个 `node server` 来渲染和提供 `rendered html`。在这个示例中，我们使用了一个 `server.js` 作为生产服务端：

```js title="server.js"
import path from "node:path";
import { fileURLToPath } from "node:url";
import fsp from "fs/promises";
import express from "express";

function resolve(p) {
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(__dirname, p);
}

// 创建一个Node生产服务端
async function createServer() {
  let app = express();
  // 为客户端打包产物提供静态文件服务，也可以将客户端构建部署到CDN或单独的开发服务器，按照你的需求。
  app.use(express.static(resolve("build")));
  // 监听 '/' 路由, 你也可以将其替换为你需要的路由.
  app.use("/", async (req, res) => {
    let url = req.originalUrl;

    try {
      let template;
      let render;

      // 加载客户端html
      template = await fsp.readFile(resolve("build/index_client.html"), "utf8");
      // 加载服务端渲染函数
      render = await import(resolve("dist/index.js")).then((m) => m.default);
      // 将应用渲染为标记
      const markup = render(url);

      let html = template.replace("<div>app-html-to-replace</div>", markup);
      // 返回包含客户端打包的rendered html
      // 客户端打包代码和服务器渲染的标记进行水和作用，
      // 并使其具有交互性
      res.setHeader("Content-Type", "text/html");
      return res.status(200).end(html);
    } catch (error) {
      console.log(error.stack);
      res.status(500).end(error.stack);
    }
  });

  return app;
}
// create and listen the server
createServer().then((app) => {
  app.listen(3000, () => {
    console.log("HTTP server is running at http://localhost:3000");
  });
});
```

我们在这里使用 `express` 作为服务端，但你可以使用任何你想要的服务端框架。渲染过程是相同的：

- 加载客户端编译后的HTML(`client index.html`)
- 从服务端脚本代码加载 `render` 函数
- 调用 `const markup = render(url)` 函数以获取应用的服务器端渲染标记
- 将 `client index.html` 中占位符替换为服务端渲染标记，并将替换后的html作为最终结果返回

## 静态站点生成(SSG)

SSG的流程与SSR相同，不同的是SSG将替换的html输出到最终产物。SSG的示例脚本：

```ts
// 加载 client html
const template = await fsp.readFile(resolve("build/index_client.html"), "utf8");
// 加载服务端渲染函数
const render = await import(resolve("dist/index.js")).then((m) => m.default);

const pages = renderDirEntry("src/pages");

for (const page of pages) {
  // 将应用渲染为标记
  const markup = render(url);
  const html = template.replace("<div>app-html-to-replace</div>", markup);
  // 输出静态生成的页面，例如将其写入硬盘
  emitPage(page, html);
}
```
