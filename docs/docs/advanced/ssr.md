# Server-Side Rendering (SSR)

Server-Side Rendering (SSR) means rendering front-end frameworks(for example React, Vue, Solid, etc) to `html` in Node.js(Server Side), and hydrating the `rendered html` on the client.

:::note
This document describes how to built a SSR application on top of Farm from scratch.
:::

## Example Projects

Farm provides a list of SSR [examples](https://github.com/farm-fe/farm/tree/main/examples) for popular frameworks:

- **[React](https://github.com/farm-fe/farm/tree/main/examples/react-ssr)**
- **[Vue](https://github.com/farm-fe/farm/tree/main/examples/vue-ssr)**
- **[Solid](https://github.com/farm-fe/farm/tree/main/examples/solid-ssr)**

## Project Structure

A [SSR typical application](https://github.com/farm-fe/farm/tree/main/examples/react-ssr) often have the following source file structure:

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

- **`index.html`**: Entry html of the application that running on the client(browser)
- **`farm.config.ts`**: farm config that builds the project to client
- **`farm.config.server.ts`**: Farm config that builds the project to Node.js(server)
- **`server.js`**: Server script that should be deployed for production
- **`src/index-client.tsx`**: Client entry scripts
- **`src/index-server.tsx`**: Server entry scripts
- **`src/main.tsx`**: Application code shared for both client and server

`index.html` need to reference `index-client.tsx` and include a placeholder where the server-rendered `markup` should injected:

```html
<div id="root"><div>app-html-to-replace</div></div>
<script src="./src/index-client.tsx"></script>
```

You should replace `<div>app-html-to-replace</div>` to the server-rendered `markup`.

:::tip
We have to build the SSR application **twice**, one for `client`(browser) and one for `server`(Node.js). So `farm.config.ts` and `farm.config.server.ts` are needed, we'll discuss the details in later sections.
:::

## Setting up Dev Server

For above example, `farm.config.ts` is used to **build the project for browser** and setting up DevServer for server rendering. The content of `farm.config.ts` normally would be:

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
      // register a middleware that render the application on the server,
      // inject server rendered markup and return final index.html
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();

          // handle index.html or SPA fallback
          if (ctx.path === "/" || ctx.status === 404) {
            // loading the server entry, and render it by ctx.path
            const render = await import(
              path.join(process.cwd(), "dist", "index.js")
            ).then((m) => m.default);
            const renderedHtml = render(ctx.path);

            // get compiled index.html content from server.getCompiler()
            // The html is compiled for client with all client bundles injected
            const template = server
              .getCompiler()
              .resource("index_client.html")
              .toString();

            // replace the placeholder to rendered markup and return it as html
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

In above example, a `middleware` is required for rendering the application to markup and serve it as html. Normal workflow for SSR in the `middleware`:

- **Load compiled server entry:** A `index-server` entry which exports a `render` function is required, we need to `import(server_entry_path)` to get the `render` function.
- **Get compiled client index.html:** All client bundles and Farm runtime are injected to `index.html`, so the client can `hydrate` successfully.
- **Replace the placeholder to rendered markup:** Replace the placeholder and return the `final html`.

:::note
In this example, we are building a `SPA` SSR application with `if (ctx.path === '/' || ctx.status === 404) {`, if you are building a `MPA` SSR application, guard `ctx.path` to your pages.
:::

## Building for Node.js

`farm.config.server.ts` is used to **build the project for Node.js**, producing the compiled server entry which can be used to rendering the application to markup on the server side.

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

For `farm.config.server.ts`, we set `input` to **server entry** and [`output.targetEnv`](/docs/config/compilation-options#output-targetenv) to `node`.

:::note
By default, Farm compiles server entry script to `esm`, if you want to compile it to `cjs`, try set [`output.format`](/docs/config/compilation-options#output-format).
:::

## Develop SSR Project

You have start compilation for both `client` and `server`, for example, you may have following `scripts` in `package.json`:

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

When starting your SSR project, you should run both `npm run start` and `npm run start:server` in different terminal.

## Building for Production

You have build both `client` and `server`, for example, you may add following command to `scripts`:

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

When building for production, you should run both `npm run build` and `npm run build:server`, the client bundles will be emitted to `build` dir, and the server bundles will be emitted to `dist` dir.

For production, you need a `node server` for `rendering` and serving `rendered html`, in this example, we provide a `server.js` as production server:

```js title="server.js"
import path from "node:path";
import { fileURLToPath } from "node:url";
import fsp from "fs/promises";
import express from "express";

function resolve(p) {
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(__dirname, p);
}

// create a node production server
async function createServer() {
  let app = express();
  // serve the client builds as static assets, you can also deploy client builds to CDN or separate dev server as you wish.
  app.use(express.static(resolve("build")));
  // listen '/' route, you can replace it to the routes you use.
  app.use("/", async (req, res) => {
    let url = req.originalUrl;

    try {
      let template;
      let render;

      // load client html
      template = await fsp.readFile(resolve("build/index_client.html"), "utf8");
      // load server render function
      render = await import(resolve("dist/index.js")).then((m) => m.default);
      // render the application to markup
      const markup = render(url);

      let html = template.replace("<div>app-html-to-replace</div>", markup);
      // return the rendered html with client bundles, the client bundles hydrate the server rendered markup and make it interactive
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

We use `express` as server here, but you can use whatever server frameworks you want. The rendering process are the same:

- Loading client compiled html
- Loading `render` function from compiled server script
- Call `const markup = render(url)` function to get the server-side rendered markup of your application
- Replace the `placeholder` in `client index.html` to the `rendered markup` and return the replaced html as final result

## Static-Site Generation(SSG)

The same flow of SSG is the same as SSR, the difference is SSG that emits to `replaced html` to the final resources. Example scripts for SSG:

```ts
// load client html
const template = await fsp.readFile(resolve("build/index_client.html"), "utf8");
// load server render function
const render = await import(resolve("dist/index.js")).then((m) => m.default);

const pages = renderDirEntry("src/pages");

for (const page of pages) {
  // render the application to markup
  const markup = render(url);
  const html = template.replace("<div>app-html-to-replace</div>", markup);
  // emit the static generated page, for example writing it to disk
  emitPage(page, html);
}
```
