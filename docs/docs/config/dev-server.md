# Dev Server Options

## DevServer Options - server

Configure the behavior of Farm Dev Server. Example:

```ts
import { defineConfig } from "farm";

export default defineConfig({
  // All dev server options are under server
  server: {
    port: 9000,
    //...
  },
});
```

type:

```ts
export interface UserServerConfig {
  headers?: OutgoingHttpHeaders | undefined;
  port?: number;
  https?: SecureServerOptions;
  protocol?: "http" | "https";
  // http2?: boolean;
  hmr?: boolean | HmrOptions;
  proxy?: Record<string, ProxiesOptions>;
  strictPort?: boolean;
  open?: boolean;
  host?: string | boolean;
  cors?: boolean | cors.Options;
  // whether to serve static assets in spa mode, default to true
  spa?: boolean;
  middlewares?: DevServerMiddleware[];
  writeToDisk?: boolean;
}
```

### port

- **default**: `9000`

The port the DevServer listens on.

### https

- **default**: `undefined`

Enable TLS + HTTP2. The value is [options](https://nodejs.org/api/http2.html#http2createsecureserveroptions-onrequesthandler) that passes to [http2.createSecureServer](https://nodejs.org/api/http2.html#http2createsecureserveroptions-onrequesthandler).

:::note
Note that a **valid certificate** is needed if `https` enabled.
:::

### headers

- **default**: `undefined`

Setup global http response headers for the DevServer.

```ts
import { defineConfig } from "farm";

export default defineConfig({
  server: {
    headers: {
      Accept: "xxxx",
    },
  },
});
```

### strictPort

- **default**: `false`

By default, Farm will automatically resolve to a new port when given port is used. For example, if `9001` is used, then `9001` will be tried. But if `strictPort` is `true`, a error will be thrown when port conflicts, instead of try other ports automatically.

### cors

- **default**: `false`

Configure [@koa/cors options](https://www.npmjs.com/package/@koa/cors).

### spa

- **default**: `true`

Enable fallback to `index.html` or not.

### hmr

- **default**: `true` for start command, false for other commands

Enable HMR. After enabling the HMR capability, it will monitor the changes of the modules involved in the compilation process. When the modules change, it will automatically trigger recompilation and push the results to Farm Runtime for update. HMR can also be configured through an object, for example:

```ts
import { defineConfig } from "farm";

export default defineConfig({
   // All dev server options are under server
   server: {
     hmr: {
       // Configure the port for web socket listening
       port: 9802
       // Configure the host for web socket listening
       host: 'localhost',
       // Files to ignore when configuring file monitoring
       ignores: ['auto_generated/*']
     }
     //...
   }
});
```

#### `hmr.port`

- **default**: `9801`

The port the Web Socket server listens on

#### `hmr.host`

- **default**: `localhost`

Host on which the Web Socket server listens.

### proxy

- **Default value**: `undefined`

Configure server proxy. farm uses `http-proxy` as a proxy for the development server. Based on [http-proxy](https://github.com/http-party/node-http-proxy?tab=readme-ov-file#options) implementation, specific options refer to its documentation, example:

```ts
import { defineConfig } from "farm";

export default defineConfig({
  server: {
    proxy: {
      "/api": {
        target: "https://music-erkelost.vercel.app/banner",
        changeOrigin: true,
        pathRewrite: (path: any) => path.replace(/^\/api/, ""),
      },
    },
  },
});
```

### open

- **default**: `false`

After the compilation is completed, the browser is automatically opened to the corresponding page.

### host

- **default**: `localhost`

The host that the Dev Server listens on.

### middlewares

- **default**: `[]`

Configuring middlewares for the dev server.

```ts
import { defineConfig } from "farm";
import compression from "koa-compress";

export default defineConfig({
  server: {
    middlewares: [compression],
  },
});
```

Note that a `middleware` is a function that returns a koa middleware.

## writeToDisk

- **default**: `false`

By default the compiled resources are stored and served in memory, set `writeToDisk` to `true` to emitted dev resources to the disk.
