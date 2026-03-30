# Dev Server Options

## DevServer Options - server

Configure the behavior of Farm Dev Server. Example:

```ts
import { defineConfig } from "@farmfe/core";

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
  https?: HttpsServerOptions;
  origin?: string;
  allowedHosts?: string[];
  hmr?: boolean | HmrOptions;
  proxy?: Record<string, ProxyOptions>;
  strictPort?: boolean;
  open?: boolean;
  host?: string | boolean;
  cors?: boolean | any;
  appType?: 'spa' | 'mpa' | 'custom';
  middlewares?: DevServerMiddleware[];
  middlewareMode?: boolean;
  writeToDisk?: boolean;
  preview?: UserPreviewServerConfig;
}
```

### port

- **default**: `9000`

The port the DevServer listens on.

### https
- **default**: `undefined`

Enable  TLS  + HTTP2. The value is [options](https://nodejs.org/api/http2.html#http2createsecureserveroptions-onrequesthandler) that passes to [http2.createSecureServer](https://nodejs.org/api/http2.html#http2createsecureserveroptions-onrequesthandler).

:::note
Note that a **valid certificate** is needed if `https` enabled.
:::

### headers
- **default**: `undefined`

Setup global http response headers for the DevServer.

```ts
import { defineConfig } from '@farmfe/core'

export default defineConfig({
  server: {
    headers: {
      'Accept': 'xxxx'
    }
  }
})
```

### strictPort
- **default**: `false`

By default, Farm will automatically resolve to a new port when given port is used. For example, if `9001` is used, then `9001` will be tried. But if `strictPort` is `true`, a error will be thrown when port conflicts, instead of try other ports automatically.

### cors
- **default**: `false`

Whether to enable CORS (Cross-Origin Resource Sharing) for the dev server.


### appType
- **default**: `'spa'`

Configure the application type, which affects how the dev server handles fallback routing:

- `'spa'`: Single Page Application. The server will fall back to `index.html` for unmatched routes.
- `'mpa'`: Multi-Page Application. The server will look for corresponding HTML files for each route.
- `'custom'`: Disable automatic HTML fallback. Useful when you have custom server-side logic.

### hmr

- **default**: `true` for start command, false for other commands

Enable HMR. After enabling the HMR capability, it will monitor the changes of the modules involved in the compilation process. When the modules change, it will automatically trigger recompilation and push the results to Farm Runtime for update. HMR can also be configured through an object, for example:

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
   // All dev server options are under server
   server: {
     hmr: {
       // Configure the port for web socket listening
       port: 9802,
       // Configure the host for web socket listening
       host: 'localhost',
       // The path for the HMR endpoint
       path: '/__hmr',
     }
     //...
   }
});
```

#### `hmr.port`

- **default**: `undefined` (falls back to `FARM_DEFAULT_HMR_PORT` env var, or the dev server port)

The port the WebSocket server listens on.

#### `hmr.host`

- **default**: `localhost`

Host on which the WebSocket server listens.

#### `hmr.clientPort`

- **default**: `9000`

The port number for the HMR client to connect to. Useful when you have a reverse proxy in front of the dev server.

#### `hmr.path`

- **default**: `'/__hmr'`

The URL path for the HMR WebSocket endpoint.

#### `hmr.timeout`

- **default**: `0`

The timeout in milliseconds for the HMR WebSocket connection.

#### `hmr.overlay`

- **default**: `true`

Whether to show an error overlay in the browser when HMR errors occur.

#### `hmr.protocol`

- **default**: `''` (auto-detected)

The WebSocket protocol to use for the HMR connection (e.g. `'ws'` or `'wss'`).

### proxy

- **Default value**: `undefined`

Configure server proxy. farm uses `http-proxy` as a proxy for the development server. Based on [http-proxy](https://github.com/http-party/node-http-proxy?tab=readme-ov-file#options) implementation, specific options refer to its documentation, example:

```ts
import { defineConfig } from "@farmfe/core";

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

Configuring middlewares for the dev server. A middleware is a function that receives the `Server` instance and returns a [connect](https://github.com/senchalabs/connect)-compatible middleware function.

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  server: {
    middlewares: [
      (server) => {
        return (req, res, next) => {
          // custom middleware logic
          next();
        };
      },
    ],
  },
});
```

### middlewareMode
- **default**: `false`

Whether to run the server in middleware mode. When `true`, the dev server will not start its own HTTP server, which is useful when you want to integrate Farm into an existing server framework.

### origin
- **default**: `''`

The origin address of the server, used to configure the origin of the dev server in the response.

### allowedHosts
- **default**: `[]`

A list of allowed hostnames for the dev server. When configured, only requests with a `Host` header matching an entry in this list will be served.

### writeToDisk
- **default**: `false`

By default the compiled resources are stored and served in memory, set `writeToDisk` to `true` to emit dev resources to the disk.

## Preview Server Options - server.preview

Configure the preview server (used with `farm preview`). Preview-specific options are nested under `server.preview`.

```ts
export interface UserPreviewServerConfig {
  headers?: OutgoingHttpHeaders | false | undefined;
  host?: string | boolean;
  port?: number;
  strictPort?: boolean;
  https?: SecureServerOptions;
  distDir?: string;
  open?: boolean | string;
  cors?: boolean | any;
  proxy?: Record<string, ProxyOptions>;
  middlewares?: PreviewServerMiddleware[];
}
```

### `preview.port`

- **default**: `1911`

The port the preview server listens on.

### `preview.host`

- **default**: `'localhost'`

The host to run the preview server on.

### `preview.strictPort`

- **default**: `false`

Whether to strictly use the specified port. If the port is occupied and this is `true`, an error will be thrown.

### `preview.distDir`

- **default**: `'dist'`

Specify where the built output directory is located. If not specified, Farm resolves it from `compilation.output.path`. If the path is relative, it is relative to `root`.

### `preview.open`

- **default**: `false`

Automatically open the preview server in the default browser.

### `preview.https`

- **default**: inherits from `server.https`

Secure server options for the preview server. Set to `false` to disable HTTPS for preview.

### `preview.headers`

- **default**: inherits from `server.headers`

HTTP headers to send with every preview server response. Set to `false` to disable.

### `preview.proxy`

- **default**: inherits from `server.proxy`

Proxy options for the preview server. Set to `false` to disable proxy.

### `preview.middlewares`

- **default**: `[]`

Middlewares for the preview server, similar to `server.middlewares`.
