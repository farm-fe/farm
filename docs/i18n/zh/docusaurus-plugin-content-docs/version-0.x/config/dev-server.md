## DevServer 选项 - server
配置 Farm Dev Server 的行为。示例：

```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // 所有 dev server 选项都在 server 下面
  server: {
    port: 9000,
    // ...
  }
});
```

类型：
```ts
export interface UserServerConfig {
  port?: number;
  // https?: boolean;
  protocol?: 'http' | 'https';
  hostname?: string;
  // http2?: boolean;
  hmr?: boolean | UserHmrConfig;
  proxy?: Record<string, ProxiesOptions>;
  strictPort?: boolean;
  open?: boolean;
  host?: string;
  cors?: boolean | cors.Options;
  // whether to serve static assets in spa mode, default to true
  spa?: boolean;
  plugins?: DevServerPlugin[];
  writeToDisk?: boolean;
}
```
### port
* **默认值**: `9000`

DevServer 监听的端口。
<!-- ### https(WIP) -->

### hmr
* **默认值**: 对于 start 命令是 `true`，其他命令是 false

启用 HMR，开启后启用 HMR 能力，将会监听编译过程中涉及到的模块的变动，当模块变化时，自动触发重编译并将结果推送给 Farm Runtime 进行更新。也可以通过一个对象来配置 HMR，例如：

```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // 所有 dev server 选项都在 server 下面
  server: {
    hmr: {
      // 配置 web socket 监听的端口
      port: 9802
      // 配置 web socket 监听的 host
      host: 'localhost',
      // 配置文件监听时，忽略的文件
      ignores: ['auto_generated/*']
    }
    // ...
  }
});
```

#### `hmr.port`
* **默认值**: `9801`

Web Socket 服务器监听的端口

#### `hmr.host`
* **默认值**: `localhost`

Web Socket 服务器监听的 Host

### proxy
* **默认值**: `undefined`

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


<!-- ### strictPort
* **默认值**: `false` -->

### open
* **默认值**: `false`

编译完成后自动打开浏览器到对应的页面。

### host
* **默认值**: `localhost`

Dev Server 监听的 host。

### plugins
* **默认值**: `[]`

配置 Farm 的 Dev Server 插件，通过 Dev Server 插件可以扩展 DevServer 的上下文，添加 middleware 等。插件就是一个函数，插件示例如下：

```ts
export function hmrPlugin(devServer: DevServer) {
  const { config, logger } = devServer;
  if (config.hmr) {
    devServer.ws = new WebSocketServer({
      port: config.hmr.port,
      host: config.hmr.host
    });
    devServer.app().use(hmr(devServer));
    devServer.hmrEngine = new HmrEngine(devServer.getCompiler(), devServer, logger);
  }
}
```

然后将该插件配置到 `server.plugins` 中。
