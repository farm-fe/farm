# DevServer and HMR
Farm provides `DevServer` and enabled `HMR` in `development` by default.

## Configuring Dev Server
Farm provides a lot of useful options to configure dev server. All dev server options are configured by [`server`](/docs/config/dev-server).

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  server: {
    port: 9801,
    cors: true,
    proxy: {
      // ...
    },
    open: true,
  }
})
```

:::note
If you are built tools on top of farm, refer to [Javascript API](/docs/api/javascript-api) for creating a Dev Server programmatically.
:::

## Dev Server Middlewares
You can use [`middlewares`](/docs/config/dev-server#middlewares) to handle dev server requests. Farm v2 uses [connect](https://github.com/senchalabs/connect)-style middleware. A middleware is a function that takes the `Server` instance and returns a `connect` handler `(req, res, next)`:

```ts title="farm.config.ts"
import type { IncomingMessage, ServerResponse } from 'node:http';
import { Server, defineConfig } from '@farmfe/core';

export function headers(devServerContext: Server) {
  const { config } = devServerContext;
  if (!config.headers) return;

  return (req: IncomingMessage, res: ServerResponse, next: () => void) => {
    if (config.headers) {
      for (const name in config.headers) {
        res.setHeader(name, config.headers[name] as string | string[]);
      }
    }
    next();
  };
}

export default defineConfig({
  server: {
    middlewares: [
      headers
    ]
  }
})
```

In above example, a Farm middleware is a function that receives the `Server` context and returns a standard `connect`-style handler. Common `connect`/`express`-compatible middlewares can be used directly, for example:

```ts {2,7}
import { defineConfig } from "@farmfe/core";
import compression from 'compression';

export default defineConfig({
  server: {
    middlewares: [
      () => compression()
    ]
  },
});
```

## Hot Module Replacement(HMR)
Farm provides a [Vite-compatible HMR API](/docs/api/hmr-api). If you are framework authors, leverage the API to update your Application instance, precise without reloading the page.

* For React, **React Refresh** are enabled automatically by official plugins [@farmfe/plugin-react](/docs/plugins/official-plugins/react).
* For Vue, Solid and other frameworks, it's HMR are supported by there plugins like `@vitejs/plugin-vue`, `vite-plugin-solid` and so on.

Farm provides official templates that set all these capabilities up already, create an app via [create-farm](/docs/quick-start) then all HMR abilities are ready.

:::note
* Usually HMR is supported out of box for app users, refer to [Vite-compatible HMR API](/docs/api/hmr-api) if you are framework author.
* Refer to [HMR Options](/docs/config/dev-server#hmr) for how to configuring HMR.
:::
