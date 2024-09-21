# Dev Server

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
  port?: number;
  // https?: boolean;
  protocol?: "http" | "https";
  hostname?: string;
  // http2?: boolean;
  hmr?: boolean | UserHmrConfig;
  proxy?: Record<string, ProxiesOptions>;
  strictPort?: boolean;
  open?: boolean;
  host?: string;
  cors?: boolean | cors.Options;
  //whether to serve static assets in spa mode, default to true
  spa?: boolean;
  plugins?: DevServerPlugin[];
  writeToDisk?: boolean;
}
```

### port

- **default**: `9000`

The port the DevServer listens on.

<!-- ### https(WIP) -->

### hmr

- **default**: `true` for start command, false for other commands

Enable HMR. After enabling the HMR capability, it will monitor the changes of the modules involved in the compilation process. When the modules change, it will automatically trigger recompilation and push the results to Farm Runtime for update. HMR can also be configured through an object, for example:

```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
   return config;
}

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

Host on which the Web Socket server listens

### proxy

- **Default value**: `undefined`

Configure server proxy. farm uses `http-proxy` as a proxy for the development server. Based on [http-proxy](https://github.com/http-party/node-http-proxy?tab=readme-ov-file#options) implementation, specific options refer to its documentation, example:

```ts
import { defineConfig } from "@farmfe/core";

function defineConfig(config: UserConfig) {
  return config;
}

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

<!-- ### strictPort
* **default**: `false` -->

### open

- **default**: `false`

After the compilation is completed, the browser is automatically opened to the corresponding page.

###host

- **default**: `localhost`

The host that the Dev Server listens on.

### plugins

- **default**: `[]`

Configure the Dev Server plug-in of Farm, through the Dev Server plug-in, you can extend the context of DevServer, add middleware, etc. A plugin is a function. Examples of plugins are as follows:

```ts
export function hmrPlugin(devServer: DevServer) {
  const { config, logger } = devServer;
  if (config.hmr) {
    devServer.ws = new WebSocketServer({
      port: config.hmr.port,
      host: config.hmr.host,
    });
    devServer.app().use(hmr(devServer));
    devServer.hmrEngine = new HmrEngine(
      devServer.getCompiler(),
      devServer,
      logger
    );
  }
}
```

Then configure the plugin into `server.plugins`.
